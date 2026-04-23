//! Movement orchestration — hold-to-move and target-seeking motor control.
//!
//! `BleController` owns the BLE session; `MoveCoordinator` owns the movement
//! lifecycle. The split keeps the session code focused on connection state
//! and avoids the `in_target_mode` boolean that previously disambiguated
//! hold-vs-target tasks sharing one slot: the `MoveTask` enum makes the
//! distinction match-exhaustive instead.
//!
//! Wake/stop/settle orchestration stays in the caller (session-level
//! responsibility) — this module only spawns and cancels the tick loops.

use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use btleplug::api::Characteristic;
use btleplug::platform::Peripheral;
use tauri::{AppHandle, Emitter};
use tokio::task::JoinHandle;
use tokio::time;
use tokio_util::sync::CancellationToken;

use super::{linak, write_no_response};
use crate::events::{DeskErrorPayload, EVT_ERROR};

/// Tick period between REFERENCE_INPUT setpoint writes. Going faster
/// doesn't help and causes drops on some firmwares.
const MOVE_TICK: Duration = Duration::from_millis(400);

/// Upper bound on any single target-seeking run.
const MOVE_TIMEOUT: Duration = Duration::from_secs(60);

/// Linak firmware parks up to ~0.8 cm short of the setpoint, especially on
/// short moves. Tighter tolerance false-positives "stuck" detection.
const TARGET_TOLERANCE_CM: f32 = 0.8;

/// Consecutive idle-at-target frames before we declare "reached".
const TARGET_IDLE_FRAMES: u32 = 3;

/// How long the desk must refuse to move *after the most recent target
/// change* before stuck detection starts counting frames. Generous so
/// chains of rapid preset clicks (which leave the firmware briefly
/// unresponsive) don't false-positive.
const STUCK_GRACE: Duration = Duration::from_millis(5000);

/// Frames after the grace window expires before we abort with a stuck error.
const STUCK_FRAMES: u32 = 10;

/// Sentinel value in `target_cell` meaning "no pending target".
const NO_TARGET: u32 = u32::MAX;

enum MoveTask {
    None,
    Hold {
        handle: JoinHandle<()>,
        cancel: CancellationToken,
    },
    Target {
        handle: JoinHandle<()>,
        cancel: CancellationToken,
    },
}

pub struct MoveCoordinator {
    task: MoveTask,
    /// Setpoint shared between coordinator and the running Target loop. The
    /// loop re-reads it every tick, so `retarget()` mid-motion just stores a
    /// new value instead of tearing down the loop and redoing wake/stop.
    target_cell: Arc<AtomicU32>,
}

impl MoveCoordinator {
    pub fn new() -> Self {
        Self {
            task: MoveTask::None,
            target_cell: Arc::new(AtomicU32::new(NO_TARGET)),
        }
    }

    pub async fn cancel(&mut self) {
        match std::mem::replace(&mut self.task, MoveTask::None) {
            MoveTask::None => {}
            MoveTask::Hold { handle, cancel } | MoveTask::Target { handle, cancel } => {
                cancel.cancel();
                let _ = handle.await;
            }
        }
    }

    /// Clear the shared setpoint. The running Target loop, if any, exits on
    /// its next tick (it checks the cell at every tick).
    pub fn clear_target(&self) {
        self.target_cell.store(NO_TARGET, Ordering::Relaxed);
    }

    /// Update the active target without tearing the loop down. Returns
    /// `true` if a live Target loop is tracking — the caller should NOT
    /// start a fresh cold-path in that case (which would burn an extra
    /// wake/stop/settle cycle on every rapid preset click).
    pub fn retarget(&self, cm: f32) -> bool {
        self.target_cell.store(cm.to_bits(), Ordering::Relaxed);
        match &self.task {
            MoveTask::Target { handle, .. } => !handle.is_finished(),
            _ => false,
        }
    }

    /// Spawn a hold-to-move loop that writes `cmd` (CMD_UP / CMD_DOWN) to
    /// the control characteristic every `MOVE_TICK` until cancelled. On
    /// cancel, writes CMD_STOP best-effort so the desk parks.
    pub async fn spawn_hold(
        &mut self,
        peripheral: Peripheral,
        ctrl_ch: Characteristic,
        cmd: [u8; 2],
    ) {
        self.cancel().await;
        self.clear_target();

        let cancel = CancellationToken::new();
        let task_cancel = cancel.clone();
        let handle = tokio::spawn(async move {
            let mut ticker = time::interval(MOVE_TICK);
            loop {
                tokio::select! {
                    _ = task_cancel.cancelled() => break,
                    _ = ticker.tick() => {
                        if write_no_response(&peripheral, &ctrl_ch, &cmd).await.is_err() {
                            break;
                        }
                    }
                }
            }
            let _ = write_no_response(&peripheral, &ctrl_ch, &linak::CMD_STOP).await;
        });
        self.task = MoveTask::Hold { handle, cancel };
    }

