use std::time::SystemTime;

use log::{info, Level::{Debug, Error, Info, Warn}};
use serde;
use tauri::{AppHandle, Manager};
use tauri_plugin_store::JsonValue;
use tokio::select;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};
use ts_rs::TS;

use discovery::{discover_streamers, StreamMagicDevice};
use payloads::{
    RequestUpdates, StreamerSystemPower, StreamerZonePosition, StreamMagicMessage, SystemInfo, SystemPower,
    ZonePosition,
};
use websocket_client::{
    WebSocketClient, WebSocketClientAction, WebSocketClientStatus, WSClientRxChannelMsg, WSClientTxChannelMsg,
};

use crate::errors::PunyTunesError;
use crate::messaging::{AppLog, AppMessageType};
use crate::persisted_state::KEY_LAST_CONNECTED_HOST;
use crate::state::PersistedBackendState;
use crate::streammagic_manager::payloads::{
    PlayControl, Presets, QueueInfo, QueueList, RecallPreset, StreamerPresets, StreamerQueueInfo, StreamerQueueList,
    StreamerSystemInfo, StreamerSystemSources, StreamerZoneNowPlaying, StreamerZonePlayState, StreamerZoneState,
    SystemSources, TransportToggleState, ZoneNowPlaying, ZonePlayState, ZoneState,
};
use crate::traits::CustomEmitters;
use crate::utils::host_from_url;

mod discovery;
mod payloads;
mod websocket_client;

// ================================================================================================
// TODO: StreamMagicManager and WebSocketClient both handle a bunch of "what's happening right
//  now" state, which is scattered throughout their respective implementations. As a result, it's
//  not easy to understand exactly what can and should be done when certain messages are received,
//  or when other actions happen. Consider a state machine approach to make this easier to reason
//  about, and (hopefully) more robust/predictable. Consider: https://crates.io/crates/rust-fsm
// ------------------------------------------------------------------------------------------------

/// Channel actions which can be sent from the invoker (the main app) to the StreamMagicManager
#[derive(Clone, Debug, serde::Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/StreamMagicManagerAction.ts")]
pub enum StreamMagicManagerAction {
    // String is the UDN of the device to activate
    ActivateUdn(String),
    // TODO: Deprecate (replaced by ActivateUdn). String is IP of streamer
    ConnectToStreamer(String),
    // Deactivate the currently-active device
    Deactivate,
    // TODO: Deprecate (replaced by Deactivate).
    DisconnectFromStreamer,
    // bool is whether to auto-activate the first discovered device
    Discover(bool),
    EmitAppLog(AppLog),
    HandleClientError,
    OnUIReady,
    ProcessDiscoveredDevice(StreamMagicDevice),
    SetIsDiscovering(bool),
    ShutDown,
    // bool is whether to delete the last-connected host from persisted state
    StopWebSocketClient(bool),
    TestConnection,
}

/// Actions which can be performed on the Streamer.
#[derive(Clone, Debug, serde::Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/StreamerAction.ts")]
pub enum StreamerAction {
    MuteSet(bool),
    NextTrack,
    Pause,
    Play,
    PlayPresetId(i32),
    PlayQueueId(i32),
    PowerOn,
    PowerStandby,
    PowerToggle,
    PreviousTrack,
    Seek(i32),
    SetRepeat(TransportToggleState),
    SetShuffle(TransportToggleState),
    SetSourceId(String),
    Stop,
    TogglePlayback,
    VolumePercentSet(u8),
    VolumeStepChange(i8),
    VolumeStepSet(u8),
}

/// All StreamMagicManager state, excluding Streamer payloads, which might be of interest to the UI.
#[derive(Clone, Debug, serde::Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/StreamMagicManagerStateMsg.ts")]
pub struct StreamMagicManagerStateMsg {
    devices: Vec<StreamMagicDevice>,
    is_activating: bool,
    is_discovering: bool,
    is_testing_connection: bool,
    websocket_client_status: WebSocketClientStatus,
}

// ------------------------------------------------------------------------------------------------

pub struct StreamMagicManagerChannel(pub mpsc::Sender<StreamMagicManagerChannelMsg>);

/// Channel messages which can be sent from the invoker (StreamMagicManager) to the WebSocketClient
#[derive(Clone, Debug)]
pub enum StreamMagicManagerChannelMsg {
    StreamerActionMsg(StreamerAction),
    StreamMagicManagerActionMsg(StreamMagicManagerAction),
}

/// StreamMagicManager manages all StreamMagic-related concerns.
///
/// StreamMagicManager expects to be running for the duration of the application. It might
/// encompass multiple Streamer connections over time.
///
/// Responsibilities:
///   * Performs discovery and retains a vector of discovered devices.
///   * Performs actions on request:
///       * `StreamerAction`: Actions to perform on the Streamer (play, pause, etc).
///       * `StreamMagicManagerAction`: General actions (discovery, Streamer activation, etc).
///   * Manages WebSocketClient connections to the Streamer.
///       * Registers for Streamer updates.
///       * Converts Streamer actions to plain text messages for sending to the Streamer.
///       * Receives plain text messages from the Streamer, parses them, stores them, and emits
///         them to the UI.
pub struct StreamMagicManager {
    app_handle: AppHandle,
    // Receive commands from the application or StreamMagicManager itself
    rx_channel: mpsc::Receiver<StreamMagicManagerChannelMsg>,
    // Send commands to itself (for use by spawned tasks such as discovery, which won't
    // have access to self)
    tx_channel: mpsc::Sender<StreamMagicManagerChannelMsg>,

