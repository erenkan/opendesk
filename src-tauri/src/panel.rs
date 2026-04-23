//! macOS NSPanel popover — openusage `v2.1` pattern.
//!
//! A plain Tauri window steals focus on `show()` and fires `Focused(false)`
//! spuriously on cmd-tab. NSPanel with `nonactivating_panel` style and a
//! `window_did_resign_key` handler behaves like Raycast / Itsycal.
//!
//! On non-macOS targets this module exposes stubs so the tray module can call
//! `show` / `toggle` unconditionally — the non-macOS code path lives in
//! `tray::show_popover_other`.

#[cfg(target_os = "macos")]
pub use imp::{show, toggle};

#[cfg(not(target_os = "macos"))]
pub fn show(_app: &tauri::AppHandle) {}

#[cfg(not(target_os = "macos"))]
pub fn toggle(_app: &tauri::AppHandle) {}

#[cfg(target_os = "macos")]
mod imp {
    // `tauri_panel!` DSL requires the explicit `-> ()` return type on
    // event handlers and macro-generated impls trip several clippy lints
    // we can't address from outside the macro.
    #![allow(clippy::unused_unit, clippy::let_unit_value)]

    use tauri::{AppHandle, Manager, Position, Size};
    use tauri_nspanel::{
        tauri_panel, CollectionBehavior, ManagerExt, PanelLevel, StyleMask, WebviewWindowExt,
    };

    tauri_panel! {
        panel!(OpenDeskPanel {
            config: {
                can_become_key_window: true,
                is_floating_panel: true
            }
        })

        panel_event!(OpenDeskPanelEventHandler {
            window_did_resign_key(notification: &NSNotification) -> ()
        })
    }

    fn ensure(app: &AppHandle) -> tauri::Result<()> {
        if app.get_webview_panel("main").is_ok() {
            return Ok(());
        }
        let window = app
            .get_webview_window("main")
            .expect("main window must exist");
        let panel = window.to_panel::<OpenDeskPanel>()?;

        panel.set_has_shadow(false);
        panel.set_opaque(false);
        panel.set_level(PanelLevel::MainMenu.value() + 1);
        panel.set_collection_behavior(
            CollectionBehavior::new()
                .move_to_active_space()
                .full_screen_auxiliary()
                .value(),
        );
        panel.set_style_mask(StyleMask::empty().nonactivating_panel().value());

        // Hide-on-blur. NSPanel resign-key fires only when the user actually
        // clicks outside, not on cmd-tab — the whole reason we picked NSPanel.
        let handler = OpenDeskPanelEventHandler::new();
        let cloned = app.clone();
        handler.window_did_resign_key(move |_notification| {
            if let Ok(p) = cloned.get_webview_panel("main") {
                p.hide();
                let _ = tauri::Emitter::emit(&cloned, crate::events::EVT_PANEL_VISIBILITY, false);
            }
        });
        panel.set_event_handler(Some(handler.as_ref()));
        Ok(())
    }

    pub fn show(app: &AppHandle) {
        if let Err(e) = ensure(app) {
            log::error!("nspanel ensure failed: {e}");
            return;
        }
        if let Ok(panel) = app.get_webview_panel("main") {
            panel.show_and_make_key();
            position_from_tray(app);
            let _ = tauri::Emitter::emit(app, crate::events::EVT_PANEL_VISIBILITY, true);
        }
    }

    pub fn toggle(app: &AppHandle) {
        if let Err(e) = ensure(app) {
            log::error!("nspanel ensure failed: {e}");
            return;
        }
        let Ok(panel) = app.get_webview_panel("main") else {
            return;
        };
        if panel.is_visible() {
            panel.hide();
            let _ = tauri::Emitter::emit(app, crate::events::EVT_PANEL_VISIBILITY, false);
        } else {
            panel.show_and_make_key();
            position_from_tray(app);
            let _ = tauri::Emitter::emit(app, crate::events::EVT_PANEL_VISIBILITY, true);
        }
    }

