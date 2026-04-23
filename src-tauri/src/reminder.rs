//! Stand-reminder timer owned by the backend.
//!
//! The UI used to run a `setInterval` for this, which macOS throttles to
//! ~1 Hz (or stalls entirely) whenever the NSPanel is hidden — so the user
//! never got the "time to stand up" nudge. Running the loop in tokio keeps
//! the countdown accurate regardless of popover visibility, and lets us
//! fire a native notification when the deadline hits.
//!
//! Frontend flow:
//!   - `start_reminder(mins)`                → spawns loop, returns new state
//!   - `get_reminder_state()`                → progress for the progress bar
//!   - `stop_reminder()`                     → cancels loop
//! Events:
//!   - `desk://reminder-fire`                → fired each time the deadline
//!     expires. Payload: `{ intervalMins }`. The loop auto-resets so the
//!     countdown starts over on its own.
//!   - `desk://reminder-state`               → fired on every start / stop
//!     so the UI mirrors authoritative backend state.

use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{self, Duration, Instant};
use tokio_util::sync::CancellationToken;

pub const EVT_FIRE: &str = "desk://reminder-fire";
pub const EVT_STATE: &str = "desk://reminder-state";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderState {
    pub running: bool,
    pub interval_mins: u32,
    /// Unix-ish milliseconds until the next fire, as seen by the frontend's
    /// own `Date.now()`. Lets the UI draw a smooth progress bar without
    /// polling the backend on every frame.
    pub deadline_ms: Option<u64>,
}

pub struct ReminderController {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    task: Option<(JoinHandle<()>, CancellationToken)>,
    interval_mins: u32,
    deadline: Option<Instant>,
    started_at_ms: u64, // wall-clock reference for the UI progress bar
}

impl ReminderController {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                task: None,
                interval_mins: 60,
                deadline: None,
                started_at_ms: 0,
            })),
        }
    }

    pub async fn snapshot(&self) -> ReminderState {
        let g = self.inner.lock().await;
        Self::state_locked(&g)
    }

    fn state_locked(g: &Inner) -> ReminderState {
        let deadline_ms = g.deadline.map(|d| {
            let now_wall = unix_ms();
            let now_mono = Instant::now();
            if d > now_mono {
                now_wall + (d - now_mono).as_millis() as u64
            } else {
                now_wall
            }
        });
        ReminderState {
            running: g.task.is_some(),
            interval_mins: g.interval_mins,
            deadline_ms,
        }
    }

    pub async fn start(&self, app: AppHandle, mins: u32) -> ReminderState {
        self.stop().await;
        let interval = Duration::from_secs(mins.max(1) as u64 * 60);
        let token = CancellationToken::new();
        let task_token = token.clone();
        let inner = self.inner.clone();
        let app_clone = app.clone();

        let deadline = Instant::now() + interval;
        {
            let mut g = self.inner.lock().await;
            g.interval_mins = mins;
            g.deadline = Some(deadline);
            g.started_at_ms = unix_ms();
        }

        let handle = tokio::spawn(async move {
            loop {
                let (wait, _started_ms) = {
                    let g = inner.lock().await;
                    let now = Instant::now();
                    let left = g
                        .deadline
                        .map(|d| d.saturating_duration_since(now))
                        .unwrap_or(Duration::ZERO);
                    (left, g.started_at_ms)
                };
                if wait.is_zero() {
                    break;
                }
                tokio::select! {
                    _ = task_token.cancelled() => return,
                    _ = time::sleep(wait) => {}
                }
                let payload_mins = {
                    let mut g = inner.lock().await;
                    g.started_at_ms = unix_ms();
                    g.deadline = Some(Instant::now() + Duration::from_secs(g.interval_mins as u64 * 60));
                    g.interval_mins
                };
                let _ = app_clone.emit(EVT_FIRE, FirePayload { interval_mins: payload_mins });
                let state = {
                    let g = inner.lock().await;
                    Self::state_locked(&g)
                };
                let _ = app_clone.emit(EVT_STATE, &state);
            }
        });

        {
            let mut g = self.inner.lock().await;
            g.task = Some((handle, token));
        }

        let state = self.snapshot().await;
        let _ = app.emit(EVT_STATE, &state);
        state
    }

    pub async fn stop(&self) {
        let mut g = self.inner.lock().await;
        if let Some((h, tok)) = g.task.take() {
            tok.cancel();
            drop(g);
            let _ = h.await;
            let mut g = self.inner.lock().await;
            g.deadline = None;
        }
    }

    pub async fn stop_and_emit(&self, app: AppHandle) {
        self.stop().await;
        let state = self.snapshot().await;
        let _ = app.emit(EVT_STATE, &state);
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FirePayload {
    interval_mins: u32,
}

fn unix_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