    // Buffer any AppLog sent to the UI before the UI is ready, so they can be sent all at once
    // when the UI is ready for them. This buffer should then be unnecessary for the remainder of
    // the session.
    buffered_app_logs: Vec<AppLog>,
    // Discovered UPnP MediaRenderer devices from Cambridge Audio
    devices: Vec<StreamMagicDevice>,
    // Keep track of the device last successfully activated. Can help with reconnects. This is
    // tracked separately from the active device in "devices" as we want it to persist separate
    // from whatever device currently happens to be active (which is none after a disconnect).
    last_active_device: Option<StreamMagicDevice>,
    // The StreamMagicManager can be activating a StreamMagicDevice (once discovery has completed),
    // *or* activating the previously-persisted streamer by IP address (i.e. not yet a known
    // StreamMagicDevice). Individual StreamMagicDevices also have an is_activating flag.
    is_activating: bool,
    is_discovering: bool,
    is_testing_connection: bool,
    count_of_disconnects_while_testing: usize,
    activation_start: Option<SystemTime>,
    activation_attempts: usize,
    activation_timeout: u128,
    ui_ready: bool,
    ws_client_join_handle: Option<tauri::async_runtime::JoinHandle<Result<(), PunyTunesError>>>,
    ws_client_receive_channel: Option<mpsc::Receiver<WSClientTxChannelMsg>>,
    ws_client_send_channel: Option<mpsc::Sender<WSClientRxChannelMsg>>,
    ws_client_status: WebSocketClientStatus,

    // Streamer payloads. Only the last (most recent) payload received is retained.
    presets: Option<StreamerPresets>,
    queue_info: Option<StreamerQueueInfo>,
    queue_list: Option<StreamerQueueList>,
    system_info: Option<StreamerSystemInfo>,
    system_power: Option<StreamerSystemPower>,
    system_sources: Option<StreamerSystemSources>,
    zone_now_playing: Option<StreamerZoneNowPlaying>,
    zone_play_state: Option<StreamerZonePlayState>,
    zone_position: Option<StreamerZonePosition>,
    zone_state: Option<StreamerZoneState>,
}

impl StreamMagicManager {
    pub fn new(
        app_handle: AppHandle,
        rx_channel: mpsc::Receiver<StreamMagicManagerChannelMsg>,
        tx_channel: mpsc::Sender<StreamMagicManagerChannelMsg>,
    ) -> StreamMagicManager {
        StreamMagicManager {
            app_handle,
            rx_channel,
            tx_channel,

            buffered_app_logs: Vec::new(),
            devices: Vec::new(),
            last_active_device: None,
            is_activating: false,
            is_discovering: false,
            is_testing_connection: false,
            count_of_disconnects_while_testing: 0,
            activation_start: None,
            activation_attempts: 0,
            activation_timeout: 15000,
            ui_ready: false,
            ws_client_join_handle: None,
            ws_client_receive_channel: None,
            ws_client_send_channel: None,
            ws_client_status: WebSocketClientStatus::Disconnected(Default::default()),

            presets: None,
            queue_info: None,
            queue_list: None,
            system_info: None,
            system_power: None,
            system_sources: None,
            zone_now_playing: None,
            zone_play_state: None,
            zone_position: None,
            zone_state: None,
        }
    }

    // --------------------------------------------------------------------------------------------
    // Message handling

    async fn emit_manager_state(&self) {
        self.app_handle
            .emit_app_message(
                AppMessageType::StreamMagicManagerState,
                StreamMagicManagerStateMsg {
                    devices: self.devices.clone(),
                    is_activating: self.is_activating,
                    is_discovering: self.is_discovering,
                    is_testing_connection: self.is_testing_connection,
                    websocket_client_status: self.ws_client_status.clone(),
                },
            )
            .await;
    }

    async fn emit_streammagic_payload(&self, msg_type: AppMessageType) {
        match msg_type {
            AppMessageType::StreamerQueueList => self.app_handle.emit_app_message(msg_type, &self.queue_list).await,
            AppMessageType::StreamerPresets => self.app_handle.emit_app_message(msg_type, &self.presets).await,
            AppMessageType::StreamerSystemInfo => self.app_handle.emit_app_message(msg_type, &self.system_info).await,
            AppMessageType::StreamerSystemPower => self.app_handle.emit_app_message(msg_type, &self.system_power).await,
            AppMessageType::StreamerSystemSources => {
                self.app_handle.emit_app_message(msg_type, &self.system_sources).await
            }
            AppMessageType::StreamerZoneNowPlaying => {
                self.app_handle.emit_app_message(msg_type, &self.zone_now_playing).await
            }
            AppMessageType::StreamerZonePlayState => {
                self.app_handle.emit_app_message(msg_type, &self.zone_play_state).await
            }
            AppMessageType::StreamerZonePosition => {
                self.app_handle.emit_app_message(msg_type, &self.zone_position).await
            }
            AppMessageType::StreamerZoneState => self.app_handle.emit_app_message(msg_type, &self.zone_state).await,
            _ => {}
        }
    }

    async fn emit_streammagic_payloads(&self) {
        self.emit_streammagic_payload(AppMessageType::StreamerQueueList).await;
        self.emit_streammagic_payload(AppMessageType::StreamerPresets).await;
        self.emit_streammagic_payload(AppMessageType::StreamerSystemInfo).await;
        self.emit_streammagic_payload(AppMessageType::StreamerSystemPower).await;
        self.emit_streammagic_payload(AppMessageType::StreamerSystemSources)
            .await;
        self.emit_streammagic_payload(AppMessageType::StreamerZoneNowPlaying)
            .await;
        self.emit_streammagic_payload(AppMessageType::StreamerZonePlayState)
            .await;
        self.emit_streammagic_payload(AppMessageType::StreamerZonePosition)
            .await;
        self.emit_streammagic_payload(AppMessageType::StreamerZoneState).await;
    }

