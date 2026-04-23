import { useState } from 'react';
import { cn } from '@/lib/cn';

const OPTS = [1, 15, 30, 45, 60, 90, 120] as const;

function label(m: number): string {
  if (m === 60) return '1 Hour';
  if (m > 60) return `${m / 60} Hours`;
  if (m === 1) return '1 min (test)';
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
              'absolute bottom-full left-0 mb-1 min-w-[80px] z-10 rounded-[7px] p-[3px]',
              'border border-chip-border bg-picker-bg backdrop-blur-[30px]',
              'shadow-[0_8px_24px_rgba(0,0,0,0.25),0_0_0_0.5px_rgba(0,0,0,0.1)]',
            )}
          >
            {OPTS.map((o) => (
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
          </div>
        </>
      )}
    </span>
  );
}
