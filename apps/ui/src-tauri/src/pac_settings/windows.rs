use crate::config::Config;
use anyhow::{bail, Result};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

const REG_PATH: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings";

/// Registry state captured before the PAC settings were applied, so removal
/// restores whatever the user had (e.g. a corporate PAC or auto-detect) instead
/// of wiping their proxy configuration.
pub(crate) struct AppliedPacSettings {
    previous_auto_config_url: Option<String>,
    previous_auto_detect: Option<u32>,
}

#[cfg(target_os = "windows")]
pub(super) fn apply_pac_settings_windows(config: &Config) -> Result<AppliedPacSettings> {
    let pac_url = config.pac_url();

    println!("[PAC][Windows] Applying PAC URL -> {}", pac_url);

    let snapshot = AppliedPacSettings {
        previous_auto_config_url: query_reg_string("AutoConfigURL"),
        previous_auto_detect: query_reg_dword("AutoDetect"),
    };

    // -------------------------
    // 1. Disable "Automatically detect settings"
    // -------------------------
    run_reg_checked(&[
        "add",
        REG_PATH,
        "/v",
        "AutoDetect",
        "/t",
        "REG_DWORD",
        "/d",
        "0",
        "/f",
    ])?;
    println!("[PAC][Windows] AutoDetect disabled.");

    // -------------------------
    // 2. Set PAC URL
    // -------------------------
    run_reg_checked(&[
        "add",
        REG_PATH,
        "/v",
        "AutoConfigURL",
        "/t",
        "REG_SZ",
        "/d",
        &pac_url,
        "/f",
    ])?;
    println!("[PAC][Windows] AutoConfigURL set.");

    // -------------------------
    // 3. Flush DNS (optional)
    // -------------------------
    flush_dns();

    Ok(snapshot)
}

#[cfg(target_os = "windows")]
pub(super) fn remove_pac_settings_windows(applied: &AppliedPacSettings) -> Result<()> {
    println!("[PAC][Windows] Restoring previous PAC settings…");

    // Restore (or remove, if it wasn't set before) exactly the two values the
    // apply step overwrote. Nothing else the user had configured is touched.
    match &applied.previous_auto_config_url {
        Some(url) => run_reg_edit(&[
            "add",
            REG_PATH,
            "/v",
            "AutoConfigURL",
            "/t",
            "REG_SZ",
            "/d",
            url,
            "/f",
        ]),
        None => run_reg_edit(&["delete", REG_PATH, "/v", "AutoConfigURL", "/f"]),
    }

    match applied.previous_auto_detect {
        Some(value) => {
            let value = value.to_string();
            run_reg_edit(&[
                "add",
                REG_PATH,
                "/v",
                "AutoDetect",
                "/t",
                "REG_DWORD",
                "/d",
                &value,
                "/f",
            ]);
        }
        None => run_reg_edit(&["delete", REG_PATH, "/v", "AutoDetect", "/f"]),
    }

    println!("[PAC][Windows] Registry values restored.");

    // -------------------------
    // Flush DNS (optional)
    // -------------------------
    flush_dns();

    //
    // Refresh WinINET (critical)
    //
    refresh_wininet_settings();
    println!("[PAC][Windows] WinINET refreshed.");

    Ok(())
}

fn run_reg_checked(args: &[&str]) -> Result<()> {
    let status = Command::new("reg")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .status()?;
    if !status.success() {
        bail!("reg {:?} failed with exit code {:?}", args, status.code());
    }
    Ok(())
}

fn run_reg_edit(args: &[&str]) {
    let _ = Command::new("reg")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .status();
}

/// Read one value from the Internet Settings key. Returns `None` when the value
/// does not exist (or `reg query` fails), which removal treats as "delete the
/// value to restore the original state".
fn query_reg_value(name: &str) -> Option<(String, String)> {
    let output = Command::new("reg")
        .args(["query", REG_PATH, "/v", name])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    // Matching line looks like:  "    AutoConfigURL    REG_SZ    http://…"
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix(name) else {
            continue;
        };
        let Some((reg_type, value)) = rest.trim_start().split_once(char::is_whitespace) else {
            continue;
        };
        return Some((reg_type.to_string(), value.trim_start().to_string()));
    }
    None
}

fn query_reg_string(name: &str) -> Option<String> {
    query_reg_value(name).map(|(_, value)| value)
}

fn query_reg_dword(name: &str) -> Option<u32> {
    let (_, value) = query_reg_value(name)?;
    // `reg query` prints REG_DWORD values as hex, e.g. "0x1".
    let digits = value.trim().trim_start_matches("0x");
    u32::from_str_radix(digits, 16).ok()
}

fn flush_dns() {
    let _ = Command::new("ipconfig")
        .args(["/flushdns"])
        .creation_flags(CREATE_NO_WINDOW)
        .status();
}

//
// Internal WinINET refresh function
//
#[cfg(target_os = "windows")]
fn refresh_wininet_settings() {
    #[link(name = "wininet")]
    extern "system" {
        fn InternetSetOptionW(
            h_internet: *mut core::ffi::c_void,
            dw_option: u32,
            lp_buffer: *mut core::ffi::c_void,
            buffer_length: u32,
        ) -> i32;
    }

    const INTERNET_OPTION_SETTINGS_CHANGED: u32 = 39;
    const INTERNET_OPTION_REFRESH: u32 = 37;

    unsafe {
        InternetSetOptionW(
            core::ptr::null_mut(),
            INTERNET_OPTION_SETTINGS_CHANGED,
            core::ptr::null_mut(),
            0,
        );
        InternetSetOptionW(
            core::ptr::null_mut(),
            INTERNET_OPTION_REFRESH,
            core::ptr::null_mut(),
            0,
        );
    }
}
