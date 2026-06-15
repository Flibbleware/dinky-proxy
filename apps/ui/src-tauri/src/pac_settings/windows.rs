use crate::config::Config;
use anyhow::{bail, Result};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(target_os = "windows")]
pub(super) fn apply_pac_settings_windows(config: &Config) -> Result<()> {
    let pac_url = config.pac_url();

    println!("[PAC][Windows] Applying PAC URL -> {}", pac_url);

    let reg_path = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings";

    // -------------------------
    // 1. Disable "Automatically detect settings"
    // -------------------------
    run_reg_checked(&[
        "add",
        reg_path,
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
        reg_path,
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
    Command::new("ipconfig")
        .args(["/flushdns"])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .ok();

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

#[cfg(target_os = "windows")]
pub(super) fn remove_pac_settings_windows() -> anyhow::Result<()> {
    println!("[PAC][Windows] Removing ALL PAC settings…");

    //
    // 1. Reset Network config via registry
    //
    run_reg_edit(&[
        "delete",
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
        "/v",
        "AutoConfigURL",
        "/f",
    ]);
    run_reg_edit(&[
        "add",
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
        "/v",
        "AutoDetect",
        "/t",
        "REG_DWORD",
        "/d",
        "0",
        "/f",
    ]);
    run_reg_edit(&[
        "delete",
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings\Wpad",
        "/f",
    ]);
    run_reg_edit(&[
        "delete",
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings\Connections",
        "/v",
        "DefaultConnectionSettings",
        "/f",
    ]);
    run_reg_edit(&[
        "delete",
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings\Connections",
        "/v",
        "SavedLegacySettings",
        "/f",
    ]);

    println!("[PAC][Windows] PAC removal complete.");

    //
    // 2. Reset WinHTTP system proxy
    //
    Command::new("netsh")
        .args(["winhttp", "reset", "proxy"])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .ok();

    println!("[PAC][Windows] WinHTTP reset.");

    // -------------------------
    // 3. Flush DNS (optional)
    // -------------------------
    Command::new("ipconfig")
        .args(["/flushdns"])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .ok();

    //
    // 4. Refresh WinINET (critical)
    //
    refresh_wininet_settings();
    println!("[PAC][Windows] WinINET refreshed.");

    Ok(())
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
