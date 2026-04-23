import type { ThemeMode } from '@/lib/constants';
import type { AccentId } from '@/lib/accents';
import type { UnitSystem } from '@/lib/units';
import { useResolvedTheme } from '@/hooks/useResolvedTheme';
import { ChevronLeft } from 'lucide-react';
import { ThemePicker } from './theme-picker';
import { AccentPicker } from './accent-picker';
import { UnitPicker } from './unit-picker';

type Props = {
  onBack: () => void;
  themeMode: ThemeMode;
  setThemeMode: (m: ThemeMode) => void;
  accent: AccentId;
  setAccent: (id: AccentId) => void;
  unit: UnitSystem;
  setUnit: (u: UnitSystem) => void;
};

export function SettingsView({
  onBack,
  themeMode,
  setThemeMode,
  accent,
  setAccent,
  unit,
  setUnit,
}: Props) {
  const dark = useResolvedTheme(themeMode) === 'dark';

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-y-auto">
      <div className="flex items-center gap-1 border-b border-divider px-2.5 py-2.5 pl-2">
        <button
          type="button"
          onClick={onBack}
          className="flex items-center gap-[3px] rounded-md border-none bg-transparent px-2 py-1 text-xs font-medium text-text-dim cursor-pointer hover:bg-chip-bg"
          style={{ fontFamily: 'inherit' }}
        >
          <ChevronLeft size={12} strokeWidth={1.8} />
          Back
        </button>
        <div className="flex-1" />
        <div className="text-[13px] font-semibold text-text-main">Settings</div>
        <div className="flex-1" />
        <div className="w-11" />
      </div>

      <div className="px-4 pt-3.5 pb-4">
        <div className="mb-2 text-[10.5px] font-semibold uppercase tracking-[0.08em] text-text-faint">
          Appearance
        </div>
        <ThemePicker mode={themeMode} onChange={setThemeMode} />

        {themeMode === 'system' && (
          <div className="mb-3.5 rounded-[7px] border border-chip-border bg-chip-bg px-2.5 py-2 text-[11px] leading-relaxed text-text-dim">
            Matches your macOS appearance — currently{' '}
            <strong className="font-semibold text-text-main">
              {dark ? 'Dark' : 'Light'}
            </strong>
            .
          </div>
        )}

        <div className="mb-1.5 text-[10.5px] font-medium text-text-dim">
          Accent
        </div>
        <AccentPicker accent={accent} onChange={setAccent} />

        <div className="mb-2 mt-3 text-[10.5px] font-semibold uppercase tracking-[0.08em] text-text-faint">
          Units
        </div>
        <UnitPicker unit={unit} onChange={setUnit} />

        <div className="my-3.5 h-px bg-divider" />

        <div className="flex justify-between text-[10.5px] tracking-[0.02em] text-text-faint">
          <span>OpenDesk 0.0.1</span>
          <span>BLE standing desk</span>
        </div>
      </div>
    </div>
  );
}
