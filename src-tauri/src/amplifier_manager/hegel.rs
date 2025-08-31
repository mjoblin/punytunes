//! AmplifierHandler implementation for Hegel amplifiers.
//!
//! Based on the Hegel IP Control Codes API:
//!   https://support.hegel.com/component/jdownloads/send/3-files/81-h120-ip-control-codes

use std::default::Default;
use std::io::Cursor;
use std::time::SystemTime;

use async_trait::async_trait;
use bytes::{Buf, BytesMut};
use log::{
    debug, info, warn,
    Level::{Info, Warn},
};
use regex::Regex;
use tauri::regex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::{sleep, timeout, Duration};

use crate::amplifier_manager::amplifier_handler::{
    AmplifierHandler, AmplifierHandlerAction, AmplifierHandlerConnectionStatus,
    AmplifierHandlerConnectionStatus::{Connected, Disconnected},
    AmplifierHandlerRxChannelMsg, AmplifierHandlerTxChannelMsg, AmplifierState,
};
use crate::amplifier_manager::discovery::AmplifierDevice;
use crate::amplifier_manager::AmplifierAction;
use crate::errors::PunyTunesError;
use crate::streammagic_manager::StreamMagicManagerChannelMsg;
use crate::utils::host_from_url;

// ================================================================================================
// Hegel command handling
// ================================================================================================

#[derive(Clone, Debug)]
enum HegelCommand {
    Error(String),
    Mute(Option<bool>),
    Power(Option<bool>),
    Source(Option<u8>),
    Volume(Option<u8>),
}

impl HegelCommand {
    pub fn code(&self) -> char {
        match self {
            HegelCommand::Error(_) => 'e',
            HegelCommand::Mute(_) => 'm',
            HegelCommand::Power(_) => 'p',
            HegelCommand::Source(_) => 'i',
            HegelCommand::Volume(_) => 'v',
        }
    }

    /// Generate a Hegel-compliant command string to request a command value.
    pub fn request(&self) -> String {
        String::from(format!("-{}.?", self.code()))
    }

    /// Generate a Hegel-compliant command string to toggle a command's value.
    pub fn toggle(&self) -> String {
        match self {
            HegelCommand::Mute(_) | HegelCommand::Power(_) => String::from(format!("-{}.t", self.code())),
            _ => String::from(""),
        }
    }

    /// Generate a Hegel-compliant command string to "up" a command's value (only makes sense for
    /// volume control).
    pub fn up(&self) -> String {
        match self {
            HegelCommand::Volume(_) => String::from(format!("-{}.u", self.code())),
            _ => String::from(""),
        }
    }

    /// Generate a Hegel-compliant command string to "down" a command's value (only makes sense for
    /// volume control).
    pub fn down(&self) -> String {
        match self {
            HegelCommand::Volume(_) => String::from(format!("-{}.d", self.code())),
            _ => String::from(""),
        }
    }
}

/// Convert a network `Frame` to a `HegelCommand`.
impl TryFrom<Frame> for HegelCommand {
    type Error = String;