    // --------------------------------------------------------------------------------------------
    // State handling

    fn get_persisted_backend_state_value(&self, key: &str) -> Option<JsonValue> {
        let state_ref = self.app_handle.state::<PersistedBackendState>();
        let state = &state_ref.0;
        let guard = state.lock().unwrap();

        if let Some(value_ref) = guard.get(key) {
            Some(value_ref.to_owned())
        } else {
            None
        }
    }

    async fn on_state_changed(&self) {
        self.emit_manager_state().await;
    }

    async fn clear_devices(&mut self) {
        self.devices = Vec::new();
        self.on_state_changed().await;
    }

    async fn add_device(&mut self, device: StreamMagicDevice) {
        self.devices.push(device);

        match self.ws_client_status.clone() {
            WebSocketClientStatus::Connected(details) => {
                self.set_device_is_active_from_url(&details.url).await;
            }
            _ => {}
        }

        self.on_state_changed().await;
    }

    async fn reset_websocket_related_state(&mut self) {
        self.set_websocket_client_status(&WebSocketClientStatus::Disconnected(Default::default())).await;
        self.set_is_testing_connection(false).await;
        self.set_is_activating(false).await;
    }

    async fn set_is_activating(&mut self, is_activating: bool) {
        if is_activating {
            if !self.is_activating {
                self.activation_start = Some(SystemTime::now());
                self.activation_attempts = 0;
                self.is_activating = true;
            }
        } else {
            self.activation_start = None;
            self.activation_attempts = 0;
            self.is_activating = false;
        }

        self.on_state_changed().await;
    }

    async fn set_is_discovering(&mut self, is_discovering: bool) {
        self.is_discovering = is_discovering;
        self.on_state_changed().await;
    }

    async fn set_is_testing_connection(&mut self, is_testing_connection: bool) {
        send_app_log!(
            &self.tx_channel, Debug, "Connection test {}",
            if is_testing_connection { "starting" } else { "completed" }
        );

        self.is_testing_connection = is_testing_connection;
        self.on_state_changed().await;
    }

    async fn set_websocket_client_status(&mut self, status: &WebSocketClientStatus) {
        self.ws_client_status = status.clone();
        self.on_state_changed().await;
    }

    // --------------------------------------------------------------------------------------------

    async fn set_presets(&mut self, presets: StreamerPresets) {
        self.presets = Some(presets);
        self.emit_streammagic_payload(AppMessageType::StreamerPresets).await;
    }

    async fn set_queue_info(&mut self, queue_info: StreamerQueueInfo) {
        self.queue_info = Some(queue_info);

        // NOTE: queue_info is not emitted to the UI. It's only for internal use, to trigger
        //  full QueueList retrievals on queue updates.
    }

    async fn set_queue_list(&mut self, queue_list: StreamerQueueList) {
        self.queue_list = Some(queue_list);
        self.emit_streammagic_payload(AppMessageType::StreamerQueueList).await;
    }

    async fn set_system_info(&mut self, info: StreamerSystemInfo) {
        self.system_info = Some(info);
        self.emit_streammagic_payload(AppMessageType::StreamerSystemInfo).await;
    }

    async fn set_system_power(&mut self, power: StreamerSystemPower) {
        self.system_power = Some(power);
        self.emit_streammagic_payload(AppMessageType::StreamerSystemPower).await;
    }

    async fn set_system_sources(&mut self, sources: StreamerSystemSources) {
        self.system_sources = Some(sources);
        self.emit_streammagic_payload(AppMessageType::StreamerSystemSources)
            .await;
    }

    async fn set_zone_now_playing(&mut self, now_playing: StreamerZoneNowPlaying) {
        self.zone_now_playing = Some(now_playing);
        self.emit_streammagic_payload(AppMessageType::StreamerZoneNowPlaying)
            .await;
    }

    async fn set_zone_play_state(&mut self, play_state: StreamerZonePlayState) {
        self.zone_play_state = Some(play_state);
        self.emit_streammagic_payload(AppMessageType::StreamerZonePlayState)
            .await;
    }

    async fn set_zone_position(&mut self, position: StreamerZonePosition) {
        self.zone_position = Some(position);
        self.emit_streammagic_payload(AppMessageType::StreamerZonePosition)
            .await;
    }

    async fn set_zone_state(&mut self, state: StreamerZoneState) {
        self.zone_state = Some(state);
        self.emit_streammagic_payload(AppMessageType::StreamerZoneState).await;
    }

    // --------------------------------------------------------------------------------------------
    // Streamer device discovery, activation, and deactivation
    //
    // Note: "Activating" means connecting to the device. Only one device can be active at a time.
    //  If there is no current WebSocketClient connection to a streamer, then none of the devices
    //  should be marked as active. The active device is the StreamMagicDevice where
    //  is_active=true.

    async fn do_discovery(&mut self, activate_discovered_device: bool) {
        // Will also optionally trigger a device activation.

        send_app_log!(&self.tx_channel, Info, "Initiating discovery");

        self.set_is_discovering(true).await;
        self.clear_devices().await;

        let channel_clone = self.tx_channel.clone();

        tauri::async_runtime::spawn(async move {
            match discover_streamers(&channel_clone, activate_discovered_device).await {
                Ok(()) => send_app_log!(&channel_clone, Info, "Streamer UPnP discovery complete"),
                Err(e) => {
                    send_app_log!(&channel_clone, Error, "Streamer discovery failed with error: {:?}", e);
                    send_manager_action!(&channel_clone, StreamMagicManagerAction::SetIsDiscovering(false));
                }
            };
        });
    }

    async fn set_all_device_activation_false(&mut self) {
        for device in &mut self.devices {
            device.is_active = false;
            device.is_activating = false;
        }

        self.emit_manager_state().await;
    }

