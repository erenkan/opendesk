use std::sync::Arc;
use std::time::Duration;

use btleplug::api::Peripheral as _;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;
use tokio::time;

use crate::ble::manager::{scan_for_address, state_log_label, ConnectionState};
use crate::ble::BleController;
use crate::events::{DeskErrorPayload, EVT_ERROR};
use crate::reminder::ReminderController;

/// How many back-to-back reconnect attempts the watchdog makes before
/// giving up and parking at `Disconnected` until the user reconnects
/// manually.
const MAX_RECONNECT_ATTEMPTS: u32 = 5;

pub struct AppState {
    pub controller: Arc<Mutex<BleController>>,
    pub reminder: Arc<ReminderController>,
}

impl AppState {
    pub fn new(app: AppHandle) -> Self {
        Self {
            controller: Arc::new(Mutex::new(BleController::new(app))),
            reminder: Arc::new(ReminderController::new()),
        }
    }

    /// Background watchdog. macOS CoreBluetooth goes silent on sleep/wake —
    /// no `didDisconnectPeripheral` fires, so the app thinks it's still
    /// Connected while the link is dead. We poll `is_connected()` every 5 s
    /// (3 s timeout to survive a stale CBPeripheral that hangs on query per
    /// btleplug #25), emit `Reconnecting { attempt }` so the UI can show
    /// progress, then drive a reconnect.
    ///
    /// Lock discipline — the reconnect runs in three phases so user
    /// commands (especially `disconnect`) stay responsive during the ~6 s
    /// scan window:
    ///   (A) short lock: mark Reconnecting + force_stale
    ///   (B) no lock:     adapter scan for the last known address
    ///   (C) short lock: install the peripheral (connect + discover +
    ///                   subscribe + DPG handshake)
    ///
    /// On `MAX_RECONNECT_ATTEMPTS` back-to-back failures we park at
    /// `Disconnected` + surface a toast — the user drives the next retry.
    pub async fn run_reconnect_loop(app: AppHandle) {
        let ctrl_arc = {
            let state = app.state::<AppState>();
            state.controller.clone()
        };

        let mut ticker = time::interval(Duration::from_secs(5));
        ticker.tick().await; // skip the immediate first fire

        let mut attempt: u32 = 0;

        loop {
            ticker.tick().await;

            // Short lock: probe whether we think we're Connected and
            // snapshot the peripheral handle for an off-lock liveness check.
            let snapshot = {
                let ctrl = ctrl_arc.lock().await;
                match ctrl.state() {
                    ConnectionState::Connected { address, .. } => {
                        ctrl.peripheral_clone().map(|p| (p, address.clone()))
                    }
                    _ => None,
                }
            };
            let Some((peripheral, last_addr)) = snapshot else {
                attempt = 0;
                continue;
            };

            let alive = match time::timeout(
                Duration::from_secs(3),
                peripheral.is_connected(),
            )
            .await
            {
                Ok(Ok(v)) => v,
                Ok(Err(err)) => {
                    log::warn!("watchdog: is_connected() error: {err}");
                    false
                }
                Err(_) => {
                    log::warn!("watchdog: is_connected() timed out on stale handle");
                    false
                }
            };
            if alive {
                attempt = 0;
                continue;
            }

            attempt = attempt.saturating_add(1);
            log::warn!(
                "watchdog: BLE link stale (attempt {attempt}/{MAX_RECONNECT_ATTEMPTS}), reconnecting"
            );
            log::debug!("watchdog: reconnect target address = {last_addr}");

            // Phase A — short lock: flip state to Reconnecting + tear down
            // the stale handle.
            {
                let mut ctrl = ctrl_arc.lock().await;
                // User may have fixed it while we were probing.
                if matches!(ctrl.state(), ConnectionState::Connected { .. }) {
                    attempt = 0;
                    continue;
                }
                ctrl.force_stale().await;
                ctrl.mark_reconnecting(attempt);
            }

            // Phase B — no lock: slow adapter scan. `disconnect()` and other
            // commands can interleave here.
            let peripheral = match scan_for_address(&last_addr).await {
                Ok(p) => p,
                Err(e) => {
                    log::warn!("watchdog: scan for {last_addr} failed: {e}");
                    handle_reconnect_failure(&ctrl_arc, &app, &mut attempt).await;
                    continue;
                }
            };

            // Phase C — short lock: install. Guard against the user having
            // manually changed state during Phase B (they could have called
            // `disconnect` or `connect_to` a different device).
            let install_result = {
                let mut ctrl = ctrl_arc.lock().await;
                if !matches!(ctrl.state(), ConnectionState::Reconnecting { .. }) {
                    log::info!(
                        "watchdog: state changed during scan ({}), aborting install",
                        state_log_label(ctrl.state())
                    );
                    attempt = 0;
                    continue;
                }
                ctrl.install_peripheral(peripheral).await
            };

            match install_result {
                Ok(info) => {
                    log::info!(
                        "watchdog: reconnected to {} on attempt {attempt}",
                        info.device
                    );
                    attempt = 0;
                }
                Err(e) => {
                    log::warn!("watchdog: install failed: {e}");
                    handle_reconnect_failure(&ctrl_arc, &app, &mut attempt).await;
                }
            }
        }
    }
}

/// Common fail handler for Phase B/C — after `MAX_RECONNECT_ATTEMPTS` we
/// park at Disconnected + emit a toast so the user knows to retry
/// manually. Below that, we just reset to Disconnected and let the next
/// tick drive another attempt.
async fn handle_reconnect_failure(
    ctrl_arc: &Arc<Mutex<BleController>>,
    app: &AppHandle,
    attempt: &mut u32,
) {
    let exhausted = *attempt >= MAX_RECONNECT_ATTEMPTS;
    {
        let mut ctrl = ctrl_arc.lock().await;
        // Respect a user override that happened concurrently.
        if matches!(ctrl.state(), ConnectionState::Connected { .. }) {
            *attempt = 0;
            return;
        }
        ctrl.force_stale().await;
    }
    if exhausted {
        log::error!(
            "watchdog: giving up after {attempt} attempts — parking Disconnected",
            attempt = *attempt
        );
        let _ = app.emit(
            EVT_ERROR,
            DeskErrorPayload {
                code: "reconnect_exhausted".into(),
                message: format!(
                    "Could not reconnect after {} attempts. Please reconnect manually.",
                    *attempt
                ),
                recoverable: true,
            },
        );
        *attempt = 0;
    }
}