    fn try_from(frame: Frame) -> Result<Self, <HegelCommand as TryFrom<Frame>>::Error> {
        match frame {
            Frame::Data(frame_data) => match Regex::new(r"^-([eimpv])\.(.*)$") {
                Ok(pattern) => {
                    if let Some(captures) = pattern.captures(&frame_data) {
                        let command_code = captures.get(1).map_or("", |m| m.as_str());
                        let command_value = captures.get(2).map_or("", |m| m.as_str());

                        return match command_code {
                            "e" => Ok(HegelCommand::Error(command_value.into())),
                            "i" => {
                                return match command_value.parse::<u8>() {
                                    Ok(source_id) => match source_id {
                                        // TODO: Different amplifiers support a different number of
                                        //  sources. We currently don't care much since we don't
                                        //  support source switching.
                                        1..=13 => Ok(HegelCommand::Source(Some(source_id))),
                                        _ => Err(String::from(format!(
                                            "Source value out of range (valid is 1-13): {source_id}"
                                        ))),
                                    },
                                    Err(_) => Err(String::from(format!("Invalid source value: {command_value}"))),
                                };
                            }
                            "m" => match command_value {
                                "0" => Ok(HegelCommand::Mute(Some(false))),
                                "1" => Ok(HegelCommand::Mute(Some(true))),
                                _ => Err(String::from(format!("Invalid mute value: {command_value}"))),
                            },
                            "p" => match command_value {
                                "0" => Ok(HegelCommand::Power(Some(false))),
                                "1" => Ok(HegelCommand::Power(Some(true))),
                                _ => Err(String::from(format!("Invalid power value: {command_value}"))),
                            },
                            "v" => match command_value.parse::<u8>() {
                                Ok(volume) => match volume {
                                    0..=100 => Ok(HegelCommand::Volume(Some(volume))),
                                    _ => Err(String::from(format!(
                                        "Volume value out of range (valid is 0-100): {volume}"
                                    ))),
                                },
                                Err(_) => Err(String::from(format!("Invalid volume value: {command_value}"))),
                            },
                            _ => Err(String::from(format!("Unexpected command code: {command_code}"))),
                        };
                    } else {
                        return Err(String::from(format!("Invalid Frame data: {frame_data}")));
                    }
                }
                Err(e) => Err(format!("Could not construct Frame Regex: {:?}", e)),
            },
        }
    }
}

// ================================================================================================
// Hegel network frame and TCP connection handling
//
// Reference: https://tokio.rs/tokio/tutorial/framing
// ================================================================================================

#[derive(Debug)]
enum FrameError {
    Incomplete,
    Parse(String),
}

