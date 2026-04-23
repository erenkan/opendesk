use tauri::{AppHandle, Manager, State};

use crate::ble::{
    manager::{DeviceInfo, MoveDir, StatusSnapshot},
    DeskError,
};
use crate::reminder::ReminderState;
use crate::state::AppState;

#[tauri::command]
pub async fn scan_and_connect(state: State<'_, AppState>) -> Result<DeviceInfo, DeskError> {
    state.controller.lock().await.scan_and_connect().await
}

/// Upper bound on BLE scan duration. Caller-provided values are clamped
/// to [SCAN_MIN_MS, SCAN_MAX_MS] so a malicious webview can't hold the
/// controller mutex indefinitely with `u64::MAX`.
const SCAN_MIN_MS: u64 = 500;
const SCAN_MAX_MS: u64 = 30_000;
const SCAN_DEFAULT_MS: u64 = 4_000;

#[tauri::command]
pub async fn scan_devices(
    state: State<'_, AppState>,
    duration_ms: Option<u64>,
) -> Result<Vec<DeviceInfo>, DeskError> {
    let duration = duration_ms
        .unwrap_or(SCAN_DEFAULT_MS)
        .clamp(SCAN_MIN_MS, SCAN_MAX_MS);
    state
        .controller
        .lock()
        .await
        .scan_devices(duration)
        .await
}

#[tauri::command]
pub async fn connect_device(
    state: State<'_, AppState>,
    address: String,
) -> Result<DeviceInfo, DeskError> {
    state.controller.lock().await.connect_to(address).await
}

#[tauri::command]
pub async fn disconnect_desk(state: State<'_, AppState>) -> Result<(), DeskError> {
    state.controller.lock().await.disconnect().await
}

#[tauri::command]
pub async fn pause_session(state: State<'_, AppState>) -> Result<(), DeskError> {
    state.controller.lock().await.pause_session().await
}

#[tauri::command]
pub async fn resume_session(state: State<'_, AppState>) -> Result<(), DeskError> {
    state.controller.lock().await.resume_session().await
}

#[tauri::command]
pub async fn move_up_start(state: State<'_, AppState>) -> Result<(), DeskError> {
    state.controller.lock().await.start_move(MoveDir::Up).await
}

#[tauri::command]
pub async fn move_down_start(state: State<'_, AppState>) -> Result<(), DeskError> {
    state
        .controller
        .lock()
        .await
        .start_move(MoveDir::Down)
        .await
}

#[tauri::command]
pub async fn move_stop(state: State<'_, AppState>) -> Result<(), DeskError> {
    state.controller.lock().await.stop_move().await
}

#[tauri::command]
pub async fn move_to(state: State<'_, AppState>, height_cm: f32) -> Result<(), DeskError> {
    state.controller.lock().await.move_to(height_cm).await
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<StatusSnapshot, DeskError> {
    Ok(state.controller.lock().await.snapshot())
}

#[tauri::command]
pub async fn reminder_start(
    app: AppHandle,
    state: State<'_, AppState>,
    mins: u32,
) -> Result<ReminderState, DeskError> {
    Ok(state.reminder.clone().start(app, mins).await)
}

#[tauri::command]
pub async fn reminder_stop(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), DeskError> {
    state.reminder.clone().stop_and_emit(app).await;
    Ok(())
}

#[tauri::command]
pub async fn reminder_state(
    state: State<'_, AppState>,
) -> Result<ReminderState, DeskError> {
    Ok(state.reminder.snapshot().await)
}

#[tauri::command]
pub async fn send_native_notification(
    app: AppHandle,
    title: String,
    body: String,
) -> Result<(), String> {
    crate::notification::send(&app, &title, &body)
}
