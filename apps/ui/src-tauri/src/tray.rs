use crate::server::ServerManager;
use image::{Rgba, RgbaImage};
use std::sync::OnceLock;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    AppHandle, Manager,
};

pub(crate) fn build_tray_menu(
    app: &AppHandle,
    toggle_text: &str,
) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::new(app)?;
    menu.append(&MenuItem::with_id(
        app,
        "toggle_server",
        toggle_text,
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        "configuration",
        "Settings",
        true,
        None::<&str>,
    )?)?;
    let separator = PredefinedMenuItem::separator(app)?;
    menu.append(&separator)?;
    menu.append(&MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?)?;
    Ok(menu)
}

fn render_icon(is_active: bool) -> RgbaImage {
    let icon_bytes = include_bytes!("../../public/pac_logo.png");
    // `expect` is safe: the PNG is embedded at compile time, so a decode
    // failure is a build defect, not a runtime condition.
    let mut img = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon")
        .to_rgba8();

    if is_active {
        let (width, height) = img.dimensions();
        let radius = width / 6;
        let cx = width - radius - (width / 10);
        let cy = height - radius - (height / 10);
        let radius_sq = (radius * radius) as i32;

        for x in 0..width {
            for y in 0..height {
                let dx = x as i32 - cx as i32;
                let dy = y as i32 - cy as i32;
                if dx * dx + dy * dy <= radius_sq {
                    img.put_pixel(x, y, Rgba([34, 197, 94, 255]));
                }
            }
        }
    }

    img
}

/// Both icon variants are rendered once and cached; tray updates on every
/// start/stop would otherwise re-decode the PNG and redraw the badge each time.
fn icon_image(is_active: bool) -> &'static RgbaImage {
    static INACTIVE: OnceLock<RgbaImage> = OnceLock::new();
    static ACTIVE: OnceLock<RgbaImage> = OnceLock::new();
    if is_active {
        ACTIVE.get_or_init(|| render_icon(true))
    } else {
        INACTIVE.get_or_init(|| render_icon(false))
    }
}

pub fn get_app_icon(is_active: bool) -> Image<'static> {
    let img = icon_image(is_active);
    let (width, height) = img.dimensions();
    Image::new_owned(img.as_raw().clone(), width, height)
}

pub async fn update_tray_state(app: &AppHandle) {
    // Recreate the menu with updated text
    let server_manager = app.state::<ServerManager>();
    let is_running = server_manager.is_running().await;
    let text = if is_running { "Disable" } else { "Enable" };

    // Get the tray by ID and update its menu
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_icon(Some(get_app_icon(is_running)));
        if let Ok(menu) = build_tray_menu(app, text) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}