impl From<FrameError> for std::io::Error {
    fn from(err: FrameError) -> Self {
        match err {
            FrameError::Incomplete => std::io::Error::new(std::io::ErrorKind::Other, "Hegel frame incomplete"),
            FrameError::Parse(detail) => {
                std::io::Error::new(std::io::ErrorKind::Other, format!("Hegel frame parse error: {detail}"))
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Frame {
    Data(String),
}

/// A network `Frame`. A Frame is a Hegel command like "-v.25" terminated with a CR.
impl Frame {
    /// Check whether a valid Frame is available for reading from the buffer.
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), FrameError> {
        get_line(src)?;

        Ok(())
    }

    /// Extract a single Frame from the buffer.
    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, FrameError> {
        let line = get_line(src)?.to_vec();

        match String::from_utf8(line) {
            Ok(string) => Ok(Frame::Data(string)),
            Err(e) => Err(FrameError::Parse(format!("{:?}", e))),
        }
    }
}

/// Retrieve a CR-terminated string from the buffer.
fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], FrameError> {
    let start = src.position() as usize;
    let end = src.get_ref().len();

    for i in start..end {
        if src.get_ref()[i] == b'\r' {
            // We found a line, update the position to be *after* the \r
            src.set_position((i + 1) as u64);

            // Return the line
            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(FrameError::Incomplete)
}

impl From<String> for Frame {
    fn from(string: String) -> Self {
        Frame::Data(format!("{string}\r"))
    }
}

/// Convert a `HegelCommand` to a network `Frame` for sending on the wire.
impl TryFrom<HegelCommand> for Frame {
    type Error = String;

    fn try_from(command: HegelCommand) -> Result<Self, Self::Error> {
        let code = command.code();

        let command_as_string = match command {
            HegelCommand::Error(value) => format!("-{code}.{value}"),
            HegelCommand::Mute(value) => match value {
                Some(is_muted) => match is_muted {
                    true => format!("-{code}.1"),
                    false => format!("-{code}.0"),
                },
                None => return Err(String::from("Mute value must be a bool")),
            },
            HegelCommand::Power(value) => match value {
                Some(is_powered_on) => match is_powered_on {
                    true => format!("-{code}.1"),
                    false => format!("-{code}.0"),
                },
                None => return Err(String::from("Power value must be a bool")),
            },
            HegelCommand::Source(value) => match value {
                Some(source_id) => match source_id {
                    1..=13 => format!("-{code}.{source_id}"),
                    _ => return Err(String::from("Source id must be between 1 and 13")),
                },
                None => return Err(String::from("Source id must be a u8 between 1 and 13")),
            },
            HegelCommand::Volume(value) => match value {
                Some(level) => match level {
                    0..=100 => format!("-{code}.{level}"),
                    _ => return Err(String::from("Volume level must be between 0 and 100")),
                },
                None => return Err(String::from("Volume level must be a u8 between 0 and 100")),
            },
        };

        return Ok(command_as_string.into());
    }
}

// ------------------------------------------------------------------------------------------------
// TCP Connection

/// Handle a TcpStream connection to a Hegel amplifier. Reads and writes Frames which represent
/// Hegel Commands.
struct HegelConnection {
    stream: TcpStream,
    buffer: BytesMut,
}

impl HegelConnection {
    pub fn new(stream: TcpStream) -> Self {
        HegelConnection {
            stream,
            buffer: BytesMut::with_capacity(1024),
        }
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>, std::io::Error> {
        let mut buf = Cursor::new(&self.buffer[..]);

        match Frame::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;
                buf.set_position(0);

                let frame = Frame::parse(&mut buf)?;
                self.buffer.advance(len);

                Ok(Some(frame))
            }
            Err(frame_error) => match frame_error {
                // Not enough data has been buffered yet
                FrameError::Incomplete => Ok(None),
                FrameError::Parse(_) => {
                    // Skip over the unparseable text
                    let len = buf.position() as usize;
                    self.buffer.advance(len);

                    Ok(None)
                }
            },
        }
    }

    /// Read a complete frame from the amplifier (when available on the wire).
    pub async fn read_frame(&mut self) -> Result<Option<Frame>, std::io::Error> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::ConnectionReset,
                        "Hegel amplifier connection reset by peer",
                    ));
                }
            }
        }
    }

    /// Write a complete frame to the amplifier.
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<(), std::io::Error> {
        match frame {
            Frame::Data(val) => {
                debug!("Sending frame: {}", &val);
                self.stream.write_all(val.as_bytes()).await?;
            }
        }

        self.stream.flush().await?;

        Ok(())
    }

    /// Close the TcpStream connection.
    pub async fn shutdown(&mut self) {
        match self.stream.shutdown().await {
            Ok(_) => info!("Connection to amplifier has been shut down"),
            Err(e) => warn!("Connection to amplifier could not be cleanly shut down: {:?}", e),
        }
    }
}

// ================================================================================================
// HegelAmplifierHandler
// ================================================================================================

/// `AmplifierHandler` implementation for Hegel amplifiers.
pub struct HegelAmplifierHandler {
    device: AmplifierDevice,
    rx_channel: Receiver<AmplifierHandlerRxChannelMsg>,
    tx_channel: Sender<AmplifierHandlerTxChannelMsg>,
    streammagic_manager_channel: Sender<StreamMagicManagerChannelMsg>,

    amplifier_state: AmplifierState,
    connection_status: AmplifierHandlerConnectionStatus,
    connection_test_start_time: Option<SystemTime>,
    connection_test_timeout: u128,
    connection_timeout: u64,
    max_heartbeat_timeout: u128,
}

#[async_trait]
impl AmplifierHandler for HegelAmplifierHandler {
    fn new(
        device: AmplifierDevice,
        rx_channel: Receiver<AmplifierHandlerRxChannelMsg>,
        tx_channel: Sender<AmplifierHandlerTxChannelMsg>,
        streammagic_manager_channel: Sender<StreamMagicManagerChannelMsg>,
    ) -> Self {
        HegelAmplifierHandler {
            device,
            rx_channel,
            tx_channel,
            streammagic_manager_channel,

            amplifier_state: Default::default(),
            connection_status: Disconnected,
            connection_test_start_time: None,
            connection_test_timeout: 1_500,
            connection_timeout: 1_500,
            max_heartbeat_timeout: 10_000,
        }
    }

