import { useState } from 'react';
import { cn } from '@/lib/cn';

const PRESETS = [15, 30, 45, 60, 90, 120] as const;
const MIN_MINS = 1;
const MAX_MINS = 240;

function label(m: number): string {
  if (m === 60) return '1 Hour';
  if (m > 60 && m % 60 === 0) return `${m / 60} Hours`;
  return `${m} min`;
}

export function IntervalPicker({
  mins,
  onChange,
}: {
  mins: number;
  onChange: (m: number) => void;
}) {
  const [open, setOpen] = useState(false);
  const [custom, setCustom] = useState<string>('');

  function commitCustom() {
    const n = Math.round(Number(custom));
    if (!Number.isFinite(n) || n < MIN_MINS || n > MAX_MINS) return;
    onChange(n);
    setCustom('');
    setOpen(false);
  }

  return (
    <span className="relative inline-block">
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        className="rounded-[5px] border border-chip-border bg-chip-bg px-[6px] py-px text-[11px] font-semibold text-text-main cursor-pointer"
      >
        {label(mins)} ▾
      </button>
      {open && (
        <>
          <div onClick={() => setOpen(false)} className="fixed inset-0 z-[9]" />
          <div
            className={cn(
              'absolute bottom-full left-0 mb-1 min-w-[110px] z-10 rounded-[7px] p-[3px]',
              'border border-chip-border bg-picker-bg backdrop-blur-[30px]',
              'shadow-[0_8px_24px_rgba(0,0,0,0.25),0_0_0_0.5px_rgba(0,0,0,0.1)]',
            )}
          >
            {PRESETS.map((o) => (
              <div
                key={o}
                onClick={() => {
                  onChange(o);
                  setOpen(false);
                }}
                className={cn(
                  'cursor-pointer rounded px-2 py-1 text-xs text-text-main',
                  o === mins && 'bg-chip-bg',
                  'hover:bg-chip-bg',
                )}
              >
                {label(o)}
              </div>
            ))}
            <div className="my-[3px] h-px bg-divider" />
            <div className="flex items-center gap-1 px-1 py-1">
              <input
                type="number"
                min={MIN_MINS}
                max={MAX_MINS}
                value={custom}
                placeholder="min"
                onChange={(e) => setCustom(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && commitCustom()}
                className="w-12 rounded border border-chip-border bg-chip-bg px-1.5 py-0.5 text-[11px] text-text-main outline-none"
              />
              <button
                type="button"
                onClick={commitCustom}
                className="rounded border border-chip-border bg-chip-bg px-1.5 py-0.5 text-[10.5px] font-medium text-text-main cursor-pointer hover:bg-chip-hover"
                style={{ fontFamily: 'inherit' }}
              >
                Set
              </button>
            </div>
          </div>
        </>
      )}
    </span>
  );
}
