use crate::config::Config;
use anyhow::{bail, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};

#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

// Hand-written so the password can never be leaked through a `{:?}` in a log or
// error chain. The derived `Debug` would print it verbatim.
impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .finish()
    }
}

pub fn load_credentials(config: &Config) -> Result<Credentials> {
    if config.username.trim().is_empty() {
        bail!("Config value 'username' is empty. Please set it in the desktop app.");
    }

    if config.password.trim().is_empty() {
        bail!("Config value 'password' is empty. Please set it in the desktop app.");
    }

    Ok(Credentials {
        username: config.username.clone(),
        password: config.password.clone(),
    })
}

// SECURITY: HTTP "Basic" auth is base64, not encryption — it is trivially
// reversible. This header (and the SOCKS5 username/password handshake) is sent in
// cleartext to the upstream proxy, which is assumed to sit on a trusted network
// which is the responsibility of the user to trust.
pub fn build_auth_header(credentials: &Credentials) -> String {
    format!(
        "Basic {}",
        STANDARD.encode(format!("{}:{}", credentials.username, credentials.password))
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ProxyProtocol};
    use std::path::PathBuf;

    fn config(username: &str, password: &str) -> Config {
        Config {
            proxy_host: String::new(),
            proxy_port: 8080,
            proxy_protocol: ProxyProtocol::Http,
            local_proxy_port: 8888,
            pac_port: 8000,
            bypass_domains: vec![],
            username: username.to_string(),
            password: password.to_string(),
            base_dir: PathBuf::from("/tmp"),
        }
    }

    #[test]
    fn auth_header_is_correctly_encoded() {
        let creds = Credentials {
            username: "user".to_string(),
            password: "pass".to_string(),
        };
        assert_eq!(build_auth_header(&creds), "Basic dXNlcjpwYXNz");
    }

    #[test]
    fn empty_username_errors() {
        assert!(load_credentials(&config("", "test-password")).is_err());
    }

    #[test]
    fn whitespace_username_errors() {
        assert!(load_credentials(&config("   ", "test-password")).is_err());
    }

    #[test]
    fn empty_password_errors() {
        assert!(load_credentials(&config("user", "")).is_err());
    }

    #[test]
    fn valid_credentials_returns_ok() {
        // dismiss false positive security warning about hardcoded password in test
        let result = load_credentials(&config("user", "test-password"));
        assert!(result.is_ok());
        let creds = result.unwrap();
        assert_eq!(creds.username, "user");
        assert_eq!(creds.password, "test-password");
    }
}
