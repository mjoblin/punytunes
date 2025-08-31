use serde::de::{Deserializer, Visitor};
use serde::{Deserialize, Serialize};
use serde_json;
use ts_rs::TS;

/// StreamMagic payloads.
///
/// Capabilities:
///   * Generate StreamMagic path update request messages (`RequestUpdates` trait).
///   * Deserialize StreamMagic path update payloads.
///
/// Note: Many fields are marked Optional to allow serde to deserialize as many fields as it finds.
///  The downside is that the UI will need to allow for potentially-missing (null) fields. It would
///  be nice to be more robust about what fields are valid in what situations.
///
/// Note: Transparent structs are used for the top-level types (instead of type aliases) to enable
///  TypeScript type exports from ts_rs. e.g. StreamerZonePosition is not defined as a type alias
///  like "pub type StreamerZonePosition = WithZone<ZonePosition>;", but instead is a transparent
///  struct. See: https://github.com/Aleph-Alpha/ts-rs/issues/51#issuecomment-969133678

#[derive(Deserialize, Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/WithZone.ts")]
pub struct WithZone<T> {
    pub zone: String,
    pub data: T,
}

#[derive(Deserialize, Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/WithoutZone.ts")]
pub struct WithoutZone<T> {
    data: T,
}

#[derive(Deserialize, Serialize)]
pub enum Params {
    WithZone,
    WithoutZone,
}

#[derive(Deserialize)]
pub struct StreamMagicMessage {
    pub path: String,
    #[serde(rename = "type")]
    pub payload_type: String,
    pub result: i32,
    pub message: String,
    pub params: serde_json::Value,
}

pub trait RequestUpdates {
    fn request_updates_msg() -> String;
}

// ================================================================================================
// Outgoing streamer control payloads
// ================================================================================================

// PlayControl ------------------------------------------------------------------------------------

// PlayControl is only used to send commands to the streamer; we don't send this information to
// the UI, so no need to support serialization.
pub struct PlayControl {}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(
    export,
    rename_all = "lowercase",
    export_to = "../src/types/generated/streammagic_payloads/TransportToggleState.ts"
)]
pub enum TransportToggleState {
    All,
    Off,
}

// NOTE: The following handles serializing/deserializing the toggle states to/from
//  "all"/TransportToggleState:All and "off"/TransportToggleState::Off. It's mostly concerned with
//  handling the lowercase form being sent to (and received from) the UI. It's a lot of work for
//  not much gain as the UI could just expect and use the uppercase variants that we'd otherwise
//  get mostly for free with serde.

impl std::fmt::Display for TransportToggleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl<'de> Deserialize<'de> for TransportToggleState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(TransportToggleStateVisitor)
    }
}

struct TransportToggleStateVisitor;

impl<'de> Visitor<'de> for TransportToggleStateVisitor {
    type Value = TransportToggleState;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string representing a variant of TransportToggleState")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match value.to_lowercase().as_str() {
            "all" => Ok(TransportToggleState::All),
            "off" => Ok(TransportToggleState::Off),
            _ => Err(serde::de::Error::custom(format!("Unknown variant: {}", value))),
        }
    }
}

