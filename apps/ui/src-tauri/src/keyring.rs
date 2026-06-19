use anyhow::Result;

#[cfg(not(debug_assertions))]
use anyhow::{anyhow, Context};
#[cfg(not(debug_assertions))]
use keyring_core::Entry;

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
        use apple_native_keyring_store::keychain;

        // keyring-core is backend-agnostic: a credential store must be registered
        // as the default before any `Entry` can be used, otherwise every operation
        // fails. Register the macOS login keychain store. (The `keychain` module is
        // the correct one for desktop apps that are not signed with a provisioning
        // profile; the `protected` store would fail with a missing-entitlement error.)
        keyring_core::set_default_store(
            keychain::Store::new().context("Failed to initialize keychain store")?,
        );

        let entry = Entry::new(SERVICE_NAME, KEY_NAME).context("Failed to create keyring entry")?;

        match entry.get_password() {
            Ok(key) => Ok(key),
            Err(keyring_core::Error::NoEntry) => {
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