    fn position_from_tray(app: &AppHandle) {
        let Some(tray) = app.tray_by_id("tray") else {
            return;
        };
        if let Ok(Some(rect)) = tray.rect() {
            position_at_tray(app, rect.position, rect.size)
        }
    }

    unsafe fn set_panel_top_left(panel: &tauri_nspanel::NSPanel, x: f64, y: f64) {
        let point = tauri_nspanel::NSPoint::new(x, y);
        let _: () = objc2::msg_send![panel, setFrameTopLeftPoint: point];
    }

    fn contains(origin_x: f64, origin_y: f64, w: f64, h: f64, px: f64, py: f64) -> bool {
        px >= origin_x && px < origin_x + w && py >= origin_y && py < origin_y + h
    }

    /// Place panel so its top-centre aligns with the tray icon's bottom-centre.
    /// Multi-monitor + HiDPI safe (openusage's algorithm).
    fn position_at_tray(app: &AppHandle, icon_pos: Position, icon_size: Size) {
        let Some(window) = app.get_webview_window("main") else {
            return;
        };

        let (icon_x, icon_y) = match icon_pos {
            Position::Physical(p) => (p.x as f64, p.y as f64),
            Position::Logical(p) => (p.x, p.y),
        };
        let (icon_w, icon_h) = match icon_size {
            Size::Physical(s) => (s.width as f64, s.height as f64),
            Size::Logical(s) => (s.width, s.height),
        };

        let Ok(monitors) = window.available_monitors() else {
            return;
        };
        let primary_logical_h = window
            .primary_monitor()
            .ok()
            .flatten()
            .map(|m| m.size().height as f64 / m.scale_factor())
            .unwrap_or(0.0);

        let center_x = icon_x + icon_w / 2.0;
        let center_y = icon_y + icon_h / 2.0;

        let monitor = monitors
            .iter()
            .find(|m| {
                let o = m.position();
                let s = m.size();
                contains(
                    o.x as f64,
                    o.y as f64,
                    s.width as f64,
                    s.height as f64,
                    center_x,
                    center_y,
                )
            })
            .cloned()
            .or_else(|| window.primary_monitor().ok().flatten());
        let Some(monitor) = monitor else {
            return;
        };

        let scale = monitor.scale_factor();
        let mon_px = monitor.position().x as f64;
        let mon_py = monitor.position().y as f64;
        let mon_lx = mon_px / scale;
        let mon_ly = mon_py / scale;

        let icon_lx = mon_lx + (icon_x - mon_px) / scale;
        let icon_ly = mon_ly + (icon_y - mon_py) / scale;
        let icon_lw = icon_w / scale;
        let icon_lh = icon_h / scale;

        let panel_width = window
            .outer_size()
            .and_then(|s| window.scale_factor().map(|f| s.width as f64 / f))
            .unwrap_or(340.0);

        // The 10px transparent padding inside the window (App.tsx) already
        // gives the visible card a breathing gap below the menubar. A
        // negative offset overlaps the tray icon area with our window's
        // transparent margin so the visible card lands the same ~6px below
        // the menubar as before the padding refactor.
        let gap_below_icon = -4.0;
        let panel_x = icon_lx + (icon_lw / 2.0) - (panel_width / 2.0);
        let panel_y = icon_ly + icon_lh + gap_below_icon;

        let target_x = panel_x;
        let target_y = primary_logical_h - panel_y;

        if let Ok(panel) = app.get_webview_panel("main") {
            let clone = panel.clone();
            if objc2_foundation::MainThreadMarker::new().is_some() {
                unsafe {
                    set_panel_top_left(clone.as_panel(), target_x, target_y);
                }
                return;
            }
            let (tx, rx) = std::sync::mpsc::channel();
            let _ = window.run_on_main_thread(move || {
                unsafe {
                    set_panel_top_left(clone.as_panel(), target_x, target_y);
                }
                let _ = tx.send(());
            });
            let _ = rx.recv();
        }
    }
}
