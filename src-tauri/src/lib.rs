pub mod audio;
pub mod client;
pub mod commands;
pub mod network;
pub mod protocol;
pub mod screen;
pub mod server;
pub mod stream_server;

use commands::*;
use stream_server::{DEFAULT_STREAM_PORT, start_stream_server};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();
    let latest_frame = app_state.latest_frame.clone();

    tauri::Builder::default()
        .manage(app_state)
        .setup(move |app| {
            // Start background video MJPEG streaming server for webview
            tauri::async_runtime::spawn(async move {
                start_stream_server(DEFAULT_STREAM_PORT, latest_frame).await;
            });

            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                let win = window.clone();
                // Safety fallback: only show if frontend fails to mount within 10s
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(10000)).await;
                    let _ = win.show();
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_network_info,
            create_room,
            join_room,
            leave_room,
            toggle_mic,
            toggle_deafen,
            get_monitors,
            start_screen_share,
            stop_screen_share,
            get_audio_devices,
            set_audio_devices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
