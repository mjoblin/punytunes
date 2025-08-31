//! `AmplifierManager` manages all amplifier-related concerns.
//!
//! AmplifierManager expects to be running for the duration of the application. It might
//! encompass multiple Amplifier connections over time.
//!
//! Responsibilities:
//!
//!  - Receiving and acting on messages from the UI (`AmplifierManagerActionMsg`).
//!  - Starting an `AmplifierHandler` when appropriate.
//!  - Sending messages to the `AmplifierHandler`
//!    - Amplifier Handler messages (`AmplifierHandlerActionMsg`) (shut down, etc)
//!    - Amplifier control messages (`AmplifierActionMsg`) coming from the UI (mute, volume, etc)
//!  - Receiving messages from the `AmplifierHandler`
//!    - Amplifier state messages (`AmplifierStateMsg`) for passing on to the UI
//!    - Amplifier connection status messages (`AmplifierHandlerConnectionStatusMsg`)

use std::default::Default;

use log::Level::{Debug, Error, Info, Warn};
use serde;
use tauri::{AppHandle};
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};
use ts_rs::TS;

use amplifier_handler::{
    AmplifierHandler, AmplifierHandlerAction, AmplifierHandlerConnectionStatus, AmplifierHandlerRxChannelMsg,
    AmplifierHandlerTxChannelMsg, AmplifierState,
};
use discovery::{discover_amplifiers, AmplifierDevice};

use crate::errors::PunyTunesError;
use crate::messaging::AppMessageType;
use crate::streammagic_manager::StreamMagicManagerChannelMsg;
use crate::traits::CustomEmitters;

mod amplifier_handler;
mod discovery;
mod hegel;

// TODO: Consider what to do when AmplifierHandler connection is lost. Worst case, a
//  user-requested discovery should pick up the amplifier again. It would be nice to
//  auto-reconnect if the connection is unexpectedly lost (without ending up in a
//  never-ending loop of attempted reconnects).

/// All `AmplifierManager` state which might be of interest to the UI.
#[derive(Clone, Debug, serde::Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/AmplifierManagerStateMsg.ts")]
pub struct AmplifierManagerStateMsg {
    is_discovering: bool,
    is_handling_amplifier: bool,
    managed_device: Option<AmplifierDevice>,
}

/// Messages which can be sent to the `AmplifierManager` for the AmplifierManager to act on.
#[derive(Clone, Debug, serde::Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/AmplifierManagerAction.ts")]
pub enum AmplifierManagerAction {
    DisconnectFromAmplifier,
    Discover,
    OnUIReady,
    ProcessDiscoveredDevice(AmplifierDevice),
    SetIsDiscovering(bool),
    ShutDown,
    TestConnection,
}

/// Messages which can be sent to the `AmplifierManager` to be forwarded on to the
/// `AmplifierHandler`.
#[derive(Clone, Debug, serde::Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/AmplifierAction.ts")]
pub enum AmplifierAction {
    MuteSet(bool),
    MuteToggle,
    PowerSet(bool),
    PowerToggle,
    SourceSet(u8),
    VolumeDown,
    VolumeSet(u8),
    VolumeUp,
}

// ------------------------------------------------------------------------------------------------

pub struct AmplifierManagerChannel(pub mpsc::Sender<AmplifierManagerChannelMsg>);

#[derive(Clone, Debug)]
pub enum AmplifierManagerChannelMsg {
    AmplifierActionMsg(AmplifierAction),
    AmplifierManagerActionMsg(AmplifierManagerAction),
}

pub struct AmplifierManager {
    app_handle: AppHandle,
    streammagic_manager_channel: Sender<StreamMagicManagerChannelMsg>,
    // Receive commands from the application or AmplifierManager itself
    rx_channel: mpsc::Receiver<AmplifierManagerChannelMsg>,
    // Send commands to itself (for use by spawned tasks such as discovery, which won't
    // have access to self)
    tx_channel: mpsc::Sender<AmplifierManagerChannelMsg>,

    amp_handler_join_handle: Option<tauri::async_runtime::JoinHandle<Result<(), PunyTunesError>>>,
    amp_handler_receive_channel: Option<mpsc::Receiver<AmplifierHandlerTxChannelMsg>>,
    amp_handler_send_channel: Option<mpsc::Sender<AmplifierHandlerRxChannelMsg>>,
    amp_state: AmplifierState,
    handler_start_count: usize,
    is_discovering: bool,
    is_handling_amplifier: bool,
    is_shutting_down: bool,
    is_testing_connection: bool,
    managed_device: Option<AmplifierDevice>,
    max_reconnect_attempts: u8,
    reconnect_attempts: u8,
    reconnect_delay: u64,
}

