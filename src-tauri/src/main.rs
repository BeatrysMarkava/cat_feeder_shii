#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ble;
mod commands;
mod server;

use std::sync::{Arc, Mutex};
use std::thread;
use tauri::Emitter;

#[derive(Default)]
pub struct AppState {
    pub backend_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

fn main() {
    let _backend_thread = thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            server::start_backend_server().await;
        });
    });

    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::scan_ble_devices,
            commands::scan_wifi_networks,
            commands::frontend_action,
            commands::feed_now_via_tauri,
            commands::connect_esp32c6,
            commands::disconnect_esp32c6,
            commands::get_device_status,
            commands::send_wifi_credentials,
            commands::health_check,
        ])
        .setup(|_app| {
            println!("Tauri app initialized");
            println!("Backend server started in separate thread");
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                println!("Window close requested");
                api.prevent_close();
                window.emit("window-close-requested", ()).unwrap();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
