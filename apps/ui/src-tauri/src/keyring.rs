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
        // keyring-core is backend-agnostic: a credential store must be registered
        // as the default before any `Entry` can be used, otherwise every operation
        // fails. Each desktop OS has its own native backend, and only that platform's
        // store crate is compiled in (see the target-gated dependencies in Cargo.toml),
        // so the store is selected per `target_os`.
        //
        // macOS: the `keychain` store is the correct one for desktop apps that are not
        // signed with a provisioning profile; the `protected` store would fail with a
        // missing-entitlement error.
        #[cfg(target_os = "macos")]
        let store = apple_native_keyring_store::keychain::Store::new()
            .context("Failed to initialize macOS keychain store")?;
        #[cfg(target_os = "windows")]
        let store = windows_native_keyring_store::Store::new()
            .context("Failed to initialize Windows credential store")?;
        #[cfg(target_os = "linux")]
        let store = linux_keyutils_keyring_store::Store::new()
            .context("Failed to initialize Linux keyutils store")?;

        keyring_core::set_default_store(store);

        let entry = Entry::new(SERVICE_NAME, KEY_NAME).context("Failed to create keyring entry")?;

        match entry.get_password() {
            Ok(key) => Ok(key),
            Err(keyring_core::Error::NoEntry) => {
                let master_key = generate_master_key()?;
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
fn generate_master_key() -> Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes)
        .map_err(|err| anyhow!("Failed to generate random master key: {err}"))?;
    Ok(hex::encode(bytes))
}
