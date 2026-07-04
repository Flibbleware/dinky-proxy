use crate::config::Config;
use anyhow::Result;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) use super::AppliedPacSettings;

/// Placeholder on platforms without PAC support; never constructed because
/// `apply_pac_settings` always errors there.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) struct AppliedPacSettings;

// `_config` prefix suppresses the unused variable warning on Linux, where neither
// cfg block compiles and the parameter goes unreferenced.
pub fn apply_pac_settings(_config: &Config) -> Result<AppliedPacSettings> {
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

pub fn remove_pac_settings(_applied: &AppliedPacSettings) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        return super::remove_pac_settings_macos(_applied);
    }

    #[cfg(target_os = "windows")]
    {
        return super::remove_pac_settings_windows(_applied);
    }

    // Default if neither Windows nor macOS
    #[allow(unreachable_code)]
    Err(anyhow::anyhow!("PAC not supported on this OS"))
}