impl AmplifierManager {
    pub fn new(
        app_handle: AppHandle,
        streammagic_manager_channel: Sender<StreamMagicManagerChannelMsg>,
        rx_channel: mpsc::Receiver<AmplifierManagerChannelMsg>,
        tx_channel: mpsc::Sender<AmplifierManagerChannelMsg>,
    ) -> AmplifierManager {
        AmplifierManager {
            app_handle,
            streammagic_manager_channel,
            rx_channel,
            tx_channel,

            amp_handler_join_handle: None,
            amp_handler_receive_channel: None,
            amp_handler_send_channel: None,
            amp_state: Default::default(),
            handler_start_count: 0,
            is_discovering: false,
            is_handling_amplifier: false,
            is_shutting_down: false,
            is_testing_connection: false,
            managed_device: None,
            max_reconnect_attempts: 3,
            reconnect_attempts: 0,
            reconnect_delay: 1000,
        }
    }

    // --------------------------------------------------------------------------------------------
    // Message handling

    /// Send `AmplifierManagerState` details to the UI.
    async fn emit_manager_state(&self) {
        self.app_handle
            .emit_app_message(
                AppMessageType::AmplifierManagerState,
                AmplifierManagerStateMsg {
                    is_discovering: self.is_discovering,
                    is_handling_amplifier: self.is_handling_amplifier,
                    managed_device: self.managed_device.clone(),
                },
            )
            .await;
    }

    /// Send `AmplifierState` details to the UI.
    async fn emit_amplifier_state(&self) {
        self.app_handle
            .emit_app_message(AppMessageType::AmplifierState, &self.amp_state.clone())
            .await;
    }

    // --------------------------------------------------------------------------------------------
    // State handling

    async fn set_amplifier_state(&mut self, amplifier_state: AmplifierState) {
        self.amp_state = amplifier_state;
        self.emit_amplifier_state().await;
    }

    async fn on_manager_state_changed(&self) {
        self.emit_manager_state().await;
    }

    async fn set_is_discovering(&mut self, is_discovering: bool) {
        self.is_discovering = is_discovering;
        self.on_manager_state_changed().await;
    }

    async fn set_is_handling_amplifier(&mut self, is_handling_amplifier: bool) {
        self.is_handling_amplifier = is_handling_amplifier;
        self.on_manager_state_changed().await;
    }

    // --------------------------------------------------------------------------------------------
    // Amplifier device discovery

    async fn do_discovery(&mut self) {
        // Will also optionally trigger a device activation.

        send_app_log!(
            &self.streammagic_manager_channel,
            Info,
            "Initiating amplifier discovery"
        );

        self.set_is_discovering(true).await;
        // self.clear_devices().await;

        let amp_channel_clone = self.tx_channel.clone();
        let sm_channel_clone = self.streammagic_manager_channel.clone();

        tauri::async_runtime::spawn(async move {
            match discover_amplifiers(&amp_channel_clone, &sm_channel_clone).await {
                Ok(()) => send_app_log!(&sm_channel_clone, Info, "Amplifier UPnP discovery complete"),
                Err(e) => {
                    send_app_log!(
                        &sm_channel_clone,
                        Error,
                        "Amplifier discovery failed with error: {:?}",
                        e
                    );
                    send_amplifier_manager_action!(&amp_channel_clone, AmplifierManagerAction::SetIsDiscovering(false));
                }
            };
        });
    }

    // --------------------------------------------------------------------------------------------
    // AmplifierHandler management

