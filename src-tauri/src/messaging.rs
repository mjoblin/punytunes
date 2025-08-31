use std::fmt::Display;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use log;
use serde;
use serde::Serializer;
use ts_rs::TS;

#[derive(Debug, TS)]
#[ts(export, export_to = "../src/types/generated/AppMessageType.ts")]
pub enum AppMessageType {
    AmplifierManagerState,
    AmplifierState,
    AppLog,
    Devices,
    IsActivating,
    IsDiscovering,
    IsInitializingStreamMagicManager,
    StreamerSystemInfo,
    StreamerSystemPower,
    StreamerSystemSources,
    StreamerPresets,
    StreamerQueueList,
    StreamerZoneNowPlaying,
    StreamerZonePlayState,
    StreamerZonePlayStatePosition,
    StreamerZonePosition,
    StreamerZoneState,
    StreamMagicManagerState,
    StreamMagicManagerStatus,
    WebSocketClientStatus,
}

impl Display for AppMessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self))
    }
}

#[derive(Clone, Debug, serde::Deserialize, TS)]
#[ts(export, rename = "Level", rename_all = "lowercase", export_to = "../src/types/generated/Level.ts")]
pub enum SerializableLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Convert from log::Level to SerializableLevel of the same name
impl From<log::Level> for SerializableLevel {
    fn from(level: log::Level) -> Self {
        match level {
            log::Level::Error => SerializableLevel::Error,
            log::Level::Warn => SerializableLevel::Warn,
            log::Level::Info => SerializableLevel::Info,
            log::Level::Debug => SerializableLevel::Debug,
            log::Level::Trace => SerializableLevel::Trace,
        }
    }
}

/// Convert from SerializableLevel to log::Level of the same name
impl From<SerializableLevel> for log::Level {
    fn from(serializable_level: SerializableLevel) -> Self {
        match serializable_level {
            SerializableLevel::Error => log::Level::Error,
            SerializableLevel::Warn => log::Level::Warn,
            SerializableLevel::Info => log::Level::Info,
            SerializableLevel::Debug => log::Level::Debug,
            SerializableLevel::Trace => log::Level::Trace,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, TS)]
#[ts(export, export_to = "../src/types/generated/AppLog.ts")]
pub struct AppLog {
    pub level: SerializableLevel,
    pub message: String,

    when: u128,
}

impl AppLog {
    pub fn new(level: log::Level, message: &str) -> Self {
        let when = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(when) => when,
            Err(_) => Duration::ZERO, // This really shouldn't happen
        };

        AppLog {
            level: level.into(),
            message: message.to_string(),
            when: when.as_millis(),
        }
    }
}

/// Display SerializableLevel as lowercase
impl Display for SerializableLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

/// Serialize SerializableLevel to lowercase
impl serde::Serialize for SerializableLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(self.to_string().to_lowercase().as_ref())
    }
}