    async fn run(&mut self) -> Result<(), PunyTunesError> {
        send_app_log!(&self.streammagic_manager_channel, Info, "AmplifierHandler is running");

        // Attempt to connect to the amplifier
        let stream = match self.connect_to_amplifier().await {
            Ok(stream) => stream,
            Err(e) => {
                send_app_log!(
                    &self.streammagic_manager_channel,
                    Info,
                    "AmplifierHandler stopping after failed connection attempt"
                );
                self.set_connection_status(Disconnected).await;

                return Err(e);
            }
        };

        send_app_log!(
            &self.streammagic_manager_channel,
            Info,
            "Connected to amplifier: {}",
            &self.device.friendly_name
        );

        let mut hegel_connection = HegelConnection::new(stream);

        self.request_initial_amplifier_state(&mut hegel_connection).await;

        // Configure an interval which will always be checked regardless of whether there's any
        // items waiting in a channel for processing.
        let mut interval = tokio::time::interval(Duration::from_millis(500));
        let mut last_amplifier_heartbeat = SystemTime::now();

        // Run forever (or until told to stop), processing messages from the Manager and from the
        // amplifier. Also perform interval checks (heartbeat, etc).

        loop {
            select! {
                // --------------------------------------------------------------------------------
                // Check for messages coming from the AmplifierManager
                incoming_cmd_check = self.rx_channel.recv() => {
                    if let Some(cmd) = incoming_cmd_check {
                        match &cmd {
                            AmplifierHandlerRxChannelMsg::AmplifierHandlerActionMsg(handler_action) => {
                                match handler_action {
                                    AmplifierHandlerAction::ShutDown => {
                                        send_app_log!(&self.streammagic_manager_channel, Info, "AmplifierHandler is shutting down");
                                        self.set_connection_status(Disconnected).await;

                                        // This breaks out of run() which will close the connection
                                        break;
                                    },
                                    AmplifierHandlerAction::TestConnection => {
                                        self.initiate_connection_test(&mut hegel_connection).await;
                                    }
                                }
                            },
                            AmplifierHandlerRxChannelMsg::AmplifierActionMsg(action) => {
                                self.send_action_to_amplifier(&mut hegel_connection, action).await;
                            },
                        }
                    }
                },

                // --------------------------------------------------------------------------------
                // Check for frames coming from the amplifier
                hegel_response = hegel_connection.read_frame() => {
                    match hegel_response {
                        Ok(possible_frame) => match possible_frame {
                            Some(frame) => {
                                debug!("Got frame: {:?}", &frame);
                                let command: Result<HegelCommand, String> = frame.clone().try_into();

                                match command {
                                    Ok(cmd) => match cmd {
                                        HegelCommand::Error(e) => {
                                            send_app_log!(
                                                &self.streammagic_manager_channel,
                                                Warn,
                                                "AmplifierHandler received error from amplifier: {e}",
                                            );
                                        },
                                        HegelCommand::Mute(is_muted) => {
                                            self.amplifier_state.is_muted = is_muted;
                                            self.emit_amplifier_state().await;
                                        },
                                        HegelCommand::Power(amp_is_powered_on) => {
                                            // The amplifier sends the current power state every few seconds,
                                            // even when unchanged. We ignore *unchanged* values in terms of
                                            // state tracking, but we track them as amplifier heartbeats.

                                            // An incoming Power message might be the result of a connection test
                                            if let Some(_) = self.connection_test_start_time {
                                                self.connection_test_start_time = None;
                                                send_app_log!(
                                                    &self.streammagic_manager_channel, Info, "Amplifier connection OK"
                                                );
                                            }

                                            last_amplifier_heartbeat = SystemTime::now();

                                            match self.amplifier_state.is_powered_on {
                                                Some(handler_power) => {
                                                    if let Some(amplifier_power) = amp_is_powered_on {
                                                        if handler_power != amplifier_power {
                                                            self.amplifier_state.is_powered_on = amp_is_powered_on;
                                                            self.emit_amplifier_state().await;
                                                        }
                                                    }
                                                }
                                                None => {
                                                    self.amplifier_state.is_powered_on = amp_is_powered_on;
                                                    self.emit_amplifier_state().await;
                                                }
                                            }
                                        },
                                        HegelCommand::Source(source_id) => {
                                            self.amplifier_state.source = source_id;
                                            self.emit_amplifier_state().await;
                                        },
                                        HegelCommand::Volume(level) => {
                                            self.amplifier_state.volume = level;
                                            self.emit_amplifier_state().await;
                                        },
                                    },
                                    Err(e) => {
                                        send_app_log!(
                                            &self.streammagic_manager_channel,
                                            Warn,
                                            "AmplifierHandler did not understand amplifier frame: {:?} : {}",
                                            frame,
                                            e,
                                        );
                                    }
                                }
                            },
                            None => {
                                send_app_log!(
                                    &self.streammagic_manager_channel,
                                    Info,
                                    "AmplifierHandler connection cleanly closed by amplifier"
                                );

                                break;
                            },
                        },
                        Err(e) => {
                            send_app_log!(
                                &self.streammagic_manager_channel, Warn, "AmplifierHandler connection error: {:?}", e
                            );
                            self.set_connection_status(Disconnected).await;

                            break;
                        },
                    }
                }

                // --------------------------------------------------------------------------------
                // Perform interval checks.

                _ = interval.tick() => {
                    // If we're in test mode, check if we haven't received a test response in time
                    if let Some(test_start) = self.connection_test_start_time {
                        match SystemTime::now().duration_since(test_start) {
                            Ok(test_duration) => {
                                if test_duration.as_millis() > self.connection_test_timeout {
                                    self.connection_test_start_time = None;
                                    send_app_log!(&self.streammagic_manager_channel, Warn, "AmplifierHandler connection test failed");

                                    hegel_connection.shutdown().await;
                                    self.set_connection_status(Disconnected).await;

                                    break;
                                }
                            }
                            _ => {},
                        };
                    }

                    // Check the amplifier heartbeat
                    let now = SystemTime::now();

                    match now.duration_since(last_amplifier_heartbeat) {
                        Ok(duration) => {
                            if duration.as_millis() > self.max_heartbeat_timeout {
                                send_app_log!(
                                    &self.streammagic_manager_channel,
                                    Warn,
                                    "AmplifierHandler hasn't received amplifier heartbeat for {}ms; assuming connection lost",
                                    self.max_heartbeat_timeout
                                );

                                hegel_connection.shutdown().await;
                                self.set_connection_status(Disconnected).await;

                                break;
                            }
                        }
                        _ => {},
                    };

                    // Check whether any state needs to be requested. Ideally this will never be
                    // the case, but the amplifier sometimes won't response to state requests
                    // (perhaps if they're sent too rapidly), like what's done in
                    // request_initial_amplifier_state(). So here we check for missing data and
                    // re-request it.
                    self.request_initial_amplifier_state(&mut hegel_connection).await;
                }
            }
        }

        send_app_log!(&self.streammagic_manager_channel, Info, "AmplifierHandler has stopped");

        Ok(())
    }
}

