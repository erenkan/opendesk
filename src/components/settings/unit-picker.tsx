import { cn } from '@/lib/cn';
import type { UnitSystem } from '@/lib/units';

type Props = {
  unit: UnitSystem;
  onChange: (u: UnitSystem) => void;
};

const OPTIONS: { id: UnitSystem; label: string; hint: string }[] = [
  { id: 'cm', label: 'cm', hint: 'Metric' },
  { id: 'in', label: 'in', hint: 'Imperial' },
];

export function UnitPicker({ unit, onChange }: Props) {
  return (
    <div className="mb-3 grid grid-cols-2 gap-2">
      {OPTIONS.map((o) => {
        const active = unit === o.id;
        return (
          <button
            key={o.id}
            type="button"
            onClick={() => onChange(o.id)}
            aria-pressed={active}
            className={cn(
              'flex flex-col items-start gap-0.5 rounded-[7px] border px-2.5 py-1.5 text-left transition-colors',
              active
                ? 'text-white shadow-[0_1px_0_rgba(255,255,255,0.3)_inset]'
                : 'border-chip-border bg-chip-bg text-text-main hover:bg-[rgba(20,18,16,0.06)] dark:hover:bg-white/[0.09]',
            )}
            style={
              active
                ? {
                    background:
                      'linear-gradient(180deg, var(--accent-base), var(--accent-ink))',
                    borderColor: 'var(--accent-ink)',
                  }
                : undefined
            }
          >
            <span className="text-[13px] font-semibold">{o.label}</span>
            <span
              className={cn(
                'text-[10px]',
                active ? 'text-white/75' : 'text-text-faint',
              )}
            >
              {o.hint}
            </span>
          </button>
        );
      })}
    </div>
  );
}
