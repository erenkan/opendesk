import { DeskIcon } from '@/lib/icons';
import { displayNumber, formatUnitSuffix, type UnitSystem } from '@/lib/units';

type Props = {
  heightCm: number;
  moving: boolean;
  activePreset: string | null;
  unit: UnitSystem;
};

export function HeightHeader({ heightCm, moving, activePreset, unit }: Props) {
  return (
    <div className="px-4 pt-[14px] pb-[10px]">
      <div className="flex items-center gap-1.5 text-[10.5px] font-semibold uppercase tracking-[0.08em] text-text-faint">
        <DeskIcon size={10} />
        <span>Current Height</span>
        {moving && (
          <span className="ml-auto flex items-center gap-[3px] text-[10px] font-semibold tracking-[0.04em] text-accent-base">
            <span className="h-[5px] w-[5px] rounded-full bg-accent-base animate-pulse-dot" />
            Moving
          </span>
        )}
      </div>
      <div className="mt-0.5 flex items-baseline gap-1 text-[38px] font-semibold leading-none -tracking-[0.02em] [font-feature-settings:'tnum']">
        <span
          className="bg-clip-text text-transparent"
          style={{
            backgroundImage: 'linear-gradient(180deg, var(--accent-base), var(--accent-ink))',
          }}
        >
          {displayNumber(heightCm, unit)}
        </span>
        <span className="text-base font-medium text-text-dim">{formatUnitSuffix(unit)}</span>
        {activePreset && (
          <span
            className="ml-auto self-center rounded-full border px-2 py-[3px] text-[11px] font-medium"
            style={{
              borderColor: 'var(--accent-ink)',
              background: 'var(--accent-soft)',
              color: 'var(--accent-ink)',
            }}
          >
            {activePreset}
          </span>
        )}
      </div>
    </div>
  );
}
