use anyhow::{bail, Result};
use std::process::Command;

use crate::config::Config;

#[cfg(target_os = "macos")]
fn get_active_network_service() -> Result<String> {
    //
    // STEP 1: Get the default network interface from `route -n get default`
    //
    let output = Command::new("route")
        .args(["-n", "get", "default"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut default_iface: Option<String> = None;

    for line in stdout.lines() {
        if let Some(iface) = line.trim_start().strip_prefix("interface:") {
            default_iface = Some(iface.trim().to_string());
            break;
        }
    }

    let default_iface = default_iface
        .ok_or_else(|| anyhow::anyhow!("Could not determine default network interface"))?;

    //
    // STEP 2: Map interface → readable name using `networksetup -listallhardwareports`
    //
    let ports = Command::new("networksetup")
        .arg("-listallhardwareports")
        .output()?;

    let ports_out = String::from_utf8_lossy(&ports.stdout);

    // Example block:
    //
    // Hardware Port: Wi-Fi
    // Device: en0
    // Ethernet Address: xx:xx:xx
    //
    let mut last_port: Option<String> = None;

    for line in ports_out.lines() {
        if let Some(stripped) = line.strip_prefix("Hardware Port:") {
            last_port = Some(stripped.trim().to_string());
        } else if let Some(stripped) = line.strip_prefix("Device:") {
            let dev = stripped.trim();

            if dev == default_iface {
                return Ok(last_port.unwrap_or(default_iface));
            }
        }
    }

    // If not found in the hardware port list, just return the interface name
    Ok(default_iface)
}

#[cfg(target_os = "macos")]
pub(super) fn apply_pac_settings_macos(config: &Config) -> Result<()> {
    let pac_url = config.pac_url();

    let service = get_active_network_service()?;
    println!("[PAC] Service found: {}", service);

    println!(
        "[PAC] Applying system PAC URL for '{}' -> {}",
        service, pac_url
    );

    for args in [
        vec!["networksetup", "-setautoproxyurl", &service, &pac_url],
        vec!["networksetup", "-setautoproxystate", &service, "off"],
        vec!["networksetup", "-setautoproxystate", &service, "on"],
    ] {
        let status = Command::new(args[0]).args(&args[1..]).status()?;
        if !status.success() {
            bail!(
                "Command {:?} failed with status {}",
                args,
                status.code().unwrap_or(-1)
            );
        } else {
            println!(
                "[PAC] Successfully ran {:?} for network service '{}'",
                &args[..args.len().saturating_sub(1)],
                service
            );
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub(super) fn remove_pac_settings_macos() -> Result<()> {
    let service = get_active_network_service()?;

    println!("[PAC] Removing system PAC settings for '{}'", service);

    // macOS does not accept an empty URL for -setautoproxyurl, so we only disable
    // the proxy state. The stale URL is inert while auto-proxy is off.
    let disable_args = ["networksetup", "-setautoproxystate", &service, "off"];
    let disable_status = Command::new(disable_args[0])
        .args(&disable_args[1..])
        .status()?;
    if !disable_status.success() {
        bail!(
            "Command {:?} failed with status {}",
            disable_args,
            disable_status.code().unwrap_or(-1)
        );
    }
    println!("[PAC] Auto-proxy disabled for '{}'", service);

    Ok(())
}