    async fn set_device_is_active_from_url(&mut self, url: &str) {
        self.set_all_device_activation_false().await;

        let mut activated_device: Option<&StreamMagicDevice> = None;

        // The provided URL is probably a ws:// url, whereas the device URL is probably an http://
        // URL to the description.xml. Regardless, we extract just the host from both and compare
        // just that.
        if let Some(given_host) = host_from_url(url) {
            for device in &mut self.devices {
                if let Some(device_host) = host_from_url(&device.url) {
                    if device_host == given_host {
                        device.is_active = true;
                        activated_device = Some(device);

                        break;
                    }
                }
            }
        }

        if let Some(device) = activated_device {
            send_app_log!(&self.tx_channel, Info, "Marked device {} as active", device);
            self.last_active_device = Some((*device).clone());
        }

        self.emit_manager_state().await;
    }

    async fn activate_device(&mut self, udn: &str) {
        self.set_is_activating(true).await;

        let now = SystemTime::now();

        // Check whether we've been attempting to activate for too long
        if let Some(activation_start) = self.activation_start {
            if let Ok(activating_duration) = now.duration_since(activation_start) {
                if activating_duration.as_millis() > self.activation_timeout {
                    send_app_log!(
                        &self.tx_channel, Error,
                        "Giving up attempting re-activation after {}ms (made {} attempts)",
                        self.activation_timeout,
                        self.activation_attempts
                    );

                    self.set_is_activating(false).await;

                    return;
                }
            }
        }

        // Exponential backoff based on activation attempt count. Max out at 1500ms.
        let base_delay = Duration::from_millis(100);
        let max_delay = Duration::from_millis(1500);

        let delay = match self.activation_attempts {
            0 => Duration::from_millis(0),
            _ => max_delay.min(base_delay * 2u32.pow(self.activation_attempts as u32)),
        };

        self.activation_attempts += 1;

        send_app_log!(
            &self.tx_channel, Info,
            "Activating device (attempt {} after {}ms delay), with UDN: {}",
            self.activation_attempts, delay.as_millis(), &udn
        );

        sleep(delay).await;

        self.set_all_device_activation_false().await;

        let matching_device = self.devices.iter_mut().find(|device| &*device.udn == udn);

        // And open a WebSocketClient connection to the device. The device will be marked as active
        // once the Connected WebSocketClient status is received.
        if let Some(device) = matching_device {
            if let Some(device_url) = host_from_url(&*device.url) {
                device.is_activating = true;
                self.on_state_changed().await;
                self.start_websocket_client(&device_url).await;
            } else {
                send_app_log!(&self.tx_channel, Error, "Could not determine device URL; not activating");
                self.set_is_activating(false).await;
            }
        }

        // Ensure the UI knows about the new device state.
        self.emit_manager_state().await;
    }

    async fn deactivate_active_device(&mut self) {
        send_app_log!(&self.tx_channel, Info, "Deactivating currently-active device");

        self.set_all_device_activation_false().await;
        self.stop_websocket_client().await;

        send_app_log!(&self.tx_channel, Info, "Deactivation complete");

        self.emit_manager_state().await;
    }

    // --------------------------------------------------------------------------------------------
    // WebSocketClient and StreamMagic device message handling

    async fn start_websocket_client(&mut self, host: &str) {
        send_app_log!(&self.tx_channel, Info, "Initiating a new WebSocketClient connection to: {host}");

        self.stop_websocket_client().await;

        let (ws_cmd_channel_tx, ws_cmd_channel_rx) = mpsc::channel(32);
        let (ws_msg_channel_tx, ws_msg_channel_rx) = mpsc::channel(32);

        // Store WSClient send channel so we can tell it to stop later
        self.ws_client_send_channel = Some(ws_cmd_channel_tx);
        self.ws_client_receive_channel = Some(ws_msg_channel_rx);

        let streamer_url = format!("ws://{}:80/smoip", host);
        let websocket_client_manager_channel = self.tx_channel.clone();
        let channel_clone = self.tx_channel.clone();

        self.ws_client_join_handle = Some(tauri::async_runtime::spawn(async move {
            let mut ws_client = WebSocketClient::new(
                &streamer_url,
                websocket_client_manager_channel,
                ws_cmd_channel_rx,
                ws_msg_channel_tx,
                false,
            );

            let ws_result = ws_client.run().await;

            if let Err(err) = &ws_result {
                send_app_log!(&channel_clone, Warn, "StreamMagicManager detected client error: {:?}", &err);
                send_manager_action!(&channel_clone, StreamMagicManagerAction::HandleClientError);
            }

            ws_result
        }));
    }

