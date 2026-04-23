import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification';
import { useEffect, useRef, useState } from 'react';
import { IntervalPicker } from '@/components/ui/interval-picker';
import { Toggle } from '@/components/ui/toggle';
import { desk, type ReminderState } from '@/lib/desk';

const DEFAULT_MINS = 60;

const IS_MACOS =
  typeof navigator !== 'undefined' &&
  /Mac|iPhone|iPad/i.test(navigator.platform || navigator.userAgent);

async function deliverNotification(title: string, body: string): Promise<void> {
  // macOS: route through our objc2 / UNUserNotificationCenter command so
  // the bundle icon attaches as a thumbnail. plugin-notification's path
  // shows a generic gray bell on LSUIElement apps.
  if (IS_MACOS) {
    await desk.sendNativeNotification(title, body);
  } else {
    sendNotification({ title, body });
  }
}

/**
 * Stand reminder driven entirely by the Rust-side `ReminderController`.
 *
 * Running a `setInterval` in React was unreliable: macOS throttles hidden
 * WKWebView timers to ~1 Hz or pauses them outright, so the user never got
 * nudged to stand up. The backend owns the deadline; the UI just renders
 * the progress bar from `Date.now()` vs the deadline and fires a native
 * `sendNotification` whenever the backend pings `desk://reminder-fire`.
 */
export function Reminder() {
  const [state, setState] = useState<ReminderState>({
    running: false,
    intervalMins: DEFAULT_MINS,
    deadlineMs: null,
  });
  const [now, setNow] = useState(() => Date.now());
  const permissionRef = useRef<boolean | null>(null);

  useEffect(() => {
    let mounted = true;
    desk.reminderState().then((s) => {
      if (mounted) setState(s);
    });
    const unlistenState = desk.onReminderState(setState);
    const unlistenFire = desk.onReminderFire(async ({ intervalMins }) => {
      // Ask once, lazily.
      if (permissionRef.current == null) {
        try {
          const granted = await isPermissionGranted();
          permissionRef.current = granted || (await requestPermission()) === 'granted';
        } catch {
          permissionRef.current = false;
        }
      }
      if (permissionRef.current) {
        const unit = intervalMins === 1 ? 'minute' : 'minutes';
        const body = `You\u2019ve been in the same position for ${intervalMins} ${unit}. Stretch, sip water, switch postures.`;
        deliverNotification('Time to move', body).catch(() => {
          // Fallback to plugin path if our command fails (e.g., bundle
          // missing on a dev hot-reload).
          sendNotification({ title: 'Time to move', body });
        });
      }
    });
    return () => {
      mounted = false;
      unlistenState.then((u) => u());
      unlistenFire.then((u) => u());
    };
  }, []);

  // Tick for the progress bar — doesn't drive the deadline, just repaints.
  useEffect(() => {
    if (!state.running || state.deadlineMs == null) return;
    const id = window.setInterval(() => setNow(Date.now()), 500);
    return () => window.clearInterval(id);
  }, [state.running, state.deadlineMs]);

  const pct = (() => {
    if (!state.running || state.deadlineMs == null) return 0;
    const totalMs = state.intervalMins * 60_000;
    const remainingMs = Math.max(0, state.deadlineMs - now);
    return Math.min(1, Math.max(0, 1 - remainingMs / totalMs));
  })();

  const remainingSecs =
    state.running && state.deadlineMs != null
      ? Math.max(0, Math.ceil((state.deadlineMs - now) / 1000))
      : 0;
  const remainingLabel =
    remainingSecs >= 60 ? `${Math.round(remainingSecs / 60)} min left` : `${remainingSecs}s left`;

  const handleToggle = (on: boolean) => {
    if (on) {
      desk.reminderStart(state.intervalMins || DEFAULT_MINS).catch(() => {});
    } else {
      desk.reminderStop().catch(() => {});
    }
  };

  const handleIntervalChange = (m: number) => {
    // Restart if already running so the new interval takes effect immediately.
    if (state.running) {
      desk.reminderStart(m).catch(() => {});
    } else {
      setState((s) => ({ ...s, intervalMins: m }));
    }
  };

  return (
    <div className="px-4 pt-3 pb-2.5">
      <div className="flex items-center gap-2.5">
        <span className="whitespace-nowrap text-[13px] font-semibold">Stand Reminder</span>
        <div className="flex-1" />
        <Toggle on={state.running} onChange={handleToggle} />
      </div>

      <div className="mt-1 flex flex-wrap items-center gap-[5px] text-[11.5px] text-text-dim [font-feature-settings:'tnum']">
        {state.running ? (
          <>
            <span>Every</span>
            <IntervalPicker mins={state.intervalMins} onChange={handleIntervalChange} />
            <span className="opacity-50">·</span>
            <span className="whitespace-nowrap">{remainingLabel}</span>
          </>
        ) : (
          <>
            <span>Paused</span>
            <span className="opacity-50">·</span>
            <IntervalPicker mins={state.intervalMins} onChange={handleIntervalChange} />
          </>
        )}
      </div>

      <div className="relative mt-2.5 h-1 overflow-hidden rounded bg-track-bg">
        <div
          className="absolute inset-0 rounded transition-[width,background] duration-200"
          style={{
            width: `${pct * 100}%`,
            background: state.running
              ? 'linear-gradient(90deg, var(--accent-base), var(--accent-ink))'
              : 'var(--text-faint)',
          }}
        />
      </div>
    </div>
  );
}
