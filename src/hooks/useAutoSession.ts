import { useEffect, useRef } from 'react';
import {
  desk,
  LAST_DEVICE_KEY,
  type ConnectionUpdate,
  type LastDevice,
} from '@/lib/desk';

const PAUSE_DELAY_MS = 15_000;

/**
 * Soft-pause the BLE session while the popover is hidden.
 *
 * Going fully `disconnect` → `connect` on every panel toggle hangs for
 * ~20 s on macOS after the desk firmware idles (btleplug #25/#413 — stale
 * CBPeripheral handles + slow Linak sleep advertising).
 *
 * Instead we only *unsubscribe* from position notifications when the panel
 * hides. The GATT link stays open, so re-showing is instant. If the
 * firmware idles its display based on active subscriptions we also get the
 * power savings; if it doesn't, at least we didn't break reconnect.
 */
export function useAutoSession(connection: ConnectionUpdate) {
  const pauseTimer = useRef<number | null>(null);
  const lastConnectionRef = useRef(connection);
  lastConnectionRef.current = connection;

  useEffect(() => {
    if (connection.state === 'connected') {
      const payload: LastDevice = {
        device: connection.device,
        address: connection.address,
      };
      try {
        localStorage.setItem(LAST_DEVICE_KEY, JSON.stringify(payload));
      } catch {
        /* private mode, ignore */
      }
    }
  }, [connection]);

  useEffect(() => {
    const clearPauseTimer = () => {
      if (pauseTimer.current != null) {
        window.clearTimeout(pauseTimer.current);
        pauseTimer.current = null;
      }
    };

    const onShown = () => {
      clearPauseTimer();
      const cur = lastConnectionRef.current;
      if (cur.state === 'connected') {
        desk.resumeSession().catch(() => {});
        return;
      }
      // Otherwise: cold boot / previous disconnect. Try to reconnect to the
      // last device so the user doesn't have to pick it again.
      const raw = localStorage.getItem(LAST_DEVICE_KEY);
      if (!raw) return;
      try {
        const last = JSON.parse(raw) as LastDevice;
        desk.connectDevice(last.address).catch(() => {
          localStorage.removeItem(LAST_DEVICE_KEY);
        });
      } catch {
        localStorage.removeItem(LAST_DEVICE_KEY);
      }
    };

    const onHidden = () => {
      clearPauseTimer();
      pauseTimer.current = window.setTimeout(() => {
        pauseTimer.current = null;
        desk.pauseSession().catch(() => {});
      }, PAUSE_DELAY_MS);
    };

    let unlisten: (() => void) | undefined;
    desk.onPanelVisibility((visible) => {
      if (visible) onShown();
      else onHidden();
    }).then((u) => {
      unlisten = u;
    });

    // Best-effort resume on mount (popover already open).
    onShown();

    return () => {
      unlisten?.();
      clearPauseTimer();
    };
  }, []);
}