impl HegelAmplifierHandler {
    async fn set_connection_status(&mut self, status: AmplifierHandlerConnectionStatus) {
        self.connection_status = status;

        match self
            .tx_channel
            .send(AmplifierHandlerTxChannelMsg::AmplifierHandlerConnectionStatusMsg(
                self.connection_status.clone(),
            ))
            .await
        {
            Ok(()) => {}
            Err(e) => {
                send_app_log!(
                    &self.streammagic_manager_channel,
                    Warn,
                    "Could not send AmplifierHandler connection status: {:?}",
                    e
                );
            }
        }
    }

    async fn emit_amplifier_state(&self) {
        match self
            .tx_channel
            .send(AmplifierHandlerTxChannelMsg::AmplifierStateMsg(
                self.amplifier_state.clone(),
            ))
            .await
        {
            Ok(()) => {}
            Err(e) => {
                send_app_log!(
                    &self.streammagic_manager_channel,
                    Warn,
                    "Could not send AmplifierHandler amplifier state: {:?}",
                    e
                );
            }
        }
    }

    /// Initiate a TCP connection to the amplifier.
    async fn connect_to_amplifier(&mut self) -> Result<TcpStream, PunyTunesError> {
        if let Some(host) = host_from_url(&self.device.url) {
            let connector = TcpStream::connect(format!("{host}:50001"));

            return match timeout(Duration::from_millis(self.connection_timeout), connector).await {
                Ok(Ok(stream)) => {
                    self.set_connection_status(Connected).await;

                    Ok(stream)
                }
                Ok(Err(e)) => Err(PunyTunesError::Io(e)),
                Err(_) => Err(PunyTunesError::Amplifier(format!(
                    "Connection attempt timed out after {:?}ms",
                    self.connection_timeout
                ))),
            };
        } else {
            Err(PunyTunesError::Amplifier(format!(
                "Could not determine hostname from '{}'",
                &self.device.url
            )))
        }
    }

