// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use std::time::SystemTime;

use log::{info, warn};
use tauri::{CustomMenuItem, Icon, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use tauri_plugin_log::{LogTarget, RotationStrategy, TimezoneStrategy};
use tauri_plugin_positioner::{Position, WindowExt};
use tauri_plugin_theme::ThemePlugin;
use time;
use tokio;
use tokio::sync::mpsc;

use punytunes::amplifier_manager::{AmplifierManager, AmplifierManagerChannel};
use punytunes::commands;
use punytunes::persisted_state::BackendState;
use punytunes::state::PersistedBackendState;
use punytunes::streammagic_manager::{StreamMagicManager, StreamMagicManagerChannel};

// #[cfg(debug_assertions)]
// const LOG_TARGETS: [LogTarget; 3] = [LogTarget::Stdout, LogTarget::Webview, LogTarget::LogDir];
const LOG_TARGETS: [LogTarget; 2] = [LogTarget::Stdout, LogTarget::LogDir];

fn main() {
    let (streammagic_manager_channel_tx, streammagic_manager_channel_rx) = mpsc::channel(32);
    let streammagic_manager_channel_tx_clone_sm = streammagic_manager_channel_tx.clone();
    let streammagic_manager_channel_tx_clone_amp = streammagic_manager_channel_tx.clone();

    let (amplifier_manager_channel_tx, amplifier_manager_channel_rx) = mpsc::channel(32);
    let amplifier_manager_channel_tx_clone = amplifier_manager_channel_tx.clone();

    let mut ctx = tauri::generate_context!();

    let quit = CustomMenuItem::new("quit".to_string(), "Quit PunyTunes").accelerator("Cmd+Q");
    let system_tray_menu = SystemTrayMenu::new().add_item(quit);

    #[cfg(target_os = "windows")]
    let system_tray = SystemTray::new()
        .with_menu(system_tray_menu)
        .with_tooltip("PunyTunes")
        .with_icon(Icon::File("icons/icon.ico".into()));

    #[cfg(target_os = "macos")]
    let system_tray = SystemTray::new()
        .with_menu(system_tray_menu)
        .with_tooltip("PunyTunes");

    tauri::Builder::default()
        .setup(move |app| {
            // Configure persisted state
            let app_handle = app.app_handle();
            info!(
                "Application data directory: {:?}",
                app_handle.path_resolver().app_data_dir().unwrap()
            );

            // Configure the backend/Rust state which is persisted to disk
            let persisted_backend_state = PersistedBackendState(Mutex::new(BackendState::new(app.app_handle())));

            let app_handle_manager_sm = app.app_handle();
            let app_handle_manager_amp = app.app_handle();
            app.manage(persisted_backend_state);

            // Start the StreamMagicManager
            tauri::async_runtime::spawn(async move {
                let mut streammagic_manager = StreamMagicManager::new(
                    app_handle_manager_sm,
                    streammagic_manager_channel_rx,
                    streammagic_manager_channel_tx_clone_sm,
                );

                // TODO: Improve Ok/Err handling; in theory we shouldn't get past run() as the
                // manager is supposed to run forever.
                match streammagic_manager.run().await {
                    Ok(()) => info!("StreamMagicManager has ended"),
                    Err(e) => info!("StreamMagicManager has ended with error: {:?}", e),
                };
            });

            // Start the AmplifierManager
            tauri::async_runtime::spawn(async move {
                let mut amplifier_manager = AmplifierManager::new(
                    app_handle_manager_amp,
                    streammagic_manager_channel_tx_clone_amp,
                    amplifier_manager_channel_rx,
                    amplifier_manager_channel_tx_clone,
                );

                // TODO: Improve Ok/Err handling; in theory we shouldn't get past run() as the
                // manager is supposed to run forever.
                match amplifier_manager.run().await {
                    Ok(()) => info!("AmplifierManager has ended"),
                    Err(e) => info!("AmplifierManager has ended with error: {:?}", e),
                };
            });

            // Hide the PunyTunes icon in the macOS dock. This prevents the "PunyTunes" menu from
            // appearing in the menu bar. The equivalent for Windows may not be necessary, but
            // including it just in case.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            #[cfg(target_os = "windows")]
            app.get_window("main").unwrap().set_skip_taskbar(true).unwrap();

            Ok(())
        })
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(ThemePlugin::init(ctx.config_mut()))
        .plugin(tauri_plugin_positioner::init())
        .enable_macos_default_menu(false)
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| {
            tauri_plugin_positioner::on_tray_event(app, &event);

            match event {
                SystemTrayEvent::LeftClick {
                    position: _,
                    size: _,
                    ..
                } => {
                    let window = app.get_window("main").unwrap();

                    // use TrayCenter as initial window position
                    let _ = window.move_window(Position::TrayCenter);

                    app.emit_all("tray-left-click", ()).unwrap();
                }
                SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                },
                _ => {}
            }
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::Focused(is_focused) => {
                // Detect click outside of the focused window and hide the app. We avoid this if
                // the window is resizable (which should only be possible in dev mode when using
                // the FLOATING config).
                let is_resizable = event.window().is_resizable().unwrap();

                if !is_focused && !is_resizable {
                    event.window().app_handle().emit_all("app-lost-focus", ()).unwrap();
                }
            }
            _ => {}
        })
        .manage(StreamMagicManagerChannel(streammagic_manager_channel_tx.clone()))
        .manage(AmplifierManagerChannel(amplifier_manager_channel_tx.clone()))
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets(LOG_TARGETS)
                .format(|out, message, record| {
                    // In macOS, logs are in ~/Library/Logs/<bundle_identifier>/
                    let date_format =
                        time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]").unwrap();

                    let colors = ColoredLevelConfig::default();

                    out.finish(format_args!(
                        "{} [{}] [{}] {}",
                        TimezoneStrategy::UseUtc.get_now().format(&date_format).unwrap(),
                        colors.color(record.level()),
                        record.target(),
                        message
                    ))
                })
                .level(log::LevelFilter::Info)
                .rotation_strategy(RotationStrategy::KeepOne)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::activate_device,
            commands::deactivate,
            commands::discover_streamer,
            commands::discover_amplifier,
            commands::emit_app_log,
            commands::send_amplifier_action,
            commands::send_streamer_action,
            commands::shutdown,
            commands::stop_websocket_client,
            commands::test_amplifier_connection,
            commands::test_streamer_connection,
            commands::ui_ready,
        ])
        .build(ctx)
        .expect("Error building PunyTunes")
        .run(|_app_handle, event| match event {
            // Keep the Rust backend running in the background
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
