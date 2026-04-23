import { ChevronDown, ChevronUp } from 'lucide-react';
import { useEffect, useRef } from 'react';
import { cn } from '@/lib/cn';
import { MAX_H, MIN_H } from '@/lib/constants';
import { desk } from '@/lib/desk';

type Props = {
  heightCm: number;
  disabled: boolean;
  moving: boolean;
  onManualMove?: () => void;
};

/**
 * Vertical "Manual" rocker — chevron buttons stacked with a small height
 * indicator strip between them. Press-and-hold mirrors a physical Linak
 * handset: the desk keeps moving until the button is released.
 */
export function SliderColumn({ heightCm, disabled, moving, onManualMove }: Props) {
  const holding = useRef<'up' | 'down' | null>(null);

  const start = (dir: 'up' | 'down') => {
    if (disabled || holding.current === dir) return;
    holding.current = dir;
    onManualMove?.();
    (dir === 'up' ? desk.moveUpStart() : desk.moveDownStart()).catch(() => {});
  };
  const stop = () => {
    if (!holding.current) return;
    holding.current = null;
    desk.moveStop().catch(() => {});
  };

  useEffect(() => {
    const onUp = () => stop();
    window.addEventListener('mouseup', onUp);
    window.addEventListener('mouseleave', onUp);
    return () => {
      window.removeEventListener('mouseup', onUp);
      window.removeEventListener('mouseleave', onUp);
    };
  }, []);

  const pct = Math.max(0, Math.min(1, (heightCm - MIN_H) / (MAX_H - MIN_H)));

  return (
    <div className="flex select-none flex-col items-center gap-2">
      <div className="-mb-0.5 text-[10.5px] font-semibold uppercase tracking-[0.08em] text-text-faint">
        Manual
      </div>

      <RockerButton dir="up" disabled={disabled} onHold={() => start('up')} onRelease={stop} />

      <div
        aria-hidden
        className="relative h-[82px] w-[6px] overflow-hidden rounded-full border border-chip-border bg-track-bg"
      >
        <div
          className={cn(
            'absolute inset-x-0 bottom-0 transition-[height] duration-100',
            moving && 'animate-pulse-dot',
          )}
          style={{
            height: `${pct * 100}%`,
            background: 'linear-gradient(to top, var(--accent-ink), var(--accent-base))',
          }}
        />
      </div>

      <RockerButton dir="down" disabled={disabled} onHold={() => start('down')} onRelease={stop} />

      <div className="-mt-0.5 text-center text-[10px] leading-tight tracking-[0.02em] text-text-faint">
        Hold
        <br />
        to move
      </div>
    </div>
  );
}

function RockerButton({
  dir,
  disabled,
  onHold,
  onRelease,
}: {
  dir: 'up' | 'down';
  disabled: boolean;
  onHold: () => void;
  onRelease: () => void;
}) {
  return (
    <button
      type="button"
      disabled={disabled}
      onMouseDown={(e) => {
        e.preventDefault();
        onHold();
      }}
      onMouseUp={onRelease}
      onMouseLeave={onRelease}
      onTouchStart={(e) => {
        e.preventDefault();
        onHold();
      }}
      onTouchEnd={onRelease}
      onTouchCancel={onRelease}
      aria-label={dir === 'up' ? 'Raise desk' : 'Lower desk'}
      className={cn(
        'flex h-[62px] w-12 items-center justify-center rounded-[10px] p-0 transition-[transform,box-shadow,background] duration-75',
        '[background:linear-gradient(180deg,#ffffff,#f3efe9)]',
        'shadow-[inset_0_1px_0_rgba(255,255,255,0.9),inset_0_-2px_0_rgba(20,18,16,0.06),0_2px_4px_rgba(20,18,16,0.08)]',
        'dark:[background:linear-gradient(180deg,rgba(255,255,255,0.09),rgba(255,255,255,0.04))]',
        'dark:shadow-[inset_0_1px_0_rgba(255,255,255,0.06),inset_0_-1px_0_rgba(0,0,0,0.3),0_2px_4px_rgba(0,0,0,0.25)]',
        'border border-chip-border text-text-main',
        'active:translate-y-px active:[background:linear-gradient(180deg,var(--accent-base),var(--accent-ink))]',
        'active:text-white active:border-[var(--accent-ink)]',
        'active:shadow-[0_1px_0_rgba(255,255,255,0.3)_inset,0_1px_2px_var(--accent-ink)66,inset_0_-2px_0_rgba(0,0,0,0.15)]',
        'disabled:opacity-40 disabled:cursor-not-allowed disabled:active:translate-y-0',
      )}
    >
      {dir === 'up' ? (
        <ChevronUp size={22} strokeWidth={2.4} />
      ) : (
        <ChevronDown size={22} strokeWidth={2.4} />
      )}
    </button>
  );
}
