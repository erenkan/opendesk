import { Check, Contrast, Moon, Sun } from 'lucide-react';
import { cn } from '@/lib/cn';
import type { ThemeMode } from '@/lib/constants';

type Props = {
  mode: ThemeMode;
  onChange: (m: ThemeMode) => void;
};

const MODES: { id: ThemeMode; label: string; icon: JSX.Element }[] = [
  { id: 'light', label: 'Light', icon: <Sun size={14} strokeWidth={1.5} /> },
  { id: 'dark', label: 'Dark', icon: <Moon size={14} strokeWidth={1.5} /> },
  { id: 'system', label: 'Auto', icon: <Contrast size={14} strokeWidth={1.5} /> },
];

export function ThemePicker({ mode, onChange }: Props) {
  return (
    <div className="mb-3 grid grid-cols-3 gap-2">
      {MODES.map((m) => {
        const active = mode === m.id;
        return (
          <button
            key={m.id}
            type="button"
            onClick={() => onChange(m.id)}
            className="flex flex-col rounded-lg border-none bg-transparent p-0 cursor-pointer"
            style={{ fontFamily: 'inherit' }}
          >
            <div
              className={cn(
                'relative h-[46px] overflow-hidden rounded-[7px] transition-all duration-150',
                active
                  ? 'ring-[3px] ring-accent-base/20'
                  : 'shadow-[inset_0_1px_0_rgba(255,255,255,0.25)]',
              )}
              style={{
                background:
                  m.id === 'light'
                    ? 'linear-gradient(135deg, #fafafa, #e8e4df)'
                    : m.id === 'dark'
                      ? 'linear-gradient(135deg, #2a2a30, #16161a)'
                      : 'linear-gradient(135deg, #fafafa 0%, #fafafa 50%, #2a2a30 50%, #16161a 100%)',
                border: active
                  ? '1.5px solid var(--accent-base)'
                  : '0.5px solid var(--chip-border)',
              }}
            >
              <div
                className="absolute left-2 bottom-[7px] top-[7px] w-[3px] rounded"
                style={{
                  background: m.id === 'light' ? 'rgba(20,18,16,0.15)' : 'rgba(255,255,255,0.2)',
                }}
              >
                <div
                  className="absolute bottom-0 left-0 right-0 rounded"
                  style={{ height: '65%', background: 'var(--accent-base)' }}
                />
              </div>
              <div
                className="absolute right-2 top-2.5 left-[18px] h-[5px] rounded"
                style={{
                  background: m.id === 'light' ? 'rgba(20,18,16,0.14)' : 'rgba(255,255,255,0.18)',
                }}
              />
              <div
                className="absolute right-3.5 top-5 left-[18px] h-1 rounded"
                style={{
                  background: m.id === 'light' ? 'rgba(20,18,16,0.09)' : 'rgba(255,255,255,0.1)',
                }}
              />
              <div
                className="absolute right-[18px] top-[30px] left-[18px] h-1 rounded"
                style={{
                  background: m.id === 'light' ? 'rgba(20,18,16,0.09)' : 'rgba(255,255,255,0.1)',
                }}
              />
              {active && (
                <div
                  className="absolute right-[3px] top-[3px] flex h-[14px] w-[14px] items-center justify-center rounded-full shadow-[0_1px_3px_rgba(0,0,0,0.3)] text-white"
                  style={{ background: 'var(--accent-base)' }}
                >
                  <Check size={8} strokeWidth={2.4} />
                </div>
              )}
            </div>
            <div
              className={cn(
                'mt-1 flex items-center justify-center gap-1 text-[11.5px] font-medium',
                active ? 'text-text-main' : 'text-text-dim',
              )}
            >
              <span className="opacity-80">{m.icon}</span>
              {m.label}
            </div>
          </button>
        );
      })}
    </div>
  );
}
