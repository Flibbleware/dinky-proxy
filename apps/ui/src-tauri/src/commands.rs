use tauri::{AppHandle, Manager};

use crate::config::{load_config, normalize_config_payload, save_config, AppConfigPayload};
use crate::keyring::MasterKey;
use crate::server::ServerManager;
use crate::tray::update_tray_state;

fn get_master_key(app: &AppHandle) -> String {
    app.state::<MasterKey>().0.clone()
}

#[tauri::command]
pub async fn load_config_command(app_handle: AppHandle) -> Result<AppConfigPayload, String> {
    let master_key = get_master_key(&app_handle);

    load_config(&app_handle, &master_key).map_err(|e| format!("Failed to load config: {}", e))
}

#[tauri::command]
pub async fn save_config_command(
    app_handle: AppHandle,
    payload: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let master_key = get_master_key(&app_handle);

    // Normalize the payload
    let normalized = normalize_config_payload(&payload);

    // Save config
    let config_path = save_config(&app_handle, &normalized, &master_key)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    // Check if server is running and restart if needed
    let was_running = {
        let server_manager = app_handle.state::<ServerManager>();
        server_manager.is_running().await
    };

    if was_running {
        let server_manager = app_handle.state::<ServerManager>();
        server_manager
            .restart(app_handle.clone(), master_key)
            .await
            .map_err(|e| format!("Failed to restart server: {}", e))?;

        update_tray_state(&app_handle).await;
    }

    Ok(serde_json::json!({
        "path": config_path.to_string_lossy(),
        "restarted": was_running
    }))
}

#[tauri::command]
pub async fn start_server_command(app_handle: AppHandle) -> Result<(), String> {
    let master_key = get_master_key(&app_handle);

    let server_manager = app_handle.state::<ServerManager>();
    server_manager
        .start(app_handle.clone(), master_key)
        .await
        .map_err(|e| format!("Failed to start server: {}", e))?;

    update_tray_state(&app_handle).await;
    Ok(())
}

#[tauri::command]
pub async fn stop_server_command(app_handle: AppHandle) -> Result<(), String> {
    let server_manager = app_handle.state::<ServerManager>();
    server_manager
        .stop()
        .await
        .map_err(|e| format!("Failed to stop server: {}", e))?;

    update_tray_state(&app_handle).await;
    Ok(())
}

#[tauri::command]
pub async fn is_server_running_command(app_handle: AppHandle) -> Result<bool, String> {
    let server_manager = app_handle.state::<ServerManager>();
    Ok(server_manager.is_running().await)
}
