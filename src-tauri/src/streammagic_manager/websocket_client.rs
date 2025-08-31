// TODO: Consider having WebSocketClient use send_app_log!() instead of app_info!()/etc. This
//  would ensure that any early logs (before the UI is ready) are buffered properly. It would also
//  require that WebSocketClient be passed a copy of the mpsc sender to then pass to send_app_log!().

use std::default::Default;
use std::time::SystemTime;

use futures_util::{SinkExt, StreamExt};
use log::{info, Level::{Error, Info, Warn}};
use serde;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::{Duration, timeout};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use ts_rs::TS;
use tungstenite;
use url;

use crate::average::RunningAverage;
use crate::errors::PunyTunesError;
use crate::streammagic_manager::StreamMagicManagerChannelMsg;

// Channel messages which can be sent from the invoker (StreamMagicManager) to the WebSocketClient
#[derive(Clone, Debug)]
pub enum WSClientRxChannelMsg {
    DataMsg(String),
    WebSocketClientActionMsg(WebSocketClientAction),
}

// Channel messages which can be sent from the WebSocketClient to the invoker (StreamMagicManager)
#[derive(Clone, Debug)]
pub enum WSClientTxChannelMsg {
    DataMsg(String),
    // WebSocket data text, as received from the StreamMagic WebSocket server
    WebSocketClientStatusMsg(WebSocketClientStatus),
}

#[derive(Clone, Debug)]
pub enum WebSocketClientAction {
    ShutDown,
    TestConnection,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/WebSocketClientConnectedDetails.ts")]
pub struct WebSocketClientConnectedDetails {
    pub url: String, // The connected server URL (e.g. "ws://{host}:80/smoip")
    pub existing: bool, // Whether the connection was pre-existing
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/WebSocketClientDisconnectedDetails.ts")]
pub struct WebSocketClientDisconnectedDetails {
    pub reason: Option<String>,
    pub consider_reconnecting: bool,
}

impl Default for WebSocketClientDisconnectedDetails {
    fn default() -> Self {
        WebSocketClientDisconnectedDetails {
            reason: None,
            consider_reconnecting: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, TS)]
#[serde(tag = "state", content = "metadata")]
#[ts(export, export_to = "../src/types/generated/WebSocketClientStatus.ts")]
pub enum WebSocketClientStatus {
    Disconnected(WebSocketClientDisconnectedDetails),
    // String is the server URL being attempted (e.g. "ws://{host}:80/smoip")
    Connecting(String),
    Connected(WebSocketClientConnectedDetails),
    TestingConnection,
}

// ------------------------------------------------------------------------------------------------

/// WebSocketClient handles a single WebSocket connection to a WebSocket server.
///
/// A single WebSocketClient instance expects to last only as long as a single connection to a
/// WebSocket server. It's possible that multiple WebSocketClient instances will be created over
/// the duration of an application session.
///
/// Responsibilities:
///   * Connects to the WebSocket server at the provided `url`.
///   * Receives messages on `rx_channel`:
///       * `DataMsg`: A plain text message to send to the WebSocket server.
///       * `WebSocketClientActionMsg`: An action to perform (e.g. ShutDown).
///   * Sends messages on `tx_channel`
///       * `DataMsg`: A plain text message (from the WebSocket server) to send to the invoker.
///       * `WebSocketClientStatusMsg`: A client status update.
///
/// WebSocketClient does not have any awareness of StreamMagic. Instead, it passes messages back
/// and forth as plain text. `StreamMagicManager` handles the StreamMagic-specific concepts.
pub struct WebSocketClient {
    url: String,
    manager_channel: Sender<StreamMagicManagerChannelMsg>,
    rx_channel: Receiver<WSClientRxChannelMsg>,
    tx_channel: Sender<WSClientTxChannelMsg>,
    status: WebSocketClientStatus,
    connection_timeout_ms: u64,
    test_connection_pong_timeout_ms: u128,
    stop_on_missing_pings: bool,
}

impl WebSocketClient {
    pub fn new(
        url: &str,
        manager_channel: Sender<StreamMagicManagerChannelMsg>,
        rx_channel: Receiver<WSClientRxChannelMsg>,
        tx_channel: Sender<WSClientTxChannelMsg>,
        stop_on_missing_pings: bool,
    ) -> Self {
        WebSocketClient {
            url: url.to_string(),
            manager_channel,
            rx_channel,
            tx_channel,
            status: WebSocketClientStatus::Disconnected(Default::default()),
            connection_timeout_ms: 2000,
            test_connection_pong_timeout_ms: 1000,
            stop_on_missing_pings,
        }
    }

