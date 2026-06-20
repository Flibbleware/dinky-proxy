#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod auth_check;
mod commands;
mod config;
mod credentials;
mod keyring;
mod net;
mod pac_server;
mod pac_settings;
mod proxy_server;
mod server;
mod socks;
mod tray;

use tauri::{tray::TrayIconBuilder, AppHandle, Manager};

use keyring::{get_or_create_master_key, MasterKey};
use server::ServerManager;

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "configuration" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "toggle_server" => {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                let server_manager = app_handle.state::<ServerManager>();
                let is_running = server_manager.is_running().await;

                if is_running {
                    let _ = commands::stop_server_command(app_handle.clone()).await;
                } else {
                    let _ = commands::start_server_command(app_handle.clone()).await;
                }

                // Update menu text and icon
                tray::update_tray_state(&app_handle).await;
            });
        }
        "quit" => {
            // Stop server before quitting
            let server_manager = app.state::<ServerManager>();
            tauri::async_runtime::block_on(async {
                if let Err(err) = server_manager.stop().await {
                    println!("[App] Failed to stop servers cleanly on quit: {}", err);
                }
            });
            app.exit(0);
        }
        _ => {}
    }
}

fn main() {
    tauri::Builder::default()
        .on_menu_event(handle_menu_event)
        .setup(|app| {
            // Fetch master key from keychain once at startup and cache it in app state.
            // This prevents repeated keychain prompts on every command call.
            let master_key =
                get_or_create_master_key().expect("Failed to retrieve master key from keychain");
            app.manage(MasterKey(master_key));

            // Initialize server manager
            app.manage(ServerManager::new());

            // Hide dock icon on macOS (menu bar only)
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            // Create system tray menu
            let menu = tray::build_tray_menu(app.handle(), "Start Server")?;

            // Create system tray icon
            let icon = tray::get_app_icon(false);

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .tooltip("DinkyProxy")
                .build(app)?;

            // Window is created by Tauri config, just hide it initially
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();

                // Handle window close event - hide instead of closing the app
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        // Prevent the window from closing and hide it instead.
                        // Best-effort: a failed hide must not panic the event loop.
                        let _ = window_clone.hide();
                        api.prevent_close();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_config_command,
            commands::save_config_command,
            commands::start_server_command,
            commands::stop_server_command,
            commands::is_server_running_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
