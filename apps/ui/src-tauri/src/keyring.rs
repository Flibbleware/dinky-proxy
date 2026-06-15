use anyhow::Result;

#[cfg(not(debug_assertions))]
use anyhow::{anyhow, Context};
#[cfg(not(debug_assertions))]
use keyring::Entry;

#[cfg(not(debug_assertions))]
const SERVICE_NAME: &str = "com.dinkyproxy.client";
#[cfg(not(debug_assertions))]
const KEY_NAME: &str = "master_key";

pub struct MasterKey(pub String);

pub fn get_or_create_master_key() -> Result<String> {
    // In debug builds skip the keychain entirely — dev binaries are unsigned so
    // macOS re-prompts on every rebuild when the binary hash changes.
    //
    // SECURITY: this hardcoded key makes config.enc trivially decryptable. It is
    // compiled in ONLY under `cfg(debug_assertions)`; release builds use the
    // keychain branch below. Release artifacts MUST be built with `--release`
    // (the default for `tauri build`) so debug builds never reach end users.
    #[cfg(debug_assertions)]
    return Ok("dinkyproxy-dev-key-not-for-production".to_string());

    #[cfg(not(debug_assertions))]
    {
        let entry = Entry::new(SERVICE_NAME, KEY_NAME).context("Failed to create keyring entry")?;

        match entry.get_password() {
            Ok(key) => Ok(key),
            Err(keyring::Error::NoEntry) => {
                let master_key = generate_master_key();
                entry
                    .set_password(&master_key)
                    .context("Failed to store master key in keychain")?;
                Ok(master_key)
            }
            Err(e) => Err(anyhow!("Failed to retrieve master key from keychain: {e}")),
        }
    }
}

#[cfg(not(debug_assertions))]
fn generate_master_key() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}
