use crate::config::Config;
use anyhow::Result;

// `_config` prefix suppresses the unused variable warning on Linux, where neither
// cfg block compiles and the parameter goes unreferenced.
pub fn apply_pac_settings(_config: &Config) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        return super::apply_pac_settings_macos(_config);
    }

    #[cfg(target_os = "windows")]
    {
        return super::apply_pac_settings_windows(_config);
    }

    // Default if neither Windows nor macOS
    #[allow(unreachable_code)]
    Err(anyhow::anyhow!("PAC not supported on this OS"))
}

pub fn remove_pac_settings() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        return super::remove_pac_settings_macos();
    }

    #[cfg(target_os = "windows")]
    {
        return super::remove_pac_settings_windows();
    }

    // Default if neither Windows nor macOS
    #[allow(unreachable_code)]
    Err(anyhow::anyhow!("PAC not supported on this OS"))
}
