import { Minus, Plus } from 'lucide-react';
import { useState } from 'react';
import { StepperButton } from '@/components/ui/nudge-button';
import { MAX_H, MIN_H } from '@/lib/constants';
import {
  displayNumber,
  formatHeight,
  formatUnitSuffix,
  stepCm,
  type UnitSystem,
} from '@/lib/units';

type Props = {
  height: number;
  label: string | null;
  unit: UnitSystem;
  onHeight: (h: number) => void;
  onLabel: (l: string | null) => void;
  /** Persist changes and close. Moving to the height is a separate action —
   *  the user clicks the preset row in the list to trigger that. */
  onSave: () => void;
  onCancel: () => void;
  currentHeight: number;
};

export function CustomEditor({
  height,
  label,
  unit,
  onHeight,
  onLabel,
  onSave,
  onCancel,
  currentHeight,
}: Props) {
  const clamp = (v: number) => Math.max(MIN_H, Math.min(MAX_H, Math.round(v)));
  const step = stepCm(unit);
  const [draft, setDraft] = useState(label || '');
  const commit = () => onLabel(draft.trim() || null);

  return (
    <div className="mt-1 flex flex-col gap-2.5 rounded-[10px] border border-chip-border bg-[rgba(0,0,0,0.03)] p-3 dark:bg-white/[0.035]">
      <div className="flex items-center justify-between">
        <span className="text-[10.5px] font-semibold uppercase tracking-[0.08em] text-text-faint">
          Edit Preset
        </span>
        <button
          type="button"
          onClick={() => onHeight(currentHeight)}
          className="rounded bg-transparent border-none px-1.5 py-0.5 text-[10.5px] font-medium cursor-pointer"
          style={{ color: 'var(--accent-base)' }}
          title="Use current desk height"
        >
          Use current ({formatHeight(currentHeight, unit)})
        </button>
      </div>

      <div>
        <div className="mb-1 text-[11px] text-text-dim">Name</div>
        <input
          type="text"
          value={draft}
          placeholder="e.g. Standup, Coffee"
          maxLength={24}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              commit();
              onSave();
            }
            if (e.key === 'Escape') onCancel();
          }}
          className="w-full rounded-md border border-chip-border bg-input-bg px-[9px] py-1.5 text-[12.5px] text-text-main outline-none"
        />
      </div>

      <div>
        <div className="mb-1 text-[11px] text-text-dim">Target height</div>
        <div className="flex items-stretch gap-2">
          <StepperButton onClick={() => onHeight(clamp(height - step))}>
            <Minus size={10} strokeWidth={1.8} />
          </StepperButton>
          <div className="flex flex-1 items-baseline justify-center gap-[3px] rounded-md border border-chip-border bg-input-bg px-2.5 py-1.5 [font-feature-settings:'tnum']">
            <span className="text-xl font-semibold -tracking-[0.01em] text-text-main">
              {displayNumber(height, unit)}
            </span>
            <span className="text-[11px] text-text-dim">{formatUnitSuffix(unit)}</span>
          </div>
          <StepperButton onClick={() => onHeight(clamp(height + step))}>
            <Plus size={10} strokeWidth={1.8} />
          </StepperButton>
        </div>

        <div className="relative mt-2 h-4">
          <div className="absolute inset-x-0 top-1/2 h-[3px] -translate-y-1/2 rounded bg-track-bg" />
          <div
            className="absolute left-0 top-1/2 h-[3px] -translate-y-1/2 rounded"
            style={{
              width: `${((height - MIN_H) / (MAX_H - MIN_H)) * 100}%`,
              background: 'linear-gradient(90deg, var(--accent-base), var(--accent-ink))',
            }}
          />
          <input
            type="range"
            min={MIN_H}
            max={MAX_H}
            step={1}
            value={height}
            onChange={(e) => onHeight(+e.target.value)}
            className="absolute inset-0 m-0 w-full cursor-pointer opacity-0"
          />
          <div
            className="pointer-events-none absolute top-1/2 h-[14px] w-[14px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-white shadow-[0_1px_4px_rgba(0,0,0,0.25),0_0_0_0.5px_rgba(0,0,0,0.1)]"
            style={{
              left: `${((height - MIN_H) / (MAX_H - MIN_H)) * 100}%`,
            }}
          />
        </div>
        <div className="mt-0.5 flex justify-between text-[10px] text-text-faint [font-feature-settings:'tnum']">
          <span>{formatHeight(MIN_H, unit)}</span>
          <span>{formatHeight(MAX_H, unit)}</span>
        </div>
      </div>

      <div className="mt-0.5 flex gap-1.5">
        <button
          type="button"
          onClick={onCancel}
          className="flex-1 rounded-md border border-chip-border bg-chip-bg px-2.5 py-[7px] text-xs font-medium text-text-main cursor-pointer"
          style={{ fontFamily: 'inherit' }}
        >
          Cancel
        </button>
        <button
          type="button"
          onClick={() => {
            commit();
            onSave();
          }}
          className="flex-[1.4] rounded-md border px-2.5 py-[7px] text-xs font-semibold text-white cursor-pointer shadow-[0_1px_0_rgba(255,255,255,0.3)_inset]"
          style={{
            fontFamily: 'inherit',
            background: 'linear-gradient(180deg, var(--accent-base), var(--accent-ink))',
            borderColor: 'var(--accent-ink)',
            boxShadow: '0 1px 0 rgba(255,255,255,0.3) inset, 0 2px 6px var(--accent-ink)44',
          }}
        >
          Save
        </button>
      </div>
    </div>
  );
}
