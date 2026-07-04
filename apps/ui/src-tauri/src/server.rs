use anyhow::{Context, Result};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::auth_check::test_proxy_auth;
use crate::config::Config;
use crate::credentials::load_credentials;
use crate::pac_server::{run_pac_server, sync_default_pac};
use crate::pac_settings::commands::{apply_pac_settings, remove_pac_settings, AppliedPacSettings};
use crate::proxy_server::run_proxy_server;

pub struct ServerManager {
    handles: Arc<Mutex<Option<ServerHandles>>>,
    // Snapshot of the system PAC state taken at apply time, so stop/quit can
    // undo exactly what was changed even if the network environment moved on.
    applied_pac: Arc<Mutex<Option<AppliedPacSettings>>>,
}

struct ServerHandles {
    proxy_handle: tokio::task::JoinHandle<Result<()>>,
    pac_handle: tokio::task::JoinHandle<Result<()>>,
}

impl ServerManager {
    pub fn new() -> Self {
        Self {
            handles: Arc::new(Mutex::new(None)),
            applied_pac: Arc::new(Mutex::new(None)),
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

        // Bind both listeners before spawning anything, so a port conflict
        // fails this command with an actionable error instead of dying silently
        // inside a spawned task while the tray reports the server as running.
        let proxy_listener = TcpListener::bind(("127.0.0.1", config.local_proxy_port))
            .await
            .with_context(|| {
                format!(
                    "Failed to bind local proxy port {} (is it already in use?)",
                    config.local_proxy_port
                )
            })?;
        let pac_listener = TcpListener::bind(("127.0.0.1", config.pac_port))
            .await
            .with_context(|| {
                format!(
                    "Failed to bind PAC server port {} (is it already in use?)",
                    config.pac_port
                )
            })?;

        // take the lock to register and spawn the servers. Re-check that
        // nothing started while we were preparing above.
        let mut handles_guard = self.handles.lock().await;

        if handles_guard.is_some() {
            println!("[ServerManager] Start called but servers are already running");
            return Ok(()); // Already running
        }

        // Start servers
        let proxy_handle = tokio::spawn(run_proxy_server(
            proxy_listener,
            config.clone(),
            credentials.clone(),
        ));

        let pac_handle = tokio::spawn(run_pac_server(pac_listener, config.clone()));

        // Apply PAC settings; if this fails, tear the just-spawned servers
        // down so we don't leak running tasks the manager isn't tracking.
        let applied = match apply_pac_settings(&config) {
            Ok(applied) => applied,
            Err(err) => {
                proxy_handle.abort();
                pac_handle.abort();
                return Err(err.context("Failed to apply system PAC settings"));
            }
        };

        // Track the applied state so we can cleanly undo it on stop/quit
        {
            let mut applied_pac_guard = self.applied_pac.lock().await;
            *applied_pac_guard = Some(applied);
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
        let mut applied_pac_guard = self.applied_pac.lock().await;

        if let Some(handles) = handles_guard.take() {
            println!("[ServerManager] Stopping proxy and PAC servers...");
            handles.proxy_handle.abort();
            handles.pac_handle.abort();
            // Await the aborted tasks: abort() only schedules cancellation, and
            // the listeners (plus every in-flight connection, via the servers'
            // JoinSets) are freed only when the futures are dropped. Without
            // this, a subsequent start() can lose a race for the port against
            // the old listener.
            let _ = handles.proxy_handle.await;
            let _ = handles.pac_handle.await;
            if let Some(applied) = applied_pac_guard.take() {
                println!("[ServerManager] Removing PAC settings before fully stopping");
                if let Err(err) = remove_pac_settings(&applied) {
                    println!(
                        "[ServerManager] Failed to remove system PAC settings: {}",
                        err
                    );
                }
            } else {
                println!("[ServerManager] No PAC settings tracked; skipping PAC removal");
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
