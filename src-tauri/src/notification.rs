//! Native macOS notifications via `UNUserNotificationCenter`.
//!
//! `tauri-plugin-notification` (and its underlying `notify-rust` →
//! `mac-notification-sys` chain) uses the deprecated `NSUserNotification`
//! API. On LSUIElement / Accessory apps macOS can't resolve our bundle
//! icon through that path and falls back to a generic gray bell. The
//! modern `UNUserNotificationCenter` lets us attach our own image, which
//! shows alongside the body text — at least the V1 coral mark appears
//! somewhere in the banner.
//!
//! Non-macOS targets fall through; the frontend keeps using
//! `@tauri-apps/plugin-notification` there.

use tauri::AppHandle;

#[cfg(target_os = "macos")]
pub fn send(app: &AppHandle, title: &str, body: &str) -> Result<(), String> {
    use objc2_foundation::{NSArray, NSString, NSURL};
    use objc2_user_notifications::{
        UNMutableNotificationContent, UNNotificationAttachment, UNNotificationRequest,
        UNUserNotificationCenter,
    };
    use tauri::{path::BaseDirectory, Manager};

    log::info!("native notify: begin title={title:?}");

    // Prefer PNG over ICNS — UNNotificationAttachment is documented for
    // image formats and PNG is the most portable.
    let icon_path = ["icon.png", "icon.icns"]
        .iter()
        .find_map(|name| app.path().resolve(name, BaseDirectory::Resource).ok())
        .filter(|p| p.exists())
        .ok_or_else(|| "no bundled icon found".to_string())?;
    let icon_path_str = icon_path.to_string_lossy().to_string();
    log::info!("native notify: icon path = {icon_path_str}");

    unsafe {
        let center = UNUserNotificationCenter::currentNotificationCenter();
        // Authorization is already requested via `@tauri-apps/plugin-notification`
        // on first reminder fire; both APIs target the same OS notification
        // center so that grant applies here too.

        let title_ns = NSString::from_str(title);
        let body_ns = NSString::from_str(body);

        let content = UNMutableNotificationContent::new();
        content.setTitle(&title_ns);
        content.setBody(&body_ns);

        let path_ns = NSString::from_str(&icon_path_str);
        let url = NSURL::fileURLWithPath(&path_ns);
        let attach_id = NSString::from_str("opendesk-icon");
        match UNNotificationAttachment::attachmentWithIdentifier_URL_options_error(
            &attach_id, &url, None,
        ) {
            Ok(attachment) => {
                log::info!("native notify: attachment ok");
                let attachments = NSArray::from_retained_slice(&[attachment]);
                content.setAttachments(&attachments);
            }
            Err(e) => {
                log::warn!("native notify: attachment failed, sending without icon: {e:?}");
            }
        }

        let req_id = NSString::from_str(&format!("opendesk-{}", uuid::Uuid::new_v4()));
        let request =
            UNNotificationRequest::requestWithIdentifier_content_trigger(&req_id, &content, None);

        center.addNotificationRequest_withCompletionHandler(&request, None);
        log::info!("native notify: request added");
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn send(_app: &AppHandle, _title: &str, _body: &str) -> Result<(), String> {
    // No-op — non-macOS targets stay on `@tauri-apps/plugin-notification`.
    Ok(())
}
