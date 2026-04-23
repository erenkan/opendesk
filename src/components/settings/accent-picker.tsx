import { Check } from 'lucide-react';
import { ACCENTS, type AccentId } from '@/lib/accents';
import { cn } from '@/lib/cn';

type Props = {
  accent: AccentId;
  onChange: (id: AccentId) => void;
};

export function AccentPicker({ accent, onChange }: Props) {
  return (
    <div className="mb-3 flex items-center gap-2">
      {ACCENTS.map((a) => {
        const active = accent === a.id;
        return (
          <button
            key={a.id}
            type="button"
            onClick={() => onChange(a.id)}
            aria-label={a.label}
            aria-pressed={active}
            title={a.label}
            className={cn(
              'flex h-7 w-7 shrink-0 cursor-pointer items-center justify-center rounded-full border-none p-0 transition-all duration-150',
              active
                ? 'scale-110 shadow-[0_0_0_2px_var(--pop-bg),0_0_0_3.5px_var(--accent-ink)]'
                : 'shadow-[inset_0_1px_0_rgba(255,255,255,0.25),0_1px_2px_rgba(0,0,0,0.15)] hover:scale-105',
            )}
            style={{ background: a.base }}
          >
            {active && (
              <Check
                size={13}
                strokeWidth={2.6}
                className="text-white drop-shadow-[0_1px_1px_rgba(0,0,0,0.35)]"
              />
            )}
          </button>
        );
      })}
    </div>
  );
}
