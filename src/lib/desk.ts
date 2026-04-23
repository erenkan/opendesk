import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// Event payloads — must stay in sync with src-tauri/src/events.rs.

export type HeightUpdate = {
  cm: number;
  mm: number;
  speed: number;
  moving: boolean;
};

export type ConnectionUpdate =
  | { state: 'disconnected' }
  | { state: 'scanning' }
  | { state: 'connecting'; device: string }
  | { state: 'connected'; device: string; address: string }
  | { state: 'reconnecting'; attempt: number };

export type DeskError = {
  code:
    | 'no_adapter'
    | 'permission_denied'
    | 'not_found'
    | 'not_connected'
    | 'missing_characteristic'
    | 'invalid_height'
    | 'move_timeout'
    | 'btleplug'
    | 'io';
  message: string;
  recoverable: boolean;
};

export type DeviceInfo = { device: string; address: string };

export type StatusSnapshot = {
  connection: ConnectionUpdate;
  lastHeight: HeightUpdate | null;
};

export type ReminderState = {
  running: boolean;
  intervalMins: number;
  /** Unix ms of the next deadline, or null if not running. */
  deadlineMs: number | null;
};

export type ReminderFirePayload = { intervalMins: number };

const EVT = {
  height: 'desk://height',
  connection: 'desk://connection',
  error: 'desk://error',
  panelVisibility: 'desk://panel-visibility',
  reminderState: 'desk://reminder-state',
  reminderFire: 'desk://reminder-fire',
} as const;

export const desk = {
  scanAndConnect: () => invoke<DeviceInfo>('scan_and_connect'),
  scanDevices: (durationMs = 4000) =>
    invoke<DeviceInfo[]>('scan_devices', { durationMs }),
  connectDevice: (address: string) =>
    invoke<DeviceInfo>('connect_device', { address }),
  disconnect: () => invoke<void>('disconnect_desk'),
  pauseSession: () => invoke<void>('pause_session'),
  resumeSession: () => invoke<void>('resume_session'),

  moveUpStart: () => invoke<void>('move_up_start'),
  moveDownStart: () => invoke<void>('move_down_start'),
  moveStop: () => invoke<void>('move_stop'),
  moveTo: (heightCm: number) => invoke<void>('move_to', { heightCm }),

  getStatus: () => invoke<StatusSnapshot>('get_status'),

  onHeight(cb: (h: HeightUpdate) => void): Promise<UnlistenFn> {
    return listen<HeightUpdate>(EVT.height, (e) => cb(e.payload));
  },
  onConnection(cb: (c: ConnectionUpdate) => void): Promise<UnlistenFn> {
    return listen<ConnectionUpdate>(EVT.connection, (e) => cb(e.payload));
  },
  onError(cb: (e: DeskError) => void): Promise<UnlistenFn> {
    return listen<DeskError>(EVT.error, (e) => cb(e.payload));
  },
  onPanelVisibility(cb: (visible: boolean) => void): Promise<UnlistenFn> {
    return listen<boolean>(EVT.panelVisibility, (e) => cb(e.payload));
  },

  reminderStart: (mins: number) => invoke<ReminderState>('reminder_start', { mins }),
  reminderStop: () => invoke<void>('reminder_stop'),
  reminderState: () => invoke<ReminderState>('reminder_state'),

  /**
   * macOS-only delivery path that bypasses `tauri-plugin-notification`'s
   * deprecated `NSUserNotification` chain. Lets us attach our own bundle
   * icon, which `mac-notification-sys` can't pick up on LSUIElement apps.
   * No-op on non-macOS — caller should fall back to `sendNotification`
   * from `@tauri-apps/plugin-notification`.
   */
  sendNativeNotification: (title: string, body: string) =>
    invoke<void>('send_native_notification', { title, body }),

  onReminderState(cb: (s: ReminderState) => void): Promise<UnlistenFn> {
    return listen<ReminderState>(EVT.reminderState, (e) => cb(e.payload));
  },
  onReminderFire(cb: (p: ReminderFirePayload) => void): Promise<UnlistenFn> {
    return listen<ReminderFirePayload>(EVT.reminderFire, (e) => cb(e.payload));
  },
};

export const LAST_DEVICE_KEY = 'opendesk:last-device';
export type LastDevice = { device: string; address: string };
