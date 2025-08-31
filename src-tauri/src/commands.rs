use log::error;

use crate::{send_app_log, send_manager_action};
use crate::amplifier_manager::{AmplifierAction, AmplifierManagerAction, AmplifierManagerChannel, AmplifierManagerChannelMsg};
use crate::streammagic_manager::{
    StreamerAction, StreamMagicManagerAction, StreamMagicManagerChannel, StreamMagicManagerChannelMsg,
};

#[tauri::command]
pub async fn activate_device(
    stream_magic_manager_channel: tauri::State<'_, StreamMagicManagerChannel>,
    udn: &str,
) -> Result<(), ()> {
    send_manager_action!(
        stream_magic_manager_channel.0,
        StreamMagicManagerAction::ActivateUdn(udn.to_string())
    );

    Ok(())
}

#[tauri::command]
pub async fn deactivate(stream_magic_manager_channel: tauri::State<'_, StreamMagicManagerChannel>) -> Result<(), ()> {
    send_manager_action!(
        stream_magic_manager_channel.0,
        StreamMagicManagerAction::Deactivate
    );

    Ok(())
}

#[tauri::command]
pub async fn discover_streamer(
    stream_magic_manager_channel: tauri::State<'_, StreamMagicManagerChannel>
) -> Result<(), ()> {
    send_manager_action!(
        stream_magic_manager_channel.0,
        StreamMagicManagerAction::Discover(false)
    );

    Ok(())
}

#[tauri::command]
pub async fn discover_amplifier(
    amplifier_manager_channel: tauri::State<'_, AmplifierManagerChannel>
) -> Result<(), ()> {
    send_amplifier_manager_action!(amplifier_manager_channel.0, AmplifierManagerAction::Discover);

    Ok(())
}

#[tauri::command]
pub async fn emit_app_log(
    stream_magic_manager_channel: tauri::State<'_, StreamMagicManagerChannel>,
    level: String,
    message: String,
) -> Result<(), ()> {
    // TODO: Figure out how to deserialize a SerializableLevel; or just rethink logging
    //  across the app...
    let log_level = match level.as_str() {
        "debug" => log::Level::Debug,
        "error" => log::Level::Error,
        "info" => log::Level::Info,
        "trace" => log::Level::Trace,
        "warn" => log::Level::Warn,
        _ => log::Level::Info,
    };

    send_app_log!(stream_magic_manager_channel.0, log_level, "{}", message);

    Ok(())
}

#[tauri::command]
pub async fn send_amplifier_action(
    stream_magic_manager_channel: tauri::State<'_, StreamMagicManagerChannel>,
    amplifier_manager_channel: tauri::State<'_, AmplifierManagerChannel>,
    action: AmplifierAction,
) -> Result<(), ()> {
    send_app_log!(
        stream_magic_manager_channel.0, log::Level::Info, "Amplifier action: {:?}", &action
    );

    match amplifier_manager_channel
        .0
        .send(AmplifierManagerChannelMsg::AmplifierActionMsg(action.clone()))
        .await
    {
        Ok(()) => {}
        Err(e) => error!(
            "Could not send amplifier action to AmplifierManager channel: {:?}: {:?}",
            action, e
        ),
    }

    Ok(())
}

#[tauri::command]
pub async fn send_streamer_action(
    stream_magic_manager_channel: tauri::State<'_, StreamMagicManagerChannel>,
    action: StreamerAction,
) -> Result<(), ()> {
    send_app_log!(
        stream_magic_manager_channel.0, log::Level::Info, "Streamer action: {:?}", &action
    );

    match stream_magic_manager_channel
        .0
        .send(StreamMagicManagerChannelMsg::StreamerActionMsg(action.clone()))
        .await
    {
        Ok(()) => {}
        Err(e) => error!(
            "Could not send streamer action to StreamMagicManager channel: {:?}: {:?}",
            action, e
        ),
    }

    Ok(())
}

#[tauri::command]
pub async fn shutdown(stream_magic_manager_channel: tauri::State<'_, StreamMagicManagerChannel>) -> Result<(), ()> {
    send_app_log!(stream_magic_manager_channel.0, log::Level::Info, "PunyTunes shutdown requested");

    // TODO: Add amplifier shutdown

    send_manager_action!(
        stream_magic_manager_channel.0,
        StreamMagicManagerAction::ShutDown
    );

    Ok(())
}

#[tauri::command]
pub async fn stop_websocket_client(
    stream_magic_manager_channel: tauri::State<'_, StreamMagicManagerChannel>,
    delete_from_persisted_state: bool,
) -> Result<(), ()> {
    send_app_log!(stream_magic_manager_channel.0, log::Level::Info, "Stopping WebSocket client");

    send_manager_action!(
        stream_magic_manager_channel.0,
        StreamMagicManagerAction::StopWebSocketClient(delete_from_persisted_state)
    );

    Ok(())
}

#[tauri::command]
pub async fn test_amplifier_connection(
    stream_magic_manager_channel: tauri::State<'_, StreamMagicManagerChannel>,
    amplifier_manager_channel: tauri::State<'_, AmplifierManagerChannel>,
) -> Result<(), ()> {
    send_app_log!(stream_magic_manager_channel.0, log::Level::Info, "Testing amplifier connection");

    send_amplifier_manager_action!(
        amplifier_manager_channel.0,
        AmplifierManagerAction::TestConnection
    );

    Ok(())
}

#[tauri::command]
pub async fn test_streamer_connection(stream_magic_manager_channel: tauri::State<'_, StreamMagicManagerChannel>) -> Result<(), ()> {
    send_app_log!(stream_magic_manager_channel.0, log::Level::Info, "Testing streamer connection");

    send_manager_action!(
        stream_magic_manager_channel.0,
        StreamMagicManagerAction::TestConnection
    );

    Ok(())
}

#[tauri::command]
pub async fn ui_ready(
    stream_magic_manager_channel: tauri::State<'_, StreamMagicManagerChannel>,
    amplifier_manager_channel: tauri::State<'_, AmplifierManagerChannel>,
) -> Result<(), ()> {
    send_app_log!(stream_magic_manager_channel.0, log::Level::Info, "UI ready");

    send_manager_action!(
        stream_magic_manager_channel.0,
        StreamMagicManagerAction::OnUIReady
    );

    send_amplifier_manager_action!(
        amplifier_manager_channel.0,
        AmplifierManagerAction::OnUIReady
    );

    Ok(())
}
