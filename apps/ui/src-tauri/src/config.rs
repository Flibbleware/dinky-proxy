use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const DEFAULT_PROXY_HOST: &str = "xx.xx.xx.xx";
const DEFAULT_PROXY_PORT: u16 = 8080;
const DEFAULT_PAC_PORT: u16 = 8000;
const DEFAULT_LOCAL_PROXY_PORT: u16 = 8888;
#[cfg(target_os = "macos")]
const DEFAULT_NETWORK_TARGET: &str = "Wi-Fi";

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProxyProtocol {
    Http,
    Socks5,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigPayload {
    pub port: u16,
    pub bypass_domains: Vec<String>,
    pub proxy_protocol: ProxyProtocol,
    pub proxy_host: String,
    pub proxy_port: u16,
    pub pac_server_port: u16,
    #[cfg(target_os = "macos")]
    pub network_target: String,
    pub username: String,
    pub password: String,
}

impl Default for AppConfigPayload {
    fn default() -> Self {
        Self {
            port: DEFAULT_LOCAL_PROXY_PORT,
            bypass_domains: vec!["imgur.com".to_string()],
            proxy_protocol: ProxyProtocol::Http,
            proxy_host: DEFAULT_PROXY_HOST.to_string(),
            proxy_port: DEFAULT_PROXY_PORT,
            pac_server_port: DEFAULT_PAC_PORT,
            #[cfg(target_os = "macos")]
            network_target: DEFAULT_NETWORK_TARGET.to_string(),
            username: String::new(),
            password: String::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub proxy_host: String,
    pub proxy_port: u16,
    pub proxy_protocol: ProxyProtocol,
    pub local_proxy_port: u16,
    pub pac_port: u16,
    pub bypass_domains: Vec<String>,
    pub username: String,
    pub password: String,
    pub base_dir: PathBuf,
}

impl Config {
    pub fn pac_path(&self) -> PathBuf {
        self.base_dir.join("proxy.pac")
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub fn pac_url(&self) -> String {
        format!("http://localhost:{}/proxy.pac", self.pac_port)
    }

    pub fn from_payload(payload: AppConfigPayload, base_dir: PathBuf) -> Self {
        Self {
            proxy_host: payload.proxy_host,
            proxy_port: payload.proxy_port,
            proxy_protocol: payload.proxy_protocol,
            local_proxy_port: payload.port,
            pac_port: payload.pac_server_port,
            bypass_domains: payload.bypass_domains,
            username: payload.username,
            password: payload.password,
            base_dir,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct EncryptedConfigFile {
    iv: String,
    tag: String,
    payload: String,
}

pub fn get_config_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .context("Failed to get app data directory")?;

    std::fs::create_dir_all(&app_data_dir).context("Failed to create app data directory")?;

    Ok(app_data_dir.join("config.enc"))
}

pub fn encrypt_config(data: &AppConfigPayload, key: &str) -> Result<EncryptedConfigFile> {
    let mut iv = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut iv);
    let key_hash = Sha256::digest(key.as_bytes());
    let cipher = Aes256Gcm::new_from_slice(&key_hash)
        .map_err(|err| anyhow!("Failed to initialise AES-256-GCM cipher: {err}"))?;

    let json = serde_json::to_string(data).context("Failed to serialize config to JSON")?;

    let nonce = Nonce::from_slice(&iv);
    let encrypted = cipher
        .encrypt(nonce, json.as_bytes())
        .map_err(|err| anyhow!("Failed to encrypt config: {err}"))?;

    // AES-GCM appends the auth tag to the end of the ciphertext
    let (payload_bytes, tag_bytes) = encrypted.split_at(encrypted.len() - 16);

    Ok(EncryptedConfigFile {
        iv: STANDARD.encode(iv),
        tag: STANDARD.encode(tag_bytes),
        payload: STANDARD.encode(payload_bytes),
    })
}

pub fn decrypt_config(encrypted: &EncryptedConfigFile, key: &str) -> Result<AppConfigPayload> {
    let key_hash = Sha256::digest(key.as_bytes());
    let cipher = Aes256Gcm::new_from_slice(&key_hash)
        .map_err(|err| anyhow!("Failed to initialise AES-256-GCM cipher: {err}"))?;

    let iv_bytes = STANDARD
        .decode(&encrypted.iv)
        .context("config.enc iv is not valid base64")?;
    if iv_bytes.len() != 12 {
        bail!(
            "config.enc iv must be 12 bytes for AES-GCM nonce, got {}",
            iv_bytes.len()
        );
    }

    let tag_bytes = STANDARD
        .decode(&encrypted.tag)
        .context("config.enc tag is not valid base64")?;
    if tag_bytes.len() != 16 {
        bail!(
            "config.enc tag must be 16 bytes for AES-GCM, got {}",
            tag_bytes.len()
        );
    }

    let mut payload = STANDARD
        .decode(&encrypted.payload)
        .context("config.enc payload is not valid base64")?;
    payload.extend_from_slice(&tag_bytes);

    let nonce = Nonce::from_slice(&iv_bytes);
    let decrypted_bytes = cipher
        .decrypt(nonce, payload.as_ref())
        .map_err(|_| anyhow!("Failed to decrypt config.enc. Is the master key correct?"))?;

    let config: AppConfigPayload = serde_json::from_slice(&decrypted_bytes)
        .context("Decrypted config payload is not valid JSON")?;

    Ok(config)
}

pub fn load_config(app_handle: &AppHandle, master_key: &str) -> Result<AppConfigPayload> {
    let config_path = get_config_path(app_handle)?;

    if !config_path.exists() {
        return Ok(AppConfigPayload::default());
    }

    let contents = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config at {}", config_path.display()))?;

    let encrypted: EncryptedConfigFile =
        serde_json::from_str(&contents).context("config.enc is not valid JSON")?;

    decrypt_config(&encrypted, master_key)
}

pub fn save_config(
    app_handle: &AppHandle,
    payload: &AppConfigPayload,
    master_key: &str,
) -> Result<PathBuf> {
    let config_path = get_config_path(app_handle)?;
    let encrypted = encrypt_config(payload, master_key)?;

    fs::write(&config_path, serde_json::to_string_pretty(&encrypted)?)
        .with_context(|| format!("Failed to write config to {}", config_path.display()))?;

    restrict_to_owner(&config_path).with_context(|| {
        format!(
            "Failed to restrict permissions on {}",
            config_path.display()
        )
    })?;

    Ok(config_path)
}

/// Restrict a file to owner read/write only (0600) on Unix so other local users
/// cannot read it. No-op on non-Unix, where the per-user profile directory ACLs
/// already limit access.
pub(crate) fn restrict_to_owner(path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

fn coerce_number(value: Option<&serde_json::Value>, fallback: u16) -> u16 {
    match value {
        Some(serde_json::Value::Number(n)) => n
            .as_u64()
            .and_then(|v| u16::try_from(v).ok())
            .unwrap_or(fallback),
        Some(serde_json::Value::String(s)) => s.parse().unwrap_or(fallback),
        _ => fallback,
    }
}

fn normalize_proxy_protocol(value: &str) -> ProxyProtocol {
    match value.to_lowercase().as_str() {
        "socks5" => ProxyProtocol::Socks5,
        _ => ProxyProtocol::Http,
    }
}

pub fn normalize_config_payload(payload: &serde_json::Value) -> AppConfigPayload {
    let empty_map = serde_json::Map::new();
    let obj = payload.as_object().unwrap_or(&empty_map);

    let proxy_protocol = obj
        .get("proxyProtocol")
        .and_then(|v| v.as_str())
        .map(normalize_proxy_protocol)
        .unwrap_or(ProxyProtocol::Http);

    AppConfigPayload {
        port: coerce_number(obj.get("port"), DEFAULT_LOCAL_PROXY_PORT),
        bypass_domains: obj
            .get("bypassDomains")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(|| vec!["imgur.com".to_string()]),
        proxy_protocol,
        proxy_host: obj
            .get("proxyHost")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| DEFAULT_PROXY_HOST.to_string()),
        proxy_port: coerce_number(obj.get("proxyPort"), DEFAULT_PROXY_PORT),
        pac_server_port: coerce_number(obj.get("pacServerPort"), DEFAULT_PAC_PORT),
        #[cfg(target_os = "macos")]
        network_target: obj
            .get("networkTarget")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| DEFAULT_NETWORK_TARGET.to_string()),
        username: obj
            .get("username")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default(),
        password: obj
            .get("password")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_payload() -> AppConfigPayload {
        AppConfigPayload {
            port: 8888,
            bypass_domains: vec!["example.com".to_string(), "internal.corp".to_string()],
            proxy_protocol: ProxyProtocol::Http,
            proxy_host: "proxy.example.com".to_string(),
            proxy_port: 3128,
            pac_server_port: 9000,
            #[cfg(target_os = "macos")]
            network_target: "Wi-Fi".to_string(),
            username: "user".to_string(),
            password: "hunter2".to_string(),
        }
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let original = sample_payload();
        let key = "my-master-key";

        let encrypted = encrypt_config(&original, key).expect("encrypt should succeed");
        let decrypted = decrypt_config(&encrypted, key).expect("decrypt should succeed");

        assert_eq!(decrypted.proxy_host, original.proxy_host);
        assert_eq!(decrypted.proxy_port, original.proxy_port);
        assert_eq!(decrypted.port, original.port);
        assert_eq!(decrypted.bypass_domains, original.bypass_domains);
        assert_eq!(decrypted.username, original.username);
        assert_eq!(decrypted.password, original.password);
    }

    #[test]
    fn wrong_key_fails_decryption() {
        let payload = sample_payload();
        let encrypted = encrypt_config(&payload, "correct-key").expect("encrypt should succeed");
        let result = decrypt_config(&encrypted, "wrong-key");
        assert!(result.is_err(), "decryption with wrong key should fail");
    }

    #[test]
    fn corrupted_payload_fails_decryption() {
        let payload = sample_payload();
        let mut encrypted = encrypt_config(&payload, "key").expect("encrypt should succeed");
        encrypted.payload = "bm90dmFsaWQ=".to_string(); // valid base64, but not valid ciphertext
        let result = decrypt_config(&encrypted, "key");
        assert!(
            result.is_err(),
            "decryption of corrupted payload should fail"
        );
    }

    #[test]
    fn each_encrypt_produces_unique_ciphertext() {
        let payload = sample_payload();
        let key = "same-key";
        let first = encrypt_config(&payload, key).expect("encrypt should succeed");
        let second = encrypt_config(&payload, key).expect("encrypt should succeed");
        assert_ne!(
            first.iv, second.iv,
            "each encrypt call should use a fresh IV"
        );
        assert_ne!(
            first.payload, second.payload,
            "ciphertexts should differ due to random IV"
        );
    }
}
