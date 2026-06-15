// src/pac_settings/mod.rs

// OS-specific implementations
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

// Re-export ONLY the implementation for the current OS
#[cfg(target_os = "macos")]
use macos::*;

#[cfg(target_os = "windows")]
use windows::*;

// Shared cross-platform API
pub mod commands;
