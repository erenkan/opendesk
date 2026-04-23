use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::path::BaseDirectory;
use tauri::tray::{MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let icon = load_icon(app)?;

    let show = MenuItem::with_id(app, "show", "Show OpenDesk", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit OpenDesk", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &sep, &quit])?;

    TrayIconBuilder::with_id("tray")
        .icon(icon)
        .icon_as_template(true)
        .tooltip("OpenDesk")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_popover(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                toggle_popover(tray.app_handle(), Some(rect));
            }
        })
        .build(app)?;
    Ok(())
}

fn load_icon(app: &AppHandle) -> tauri::Result<Image<'static>> {
    // Prefer the bundled resource at runtime; during dev fall back to the
    // icon we include at compile time so builds never depend on pnpm
    // having copied resources first.
    if let Ok(path) = app
        .path()
        .resolve("icons/tray-icon.png", BaseDirectory::Resource)
    {
        if path.exists() {
            return Image::from_path(path);
        }
    }
    Ok(Image::from_bytes(include_bytes!(
        "../icons/tray-icon.png"
    ))?)
}

#[cfg(target_os = "macos")]
fn show_popover(app: &AppHandle) {
    crate::panel::show(app);
}

#[cfg(target_os = "macos")]
fn toggle_popover(app: &AppHandle, _rect: Option<tauri::Rect>) {
    crate::panel::toggle(app);
}

#[cfg(not(target_os = "macos"))]
fn show_popover(app: &AppHandle) {
    show_popover_other(app, None);
}

#[cfg(not(target_os = "macos"))]
fn toggle_popover(app: &AppHandle, rect: Option<tauri::Rect>) {
    let Some(w) = app.get_webview_window("main") else {
        return;
    };
    if w.is_visible().unwrap_or(false) {
        let _ = w.hide();
        let _ = tauri::Emitter::emit(app, crate::events::EVT_PANEL_VISIBILITY, false);
    } else {
        show_popover_other(app, rect);
    }
}

#[cfg(not(target_os = "macos"))]
fn show_popover_other(app: &AppHandle, rect: Option<tauri::Rect>) {
    let Some(w) = app.get_webview_window("main") else {
        return;
    };

    if let Some(rect) = rect {
        if let Some(pos) = place_under_tray_rect(&w, &rect) {
            let _ = w.set_position(pos);
        }
    }
    let _ = w.show();
    let _ = w.set_focus();
    let _ = tauri::Emitter::emit(app, crate::events::EVT_PANEL_VISIBILITY, true);
}

#[cfg(not(target_os = "macos"))]
fn place_under_tray_rect(
    window: &tauri::WebviewWindow,
    rect: &tauri::Rect,
) -> Option<tauri::Position> {
    use tauri::{PhysicalPosition, Position as TPosition};

    let (ix, iy, iw, _ih) = match (rect.position, rect.size) {
        (tauri::Position::Physical(p), tauri::Size::Physical(s)) => {
            (p.x as f64, p.y as f64, s.width as f64, s.height as f64)
        }
        (tauri::Position::Logical(p), tauri::Size::Logical(s)) => {
            let scale = window.scale_factor().unwrap_or(1.0);
            (p.x * scale, p.y * scale, s.width * scale, s.height * scale)
        }
        _ => return None,
    };

    let outer = window.outer_size().ok()?;
    let panel_w = outer.width as f64;

    let center_x = ix + iw / 2.0;
    let x = (center_x - panel_w / 2.0).round() as i32;
    // Windows tray is usually at bottom — prefer placing ABOVE the icon.
    let y = (iy - outer.height as f64 - 6.0).max(0.0).round() as i32;

    Some(TPosition::Physical(PhysicalPosition::new(x, y)))
}
