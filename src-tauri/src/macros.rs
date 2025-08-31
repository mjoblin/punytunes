use log::warn;
use tokio::sync::mpsc;

use crate::amplifier_manager::{AmplifierManagerAction, AmplifierManagerChannelMsg};
use crate::streammagic_manager::{StreamMagicManagerAction, StreamMagicManagerChannelMsg};

/// The app_info, app_warn, and app_error macros assume they're being called from within a struct
/// implementation function, where the struct has an `app_handle` field which is a Tauri
/// `AppHandle`.
#[macro_export]
macro_rules! app_info {
    ($self:ident, $($arg:tt)*) => {
        let formatted_str = format!($($arg)*);
        let target = module_path!();

        $self.app_handle.emit_app_log_with_target(target, log::Level::Info, formatted_str.as_str()).await;
    };
}

#[macro_export]
macro_rules! app_warn {
    ($self:ident, $($arg:tt)*) => {
        let formatted_str = format!($($arg)*);
        let target = module_path!();

        $self.app_handle.emit_app_log_with_target(target, log::Level::Warn, formatted_str.as_str()).await;
    };
}

#[macro_export]
macro_rules! app_error {
    ($self:ident, $($arg:tt)*) => {
        let formatted_str = format!($($arg)*);
        let target = module_path!();

        $self.app_handle.emit_app_log_with_target(target, log::Level::Error, formatted_str.as_str()).await;
    };
}

/// send_manager_action sends a StreamMagicManagerActionMsg to the given sender.

pub async fn send_manager_action_impl(
    sender: &mpsc::Sender<StreamMagicManagerChannelMsg>,
    action: StreamMagicManagerAction,
) {
    match sender.send(StreamMagicManagerChannelMsg::StreamMagicManagerActionMsg(action.clone())).await
    {
        Ok(_) => {}
        Err(e) =>
            warn!("Could not send manager action {:?}: {:?}", &action, e),
    }
}

#[macro_export]
macro_rules! send_manager_action {
    ($sender:expr, $action:expr) => {
        {
            let sender = $sender.clone();
            let action = $action;

            tokio::spawn(async move {
                crate::macros::send_manager_action_impl(&sender, action).await;
            });
        }
    };
}

/// send_amplifier_manager_action sends a AmplifierManagerActionMsg to the given sender.

pub async fn send_amplifier_manager_action_impl(
    sender: &mpsc::Sender<AmplifierManagerChannelMsg>,
    action: AmplifierManagerAction,
) {
    match sender.send(AmplifierManagerChannelMsg::AmplifierManagerActionMsg(action.clone())).await
    {
        Ok(_) => {}
        Err(e) =>
            warn!("Could not send amplifier manager action {:?}: {:?}", &action, e),
    }
}

#[macro_export]
macro_rules! send_amplifier_manager_action {
    ($sender:expr, $action:expr) => {
        {
            let sender = $sender.clone();
            let action = $action;

            tokio::spawn(async move {
                crate::macros::send_amplifier_manager_action_impl(&sender, action).await;
            });
        }
    };
}

/// send_app_log sends a StreamMagicManagerAction::EmitAppLog to the given sender.
#[macro_export]
macro_rules! send_app_log {
    ($sender:expr, $level:expr, $($arg:tt)*) => {
        {
            let sender = $sender.clone();
            let level = $level;
            let formatted_str = format!($($arg)*);

            tokio::spawn(async move {
                crate::macros::send_manager_action_impl(
                    &sender,
                    crate::streammagic_manager::StreamMagicManagerAction::EmitAppLog(
                        crate::messaging::AppLog::new(level, formatted_str.as_str())
                    ),
                ).await;
            });
        }
    };
}