    /// Send the given AmplifierHandlerAction to the amplifier handler.
    async fn send_handler_action_to_amplifier_handler(&mut self, handler_action: AmplifierHandlerAction) {
        send_app_log!(
            &self.streammagic_manager_channel,
            Debug,
            "Sending {:?} to AmplifierHandler",
            &handler_action
        );

        match &self.amp_handler_send_channel {
            Some(amp_handler_channel) => {
                match amp_handler_channel
                    .send(AmplifierHandlerRxChannelMsg::AmplifierHandlerActionMsg(
                        handler_action.clone(),
                    ))
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        send_app_log!(
                            &self.streammagic_manager_channel,
                            Warn,
                            "Could not send {:?} to AmplifierHandler: {:?}",
                            &handler_action,
                            e
                        );
                    }
                }
            }
            None => {
                send_app_log!(
                    &self.streammagic_manager_channel,
                    Warn,
                    "No AmplifierHandler send channel found"
                );
            }
        }
    }

    /// Stop any running amplifier handler.
    async fn stop_amplifier_handler(&mut self) {
        if let Some(handle) = self.amp_handler_join_handle.take() {
            send_app_log!(
                &self.streammagic_manager_channel,
                Info,
                "Stopping existing AmplifierHandler"
            );
            self.send_handler_action_to_amplifier_handler(AmplifierHandlerAction::ShutDown)
                .await;

            match handle.await {
                Ok(amp_handler_result) => match amp_handler_result {
                    Ok(_) => {
                        send_app_log!(
                            &self.streammagic_manager_channel,
                            Info,
                            "AmplifierHandler successfully stopped"
                        );
                    }
                    Err(e) => {
                        send_app_log!(
                            &self.streammagic_manager_channel,
                            Warn,
                            "AmplifierHandler stopped with error: {:?}",
                            e
                        );
                    }
                },
                Err(e) => {
                    send_app_log!(
                        &self.streammagic_manager_channel,
                        Warn,
                        "Could not stop AmplifierHandler: {:?}",
                        e
                    );
                }
            }

            self.amp_handler_join_handle = None;
        } else if self.handler_start_count > 0 {
            send_app_log!(
                &self.streammagic_manager_channel,
                Info,
                "No existing AmplifierHandler to stop"
            );
        }
    }

    /// Initiate the handling of a single amplifier connection. There might be multiple of these
    /// over the lifetime of the manager (although only one at a time).
    async fn handle_amplifier(&mut self, device: AmplifierDevice) {
        // Stop any existing AmplifierHandler
        self.stop_amplifier_handler().await;
        self.managed_device = None;

        self.set_amplifier_state(Default::default()).await;

        if device.manufacturer != "Hegel" {
            send_app_log!(
                &self.streammagic_manager_channel,
                Info,
                "Ignoring unsupported amplifier manufacturer: {}",
                &device.manufacturer
            );
            self.emit_manager_state().await;
            return;
        }

        self.managed_device = Some(device.clone());
        self.emit_manager_state().await;

        // Set up the channels to talk to, and receive from, the new AmplifierHandler
        let (handler_cmd_channel_tx, handler_cmd_channel_rx) = mpsc::channel(32);
        let (handler_msg_channel_tx, handler_msg_channel_rx) = mpsc::channel(32);

        self.amp_handler_send_channel = Some(handler_cmd_channel_tx);
        self.amp_handler_receive_channel = Some(handler_msg_channel_rx);

        let streammagic_mgr_channel = self.streammagic_manager_channel.clone();

        // Start the AmplifierHandler and wait for it to complete
        self.amp_handler_join_handle = Some(tauri::async_runtime::spawn(async move {
            let mut amp_handler = hegel::HegelAmplifierHandler::new(
                device,
                handler_cmd_channel_rx,
                handler_msg_channel_tx,
                streammagic_mgr_channel.clone(),
            );

            let amp_handler_result = amp_handler.run().await;

            if let Err(err) = &amp_handler_result {
                send_app_log!(
                    &streammagic_mgr_channel,
                    Warn,
                    "AmplifierManager detected handler error: {:?}",
                    &err
                );
            }

            amp_handler_result
        }));

        self.handler_start_count += 1;
        self.emit_manager_state().await;
    }

    // --------------------------------------------------------------------------------------------

    async fn initialize(&mut self) {
        self.do_discovery().await;
    }

    // --------------------------------------------------------------------------------------------

    /// Main run loop for the AmplifierManager. This should run forever (unless told to shut down).
    pub async fn run(&mut self) -> Result<(), PunyTunesError> {
        self.initialize().await;
        self.emit_amplifier_state().await;

        // Configure an interval which will always be checked regardless of whether there's any
        // items waiting in a channel for processing.
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            select! {
                // --------------------------------------------------------------------------------
                // Incoming commands requested of the Manager. These commands will be coming from
                // either a Tauri command (requested by the UI), or from the Manager itself (e.g.
                // from an async discovery task which wants to inform the Manager of the discovered
                // devices).
                incoming_cmd_check = self.rx_channel.recv() => {
                    if let Some(cmd) = incoming_cmd_check {
                        match cmd {
                            // Manager-level action
                            AmplifierManagerChannelMsg::AmplifierManagerActionMsg(manager_action) => {
                                match manager_action {
                                    AmplifierManagerAction::DisconnectFromAmplifier => {
                                        self.stop_amplifier_handler().await;
                                    },
                                    AmplifierManagerAction::Discover => {
                                        self.do_discovery().await;
                                    },
                                    AmplifierManagerAction::OnUIReady => {
                                        self.emit_manager_state().await;
                                        self.emit_amplifier_state().await;
                                    },
                                    AmplifierManagerAction::ProcessDiscoveredDevice(device) => {
                                        send_app_log!(&self.streammagic_manager_channel, Info, "Processing discovered amplifier: {}", device);
                                        self.reconnect_attempts = 0;
                                        self.handle_amplifier(device).await;
                                    },
                                    AmplifierManagerAction::SetIsDiscovering(is_discovering) => {
                                        self.set_is_discovering(is_discovering).await;
                                    },
                                    AmplifierManagerAction::ShutDown => {
                                        send_app_log!(&self.streammagic_manager_channel, Info, "AmplifierManager shutdown requested");
                                        self.is_shutting_down = true;
                                        self.stop_amplifier_handler().await;
                                        break;
                                    },
                                    AmplifierManagerAction::TestConnection => {
                                        if self.is_handling_amplifier {
                                            self.is_testing_connection = true;
                                            self.send_handler_action_to_amplifier_handler(
                                                AmplifierHandlerAction::TestConnection
                                            ).await;
                                        } else {
                                            send_app_log!(
                                                &self.streammagic_manager_channel, Warn,
                                                "Ignoring request to test connection (not currently handling an amplifier)"
                                            );
                                        }
                                    },
                                }
                            },
                            // Handler-level action to be passed on to the AmplifierHandler
                            AmplifierManagerChannelMsg::AmplifierActionMsg(amplifier_action) => {
                                if self.is_handling_amplifier {
                                    if let Some(sender) = &self.amp_handler_send_channel {
                                        match sender.send(AmplifierHandlerRxChannelMsg::AmplifierActionMsg(amplifier_action.clone())).await {
                                            Ok(_) => {}
                                            Err(e) => {
                                                send_app_log!(&self.streammagic_manager_channel, Warn,
                                                    "Could not send command {:?} to AmplifierHandler: {:?}",
                                                    amplifier_action, e
                                                );
                                            }
                                        }
                                    }
                                } else {
                                    send_app_log!(
                                        &self.streammagic_manager_channel,
                                        Warn,
                                        "AmplifierManager is not handling an amplifier connection"
                                    );
                                }
                            },
                        }
                    }
                }

                // --------------------------------------------------------------------------------
                // Messages coming from the AmplifierHandler.
                amp_handler_message = async {
                    self.amp_handler_receive_channel.as_mut().expect("AmplifierHandler crash").recv().await
                }, if &self.amp_handler_receive_channel.is_some() => {

                    if let Some(update) = amp_handler_message {
                        match update {
                            AmplifierHandlerTxChannelMsg::AmplifierStateMsg(amp_state) => {
                                self.set_amplifier_state(amp_state).await;
                            },
                            AmplifierHandlerTxChannelMsg::AmplifierHandlerConnectionStatusMsg(status) => {
                                match &status {
                                    AmplifierHandlerConnectionStatus::Connected => {
                                        self.set_is_handling_amplifier(true).await;
                                        self.reconnect_attempts = 0;
                                    },
                                    AmplifierHandlerConnectionStatus::Disconnected => {
                                        self.set_is_handling_amplifier(false).await;
                                        self.stop_amplifier_handler().await;

                                        if !self.is_shutting_down {
                                            // Attempt a reconnect
                                            if let Some(existing_device) = &self.managed_device {
                                                self.reconnect_attempts += 1;

                                                if self.reconnect_attempts <= self.max_reconnect_attempts {
                                                    send_app_log!(
                                                        &self.streammagic_manager_channel,
                                                        Warn,
                                                        "AmplifierManager is attempting an amplifier reconnect (#{})",
                                                        self.reconnect_attempts,
                                                    );

                                                    sleep(Duration::from_millis(self.reconnect_delay)).await;
                                                    self.handle_amplifier(existing_device.clone()).await;
                                                } else {
                                                    send_app_log!(
                                                        &self.streammagic_manager_channel,
                                                        Warn,
                                                        "AmplifierManager is giving up on amplifier reconnects",
                                                    );

                                                    self.reconnect_attempts = 0;
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                        }
                    }
                }

                // --------------------------------------------------------------------------------
                // Do some checks every interval, regardless of incoming messages.
                _ = interval.tick() => {
                },
            }
        }

        // We don't expect to get here unless the application is ending.
        send_app_log!(&self.streammagic_manager_channel, Info, "AmplifierManager has ended");

        Ok(())
    }
}
