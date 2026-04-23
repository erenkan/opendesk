import { useState } from 'react';
import { cn } from '@/lib/cn';
import { Settings } from 'lucide-react';
import { DevicePicker } from './device-picker';
import type { ConnectionUpdate } from '@/lib/desk';

type Props = {
  connection: ConnectionUpdate;
  onDisconnect: () => void;
  onSettings: () => void;
};

export function Footer({ connection, onDisconnect, onSettings }: Props) {
  const [pickerOpen, setPickerOpen] = useState(false);

  const connected = connection.state === 'connected';
  const connLabel = (() => {
    switch (connection.state) {
      case 'connected':
        return connection.device;
      case 'connecting':
        return `Connecting… ${connection.device}`;
      case 'scanning':
        return 'Scanning…';
      case 'reconnecting':
        return `Reconnecting (${connection.attempt})`;
      default:
        return 'Not connected';
    }
  })();

  return (
    <>
      {pickerOpen && <DevicePicker onPicked={() => setPickerOpen(false)} />}

      <div className="flex shrink-0 items-center gap-1 border-t border-divider bg-footer-bg px-3 pb-2.5 pt-2">
        <button
          type="button"
          onClick={() => setPickerOpen((o) => !o)}
          className="flex min-w-0 items-center gap-1.5 border-none bg-transparent p-0 text-[11px] text-text-dim cursor-pointer"
          title={connected ? 'Open device picker' : 'Scan devices'}
        >
          <span
            className={cn(
              'relative h-2 w-2 shrink-0 rounded-full',
              connected ? 'bg-green-live' : 'bg-red-dead',
            )}
          >
            {!connected && (
              <span className="absolute inset-0 rounded-full bg-red-dead animate-ping-slow" />
            )}
          </span>
          <span className="whitespace-nowrap font-medium tracking-[0.01em]">
            {connected ? 'BLE Connected' : 'BLE'}
          </span>
          <span className="max-w-[160px] overflow-hidden text-ellipsis whitespace-nowrap text-text-faint">
            · {connLabel}
          </span>
          <span className="text-[10px] text-text-faint">{pickerOpen ? '▾' : '▸'}</span>
        </button>
        <div className="flex-1" />
        {connected && (
          <button
            type="button"
            onClick={onDisconnect}
            title="Disconnect"
            className="rounded-[5px] border-none bg-transparent px-2 py-1 text-[11px] text-text-dim cursor-pointer hover:bg-chip-bg"
          >
            ✕
          </button>
        )}
        <button
          type="button"
          onClick={onSettings}
          title="Settings"
          className="flex items-center gap-1 rounded-[5px] border-none bg-transparent px-1.5 py-1 text-text-dim cursor-pointer hover:bg-chip-bg"
        >
          <Settings size={13} strokeWidth={1.6} />
        </button>
      </div>
    </>
  );
}