    pub async fn run(&mut self) -> Result<(), PunyTunesError> {
        let (ws_stream, _) = self.connect().await?;
        let (mut ws_write, mut ws_read) = ws_stream.split();

        // Configure an interval which will always be checked regardless of whether there's any
        // items waiting in a channel for processing.
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        // Track WebSocket server ping times. This is done to establish when the client may have
        // lost the connection (perhaps due to going to sleep). This is different from the server
        // explicitly closing the connection (see tungstenite::Message::Close). The average ping
        // duration is tracked, and if enough time has passed since the last ping then we assume
        // the connection is lost. We wait for at least 2 pings to come in before calculating the
        // average ping delay.
        let mut ping_avg = RunningAverage::new(10);
        let mut last_ping_time = SystemTime::now();
        let mut have_ignored_first_ping = false;

        // Track pong times as part of WebSocketClientStatus::TestingConnection.
        let mut test_connection_ping_time = SystemTime::now();

        const START_PING_THRESHOLD_SECS: u64 = 600; // For use until average ping time is known
        const PING_DURATION_BUFFER_FACTOR: f64 = 1.25; // 125% is considered too long to wait

        loop {
            select! {
                incoming_cmd_check = self.rx_channel.recv() => {
                    if let Some(cmd) = incoming_cmd_check {
                        match &cmd {
                            WSClientRxChannelMsg::DataMsg(msg) => {
                                // Send raw message text to WebSocket server
                                match ws_write.send(msg.clone().into()).await {
                                    Ok(()) => {},
                                    Err(e) => {
                                        send_app_log!(&self.manager_channel, Error, "WebSocket send error: {:?}", e);
                                    } ,
                                }
                            }
                            WSClientRxChannelMsg::WebSocketClientActionMsg(action_msg) => {
                                match action_msg {
                                    WebSocketClientAction::ShutDown => {
                                        send_app_log!(&self.manager_channel, Info, "WebSocketClient shutting down");
                                        self.set_status(WebSocketClientStatus::Disconnected(
                                            WebSocketClientDisconnectedDetails { reason: None, consider_reconnecting: false }
                                        )).await;

                                        // This breaks out of run() which will close the connection
                                        break;
                                    }
                                    WebSocketClientAction::TestConnection => {
                                        match self.status {
                                            WebSocketClientStatus::Connected(_) => {
                                                // Initiate the test by sending a ping. If a pong response is not received
                                                // within a given time limit then the connection is considered lost. Pong
                                                // checks are performed in interval.tick().
                                                match ws_write.send(tungstenite::protocol::Message::Ping("PING".into())).await {
                                                    Ok(_) => {
                                                        test_connection_ping_time = SystemTime::now();
                                                        self.set_status(WebSocketClientStatus::TestingConnection).await;
                                                        info!("Test connection ping sent");
                                                    },
                                                    Err(e) => {
                                                        info!("Test connection ping error: {:?}", e);
                                                    }
                                                }
                                            },
                                            _ => {
                                                // We were asked to test the connection while not connected, so
                                                // just respond with the non-Connected current state.
                                                self.set_status(self.status.clone()).await;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                Some(read_item) = ws_read.next() => {
                    match read_item {
                        Ok(message) => match message {
                            tungstenite::Message::Ping(_) => {
                                let now = SystemTime::now();

                                // Keep track of how long we're waiting between pings. Ignore the first
                                // ping because it might throw off the average wait time calculation.
                                if have_ignored_first_ping {
                                    match now.duration_since(last_ping_time) {
                                        Ok(duration) => ping_avg.add(duration.as_secs() as f64),
                                        _ => {},
                                    };
                                } else {
                                    have_ignored_first_ping = true;
                                }

                                last_ping_time = now;

                                info!("Ping average: {}", ping_avg.average());
                            },
                            tungstenite::Message::Pong(_) => {
                                if self.status == WebSocketClientStatus::TestingConnection {
                                    // Streamer responded to our test ping
                                    self.set_status(WebSocketClientStatus::Connected(
                                        WebSocketClientConnectedDetails {
                                            url: self.url.clone(),
                                            existing: true,
                                        }
                                    )).await;
                                }
                            },
                            tungstenite::Message::Close(_) => {
                                // Explicit server connection close. This is distinct from the client
                                // losing the connection for other reasons (which is detected by ping
                                // time checks).
                                let reason = String::from("WebSocket connection closed by server");
                                send_app_log!(&self.manager_channel, Warn, "{}", reason.clone());
                                self.set_status(WebSocketClientStatus::Disconnected(
                                    WebSocketClientDisconnectedDetails { reason: Some(reason), consider_reconnecting: true }
                                )).await;

                                return Err(PunyTunesError::WebSocketConnectionClosed);
                            },
                            tungstenite::Message::Text(message_text) => {
                                // Standard incoming data from the WebSocket server
                                match self.tx_channel.send(WSClientTxChannelMsg::DataMsg(message_text)).await {
                                    Ok(()) => {},
                                    Err(e) => {
                                        send_app_log!(&self.manager_channel, Error, "Error sending message to StreamMagic Manager: {:?}", e);
                                    },
                                }
                            },
                            unexpected => {
                                send_app_log!(&self.manager_channel, Warn, "Ignoring unexpected WebSocket message type: {:?}", unexpected);
                            },
                        },
                        Err(e) => {
                            if self.status == WebSocketClientStatus::TestingConnection {
                                // A test ping can be followed by this read error. This might be
                                // what happens when we attempt to send a test ping to a websocket
                                // server which has already dropped our connection (perhaps due to
                                // our machine going to sleep).
                                let reason = format!("WebSocketClient test failed (message read error: {})", e);
                                send_app_log!(&self.manager_channel, Warn, "{}", reason.clone());
                                self.set_status(WebSocketClientStatus::Disconnected(
                                    WebSocketClientDisconnectedDetails { reason: Some(reason), consider_reconnecting: true }
                                )).await;

                                return Err(PunyTunesError::WebSocketClientLostConnection);
                            } else {
                                let reason = format!("Could not read next item from WebSocket server stream: {}", e);
                                send_app_log!(&self.manager_channel, Warn, "{}", &reason);
                                self.set_status(WebSocketClientStatus::Disconnected(
                                    WebSocketClientDisconnectedDetails { reason: Some(reason.clone()), consider_reconnecting: true }
                                )).await;

                                return Err(PunyTunesError::WebSocket(reason))
                            }
                        },
                    }
                }

                _ = interval.tick() => {
                    // Do some checks every interval, regardless of incoming messages.

                    if self.status == WebSocketClientStatus::TestingConnection {
                        // Check whether we haven't received a test pong response in the allotted time
                        let now = SystemTime::now();

                        if let Ok(duration) = now.duration_since(test_connection_ping_time) {
                            if duration.as_millis() > self.test_connection_pong_timeout_ms {
                                let reason = String::from("WebSocketClient test failed (pong timeout)");
                                send_app_log!(&self.manager_channel, Warn, "{}", reason.clone());
                                self.set_status(WebSocketClientStatus::Disconnected(
                                    WebSocketClientDisconnectedDetails { reason: Some(reason), consider_reconnecting: true }
                                )).await;

                                return Err(PunyTunesError::WebSocketClientLostConnection);
                            }
                        }
                    }

                    if self.stop_on_missing_pings {
                        // Check if it's been too long since we last received a server ping.
                        match SystemTime::now().duration_since(last_ping_time) {
                            Ok(duration) => {
                                let ping_duration_check = match ping_avg.len() {
                                    len if len > 2 => ping_avg.average() * PING_DURATION_BUFFER_FACTOR,
                                    _ => START_PING_THRESHOLD_SECS as f64,
                                };

                                if duration.as_secs() as f64 > ping_duration_check {
                                    send_app_log!(&self.manager_channel, Warn, "WebSocket ping not received for {ping_duration_check} secs; connection lost");

                                    return Err(PunyTunesError::WebSocketClientLostConnection);
                                }
                            },
                            Err(e) => {
                                let error = format!("Could not determine last WebSocket ping duration: {:?}", e);
                                send_app_log!(&self.manager_channel, Error, "{}", &error);

                                return Err(PunyTunesError::WebSocket(error));
                            }
                        }
                    }
                },
            }
        }

        send_app_log!(&self.manager_channel, Info, "WebSocketClient has stopped");

        Ok(())
    }

    async fn connect(
        &mut self,
    ) -> Result<
        (
            WebSocketStream<MaybeTlsStream<TcpStream>>,
            tungstenite::handshake::client::Response,
        ),
        PunyTunesError,
    > {
        send_app_log!(&self.manager_channel, Info, "WebSocketClient connecting to {}", &self.url);
        self.set_status(WebSocketClientStatus::Connecting(self.url.clone())).await;

        // Create a WebSocket connection Request from the given URL.
        let url = match url::Url::parse(&self.url) {
            Ok(url) => url,
            Err(e) => {
                let error = format!("URL parsing error: {:?}", e);
                self.set_status(WebSocketClientStatus::Disconnected(
                    WebSocketClientDisconnectedDetails { reason: Some(error.clone()), consider_reconnecting: false }
                )).await;

                return Err(PunyTunesError::WebSocket(error));
            }
        };

        // StreamMagic requires a custom "Origin" header to be included in the connection request,
        // so we create a custom default WebSocket Request and modify its headers.
        let mut request = match url.into_client_request() {
            Ok(r) => r,
            Err(e) => {
                let error = format!("Could not create WebSocket connection request: {:?}", e);
                self.set_status(WebSocketClientStatus::Disconnected(
                    WebSocketClientDisconnectedDetails { reason: Some(error.clone()), consider_reconnecting: false }
                )).await;

                return Err(PunyTunesError::WebSocket(error));
            }
        };

        let headers = request.headers_mut();
        headers.insert("Origin", "punytunes".parse().unwrap());

        // Detect connection attempt timeouts.
        let connect_timeout = Duration::from_millis(self.connection_timeout_ms);
        let connect_attempt = connect_async(request);

        // Attempt the connection.
        return match timeout(connect_timeout, connect_attempt).await {
            Ok(Ok(result)) => {
                self.set_status(WebSocketClientStatus::Connected(
                    WebSocketClientConnectedDetails {
                        url: self.url.clone(),
                        existing: false,
                    }
                )).await;

                Ok(result)
            }
            Ok(Err(e)) => {
                let error_message = match e {
                    tungstenite::Error::Io(e) => format!("{}", e.to_string().replace(r#"\""#, "")),
                    _ => format!("{:?}", e),
                };

                let reported_error = format!("WebSocketClient connection error: {:?}", error_message);
                send_app_log!(&self.manager_channel, Warn, "{}", &reported_error);

                Err(PunyTunesError::WebSocket(reported_error))
            }
            Err(_) => {
                send_app_log!(&self.manager_channel, Warn, "WebSocketClient connection timed out");

                Err(PunyTunesError::WebSocketTimeout)
            },
        };
    }

    async fn set_status(&mut self, status: WebSocketClientStatus) {
        self.status = status;

        match self
            .tx_channel
            .send(WSClientTxChannelMsg::WebSocketClientStatusMsg(self.status.clone()))
            .await
        {
            Ok(()) => {}
            Err(e) => {
                send_app_log!(&self.manager_channel, Warn, "Could not send WebSocketClientStatus update: {:?}", e);
            }
        }
    }
}