    async fn send_shutdown_request_to_websocket_client(&mut self) {
        send_app_log!(&self.tx_channel, Info, "Sending ShutDown request to WebSocketClient");

        match &self.ws_client_send_channel {
            Some(ws_client_channel) => {
                match ws_client_channel
                    .send(WSClientRxChannelMsg::WebSocketClientActionMsg(
                        WebSocketClientAction::ShutDown,
                    ))
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        send_app_log!(&self.tx_channel, Warn,
                            "Could not send command {:?} to WebSocketClient: {:?}",
                            WebSocketClientAction::ShutDown, e
                        );
                    }
                }
            }
            None => {
                send_app_log!(&self.tx_channel, Warn, "No WebSocketClient send channel found");
            }
        }
    }

    async fn send_test_connection_request_to_websocket_client(&mut self) {
        send_app_log!(&self.tx_channel, Debug, "Sending TestConnection request to WebSocketClient");
        self.set_is_testing_connection(true).await;
        self.count_of_disconnects_while_testing = 0;

        match &self.ws_client_send_channel {
            Some(ws_client_channel) => {
                match ws_client_channel
                    .send(WSClientRxChannelMsg::WebSocketClientActionMsg(
                        WebSocketClientAction::TestConnection,
                    ))
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        send_app_log!(&self.tx_channel, Warn,
                            "Could not send command {:?} to WebSocketClient: {:?}",
                            WebSocketClientAction::TestConnection, e
                        );
                        self.set_is_testing_connection(false).await;
                    }
                }
            }
            None => {
                send_app_log!(&self.tx_channel, Warn, "No WebSocketClient send channel found");
                self.set_is_testing_connection(false).await;
            }
        }
    }

    async fn stop_websocket_client(&mut self) {
        if let Some(handle) = self.ws_client_join_handle.take() {
            send_app_log!(&self.tx_channel, Info, "Stopping existing WebSocketClient task");
            self.send_shutdown_request_to_websocket_client().await;

            match handle.await {
                Ok(ws_client_result) => match ws_client_result {
                    Ok(_) => {
                        send_app_log!(&self.tx_channel, Info, "WebSocketClient successfully stopped");
                    }
                    Err(e) => {
                        send_app_log!(&self.tx_channel, Warn, "WebSocketClient stopped with error: {:?}", e);
                    }
                },
                Err(e) => {
                    send_app_log!(&self.tx_channel, Warn, "Could not stop WebSocketClient: {:?}", e);
                }
            }

            self.ws_client_join_handle = None;
        }
    }

    async fn send_websocket_message(&mut self, msg: &str) {
        if let Some(send_channel) = &self.ws_client_send_channel {
            match send_channel.send(WSClientRxChannelMsg::DataMsg(msg.to_string())).await {
                Ok(()) => {}
                Err(e) => {
                    send_app_log!(&self.tx_channel, Warn, "Could not send message to WebSocketClient channel: {:?}", e);
                }
            }
        }
    }

    async fn register_for_streammagic_updates(&mut self) {
        self.send_websocket_message(&Presets::request_updates_msg()).await;
        self.send_websocket_message(&QueueInfo::request_updates_msg()).await;
        self.send_websocket_message(&SystemInfo::request_updates_msg()).await;
        self.send_websocket_message(&SystemPower::request_updates_msg()).await;
        self.send_websocket_message(&SystemSources::request_updates_msg()).await;
        self.send_websocket_message(&ZoneNowPlaying::request_updates_msg())
            .await;
        self.send_websocket_message(&ZonePlayState::request_updates_msg()).await;
        self.send_websocket_message(&ZonePosition::request_updates_msg()).await;
        self.send_websocket_message(&ZoneState::request_updates_msg()).await;
    }

    async fn process_streammagic_message(&mut self, message: &str) -> Result<(), PunyTunesError> {
        match serde_json::from_str::<StreamMagicMessage>(message) {
            Ok(message) => match message.path.as_ref() {
                "/queue/info" => match serde_json::from_value::<StreamerQueueInfo>(message.params) {
                    Ok(payload) => {
                        self.set_queue_info(payload).await;

                        // A QueueInfo update is used to trigger a retrieval of the presumably
                        // updated QueueList.
                        self.send_websocket_message(&QueueList::request_state_msg()).await;
                    }
                    Err(e) => {
                        send_app_log!(&self.tx_channel, Warn, "Could not decode {} payload: {:?}", &message.path, e);
                    }
                },
                "/queue/list" => match serde_json::from_value::<StreamerQueueList>(message.params) {
                    Ok(payload) => self.set_queue_list(payload).await,
                    Err(e) => {
                        send_app_log!(&self.tx_channel, Warn, "Could not decode {} payload: {:?}", &message.path, e);
                    }
                },
                "/presets/list" => match serde_json::from_value::<StreamerPresets>(message.params) {
                    Ok(payload) => self.set_presets(payload).await,
                    Err(e) => {
                        send_app_log!(&self.tx_channel, Warn, "Could not decode {} payload: {:?}", &message.path, e);
                    }
                },
                "/system/info" => match serde_json::from_value::<StreamerSystemInfo>(message.params) {
                    Ok(payload) => self.set_system_info(payload).await,
                    Err(e) => {
                        send_app_log!(&self.tx_channel, Warn, "Could not decode {} payload: {:?}", &message.path, e);
                    }
                },
                "/system/power" => match serde_json::from_value::<StreamerSystemPower>(message.params) {
                    Ok(payload) => self.set_system_power(payload).await,
                    Err(e) => {
                        send_app_log!(&self.tx_channel, Warn, "Could not decode {} payload: {:?}", &message.path, e);
                    }
                },
                "/system/sources" => match serde_json::from_value::<StreamerSystemSources>(message.params) {
                    Ok(payload) => self.set_system_sources(payload).await,
                    Err(e) => {
                        send_app_log!(&self.tx_channel, Warn, "Could not decode {} payload: {:?}", &message.path, e);
                    }
                },
                "/zone/now_playing" => match serde_json::from_value::<StreamerZoneNowPlaying>(message.params) {
                    Ok(payload) => self.set_zone_now_playing(payload).await,
                    Err(e) => {
                        send_app_log!(&self.tx_channel, Warn, "Could not decode {} payload: {:?}", &message.path, e);
                    }
                },
                "/zone/play_state" => match serde_json::from_value::<StreamerZonePlayState>(message.params) {
                    Ok(payload) => self.set_zone_play_state(payload).await,
                    Err(e) => {
                        send_app_log!(&self.tx_channel, Warn, "Could not decode {} payload: {:?}", &message.path, e);
                    }
                },
                "/zone/play_state/position" => match serde_json::from_value::<StreamerZonePosition>(message.params) {
                    Ok(payload) => self.set_zone_position(payload).await,
                    Err(e) => {
                        // Position information is sometimes unavailable, which manifests here as
                        // a missing "data" field in the message payload. We choose to ignore this
                        // as it's not a major issue. Alternatively we could instead set a None
                        // state for ZonePosition::position, but that doesn't seem useful (at the
                        // time of writing this comment).
                        let error_str = format!("{:?}", e);
                        if !error_str.contains("missing field `data`") {
                            send_app_log!(&self.tx_channel, Warn, "Could not decode {} payload: {:?}", &message.path, e);
                        }
                    }
                },
                "/zone/state" => match serde_json::from_value::<StreamerZoneState>(message.params) {
                    Ok(payload) => self.set_zone_state(payload).await,
                    Err(e) => {
                        send_app_log!(&self.tx_channel, Warn, "Could not decode {} payload: {:?}", &message.path, e);
                    }
                },
                // We expect to receive a play_control message whenever we send a streamer action
                // like "NextTrack"; but we don't need to do anything with it. Similarly we want
                // to ignore /zone/recall_preset messages.
                "/zone/play_control" | "/zone/recall_preset" => {}
                unmatched => {
                    // We shouldn't see this, if we're appropriately acting on any incoming message
                    // path that we've actively subscribed to or that we're triggering as the result
                    // of a streamer action.
                    send_app_log!(&self.tx_channel, Warn, "Received unhandled StreamMagic message path: {unmatched}");
                }
            },
            Err(e) => {
                send_app_log!(
                    &self.tx_channel, Warn,
                    "Could not deserialize StreamMagic message; error: {:?} :: message: {}",
                    e, &message
                );
            }
        }

        Ok(())
    }

    // --------------------------------------------------------------------------------------------

    async fn initialize(&mut self) {
        // Auto-connect to the last-known host

        let last_connected_host_maybe = self.get_persisted_backend_state_value(KEY_LAST_CONNECTED_HOST);
        let mut activate_discovered_device = false;

        if let Some(last_connected_host) = last_connected_host_maybe {
            send_app_log!(
                &self.tx_channel,
                Info,
                "Using stored streamer host to initialize connection: {last_connected_host}"
            );

            if let Some(host) = last_connected_host.as_str() {
                let host_owned = host.to_owned();

                // Treat this like an activation. We can't invoke activate_device() at this point
                // as discovery has not yet completed, and we don't want to wait for discovery to
                // complete as we'd like to connect ASAP if the last device is still on the network.
                self.set_is_activating(true).await;
                self.start_websocket_client(&host_owned.clone()).await;
            }
        } else {
            activate_discovered_device = true; // Only auto-activate if there's no persisted host
        }

        // TODO: Handle case where persisted host no longer exists (connection will fail, and it
        //  would be nice to instead auto-connect to the first discovered device).
        self.do_discovery(activate_discovered_device).await;
    }

    // --------------------------------------------------------------------------------------------

    pub async fn run(&mut self) -> Result<(), PunyTunesError> {
        self.initialize().await;

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
                            StreamMagicManagerChannelMsg::StreamMagicManagerActionMsg(manager_action) => {
                                match manager_action {
                                    StreamMagicManagerAction::ActivateUdn(udn) => {
                                        self.activate_device(&udn).await;
                                    },
                                    // TODO: Deprecate (replaced by ActivateUdn)
                                    StreamMagicManagerAction::ConnectToStreamer(host) => {
                                        self.start_websocket_client(&host).await;
                                    },
                                    StreamMagicManagerAction::Deactivate => {
                                        self.deactivate_active_device().await;
                                    },
                                    // TODO: Deprecate (replaced by Deactivate)
                                    StreamMagicManagerAction::DisconnectFromStreamer => {
                                        self.stop_websocket_client().await;
                                    },
                                    StreamMagicManagerAction::Discover(activate) => {
                                        self.do_discovery(activate).await;
                                    },
                                    StreamMagicManagerAction::EmitAppLog(app_log) => {
                                        if self.ui_ready {
                                            // We unpack the AppLog to pass it into emit_app_log(),
                                            // where it gets repackaged back into an AppLog. This
                                            // feels a little weird but we want to use emit_app_log
                                            // (and not emit_app_message) as we want the standard
                                            // Rust log entry to be emitted as well.
                                            //
                                            // The same happens in the handler below for
                                            // StreamMagicManagerAction::OnUIReady.
                                            self.app_handle.emit_app_log_with_target(
                                                module_path!(),
                                                app_log.level.into(),
                                                &app_log.message
                                            ).await;
                                        } else {
                                            // Log here too, just in case the UI is never ready and
                                            // the buffered logs are never emitted.
                                            info!("Buffering log: {:?}", app_log);

                                            self.buffered_app_logs.push(app_log);
                                        }
                                    },
                                    StreamMagicManagerAction::HandleClientError => {
                                        if self.is_testing_connection {
                                            send_app_log!(&self.tx_channel, Warn, "Test resulted in client error");
                                            self.set_is_testing_connection(false).await;
                                        }

                                        // Consider all WebSocketClient-sourced PunyTunesErrors as potentially
                                        // recoverable via a re-activation.
                                        if let Some(last_device) = &self.last_active_device {
                                            send_app_log!(&self.tx_channel, Info,"Client error; retrying last-active device");
                                            let _ = &self.activate_device(&last_device.udn.clone()).await;
                                        } else {
                                            send_app_log!(
                                                &self.tx_channel, Info, "Client error; not trying again (no last-active device)"
                                            );
                                        }
                                    },
                                    StreamMagicManagerAction::OnUIReady => {
                                        self.ui_ready = true;
                                        self.emit_manager_state().await;
                                        self.emit_streammagic_payloads().await;

                                        for buffered_app_log in &self.buffered_app_logs {
                                            let log_clone = buffered_app_log.clone();

                                            self.app_handle.emit_app_log_with_target(
                                                module_path!(),
                                                log_clone.level.into(),
                                                &log_clone.message
                                            ).await;
                                        }

                                        self.buffered_app_logs.clear();
                                    },
                                    StreamMagicManagerAction::ProcessDiscoveredDevice(device) => {
                                        send_app_log!(&self.tx_channel, Info, "Processing discovered device: {}", device);
                                        self.add_device(device).await;
                                    },
                                    StreamMagicManagerAction::SetIsDiscovering(is_discovering) => {
                                        self.set_is_discovering(is_discovering).await;
                                    },
                                    StreamMagicManagerAction::ShutDown => {
                                        send_app_log!(&self.tx_channel, Info, "StreamMagicManager shutdown requested");
                                        self.stop_websocket_client().await;

                                        // TODO: Determine what to do about shutting down the StreamMagic
                                        //  Manager. Should it be restartable by the UI?
                                        break;
                                    },
                                    StreamMagicManagerAction::StopWebSocketClient(remove_from_persisted_state) => {
                                        send_app_log!(&self.tx_channel, Info, "StreamMagicManager WebSocket halt requested");
                                        self.stop_websocket_client().await;
                                        self.reset_websocket_related_state().await;

                                        if remove_from_persisted_state {
                                            let persisted_backend_state = self.app_handle.state::<PersistedBackendState>();
                                            let mut state_guard = persisted_backend_state.0.lock().unwrap();

                                            if let Err(delete_error) = state_guard.delete(KEY_LAST_CONNECTED_HOST) {
                                                send_app_log!(
                                                    self.tx_channel, Warn,
                                                    "Could not remove last connected host from persisted state: {:?}",
                                                    delete_error
                                                );
                                            }
                                        }
                                    },
                                    StreamMagicManagerAction::TestConnection => {
                                        match self.ws_client_status {
                                            WebSocketClientStatus::Connected(_) => {
                                                // When actively connected, ask the WebSocketClient to do a test
                                                self.send_test_connection_request_to_websocket_client().await;
                                            },
                                            _ => {
                                                // When not actively connected, attempt a reconnect to the last-active device
                                                if let Some(last_device) = &self.last_active_device {
                                                    send_app_log!(
                                                        &self.tx_channel,
                                                        Info,
                                                        "No active WebSocketClient connection to test; attempting to re-activate last-seen device"
                                                    );

                                                    let _ = &self.activate_device(&last_device.udn.clone()).await;
                                                } else {
                                                    send_app_log!(
                                                        &self.tx_channel,
                                                        Warn,
                                                        "Cannot perform test; no active WebSocketClient connection, and no known last-seen device to re-activate"
                                                    );
                                                }
                                            }
                                        }
                                    },
                                }
                            },
                            StreamMagicManagerChannelMsg::StreamerActionMsg(streamer_action) => {
                                match streamer_action {
                                    StreamerAction::MuteSet(is_muted) => {
                                        self.send_websocket_message(&ZoneState::set_mute_msg(is_muted)).await;
                                    },
                                    StreamerAction::NextTrack => {
                                        self.send_websocket_message(&PlayControl::next_track_msg()).await;
                                    },
                                    StreamerAction::Pause => {
                                        self.send_websocket_message(&PlayControl::pause_msg()).await;
                                    },
                                    StreamerAction::PlayQueueId(queue_id) => {
                                        self.send_websocket_message(&PlayControl::play_queue_id_msg(queue_id)).await;
                                    },
                                    StreamerAction::PlayPresetId(preset_id) => {
                                        self.send_websocket_message(&RecallPreset::play_preset_id_msg(preset_id)).await;
                                    },
                                    StreamerAction::PowerOn => {
                                        self.send_websocket_message(&SystemPower::on_msg()).await;
                                    },
                                    StreamerAction::PowerStandby => {
                                        self.send_websocket_message(&SystemPower::standby_msg()).await;
                                    },
                                    StreamerAction::PowerToggle => {
                                        self.send_websocket_message(&SystemPower::toggle_msg()).await;
                                    },
                                    StreamerAction::Play => {
                                        self.send_websocket_message(&PlayControl::play_msg()).await;
                                    },
                                    StreamerAction::PreviousTrack => {
                                        self.send_websocket_message(&PlayControl::previous_track_msg()).await;
                                    },
                                    StreamerAction::Seek(position) => {
                                        self.send_websocket_message(&PlayControl::seek_msg(position)).await;
                                    },
                                    StreamerAction::SetRepeat(state) => {
                                        self.send_websocket_message(&PlayControl::set_repeat_msg(state)).await;
                                    },
                                    StreamerAction::SetShuffle(state) => {
                                        self.send_websocket_message(&PlayControl::set_shuffle_msg(state)).await;
                                    },
                                    StreamerAction::SetSourceId(source_id) => {
                                        self.send_websocket_message(&ZoneState::set_source_id_msg(source_id)).await;
                                    },
                                    StreamerAction::Stop => {
                                        self.send_websocket_message(&PlayControl::stop_msg()).await;
                                    },
                                    StreamerAction::TogglePlayback => {
                                        self.send_websocket_message(&PlayControl::toggle_playback_msg()).await;
                                    },
                                    StreamerAction::VolumePercentSet(percent) => {
                                        self.send_websocket_message(&ZoneState::set_volume_percent_msg(percent)).await;
                                    },
                                    StreamerAction::VolumeStepChange(degree) => {
                                        self.send_websocket_message(&ZoneState::change_volume_step_msg(degree)).await;
                                    },
                                    StreamerAction::VolumeStepSet(step) => {
                                        self.send_websocket_message(&ZoneState::set_volume_step_msg(step)).await;
                                    },
                                }
                            },
                        }
                    }
                }

                // --------------------------------------------------------------------------------
                // WebSocket messages coming from the WebSocketClient.
                //
                // NOTE: This is a slightly off-feeling way of only checking for incoming WebSocket
                //  client messages when the channel is Some and not None (the channel will be None
                //  when there's no active WebSocket connection). The "expect()" should never crash
                //  -- in theory -- due to the "if ...is_some()".
                //
                // ref: https://github.com/tokio-rs/tokio/issues/2583#issuecomment-638212772
                ws_client_message = async {
                    self.ws_client_receive_channel.as_mut().expect("websocket client handler crash").recv().await
                }, if &self.ws_client_receive_channel.is_some() => {

                    if let Some(update) = ws_client_message {
                        match update {
                            WSClientTxChannelMsg::DataMsg(message) => {
                                match self.process_streammagic_message(&message).await {
                                    Ok(_) => {},
                                    Err(e) => {
                                        send_app_log!(&self.tx_channel, Warn, "Could not process StreamMagic message: {:?}", e);
                                        send_app_log!(&self.tx_channel, Warn, "Unprocessable message: {}", &message);
                                    },
                                }
                            },
                            WSClientTxChannelMsg::WebSocketClientStatusMsg(status) => {
                                if !self.is_testing_connection {
                                    send_app_log!(&self.tx_channel, Info, "WebSocketClient status: {:?}", &status);
                                }

                                match &status {
                                    WebSocketClientStatus::Connected(details) => {
                                        if self.is_testing_connection {
                                            // The WebSocketClient responds with a Connected state if it's been asked
                                            // to test the connection and the connection is OK.
                                            send_app_log!(&self.tx_channel, Info, "Streamer connection OK");
                                            self.set_is_testing_connection(false).await;
                                        }

                                        if !details.existing {
                                            // This is a new connection not an existing one, so do some "we have a new
                                            // connection" tasks.

                                            if let Some(host) = host_from_url(&details.url) {
                                                // Persist this successful host for later use.
                                                send_app_log!(&self.tx_channel, Info, "Persisting last connected host: {}", host);
                                                let persisted_backend_state = self.app_handle.state::<PersistedBackendState>();
                                                let mut state_guard = persisted_backend_state.0.lock().unwrap();

                                                if let Err(set_error) = state_guard.set(KEY_LAST_CONNECTED_HOST, host.into()) {
                                                    send_app_log!(
                                                        self.tx_channel, Error,
                                                        "Could not persist last connected host: {:?}",
                                                        set_error
                                                    );
                                                }
                                            }

                                            self.register_for_streammagic_updates().await;

                                            // Full QueueList details do not come in via updates.
                                            // Instead, a request is made here -- and whenever a
                                            // QueueInfo update is received.
                                            self.send_websocket_message(&QueueList::request_state_msg()).await;

                                            // Whenever we get a Connected message for a non-existing connection, we
                                            // mark the device with the matching host name as active.
                                            self.set_device_is_active_from_url(&details.url).await;
                                        }

                                        self.set_is_activating(false).await;
                                    },
                                    WebSocketClientStatus::Disconnected(details) => {
                                        if let Some(reason) = &details.reason {
                                            send_app_log!(&self.tx_channel, Info, "WebSocketClient has disconnected with reason: {reason}");
                                        } else {
                                            send_app_log!(&self.tx_channel, Info, "WebSocketClient has disconnected with no message");
                                        }

                                        // If we get a Disconnected then we assume no devices are
                                        // currently active.
                                        self.set_all_device_activation_false().await;

                                        // TODO: Check if this is necessary; likely done elsewhere
                                        //  (say during WebSocketClient disconnect handling).
                                        self.ws_client_join_handle = None;
                                        self.ws_client_send_channel = None;
                                        self.ws_client_receive_channel = None;

                                        // If the connection is Disconnected and we were in test mode,
                                        // then switch from test mode to (re)activation mode.
                                        if self.is_testing_connection {
                                             send_app_log!(&self.tx_channel, Warn, "Test resulted in Disconnect");
                                             self.set_is_testing_connection(false).await;

                                             if let Some(last_active_device) = &self.last_active_device {
                                                 send_app_log!(
                                                     &self.tx_channel, Info, "Attempting re-activation of {}", &last_active_device.udn
                                                 );
                                                 self.activate_device(&last_active_device.udn.clone()).await;
                                            } else {
                                                send_app_log!(
                                                    &self.tx_channel, Info,
                                                    "Giving up on connection test (no last-active device to reactivate)"
                                                );
                                            }
                                        } else if details.consider_reconnecting {
                                            send_app_log!(&self.tx_channel, Info, "Potentially-recoverable disconnect");

                                            if let Some(last_active_device) = &self.last_active_device {
                                                send_app_log!(
                                                    &self.tx_channel, Info, "Attempting re-activation of {}", &last_active_device.udn
                                                );
                                                self.activate_device(&last_active_device.udn.clone()).await;
                                            } else {
                                                send_app_log!(
                                                    &self.tx_channel, Warn,
                                                    "Unable to determine last active UDN; not attempting reconnect"
                                                );
                                                self.set_is_activating(false).await;
                                            }
                                        } else {
                                            self.set_is_activating(false).await;
                                        }
                                    },
                                    WebSocketClientStatus::TestingConnection => {
                                        send_app_log!(&self.tx_channel, Debug, "WebSocketClient is testing its connection");
                                    },
                                    _ => {},
                                }

                                // Update the manager's tracking of the client status (which will
                                // also update the UI).
                                self.set_websocket_client_status(&status).await;
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
        send_app_log!(&self.tx_channel, Info, "StreamMagicManager has ended");

        Ok(())
    }
}