    /// Send a request (i.e. "<cmd>.?") frame to the amplifier for the given command.
    async fn send_command_request(&mut self, hegel_connection: &mut HegelConnection, command: HegelCommand) {
        match hegel_connection.write_frame(&command.request().into()).await {
            Ok(_) => {}
            Err(e) => {
                send_app_log!(
                    &self.streammagic_manager_channel,
                    Warn,
                    "Could not send {:?} command: {e}",
                    &command
                );
            }
        }
    }

    /// Request current state from the amplifier. The amp seems to get overwhelmed if we send
    /// frames too quickly, so we do some sleeps in between.
    async fn request_initial_amplifier_state(&mut self, hegel_connection: &mut HegelConnection) {
        if self.amplifier_state.is_muted.is_none() {
            self.send_command_request(hegel_connection, HegelCommand::Mute(None))
                .await;
            sleep(Duration::from_millis(100)).await;
        }

        if self.amplifier_state.is_powered_on.is_none() {
            self.send_command_request(hegel_connection, HegelCommand::Power(None))
                .await;
            sleep(Duration::from_millis(100)).await;
        }

        if self.amplifier_state.source.is_none() {
            self.send_command_request(hegel_connection, HegelCommand::Source(None))
                .await;
            sleep(Duration::from_millis(100)).await;
        }

        if self.amplifier_state.volume.is_none() {
            self.send_command_request(hegel_connection, HegelCommand::Volume(None))
                .await;
        }
    }

    /// Send an `AmplifierAction` (as received from the `AmplifierManager`) to the amplifier.
    async fn send_action_to_amplifier(&mut self, connection: &mut HegelConnection, action: &AmplifierAction) {
        debug!("AmplifierHandler sending AmplifierAction to amplifier: {:?}", action);

        // Attempt to create a Hegel Frame from the given AmplifierAction. Frames can be created
        // from either a Command (using try_into(), which returns a Result); or from a String (e.g.
        // using the Command::toggle() method) which then needs to be wrapped in Ok().
        let hegel_frame_result: Result<Frame, String> = match action {
            AmplifierAction::MuteSet(mute) => HegelCommand::Mute(Some(*mute)).try_into(),
            AmplifierAction::MuteToggle => Ok(HegelCommand::Mute(None).toggle().into()),
            AmplifierAction::PowerSet(on_or_off) => HegelCommand::Power(Some(*on_or_off)).try_into(),
            AmplifierAction::PowerToggle => Ok(HegelCommand::Power(None).toggle().into()),
            AmplifierAction::SourceSet(source_id) => HegelCommand::Source(Some(*source_id)).try_into(),
            AmplifierAction::VolumeDown => Ok(HegelCommand::Volume(None).down().into()),
            AmplifierAction::VolumeSet(level) => HegelCommand::Volume(Some(*level)).try_into(),
            AmplifierAction::VolumeUp => Ok(HegelCommand::Volume(None).up().into()),
        };

        // Send the Hegel Frame to the amplifier
        match hegel_frame_result {
            Ok(hegel_frame) => {
                if let Err(e) = connection.write_frame(&hegel_frame).await {
                    send_app_log!(
                        &self.streammagic_manager_channel,
                        Warn,
                        "Could not send amplifier frame '{:?}' for {:?} action to amplifier: {e}",
                        &hegel_frame,
                        &action
                    );
                }
            }
            Err(e) => {
                send_app_log!(
                    &self.streammagic_manager_channel,
                    Warn,
                    "Could not generate amplifier command: {e}"
                );
            }
        }
    }

