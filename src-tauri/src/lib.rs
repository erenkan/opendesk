mod ble;
mod commands;
mod events;
mod notification;
mod panel;
mod reminder;
mod state;
mod tray;

use tauri::Manager;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri_plugin_log::{Target, TargetKind};

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .level_for("opendesk_lib", log::LevelFilter::Debug)
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init());

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder = builder
        .invoke_handler(tauri::generate_handler![
            commands::scan_and_connect,
            commands::scan_devices,
            commands::connect_device,
            commands::disconnect_desk,
            commands::pause_session,
            commands::resume_session,
            commands::move_up_start,
            commands::move_down_start,
            commands::move_stop,
            commands::move_to,
            commands::get_status,
            commands::reminder_start,
            commands::reminder_stop,
            commands::reminder_state,
            commands::send_native_notification,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                // `LSUIElement`/Accessory apps aren't represented in the dock,
                // so macOS falls back to a generic bell glyph in the
                // Notification Center banner. Pointing NSApp at our bundle
                // icon fixes that.
                set_ns_app_icon(app.handle());
            }

            app.manage(AppState::new(app.handle().clone()));

            tray::create(&app.handle().clone())?;

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                AppState::run_reconnect_loop(handle).await;
            });

            Ok(())
        });

    // Non-macOS: plain window + focus-lost hide. Also emits the visibility
    // event so `useAutoSession` pauses the BLE stream — macOS gets the same
    // event via `window_did_resign_key` in `panel.rs`.
    #[cfg(not(target_os = "macos"))]
    {
        builder = builder.on_window_event(|window, event| {
            if let tauri::WindowEvent::Focused(false) = event {
                if window.label() == "main" {
                    let _ = window.hide();
                    let _ = tauri::Emitter::emit(
                        window.app_handle(),
                        events::EVT_PANEL_VISIBILITY,
                        false,
                    );
                }
            }
        });
    }

    builder
        .run(tauri::generate_context!())
        .expect("failed to run OpenDesk");
}

#[cfg(target_os = "macos")]
fn set_ns_app_icon(app: &tauri::AppHandle) {
    use objc2::AllocAnyThread;
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::{MainThreadMarker, NSString};
    use tauri::path::BaseDirectory;

    // Prefer the bundled .icns at Resources/; fall back to icon.png.
    let Some(resource) = ["icon.icns", "icon.png"]
        .iter()
        .find_map(|name| app.path().resolve(name, BaseDirectory::Resource).ok())
        .filter(|p| p.exists())
    else {
        log::warn!("set_ns_app_icon: no icon.icns or icon.png in Resources/");
        return;
    };
    let Some(path_str) = resource.to_str() else {
        return;
    };

    let Some(mtm) = MainThreadMarker::new() else {
        log::warn!("set_ns_app_icon: not on main thread, skipping");
        return;
    };

    unsafe {
        let ns_path = NSString::from_str(path_str);
        let Some(image) = NSImage::initWithContentsOfFile(NSImage::alloc(), &ns_path) else {
            log::warn!("set_ns_app_icon: NSImage init failed for {path_str}");
            return;
        };
        NSApplication::sharedApplication(mtm).setApplicationIconImage(Some(&image));
        log::info!("set_ns_app_icon: NSApp icon set from {path_str}");
    }
}