    /// Spawn a target-seeking loop. Reads `target_cell` each tick and writes
    /// the corresponding REFERENCE_INPUT payload. Exits when the carriage
    /// parks within tolerance, the target is cleared (`NO_TARGET`), the
    /// cancel token fires, or stuck/timeout conditions trip.
    #[allow(clippy::too_many_arguments)]
    pub async fn spawn_target(
        &mut self,
        peripheral: Peripheral,
        ref_ch: Characteristic,
        ctrl_ch: Characteristic,
        last_cm: Arc<AtomicU32>,
        last_speed: Arc<AtomicI32>,
        app: AppHandle,
        initial_cm: f32,
    ) {
        self.cancel().await;
        self.target_cell
            .store(initial_cm.to_bits(), Ordering::Relaxed);

        let target_cell = self.target_cell.clone();
        let cancel = CancellationToken::new();
        let task_cancel = cancel.clone();

        let handle = tokio::spawn(async move {
            run_target_loop(
                peripheral,
                ref_ch,
                ctrl_ch,
                last_cm,
                last_speed,
                target_cell.clone(),
                app,
                task_cancel,
            )
            .await;
            target_cell.store(NO_TARGET, Ordering::Relaxed);
        });
        self.task = MoveTask::Target { handle, cancel };
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_target_loop(
    peripheral: Peripheral,
    ref_ch: Characteristic,
    ctrl_ch: Characteristic,
    last_cm: Arc<AtomicU32>,
    last_speed: Arc<AtomicI32>,
    target_cell: Arc<AtomicU32>,
    app: AppHandle,
    cancel: CancellationToken,
) {
    let mut ticker = time::interval(MOVE_TICK);
    let mut idle_frames = 0u32;
    let mut stuck_frames = 0u32;
    let started = time::Instant::now();
    let mut last_logged_target = f32::NAN;
    let mut target_changed_at = time::Instant::now();

    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = ticker.tick() => {
                let bits = target_cell.load(Ordering::Relaxed);
                if bits == NO_TARGET {
                    break;
                }
                let target_cm = f32::from_bits(bits);
                let payload = linak::encode_height_cm(target_cm);

                if (target_cm - last_logged_target).abs() > 0.01 {
                    log::info!(
                        "move_to: tick -> target={target_cm:.1}cm raw={payload:?}"
                    );
                    last_logged_target = target_cm;
                    target_changed_at = time::Instant::now();
                    idle_frames = 0;
                    stuck_frames = 0;
                }

                if let Err(err) = write_no_response(&peripheral, &ref_ch, &payload).await {
                    log::warn!("move_to ref_input write failed: {err}");
                    let _ = app.emit(EVT_ERROR, DeskErrorPayload {
                        code: err.code().to_string(),
                        message: format!("move write failed: {err}"),
                        recoverable: true,
                    });
                    break;
                }

                let cm = f32::from_bits(last_cm.load(Ordering::Relaxed));
                let speed = last_speed.load(Ordering::Relaxed) as i16;

                if speed == 0 && (cm - target_cm).abs() < TARGET_TOLERANCE_CM {
                    idle_frames += 1;
                    if idle_frames >= TARGET_IDLE_FRAMES {
                        log::info!(
                            "move_to: reached target {target_cm:.1}cm (final {cm:.2}cm), exiting"
                        );
                        break;
                    }
                } else {
                    idle_frames = 0;
                }

                // Stuck only after a grace window post-target-change, so a
                // natural coast-to-stop between ticks doesn't false-positive.
                if speed == 0
                    && (cm - target_cm).abs() >= TARGET_TOLERANCE_CM
                    && target_changed_at.elapsed() > STUCK_GRACE
                {
                    stuck_frames += 1;
                    if stuck_frames >= STUCK_FRAMES {
                        log::warn!(
                            "move_to: stuck at {cm:.1}cm, target {target_cm:.1}cm — aborting"
                        );
                        let _ = write_no_response(&peripheral, &ctrl_ch, &linak::CMD_STOP).await;
                        let _ = app.emit(EVT_ERROR, DeskErrorPayload {
                            code: "stuck".into(),
                            message: format!(
                                "Desk stopped at {cm:.1} cm before reaching {target_cm:.1} cm. \
                                 Check for obstruction or firmware error. \
                                 Reset the desk panel if it persists."
                            ),
                            recoverable: true,
                        });
                        break;
                    }
                } else {
                    stuck_frames = 0;
                }

                if started.elapsed() > MOVE_TIMEOUT {
                    log::warn!("move_to: timeout");
                    break;
                }
            }
        }
    }
}