    /// Initiate a connection test.
    ///
    /// Sends a Power request. If a response is not received in time, then the connection will be
    /// considered lost.
    async fn initiate_connection_test(&mut self, hegel_connection: &mut HegelConnection) {
        match hegel_connection
            .write_frame(&HegelCommand::Power(None).request().into())
            .await
        {
            Ok(_) => {
                self.connection_test_start_time = Some(SystemTime::now());
            }
            Err(e) => {
                send_app_log!(
                    &self.streammagic_manager_channel,
                    Warn,
                    "Could not initiate amplifier connection test: {:?}",
                    e
                );
            }
        }
    }
}

// ================================================================================================
// Tests
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Invalid Frames into Commands

    #[test]
    fn test_command_from_invalid_frames() {
        let invalid_frame_data = [
            // Invalid format
            "",
            "v",
            "10",
            "v.10",
            "v10",
            "-v10",
            "-v.10.90",
            // Invalid codes
            "-f.10",
            // Invalid values
            "-p.-1",
            "-p.2",
            "-p.i",
            "-p.invalid",
            "-i.-1",
            "-i.0",
            "-i.14",
            "-i.i",
            "-i.invalid",
            "-v.-1",
            "-v.101",
            "-v.i",
            "-v.invalid",
            "-m.-1",
            "-m.2",
            "-m.i",
            "-m.invalid",
        ];

        for invalid in invalid_frame_data {
            let result: Result<HegelCommand, _> = Frame::Data(invalid.into()).try_into();

            match result {
                Ok(command) => panic!("Invalid Frame data was incorrectly converted into a Command: {invalid}"),
                Err(_) => {}
            }
        }
    }

    // Valid Frames into Commands

    #[test]
    fn test_mute_off_command_from_frame() {
        let frame = Frame::Data(String::from("-m.0"));
        let result: Result<HegelCommand, _> = frame.try_into();

        match result {
            Ok(command) => {
                if let HegelCommand::Mute(value) = command {
                    assert_eq!(value, Some(false));
                } else {
                    panic!("Expected Command::Mute, but got a different variant");
                }
            }
            Err(_) => panic!("Could not convert Frame into a Command::Mute"),
        }
    }

    #[test]
    fn test_mute_on_command_from_frame() {
        let frame = Frame::Data(String::from("-m.1"));
        let result: Result<HegelCommand, _> = frame.try_into();

        match result {
            Ok(command) => {
                if let HegelCommand::Mute(value) = command {
                    assert_eq!(value, Some(true));
                } else {
                    panic!("Expected Command::Mute, but got a different variant");
                }
            }
            Err(_) => panic!("Could not convert Frame into a Command::Mute"),
        }
    }

    #[test]
    fn test_power_off_command_from_frame() {
        let frame = Frame::Data(String::from("-p.0"));
        let result: Result<HegelCommand, _> = frame.try_into();

        match result {
            Ok(command) => {
                if let HegelCommand::Power(value) = command {
                    assert_eq!(value, Some(false));
                } else {
                    panic!("Expected Command::Power, but got a different variant");
                }
            }
            Err(_) => panic!("Could not convert Frame into a Command::Power"),
        }
    }

    #[test]
    fn test_power_on_command_from_frame() {
        let frame = Frame::Data(String::from("-p.1"));
        let result: Result<HegelCommand, _> = frame.try_into();

        match result {
            Ok(command) => {
                if let HegelCommand::Power(value) = command {
                    assert_eq!(value, Some(true));
                } else {
                    panic!("Expected Command::Power, but got a different variant");
                }
            }
            Err(_) => panic!("Could not convert Frame into a Command::Power"),
        }
    }

    #[test]
    fn test_source_command_from_frame() {
        let frame = Frame::Data(String::from("-i.5"));
        let result: Result<HegelCommand, _> = frame.try_into();

        match result {
            Ok(command) => {
                if let HegelCommand::Source(value) = command {
                    assert_eq!(value, Some(5));
                } else {
                    panic!("Expected Command::Source, but got a different variant");
                }
            }
            Err(_) => panic!("Could not convert Frame into a Command::Source"),
        }
    }

    #[test]
    fn test_volume_command_from_frame() {
        let frame = Frame::Data(String::from("-v.10"));
        let result: Result<HegelCommand, _> = frame.try_into();

        match result {
            Ok(command) => {
                if let HegelCommand::Volume(value) = command {
                    assert_eq!(value, Some(10));
                } else {
                    panic!("Expected Command::Volume, but got a different variant");
                }
            }
            Err(_) => panic!("Could not convert Frame into a Command::Volume"),
        }
    }

    #[test]
    fn test_error_command_from_frame() {
        let frame = Frame::Data(String::from("-e.2"));
        let result: Result<HegelCommand, _> = frame.try_into();

        match result {
            Ok(command) => {
                if let HegelCommand::Error(value) = command {
                    assert_eq!(value, "2");
                } else {
                    panic!("Expected Command::Error, but got a different variant");
                }
            }
            Err(_) => panic!("Could not convert Frame into a Command::Error"),
        }
    }

    // Valid Commands into strings

    #[test]
    fn test_mute_off_frame_from_command() {
        let command = HegelCommand::Mute(Some(false));
        let result: Result<Frame, _> = command.try_into();

        match result {
            Ok(frame) => {
                if let Frame::Data(frame_string) = frame {
                    assert_eq!(frame_string, "-m.0\r");
                }
            }
            Err(e) => panic!("Could not convert Command::Mute (off) into Frame: {e}"),
        }
    }

    #[test]
    fn test_mute_on_frame_from_command() {
        let command = HegelCommand::Mute(Some(true));
        let result: Result<Frame, _> = command.try_into();

        match result {
            Ok(frame) => {
                if let Frame::Data(frame_string) = frame {
                    assert_eq!(frame_string, "-m.1\r");
                }
            }
            Err(e) => panic!("Could not convert Command::Mute (on) into Frame: {e}"),
        }
    }

    #[test]
    fn test_power_off_frame_from_command() {
        let command = HegelCommand::Power(Some(false));
        let result: Result<Frame, _> = command.try_into();

        match result {
            Ok(frame) => {
                if let Frame::Data(frame_string) = frame {
                    assert_eq!(frame_string, "-p.0\r");
                }
            }
            Err(e) => panic!("Could not convert Command::Power (off) into Frame: {e}"),
        }
    }

    #[test]
    fn test_power_on_frame_from_command() {
        let command = HegelCommand::Power(Some(true));
        let result: Result<Frame, _> = command.try_into();

        match result {
            Ok(frame) => {
                if let Frame::Data(frame_string) = frame {
                    assert_eq!(frame_string, "-p.1\r");
                }
            }
            Err(e) => panic!("Could not convert Command::Power (on) into Frame: {e}"),
        }
    }

    #[test]
    fn test_source_frame_from_command() {
        let command = HegelCommand::Source(Some(5));
        let result: Result<Frame, _> = command.try_into();

        match result {
            Ok(frame) => {
                if let Frame::Data(frame_string) = frame {
                    assert_eq!(frame_string, "-i.5\r");
                }
            }
            Err(e) => panic!("Could not convert Command::Source into Frame: {e}"),
        }
    }

    #[test]
    fn test_volume_frame_from_command() {
        let command = HegelCommand::Volume(Some(25));
        let result: Result<Frame, _> = command.try_into();

        match result {
            Ok(frame) => {
                if let Frame::Data(frame_string) = frame {
                    assert_eq!(frame_string, "-v.25\r");
                }
            }
            Err(e) => panic!("Could not convert Command::Volume into Frame: {e}"),
        }
    }
}
