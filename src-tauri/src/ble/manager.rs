use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use btleplug::api::{
    Central, CharPropFlags, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::StreamExt;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::task::JoinHandle;
use tokio::time;
use tokio_util::sync::CancellationToken;

use crate::events::{
    ConnectionUpdate, DeskErrorPayload, HeightUpdate, EVT_CONNECTION, EVT_ERROR, EVT_HEIGHT,
};

use super::move_coord::MoveCoordinator;
use super::{linak, write_no_response, DeskError};

const SCAN_TIMEOUT: Duration = Duration::from_secs(12);
const PERMISSION_SILENT_WINDOW: Duration = Duration::from_secs(10);
const NOTIFY_EMIT_INTERVAL: Duration = Duration::from_millis(33);

/// Idle desk drops the first command after sleep — wake + short delay
/// before the real write.
const WAKE_DELAY: Duration = Duration::from_millis(150);

/// Before a fresh target-seek we STOP the carriage and poll until speed
/// hits zero. Prevents the firmware reversal-guard E16 when the user
/// switches direction mid-motion.
const SETTLE_POLL_INTERVAL: Duration = Duration::from_millis(100);
const SETTLE_TIMEOUT: Duration = Duration::from_millis(1500);

/// Grace period after SETTLE before the first REFERENCE_INPUT tick — some
/// firmwares still have internal state to flush, especially after a chain
/// of rapid direction reversals. Too short here and the firmware silently
/// ignores the next setpoint, which later trips the stuck detector.
const PRE_TARGET_GRACE: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ConnectionState {
    Disconnected,
    Scanning,
    Connecting { device: String },
    Connected { device: String, address: String },
    Reconnecting { attempt: u32 },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub device: String,
    pub address: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusSnapshot {
    pub connection: ConnectionUpdate,
    pub last_height: Option<HeightUpdate>,
}

pub struct BleController {
    app: AppHandle,
    peripheral: Option<Peripheral>,
    char_control: Option<Characteristic>,
    char_refinput: Option<Characteristic>,
    char_position: Option<Characteristic>,
    state: ConnectionState,
    notify_task: Option<JoinHandle<()>>,
    notify_cancel: Option<CancellationToken>,
    move_coord: MoveCoordinator,
    last_cm_bits: Arc<AtomicU32>,
    last_speed: Arc<std::sync::atomic::AtomicI32>,
    /// Peripherals discovered in the most recent `scan_devices`, keyed by
    /// `PeripheralId.to_string()`. Lets `connect_to` skip a second scan on
    /// macOS where the CBPeripheral handle is what we actually need.
    scan_cache: std::collections::HashMap<String, Peripheral>,
}

impl BleController {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            peripheral: None,
            char_control: None,
            char_refinput: None,
            char_position: None,
            state: ConnectionState::Disconnected,
            notify_task: None,
            notify_cancel: None,
            move_coord: MoveCoordinator::new(),
            last_cm_bits: Arc::new(AtomicU32::new(0)),
            last_speed: Arc::new(std::sync::atomic::AtomicI32::new(0)),
            scan_cache: std::collections::HashMap::new(),
        }
    }

    pub fn state(&self) -> &ConnectionState {
        &self.state
    }

    pub fn snapshot(&self) -> StatusSnapshot {
        let bits = self.last_cm_bits.load(Ordering::Relaxed);
        let last_height = if bits == 0 {
            None
        } else {
            let cm = f32::from_bits(bits);
            let raw = ((cm - linak::HEIGHT_MIN_CM) * 100.0).round() as u16;
            let speed = self.last_speed.load(Ordering::Relaxed) as i16;
            Some(HeightUpdate {
                cm,
                mm: raw,
                speed,
                moving: speed != 0,
            })
        };
        StatusSnapshot {
            connection: connection_to_update(&self.state),
            last_height,
        }
    }

    /// Does NOT touch the connection state if we're already `Connected` —
    /// rescanning from the device picker used to downgrade Connected →
    /// Scanning → Disconnected, stranding the live notification stream with
    /// a "Not connected" badge.
    pub async fn scan_devices(&mut self, duration_ms: u64) -> Result<Vec<DeviceInfo>, DeskError> {
        let was_connected = matches!(self.state, ConnectionState::Connected { .. });
        if !was_connected {
            self.set_state(ConnectionState::Scanning);
        }

        let manager = Manager::new().await?;
        let adapter = manager
            .adapters()
            .await?
            .into_iter()
            .next()
            .ok_or(DeskError::NoAdapter)?;

        adapter.start_scan(ScanFilter::default()).await?;
        time::sleep(Duration::from_millis(duration_ms.max(500))).await;
        let peripherals = adapter.peripherals().await?;
        let _ = adapter.stop_scan().await;

        self.scan_cache.clear();
        let mut out = Vec::with_capacity(peripherals.len());
        for p in peripherals {
            let props = match p.properties().await {
                Ok(Some(pr)) => pr,
                _ => continue,
            };
            let name = props
                .local_name
                .clone()
                .unwrap_or_else(|| "(unnamed)".into());
            // On macOS CoreBluetooth hides the real MAC; `address()` returns
            // all zeros. Use `id()` (a stable PeripheralId — BDAddr on
            // Linux/Windows, CBPeripheral UUID on macOS) as the unique key.
            let id = p.id().to_string();
            self.scan_cache.insert(id.clone(), p);
            out.push(DeviceInfo {
                device: name,
                address: id,
            });
        }

        // Only clear the spinner if we were the ones who set it. Leave
        // Connected alone so a rescan from the picker doesn't wipe it.
        if !was_connected && matches!(self.state, ConnectionState::Scanning) {
            self.set_state(ConnectionState::Disconnected);
        }
        Ok(out)
    }

    /// Connect to an id returned by `scan_devices` or remembered from the
    /// previous session. Tries the cached handle first, but falls back to a
    /// fresh rescan if the cached `CBPeripheral` is stale (post-disconnect).
    pub async fn connect_to(&mut self, address: String) -> Result<DeviceInfo, DeskError> {
        // Fast path: cached peripheral from the last `scan_devices` call.
        // Address (BDAddr on Linux/Windows, CBPeripheral UUID on macOS) is
        // device identity — keep it at debug level so it doesn't land in
        // release logs.
        if let Some(peripheral) = self.scan_cache.get(&address).cloned() {
            log::debug!("connect_to: trying cached handle for {address}");
            match time::timeout(Duration::from_secs(6), self.complete_connection(peripheral)).await
            {
                Ok(result) => return result,
                Err(_) => {
                    log::warn!(
                        "connect_to: cached handle timed out, dropping cache and rescanning"
                    );
                    self.scan_cache.remove(&address);
                    // Make sure we don't leave the controller in Connecting.
                    self.set_state(ConnectionState::Disconnected);
                }
            }
        }

        // Fresh rescan path. Required after a previous disconnect on macOS,
        // where re-using the old peripheral handle hangs for ~20 s.
        self.set_state(ConnectionState::Scanning);
        let peripheral = match scan_for_address(&address).await {
            Ok(p) => p,
            Err(e) => {
                self.set_state(ConnectionState::Disconnected);
                return Err(e);
            }
        };

        // Cache the fresh handle so the next call (likely the resume after
        // panel hide) has a hot path.
        self.scan_cache.insert(address.clone(), peripheral.clone());
        self.complete_connection(peripheral).await
    }

    /// Watchdog entry point. Installs a peripheral that was scanned off-lock
    /// into the session. Same pipeline as `complete_connection` (connect,
    /// discover, subscribe, DPG handshake, emit Connected) — exposed so the
    /// reconnect loop can do the slow scan phase without holding the
    /// controller mutex.
    pub async fn install_peripheral(
        &mut self,
        peripheral: Peripheral,
    ) -> Result<DeviceInfo, DeskError> {
        self.complete_connection(peripheral).await
    }

    pub async fn scan_and_connect(&mut self) -> Result<DeviceInfo, DeskError> {
        self.set_state(ConnectionState::Scanning);

        let manager = Manager::new().await?;
        let adapter = manager
            .adapters()
            .await?
            .into_iter()
            .next()
            .ok_or(DeskError::NoAdapter)?;

        adapter.start_scan(ScanFilter::default()).await?;
        let peripheral = self.find_desk(&adapter).await?;
        let _ = adapter.stop_scan().await;

        self.complete_connection(peripheral).await
    }

    async fn complete_connection(
        &mut self,
        peripheral: Peripheral,
    ) -> Result<DeviceInfo, DeskError> {
        let local_name = peripheral
            .properties()
            .await?
            .and_then(|p| p.local_name)
            .unwrap_or_else(|| "Unknown Desk".into());
        let address = peripheral.id().to_string();

        self.set_state(ConnectionState::Connecting {
            device: local_name.clone(),
        });

        peripheral.connect().await?;
        peripheral.discover_services().await?;

        let chars = peripheral.characteristics();
        let find =
            |uuid| {
                chars.iter().find(|c| c.uuid == uuid).cloned().ok_or(
                    DeskError::MissingCharacteristic(match uuid {
                        u if u == linak::CHAR_CONTROL => "control",
                        u if u == linak::CHAR_REFINPUT => "refinput",
                        u if u == linak::CHAR_POSITION => "position",
                        _ => "unknown",
                    }),
                )
            };
        self.char_control = Some(find(linak::CHAR_CONTROL)?);
        self.char_refinput = Some(find(linak::CHAR_REFINPUT)?);
        let pos_char = find(linak::CHAR_POSITION)?;
        if !pos_char.properties.contains(CharPropFlags::NOTIFY) {
            return Err(DeskError::MissingCharacteristic("position-notify"));
        }
        self.char_position = Some(pos_char.clone());
        let dpg_char = find(linak::CHAR_DPG).ok();
        self.peripheral = Some(peripheral.clone());

        self.subscribe_position().await?;

        // DPG1C handshake: make sure user_id[0] == 1. Without this the desk
        // accepts position notifications fine but silently drops every write
        // to REFERENCE_INPUT. Harmless on non-DPG1C controllers.
        if let Some(dpg) = dpg_char {
            if let Err(e) = prime_user_id(&peripheral, &dpg).await {
                log::warn!("DPG user_id handshake failed (continuing): {e}");
            }
        }

        self.set_state(ConnectionState::Connected {
            device: local_name.clone(),
            address: address.clone(),
        });

        Ok(DeviceInfo {
            device: local_name,
            address,
        })
    }

    pub async fn disconnect(&mut self) -> Result<(), DeskError> {
        self.teardown_session(/* attempt_disconnect */ true).await;
        Ok(())
    }

    /// Clone of the current peripheral handle for out-of-lock liveness probes
    /// (the watchdog calls `is_connected()` on it without holding the mutex).
    pub fn peripheral_clone(&self) -> Option<Peripheral> {
        self.peripheral.clone()
    }

    /// Emits `Reconnecting { attempt }` so the UI can render a reconnect
    /// spinner with the current attempt number. Watchdog-only entry point.
    pub fn mark_reconnecting(&mut self, attempt: u32) {
        self.set_state(ConnectionState::Reconnecting { attempt });
    }

    /// Tear down after detecting the underlying BLE link went away (sleep,
    /// controller power cycle, RF drop). Like `disconnect()` but skips
    /// `peripheral.disconnect()` on the stale handle — on macOS that call
    /// hangs ~20 s against a dead CBPeripheral (btleplug #25). Leaves the
    /// state at `Disconnected` so the watchdog can drive reconnect.
    pub async fn force_stale(&mut self) {
        self.teardown_session(/* attempt_disconnect */ false).await;
    }

    /// Light-touch "pause". Keeps the BLE link open so reconnect is instant,
    /// but stops the position notification stream. Some Linak firmwares
    /// dim the controller display once no host is actively subscribed; for
    /// the ones that don't it's at least not costing reconnect latency.
    pub async fn pause_session(&mut self) -> Result<(), DeskError> {
        let Some(peripheral) = self.peripheral.clone() else {
            return Ok(());
        };
        self.move_coord.cancel().await;
        if let Some(tok) = self.notify_cancel.take() {
            tok.cancel();
        }
        if let Some(task) = self.notify_task.take() {
            let _ = task.await;
        }
        if let Some(ch) = self.char_position.clone() {
            let _ = peripheral.unsubscribe(&ch).await;
        }
        Ok(())
    }

    /// Resume a paused session: re-subscribe to position notifications so
    /// the UI gets live telemetry again. No-op if we're fully disconnected.
    pub async fn resume_session(&mut self) -> Result<(), DeskError> {
        if self.peripheral.is_none() {
            return Ok(());
        }
        self.subscribe_position().await
    }

    pub async fn start_move(&mut self, direction: MoveDir) -> Result<(), DeskError> {
        let cmd = match direction {
            MoveDir::Up => linak::CMD_UP,
            MoveDir::Down => linak::CMD_DOWN,
        };
        let ctrl_ch = self.char_control.clone().ok_or(DeskError::NotConnected)?;
        let peripheral = self.peripheral.clone().ok_or(DeskError::NotConnected)?;

        // Wake first — idle desk ignores the first command otherwise.
        write_no_response(&peripheral, &ctrl_ch, &linak::CMD_WAKE).await?;
        time::sleep(WAKE_DELAY).await;

        self.move_coord.spawn_hold(peripheral, ctrl_ch, cmd).await;
        Ok(())
    }

    pub async fn stop_move(&mut self) -> Result<(), DeskError> {
        // Clear the shared setpoint first — a running target loop will exit
        // on its next tick. Then cancel anything that's still live.
        self.move_coord.clear_target();
        self.move_coord.cancel().await;
        let peripheral = self.peripheral.clone().ok_or(DeskError::NotConnected)?;
        let ctrl_ch = self.char_control.clone().ok_or(DeskError::NotConnected)?;
        write_no_response(&peripheral, &ctrl_ch, &linak::CMD_STOP).await
    }

    pub async fn move_to(&mut self, target_cm: f32) -> Result<(), DeskError> {
        if !(linak::HEIGHT_MIN_CM..=linak::HEIGHT_MAX_CM).contains(&target_cm) {
            return Err(DeskError::InvalidHeight(target_cm));
        }

        // Direction-reversal guard. Linak firmware throws E16 when a new
        // REFERENCE_INPUT setpoint flips the carriage's direction without a
        // STOP+settle in between. Hot retarget only if the new target is on
        // the same side as the current motion (or the carriage is idle);
        // otherwise fall through to the cold path which settles first.
        let current_cm = f32::from_bits(self.last_cm_bits.load(Ordering::Relaxed));
        let current_speed = self.last_speed.load(Ordering::Relaxed) as i16;
        let reversal = is_direction_reversal(current_cm, current_speed, target_cm);

        if !reversal && self.move_coord.retarget(target_cm) {
            log::info!("move_to: retarget existing loop -> {target_cm:.1}cm");
            return Ok(());
        }

        // Cold path: spin up a fresh target loop. Either no loop was
        // running, or we're reversing direction (needs STOP + settle to
        // avoid E16).
        //
        // CRITICAL: cancel the existing loop BEFORE sending STOP / waiting
        // for settle. An active Target loop keeps writing
        // `REFERENCE_INPUT = old_target` every MOVE_TICK (400 ms), which
        // overrides our STOP command on the wire — the carriage never
        // actually halts and settle times out. Cancel first so the wire is
        // quiet while we stop and wait.
        self.move_coord.cancel().await;

        let peripheral = self.peripheral.clone().ok_or(DeskError::NotConnected)?;
        let ref_ch = self.char_refinput.clone().ok_or(DeskError::NotConnected)?;
        let ctrl_ch = self.char_control.clone().ok_or(DeskError::NotConnected)?;

        if reversal {
            log::info!(
                "move_to: reversal (cur={current_cm:.1}cm speed={current_speed} -> target={target_cm:.1}cm) — cold path"
            );
        } else {
            log::info!("move_to: starting loop -> {target_cm:.1}cm");
        }
        write_no_response(&peripheral, &ctrl_ch, &linak::CMD_WAKE).await?;
        time::sleep(WAKE_DELAY).await;
        write_no_response(&peripheral, &ctrl_ch, &linak::CMD_STOP).await?;

        // Wait for the carriage to actually stop before writing the first
        // REFERENCE_INPUT — prevents the firmware's reversal-guard E16.
        let settle_deadline = time::Instant::now() + SETTLE_TIMEOUT;
        while time::Instant::now() < settle_deadline {
            if self.last_speed.load(Ordering::Relaxed) == 0 {
                break;
            }
            time::sleep(SETTLE_POLL_INTERVAL).await;
        }
        time::sleep(PRE_TARGET_GRACE).await;

        self.move_coord
            .spawn_target(
                peripheral,
                ref_ch,
                ctrl_ch,
                self.last_cm_bits.clone(),
                self.last_speed.clone(),
                self.app.clone(),
                target_cm,
            )
            .await;
        Ok(())
    }

    /// Common tear-down used by `disconnect` (send real BLE disconnect) and
    /// `force_stale` (skip the disconnect — handle is already dead). Leaves
    /// the controller at `Disconnected`.
    async fn teardown_session(&mut self, attempt_disconnect: bool) {
        self.move_coord.cancel().await;
        if let Some(tok) = self.notify_cancel.take() {
            tok.cancel();
        }
        if let Some(task) = self.notify_task.take() {
            let _ = task.await;
        }
        if let Some(p) = self.peripheral.take() {
            if attempt_disconnect {
                let _ = p.disconnect().await;
            }
        }
        self.char_control = None;
        self.char_refinput = None;
        self.char_position = None;
        // Drop cached CBPeripheral handles. On macOS they become stale after
        // a disconnect (btleplug #25, #413) and re-using them hangs the next
        // `peripheral.connect()` for ~20 s.
        self.scan_cache.clear();
        self.set_state(ConnectionState::Disconnected);
    }

    async fn find_desk(&mut self, adapter: &Adapter) -> Result<Peripheral, DeskError> {
        let deadline = time::Instant::now() + SCAN_TIMEOUT;
        let permission_checkpoint = time::Instant::now() + PERMISSION_SILENT_WINDOW;
        let mut warned_permission = false;

        loop {
            if time::Instant::now() >= deadline {
                return Err(DeskError::NotFound);
            }
            let peripherals = adapter.peripherals().await?;
            if peripherals.is_empty()
                && !warned_permission
                && time::Instant::now() >= permission_checkpoint
            {
                self.emit_error(&DeskError::PermissionDenied);
                warned_permission = true;
            }
            for p in peripherals {
                let props = match p.properties().await {
                    Ok(Some(props)) => props,
                    _ => continue,
                };
                let name = match &props.local_name {
                    Some(n) => n.clone(),
                    None => continue,
                };
                let services_match = props.services.contains(&linak::SERVICE_UUID);
                if services_match || linak::is_desk_name(&name) {
                    return Ok(p);
                }
            }
            time::sleep(Duration::from_millis(500)).await;
        }
    }

    async fn subscribe_position(&mut self) -> Result<(), DeskError> {
        let peripheral = self.peripheral.clone().ok_or(DeskError::NotConnected)?;
        let ch = self.char_position.clone().ok_or(DeskError::NotConnected)?;

        // One-shot GATT read so the UI doesn't stare at the floor value
        // until the desk happens to send its next notification.
        if let Ok(initial) = peripheral.read(&ch).await {
            if let Some(pos) = linak::decode_position(&initial) {
                log::info!(
                    "position initial read: bytes={:02x?} raw_u16={} decoded_cm={:.2}",
                    initial,
                    pos.raw,
                    pos.cm,
                );
                self.last_cm_bits.store(pos.cm.to_bits(), Ordering::Relaxed);
                self.last_speed.store(pos.speed as i32, Ordering::Relaxed);
                let _ = self.app.emit(
                    EVT_HEIGHT,
                    HeightUpdate {
                        cm: round1(pos.cm),
                        mm: pos.raw,
                        speed: pos.speed,
                        moving: pos.speed != 0,
                    },
                );
            }
        }

        peripheral.subscribe(&ch).await?;

        // Also subscribe to firmware error codes (E16 etc.). Best-effort.
        let err_char = peripheral
            .characteristics()
            .into_iter()
            .find(|c| c.uuid == linak::CHAR_CONTROL_ERROR);
        if let Some(ref ec) = err_char {
            if ec.properties.contains(CharPropFlags::NOTIFY) {
                let _ = peripheral.subscribe(ec).await;
            }
        }

        let mut stream = peripheral.notifications().await?;
        let app = self.app.clone();
        let last_cm = self.last_cm_bits.clone();
        let last_speed = self.last_speed.clone();
        let token = CancellationToken::new();
        let task_token = token.clone();

        let task = tokio::spawn(async move {
            let mut last_emit = time::Instant::now()
                .checked_sub(NOTIFY_EMIT_INTERVAL)
                .unwrap_or_else(time::Instant::now);
            let mut last_diag = time::Instant::now();
            let mut first = true;
            loop {
                tokio::select! {
                    _ = task_token.cancelled() => break,
                    next = stream.next() => {
                        let Some(n) = next else { break; };
                        if n.uuid == linak::CHAR_CONTROL_ERROR {
                            log::warn!("control-error notify: bytes={:02x?}", n.value);
                            // Payload layout `[0x01, <sz>, <code>, ...]` — the
                            // first byte is a response marker, not the error
                            // code. Pick the payload byte instead.
                            let code = n.value.get(2).copied().unwrap_or(0);
                            if code != 0 {
                                let _ = app.emit(EVT_ERROR, DeskErrorPayload {
                                    code: format!("desk_err_{code:02x}"),
                                    message: format!(
                                        "Desk firmware reported error E{code:02} (0x{code:02X}). \
                                         Common causes: sudden reversal, obstruction, overload. \
                                         Wait a second then try again, or reset the panel if it persists."
                                    ),
                                    recoverable: true,
                                });
                            }
                            continue;
                        }
                        if n.uuid != linak::CHAR_POSITION { continue; }
                        let Some(pos) = linak::decode_position(&n.value) else { continue; };
                        if first {
                            log::info!(
                                "position first notify: bytes={:02x?} raw_u16={} decoded_cm={:.2} speed={}",
                                n.value, pos.raw, pos.cm, pos.speed,
                            );
                            first = false;
                        }
                        if last_diag.elapsed() >= Duration::from_secs(1) {
                            log::debug!(
                                "position diag: raw_u16={} decoded_cm={:.2} speed={}",
                                pos.raw, pos.cm, pos.speed,
                            );
                            last_diag = time::Instant::now();
                        }
                        last_cm.store(pos.cm.to_bits(), Ordering::Relaxed);
                        last_speed.store(pos.speed as i32, Ordering::Relaxed);
                        if time::Instant::now().duration_since(last_emit) < NOTIFY_EMIT_INTERVAL {
                            continue;
                        }
                        last_emit = time::Instant::now();
                        let _ = app.emit(EVT_HEIGHT, HeightUpdate {
                            cm: round1(pos.cm),
                            mm: pos.raw,
                            speed: pos.speed,
                            moving: pos.speed != 0,
                        });
                    }
                }
            }
        });
        self.notify_task = Some(task);
        self.notify_cancel = Some(token);
        Ok(())
    }

    fn set_state(&mut self, new: ConnectionState) {
        // Log at INFO without the address (device identity lands on disk
        // otherwise). Full struct goes to DEBUG for diagnosis.
        log::info!("state -> {}", state_log_label(&new));
        log::debug!("state full: {new:?}");
        self.state = new;
        let _ = self
            .app
            .emit(EVT_CONNECTION, connection_to_update(&self.state));
    }

    fn emit_error(&self, err: &DeskError) {
        let _ = self.app.emit(
            EVT_ERROR,
            DeskErrorPayload {
                code: err.code().to_string(),
                message: err.to_string(),
                recoverable: err.recoverable(),
            },
        );
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MoveDir {
    Up,
    Down,
}

/// INFO-safe connection state label — drops the address so device identity
/// doesn't end up in the release log file. Full `{:?}` goes to DEBUG.
/// Shared with `state.rs` so the watchdog's "state changed during scan"
/// message stays consistent with the main transition logs.
pub(crate) fn state_log_label(s: &ConnectionState) -> String {
    match s {
        ConnectionState::Disconnected => "Disconnected".into(),
        ConnectionState::Scanning => "Scanning".into(),
        ConnectionState::Connecting { device } => format!("Connecting({device})"),
        ConnectionState::Connected { device, .. } => format!("Connected({device})"),
        ConnectionState::Reconnecting { attempt } => format!("Reconnecting(attempt={attempt})"),
    }
}

fn connection_to_update(s: &ConnectionState) -> ConnectionUpdate {
    match s {
        ConnectionState::Disconnected => ConnectionUpdate::Disconnected,
        ConnectionState::Scanning => ConnectionUpdate::Scanning,
        ConnectionState::Connecting { device } => ConnectionUpdate::Connecting {
            device: device.clone(),
        },
        ConnectionState::Connected { device, address } => ConnectionUpdate::Connected {
            device: device.clone(),
            address: address.clone(),
        },
        ConnectionState::Reconnecting { attempt } => {
            ConnectionUpdate::Reconnecting { attempt: *attempt }
        }
    }
}

fn round1(v: f32) -> f32 {
    (v * 10.0).round() / 10.0
}

/// True if the carriage is currently moving AND the new target lies on the
/// opposite side of where it's heading. Linak firmware throws E16 when a
/// REFERENCE_INPUT setpoint flips direction without a STOP + settle in
/// between; the caller must route such moves through the cold path.
///
/// A small dead zone (0.5 cm) around the current position prevents
/// micro-jitter near the setpoint from registering as a direction change.
fn is_direction_reversal(current_cm: f32, current_speed: i16, target_cm: f32) -> bool {
    if current_speed == 0 {
        return false;
    }
    const DEAD_ZONE_CM: f32 = 0.5;
    let target_above = target_cm > current_cm + DEAD_ZONE_CM;
    let target_below = target_cm < current_cm - DEAD_ZONE_CM;
    let moving_up = current_speed > 0;
    let moving_down = current_speed < 0;
    (moving_up && target_below) || (moving_down && target_above)
}

#[cfg(test)]
mod tests {
    use super::is_direction_reversal;

    #[test]
    fn stopped_is_never_reversal() {
        assert!(!is_direction_reversal(80.0, 0, 120.0));
        assert!(!is_direction_reversal(120.0, 0, 80.0));
    }

    #[test]
    fn same_direction_is_not_reversal() {
        assert!(!is_direction_reversal(80.0, 100, 120.0));
        assert!(!is_direction_reversal(120.0, -100, 80.0));
    }

    #[test]
    fn opposite_direction_is_reversal() {
        assert!(is_direction_reversal(100.0, -100, 120.0));
        assert!(is_direction_reversal(100.0, 100, 80.0));
    }

    #[test]
    fn target_within_dead_zone_is_not_reversal() {
        assert!(!is_direction_reversal(100.0, 100, 99.7));
        assert!(!is_direction_reversal(100.0, -100, 100.3));
    }
}

/// Adapter-level scan for a specific peripheral id. Free function (no
/// `&self`) so the watchdog can run it without holding the controller
/// mutex — commands like `disconnect` stay responsive during the ~6 s
/// scan window. Caller owns transitioning the state machine.
pub(crate) async fn scan_for_address(address: &str) -> Result<Peripheral, DeskError> {
    let manager = Manager::new().await?;
    let adapter = manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .ok_or(DeskError::NoAdapter)?;

    adapter.start_scan(ScanFilter::default()).await?;

    let deadline = time::Instant::now() + Duration::from_secs(6);
    let mut peripheral: Option<Peripheral> = None;
    while time::Instant::now() < deadline {
        for p in adapter.peripherals().await? {
            if p.id().to_string().eq_ignore_ascii_case(address) {
                peripheral = Some(p);
                break;
            }
        }
        if peripheral.is_some() {
            break;
        }
        time::sleep(Duration::from_millis(300)).await;
    }
    let _ = adapter.stop_scan().await;

    peripheral.ok_or(DeskError::NotFound)
}

/// DPG1C user-id handshake. The controller expects `user_id[0] == 1` before
/// it accepts REFERENCE_INPUT writes. Protocol:
///
///   write [0x7F, 134, 0x00] to DPG (read-user-id request)
///   wait briefly, then read DPG; response is `[0x01, <sz>, <byte0>, ...]`
///   if `response[2] != 0x01`, write `[0x7F, 134, 0x80, 0x01, <rest>]` to
///   update user_id[0] to 1.
async fn prime_user_id(peripheral: &Peripheral, dpg: &Characteristic) -> Result<(), DeskError> {
    // Request current user_id. Uses WithResponse because DPG reads are
    // request/response, not fire-and-forget like the control writes.
    peripheral
        .write(
            dpg,
            &[0x7F, linak::DPG_CMD_USER_ID, 0x00],
            WriteType::WithResponse,
        )
        .await?;
    time::sleep(Duration::from_millis(120)).await;

    let resp = match peripheral.read(dpg).await {
        Ok(v) => v,
        Err(e) => {
            log::warn!("DPG read failed: {e}");
            return Ok(());
        }
    };
    // DPG bytes include controller fingerprint; keep them at debug so
    // release logs don't carry device identity.
    log::debug!("DPG user_id response: {:02x?}", resp);

    // Valid response starts with 0x01; payload bytes live from index 2.
    if resp.len() >= 3 && resp[0] == 0x01 && resp[2] != 0x01 {
        let mut payload: Vec<u8> = vec![0x7F, linak::DPG_CMD_USER_ID, 0x80, 0x01];
        payload.extend_from_slice(&resp[3..]);
        log::debug!("DPG setting user_id[0]=1 via {:02x?}", payload);
        peripheral
            .write(dpg, &payload, WriteType::WithResponse)
            .await?;
    }
    Ok(())
}
