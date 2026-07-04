use anyhow::{Context, Result};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

use crate::auth_check::test_proxy_auth;
use crate::config::Config;
use crate::credentials::load_credentials;
use crate::pac_server::{run_pac_server, sync_default_pac};
use crate::pac_settings::commands::apply_pac_settings;
use crate::proxy_server::run_proxy_server;

pub struct ServerManager {
    handles: Arc<Mutex<Option<ServerHandles>>>,
    active_config: Arc<Mutex<Option<Config>>>,
}

struct ServerHandles {
    proxy_handle: tokio::task::JoinHandle<Result<()>>,
    pac_handle: tokio::task::JoinHandle<Result<()>>,
}

impl ServerManager {
    pub fn new() -> Self {
        Self {
            handles: Arc::new(Mutex::new(None)),
            active_config: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start(&self, app_handle: AppHandle, master_key: String) -> Result<()> {
        // Ensure no lingering servers from a previous start in this process
        if self.is_running().await {
            println!(
                "[ServerManager] Detected existing servers; stopping them before starting fresh"
            );
            if let Err(err) = self.stop().await {
                println!(
                    "[ServerManager] Warning: failed to stop existing servers cleanly: {}",
                    err
                );
            }
        }

        println!("[ServerManager] Starting proxy and PAC servers...");

        // Prepare config and validate proxy auth without holding the `handles`
        // lock, so concurrent status checks, stops, and config saves stay
        // responsive instead of serializing behind it and freezing the UI on a
        // misconfigured proxy.

        // Load config
        let config_payload = crate::config::load_config(&app_handle, &master_key)
            .context("Failed to load config")?;

        let base_dir = app_handle
            .path()
            .app_data_dir()
            .context("Failed to get app data directory")?;

        let config = Config::from_payload(config_payload, base_dir);

        // Sync PAC file
        sync_default_pac(&config)?;

        // Load credentials
        let credentials = load_credentials(&config).context("Failed to load credentials")?;

        // Test proxy auth
        test_proxy_auth(&config, &credentials)
            .await
            .context("Proxy authentication failed")?;

        // take the lock to register and spawn the servers. Re-check that
        // nothing started while we were preparing above.
        let mut handles_guard = self.handles.lock().await;

        if handles_guard.is_some() {
            println!("[ServerManager] Start called but servers are already running");
            return Ok(()); // Already running
        }

        // Start servers
        let config_clone = config.clone();
        let credentials_clone = credentials.clone();
        let proxy_handle = tokio::spawn(run_proxy_server(config_clone, credentials_clone));

        let config_clone = config.clone();
        let pac_handle = tokio::spawn(run_pac_server(config_clone));

        // Apply PAC settings
        apply_pac_settings(&config)?;

        // Track active config so we can cleanly remove PAC settings on stop/quit
        {
            let mut active_config_guard = self.active_config.lock().await;
            *active_config_guard = Some(config.clone());
        }

        *handles_guard = Some(ServerHandles {
            proxy_handle,
            pac_handle,
        });

        println!("[ServerManager] Servers started successfully");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut handles_guard = self.handles.lock().await;
        let mut active_config_guard = self.active_config.lock().await;

        if let Some(handles) = handles_guard.take() {
            println!("[ServerManager] Stopping proxy and PAC servers...");
            handles.proxy_handle.abort();
            handles.pac_handle.abort();
            if let Some(_config) = active_config_guard.take() {
                println!("[ServerManager] Removing PAC settings before fully stopping");
                if let Err(err) = crate::pac_settings::commands::remove_pac_settings() {
                    println!(
                        "[ServerManager] Failed to remove system PAC settings: {}",
                        err
                    );
                }
            } else {
                println!("[ServerManager] No active config tracked; skipping PAC removal");
            }
            println!("[ServerManager] Servers stopped successfully");
        } else {
            println!("[ServerManager] Stop called but no servers were running");
        }

        Ok(())
    }

    pub async fn restart(&self, app_handle: AppHandle, master_key: String) -> Result<()> {
        println!("[ServerManager] Restarting servers...");
        if let Err(err) = self.stop().await {
            println!(
                "[ServerManager] Failed to stop cleanly during restart: {}",
                err
            );
        }
        // Small delay to ensure cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        self.start(app_handle, master_key).await
    }

    pub async fn is_running(&self) -> bool {
        let handles_guard = self.handles.lock().await;
        handles_guard.is_some()
    }
}

impl Default for ServerManager {
    fn default() -> Self {
        Self::new()
    }
}
