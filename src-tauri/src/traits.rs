use async_trait::async_trait;
use log::{
    debug, error, info, trace, warn,
    Level::{Debug, Error, Info, Trace, Warn},
};
use tauri::{AppHandle, Manager};

use crate::messaging::{AppLog, AppMessageType};

#[async_trait]
pub trait CustomEmitters {
    async fn emit_app_message<S: serde::Serialize + Clone + Send>(&self, msg_type: AppMessageType, payload: S);
    async fn emit_app_log(&self, level: log::Level, message: &str);
    async fn emit_app_log_with_target(&self, target: &str, level: log::Level, message: &str);
}

#[async_trait]
impl CustomEmitters for AppHandle {
    async fn emit_app_message<S: serde::Serialize + Clone + Send>(&self, msg_type: AppMessageType, payload: S) {
        let msg_type_str = msg_type.to_string();

        match self.emit_all(&msg_type_str, payload) {
            Ok(_) => {}
            Err(e) => warn!("Could not emit {} message: {:?}", &msg_type_str, e),
        }
    }

    async fn emit_app_log(&self, level: log::Level, message: &str) {
        // TODO: There's probably a better way to map a log level to a log macro
        match level {
            Debug => debug!("{}", message),
            Error => error!("{}", message),
            Info => info!("{}", message),
            Trace => trace!("{}", message),
            Warn => warn!("{}", message),
        }

        self.emit_app_message(AppMessageType::AppLog, AppLog::new(level, message))
            .await;
    }

    async fn emit_app_log_with_target(&self, target: &str, level: log::Level, message: &str) {
        // TODO: There's probably a better way to map a log level to a log macro
        match level {
            Debug => debug!(target: target, "{}", message),
            Error => error!(target: target, "{}", message),
            Info => info!(target: target, "{}", message),
            Trace => trace!(target: target, "{}", message),
            Warn => warn!(target: target, "{}", message),
        }

        self.emit_app_message(AppMessageType::AppLog, AppLog::new(level, message))
            .await;
    }
}
