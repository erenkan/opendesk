import { useCallback, useEffect, useRef, useState } from 'react';
import type { UnlistenFn } from '@tauri-apps/api/event';

import {
  desk,
  type ConnectionUpdate,
  type DeskError,
  type HeightUpdate,
} from '../lib/desk';
import { clampCm } from '../lib/constants';

export type UseDesk = {
  connection: ConnectionUpdate;
  height: HeightUpdate | null;
  error: DeskError | null;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  startUp: () => Promise<void>;
  startDown: () => Promise<void>;
  stop: () => Promise<void>;
  moveTo: (cm: number) => Promise<void>;
};

/**
 * Single source of truth for BLE desk state in the React tree. Subscribes to
 * all three backend events and primes the initial snapshot via `get_status`.
 *
 * Commands auto-clear the previous error and re-raise it on failure so the
 * UI can toast without custom try/catch at every call site.
 */
export function useDesk(): UseDesk {
  const [connection, setConnection] = useState<ConnectionUpdate>({ state: 'disconnected' });
  const [height, setHeight] = useState<HeightUpdate | null>(null);
  const [error, setError] = useState<DeskError | null>(null);

  const unsubs = useRef<UnlistenFn[]>([]);

  useEffect(() => {
    let cancelled = false;

    Promise.all([
      desk.onConnection((c) => { if (!cancelled) setConnection(c); }),
      desk.onHeight((h) => { if (!cancelled) setHeight(h); }),
      desk.onError((e) => { if (!cancelled) setError(e); }),
    ]).then((fns) => {
      if (cancelled) fns.forEach((f) => f());
      else unsubs.current = fns;
    });

    desk.getStatus().then((snap) => {
      if (cancelled) return;
      setConnection(snap.connection);
      if (snap.lastHeight) setHeight(snap.lastHeight);
    }).catch(() => { /* first boot before any event — ignore */ });

    return () => {
      cancelled = true;
      unsubs.current.forEach((f) => f());
      unsubs.current = [];
    };
  }, []);

  const run = useCallback(async <T,>(op: () => Promise<T>): Promise<T | void> => {
    setError(null);
    try {
      return await op();
    } catch (e) {
      setError(e as DeskError);
    }
  }, []);

  return {
    connection,
    height,
    error,
    connect: () => run(() => desk.scanAndConnect()).then(() => undefined),
    disconnect: () => run(() => desk.disconnect()).then(() => undefined),
    startUp: () => run(() => desk.moveUpStart()).then(() => undefined),
    startDown: () => run(() => desk.moveDownStart()).then(() => undefined),
    stop: () => run(() => desk.moveStop()).then(() => undefined),
    moveTo: (cm: number) => run(() => desk.moveTo(clampCm(cm))).then(() => undefined),
  };
}
