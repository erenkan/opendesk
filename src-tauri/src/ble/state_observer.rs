//! Adapter state observer — the primary recovery path on macOS sleep/wake.
//!
//! Native Swift BLE apps (DeskControl, idasen-controller-mac) rely on
//! `centralManagerDidUpdateState` for sleep/wake recovery rather than polling
//! `is_connected()`. btleplug 0.12 exposes the same signal via
//! `CentralEvent::StateUpdate`, so we subscribe to a long-lived `Adapter`
//! stream and react to transitions:
//!
//! * `PoweredOff` / `Unknown` → drop peripheral refs (they become invalid the
//!   moment CoreBluetooth powers down; any later `connect()` on a cached
//!   handle silently misfires).
//! * `PoweredOn` → if we had a previous connection, rescan for that address
//!   and reinstall. Gives transparent recovery without a process relaunch.
//!
//! The polling watchdog in `state.rs` stays on as a fallback for the cases
//! where `StateUpdate` doesn't fire (CoreBluetooth XPC can silently stall),
//! but this observer is the cheap happy path.

use std::time::Duration;

use btleplug::api::{Central, CentralEvent, CentralState, Manager as _};
use btleplug::platform::Manager;
use futures::StreamExt;
use tauri::{AppHandle, Manager as _AppManager};
use tokio::time;

use crate::ble::manager::ConnectionState;
use crate::state::AppState;

pub async fn run(app: AppHandle) {
    // Retry forever — adapter creation can fail right at boot on a cold
    // Bluetooth stack; also cover the case where CoreBluetooth resets the
    // adapter (we'd need a fresh event stream).
    loop {
        if let Err(e) = observe_once(&app).await {
            log::warn!("state_observer: {e}, restarting in 5s");
        } else {
            log::info!("state_observer: event stream ended, restarting in 5s");
        }
        time::sleep(Duration::from_secs(5)).await;
    }
}

async fn observe_once(app: &AppHandle) -> Result<(), String> {
    let manager = Manager::new()
        .await
        .map_err(|e| format!("Manager::new: {e}"))?;
    let adapter = manager
        .adapters()
        .await
        .map_err(|e| format!("adapters(): {e}"))?
        .into_iter()
        .next()
        .ok_or_else(|| "no BLE adapter".to_string())?;
    let mut events = adapter
        .events()
        .await
        .map_err(|e| format!("events(): {e}"))?;

    log::info!("state_observer: subscribed to adapter events");

    while let Some(event) = events.next().await {
        if let CentralEvent::StateUpdate(state) = event {
            log::info!("state_observer: adapter state -> {state:?}");
            match state {
                CentralState::PoweredOff | CentralState::Unknown => {
                    force_stale_if_connected(app).await;
                }
                CentralState::PoweredOn => {
                    try_reconnect(app).await;
                }
            }
        }
    }
    Ok(())
}

async fn force_stale_if_connected(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut ctrl = state.controller.lock().await;
    if matches!(ctrl.state(), ConnectionState::Connected { .. }) {
        log::info!("state_observer: adapter off — tearing down session");
        ctrl.force_stale().await;
    }
}

async fn try_reconnect(app: &AppHandle) {
    let state = app.state::<AppState>();

    // Grab the last known address off the lock, then try `connect_to` which
    // handles its own scan + complete_connection pipeline.
    let target = {
        let ctrl = state.controller.lock().await;
        match ctrl.state() {
            // Already connected — nothing to do (spurious re-powered event).
            ConnectionState::Connected { .. } => None,
            _ => ctrl.last_connected_address().map(String::from),
        }
    };
    let Some(address) = target else {
        return;
    };

    log::info!("state_observer: adapter on — reconnecting to {address}");
    let mut ctrl = state.controller.lock().await;
    if let Err(e) = ctrl.connect_to(address).await {
        log::warn!("state_observer: reconnect failed: {e}");
    }
}
