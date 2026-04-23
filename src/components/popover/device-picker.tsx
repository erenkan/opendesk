import { useEffect, useState } from 'react';
import { type DeviceInfo, desk } from '@/lib/desk';

type Props = {
  onPicked: () => void;
};

export function DevicePicker({ onPicked }: Props) {
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
  const [scanning, setScanning] = useState(false);

  async function scan() {
    setScanning(true);
    setDevices([]);
    try {
      const list = await desk.scanDevices(5000);
      setDevices(list);
    } finally {
      setScanning(false);
    }
  }

  useEffect(() => {
    scan();
  }, []);

  async function pick(address: string) {
    try {
      await desk.connectDevice(address);
      onPicked();
    } catch {
      // Error surfaces via useDesk subscription.
    }
  }

  return (
    <div className="max-h-[220px] overflow-y-auto border-t border-divider bg-picker-panel-bg px-[14px] py-2">
      <div className="mb-1.5 flex items-center gap-1.5">
        <span className="text-[10.5px] font-semibold uppercase tracking-[0.08em] text-text-faint">
          Devices
        </span>
        <div className="flex-1" />
        <button
          type="button"
          onClick={scan}
          disabled={scanning}
          className="rounded-[5px] border border-chip-border bg-chip-bg px-2 py-[3px] text-[11px] text-text-main cursor-pointer disabled:cursor-default disabled:opacity-70"
        >
          {scanning ? 'Scanning…' : 'Rescan'}
        </button>
      </div>

      {devices.length === 0 && !scanning && (
        <div className="py-1 text-[11px] text-text-dim">
          No devices. Put the desk in pairing mode and tap Rescan.
        </div>
      )}

      {devices.map((d) => (
        <button
          key={d.address}
          type="button"
          onClick={() => pick(d.address)}
          className="mb-0.5 flex w-full items-center gap-2 rounded-md border border-chip-border bg-transparent px-2 py-1.5 text-left text-xs text-text-main cursor-pointer hover:bg-chip-bg"
        >
          <span className="flex-1 overflow-hidden text-ellipsis whitespace-nowrap">{d.device}</span>
          <span className="font-mono text-[10px] text-text-faint">{d.address.slice(-8)}</span>
        </button>
      ))}
    </div>
  );
}