/// Generate StreamMagic payloads for streamer PlayControl actions.
impl PlayControl {
    pub fn next_track_msg() -> String {
        String::from(r#"{"path": "/zone/play_control", "params": {"skip_track": 1}}"#)
    }

    pub fn pause_msg() -> String {
        String::from(r#"{"path": "/zone/play_control", "params": {"action": "pause"}}"#)
    }

    pub fn play_msg() -> String {
        String::from(r#"{"path": "/zone/play_control", "params": {"action": "play"}}"#)
    }

    pub fn play_queue_id_msg(queue_id: i32) -> String {
        String::from(format!(
            r#"
            {{"path": "/zone/play_control", "params": {{"queue_id": {queue_id}}}}}
        "#
        ))
    }

    pub fn previous_track_msg() -> String {
        String::from(r#"{"path": "/zone/play_control", "params": {"skip_track": -1}}"#)
    }

    pub fn seek_msg(position: i32) -> String {
        String::from(format!(
            r#"
            {{"path": "/zone/play_control", "params": {{"position": {position}}}}}
        "#
        ))
    }

    pub fn set_repeat_msg(state: TransportToggleState) -> String {
        String::from(format!(
            r#"
            {{"path": "/zone/play_control", "params": {{"mode_repeat": "{state}"}}}}
        "#
        ))
    }

    pub fn set_shuffle_msg(state: TransportToggleState) -> String {
        String::from(format!(
            r#"
            {{"path": "/zone/play_control", "params": {{"mode_shuffle": "{state}"}}}}
        "#
        ))
    }

    pub fn stop_msg() -> String {
        String::from(r#"{"path": "/zone/play_control", "params": {"action": "stop"}}"#)
    }

    pub fn toggle_playback_msg() -> String {
        String::from(r#"{"path": "/zone/play_control", "params": {"action": "toggle"}}"#)
    }
}

// RecallPreset -----------------------------------------------------------------------------------

// RecallPreset is only used to send commands to the streamer; we don't send this information to
// the UI, so no need to support serialization.
pub struct RecallPreset {}

impl RecallPreset {
    pub fn play_preset_id_msg(preset_id: i32) -> String {
        String::from(format!(
            r#"
            {{"path": "/zone/recall_preset", "params": {{"preset": {preset_id}}}}}
        "#
        ))
    }
}

// ================================================================================================
// Incoming streamer update payloads
// ================================================================================================

// QueueList --------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../src/types/generated/streammagic_payloads/QueueListItemMetadata.ts"
)]
pub struct QueueListItemMetadata {
    pub class: Option<String>,
    pub source: Option<String>,
    pub name: Option<String>,
    pub title: Option<String>,
    pub art_url: Option<String>,
    pub track_number: Option<i32>,
    pub duration: Option<i32>,
    pub genre: Option<String>,
    pub album: Option<String>,
    pub artist: Option<String>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/QueueListItem.ts")]
pub struct QueueListItem {
    pub id: Option<i32>,
    pub position: Option<i32>,
    pub metadata: Option<QueueListItemMetadata>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/QueueList.ts")]
pub struct QueueList {
    pub start: Option<i32>,
    pub count: Option<i32>,
    pub total: Option<i32>,
    pub play_postition: Option<i32>, // TODO: Is this a typo?
    pub play_id: Option<i32>,
    pub items: Option<Vec<QueueListItem>>,
}

impl QueueList {
    pub fn request_state_msg() -> String {
        String::from(r#"{"path": "/queue/list"}"#)
    }
}

#[derive(Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../src/types/generated/streammagic_payloads/StreamerQueueList.ts"
)]
pub struct StreamerQueueList(WithoutZone<QueueList>);

// ------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
pub struct QueueInfo {
    pub ids: Option<Vec<i64>>,
    pub total: Option<i64>,
    pub id_array_token: Option<i64>,
    pub play_postition: Option<i64>,
    pub play_id: Option<i64>,
}

impl RequestUpdates for QueueInfo {
    fn request_updates_msg() -> String {
        String::from(r#"{"path": "/queue/info", "params": {"update": 1}}"#)
    }
}

#[derive(Serialize, Deserialize)]
pub struct StreamerQueueInfo(WithoutZone<QueueInfo>);

// Presets ----------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/PresetItem.ts")]
pub struct PresetItem {
    pub id: Option<i32>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub r#type: Option<String>,
    pub class: Option<String>,
    pub state: Option<String>,
    pub is_playing: Option<bool>,
    pub art_url: Option<String>,
    pub airable_radio_id: Option<i64>, // These can be big numbers so bumping to i64
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/Presets.ts")]
pub struct Presets {
    pub start: Option<i32>,
    pub end: Option<i32>,
    pub max_presets: Option<i32>,
    pub presettable: Option<bool>,
    pub presets: Option<Vec<PresetItem>>,
}

impl RequestUpdates for Presets {
    fn request_updates_msg() -> String {
        String::from(r#"{"path": "/presets/list", "params": {"update": 1}}"#)
    }
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/StreamerPresets.ts")]
pub struct StreamerPresets(WithoutZone<Presets>);

// SystemInfo -------------------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/SystemInfoVersion.ts")]
pub struct SystemInfoVersion {
    pub component: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/SystemInfo.ts")]
pub struct SystemInfo {
    pub name: Option<String>,
    pub timezone: Option<String>,
    pub locale: Option<String>,
    pub usage_reports: Option<bool>,
    pub setup: Option<bool>,
    pub sources_setup: Option<bool>,
    pub versions: Option<Vec<SystemInfoVersion>>,
    pub udn: Option<String>,
    pub hcv: Option<i64>,
    pub model: Option<String>,
    pub unit_id: Option<String>,
    pub max_http_body_size: Option<i64>,
    pub api: Option<String>,
}

impl RequestUpdates for SystemInfo {
    fn request_updates_msg() -> String {
        String::from(r#"{"path": "/system/info", "params": {"update": 1}}"#)
    }
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/StreamerSystemInfo.ts")]
pub struct StreamerSystemInfo(WithoutZone<SystemInfo>);

// SystemPower ------------------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/SystemPower.ts")]
pub struct SystemPower {
    pub power: String,
}

impl SystemPower {
    pub fn on_msg() -> String {
        String::from(r#"{"path": "/system/power", "params": {"power": "ON"}}"#)
    }

    pub fn standby_msg() -> String {
        String::from(r#"{"path": "/system/power", "params": {"power": "NETWORK"}}"#)
    }

    pub fn toggle_msg() -> String {
        String::from(r#"{"path": "/system/power", "params": {"power": "toggle"}}"#)
    }
}

impl RequestUpdates for SystemPower {
    fn request_updates_msg() -> String {
        String::from(r#"{"path": "/system/power", "params": {"update": 1}}"#)
    }
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/StreamerSystemPower.ts")]
pub struct StreamerSystemPower(WithoutZone<SystemPower>);

// SystemSources ----------------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/Source.ts")]
pub struct Source {
    pub id: String,
    pub name: String,
    pub default_name: String,
    pub class: String,
    pub nameable: bool,
    pub ui_selectable: bool,
    pub description: String,
    pub description_locale: String,
    pub preferred_order: i64,
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/SystemSources.ts")]
pub struct SystemSources {
    pub sources: Vec<Source>,
}

impl RequestUpdates for SystemSources {
    fn request_updates_msg() -> String {
        String::from(r#"{"path": "/system/sources", "params": {"update": 1}}"#)
    }
}

#[derive(Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../src/types/generated/streammagic_payloads/StreamerSystemSources.ts"
)]
pub struct StreamerSystemSources(WithoutZone<SystemSources>);

// ZoneNowPlaying ---------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../src/types/generated/streammagic_payloads/ZoneNowPlayingQueue.ts"
)]
pub struct ZoneNowPlayingQueue {
    pub length: Option<i32>,
    pub position: Option<i32>,
    pub shuffle: Option<String>,
    pub repeat: Option<String>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../src/types/generated/streammagic_payloads/ZoneNowPlayingProgress.ts"
)]
pub struct ZoneNowPlayingProgress {
    pub position: Option<i32>,
    pub duration: Option<i32>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../src/types/generated/streammagic_payloads/ZoneNowPlayingDisplay.ts"
)]
pub struct ZoneNowPlayingDisplay {
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub line3: Option<String>,
    pub format: Option<String>,
    pub mqa: Option<String>,
    pub playback_source: Option<String>,
    pub class: Option<String>,
    pub art_url: Option<String>,
    pub art_file: Option<String>,
    pub progress: Option<ZoneNowPlayingProgress>,
    pub context: Option<String>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../src/types/generated/streammagic_payloads/ZoneNowPlayingSource.ts"
)]
pub struct ZoneNowPlayingSource {
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/ZoneNowPlaying.ts")]
pub struct ZoneNowPlaying {
    pub state: Option<String>,
    pub source: Option<ZoneNowPlayingSource>,
    pub display: Option<ZoneNowPlayingDisplay>,
    pub queue: Option<ZoneNowPlayingQueue>,
    pub controls: Option<Vec<String>>,
}

impl RequestUpdates for ZoneNowPlaying {
    fn request_updates_msg() -> String {
        String::from(r#"{"path": "/zone/now_playing", "params": {"update": 1}}"#)
    }
}

#[derive(Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../src/types/generated/streammagic_payloads/StreamerZoneNowPlaying.ts"
)]
pub struct StreamerZoneNowPlaying(WithZone<ZoneNowPlaying>);

// ZonePlayState ----------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../src/types/generated/streammagic_payloads/ZonePlayStateMetadata.ts"
)]
pub struct ZonePlayStateMetadata {
    pub class: Option<String>,
    pub source: Option<String>,
    pub name: Option<String>,
    pub playback_source: Option<String>,
    pub track_number: Option<i32>,
    pub duration: Option<i32>,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub art_url: Option<String>,
    pub sample_format: Option<String>,
    pub mqa: Option<String>,
    pub codec: Option<String>,
    pub lossless: Option<bool>,
    pub sample_rate: Option<i32>,
    pub bit_depth: Option<i32>,
    pub encoding: Option<String>,
    pub station: Option<String>,
    pub bitrate: Option<i64>,
    pub radio_id: Option<i64>, // These can be big numbers so bumping to i64
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/ZonePlayStateState.ts")]
pub enum ZonePlayStateState {
    Buffering,
    Connecting,
    NoSignal,
    NotReady,
    Pause,
    Play,
    Ready,
    Stop,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/ZonePlayState.ts")]
pub struct ZonePlayState {
    pub state: Option<ZonePlayStateState>,
    pub position: Option<i32>,
    pub presettable: Option<bool>,
    pub queue_index: Option<i32>,
    pub queue_length: Option<i32>,
    pub queue_id: Option<i32>,
    pub mode_repeat: Option<String>,
    pub mode_shuffle: Option<String>,
    pub metadata: Option<ZonePlayStateMetadata>,
}

impl RequestUpdates for ZonePlayState {
    fn request_updates_msg() -> String {
        String::from(r#"{"path": "/zone/play_state", "params": {"update": 1}}"#)
    }
}

#[derive(Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../src/types/generated/streammagic_payloads/StreamerZonePlayState.ts"
)]
pub struct StreamerZonePlayState(WithZone<ZonePlayState>);

// ZonePosition -----------------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/ZonePosition.ts")]
pub struct ZonePosition {
    pub position: i32,
}

impl RequestUpdates for ZonePosition {
    fn request_updates_msg() -> String {
        String::from(r#"{"path": "/zone/play_state/position", "params": {"update": 1}}"#)
    }
}

#[derive(Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../src/types/generated/streammagic_payloads/StreamerZonePosition.ts"
)]
pub struct StreamerZonePosition(WithZone<ZonePosition>);

// ZoneState --------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/streammagic_payloads/ZoneState.ts")]
pub struct ZoneState {
    pub source: Option<String>,
    pub power: Option<bool>,
    pub pre_amp_mode: Option<bool>,
    pub pre_amp_state: Option<String>,
    pub mute: Option<bool>,
    pub volume_step: Option<u8>,
    pub volume_percent: Option<u8>,
    pub volume_db: Option<i64>,
    pub cbus: Option<String>,
}

impl ZoneState {
    pub fn set_mute_msg(is_muted: bool) -> String {
        String::from(format!(
            r#"{{"path": "/zone/state", "params": {{"mute": {is_muted}}}}}"#
        ))
    }

    pub fn set_source_id_msg(source_id: String) -> String {
        String::from(format!(
            r#"{{"path": "/zone/state", "params": {{"source": "{source_id}"}}}}"#
        ))
    }

    pub fn set_volume_percent_msg(percent: u8) -> String {
        String::from(format!(
            r#"{{"path": "/zone/state", "params": {{"volume_percent": {percent}}}}}"#
        ))
    }

    pub fn change_volume_step_msg(degree: i8) -> String {
        String::from(format!(
            r#"{{"path": "/zone/state", "params": {{"volume_step_change": {degree}}}}}"#
        ))
    }

    pub fn set_volume_step_msg(step: u8) -> String {
        String::from(format!(
            r#"{{"path": "/zone/state", "params": {{"volume_step": {step}}}}}"#
        ))
    }
}

impl RequestUpdates for ZoneState {
    fn request_updates_msg() -> String {
        String::from(r#"{"path": "/zone/state", "params": {"update": 1}}"#)
    }
}

#[derive(Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../src/types/generated/streammagic_payloads/StreamerZoneState.ts"
)]
pub struct StreamerZoneState(WithZone<ZoneState>);
