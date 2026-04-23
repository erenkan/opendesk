import type { Update } from '@tauri-apps/plugin-updater';
import { ChevronLeft, Loader2 } from 'lucide-react';
import { useState } from 'react';
import { useResolvedTheme } from '@/hooks/useResolvedTheme';
import type { AccentId } from '@/lib/accents';
import type { ThemeMode } from '@/lib/constants';
import type { UnitSystem } from '@/lib/units';
import { checkForUpdate, installAndRelaunch, type UpdateCheck } from '@/lib/updater';
import { AccentPicker } from './accent-picker';
import { ThemePicker } from './theme-picker';
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
  const [busy, setBusy] = useState<'idle' | 'checking' | 'installing'>('idle');
  const [status, setStatus] = useState<UpdateCheck | null>(null);
  const [pending, setPending] = useState<Update | null>(null);

  async function onCheck() {
    setBusy('checking');
    setStatus(null);
    const { update, result } = await checkForUpdate();
    setStatus(result);
    setPending(update);
    setBusy('idle');
  }

  async function onInstall() {
    if (!pending) return;
    setBusy('installing');
    try {
      await installAndRelaunch(pending);
    } catch (e) {
      setStatus({ kind: 'error', message: String(e) });
      setBusy('idle');
    }
  }

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
            <strong className="font-semibold text-text-main">{dark ? 'Dark' : 'Light'}</strong>.
          </div>
        )}

        <div className="mb-1.5 text-[10.5px] font-medium text-text-dim">Accent</div>
        <AccentPicker accent={accent} onChange={setAccent} />

        <div className="mb-2 mt-3 text-[10.5px] font-semibold uppercase tracking-[0.08em] text-text-faint">
          Units
        </div>
        <UnitPicker unit={unit} onChange={setUnit} />

        <div className="my-3.5 h-px bg-divider" />

        <div className="mb-2 flex items-center justify-between text-[10.5px] tracking-[0.02em] text-text-faint">
          <span>OpenDesk 0.0.1</span>
          <button
            type="button"
            onClick={status?.kind === 'available' ? onInstall : onCheck}
            disabled={busy !== 'idle'}
            className="flex items-center gap-1 rounded-md border border-chip-border bg-chip-bg px-2 py-1 text-[10.5px] font-medium text-text-main cursor-pointer hover:bg-chip-hover disabled:opacity-60 disabled:cursor-not-allowed"
            style={{ fontFamily: 'inherit' }}
          >
            {busy !== 'idle' && <Loader2 size={10} strokeWidth={2} className="animate-spin" />}
            {busy === 'checking' && 'Checking…'}
            {busy === 'installing' && 'Installing…'}
            {busy === 'idle' && status?.kind === 'available' && `Install ${status.version}`}
            {busy === 'idle' && status?.kind !== 'available' && 'Check for updates'}
          </button>
        </div>

        {status?.kind === 'none' && (
          <div className="text-[10px] text-text-faint">You're on the latest version.</div>
        )}
        {status?.kind === 'error' && (
          <div className="text-[10px] text-red-500">{status.message}</div>
        )}
        {status?.kind === 'available' && status.notes && (
          <div className="rounded border border-chip-border bg-chip-bg px-2 py-1.5 text-[10px] leading-relaxed text-text-dim whitespace-pre-wrap">
            {status.notes}
          </div>
        )}
      </div>
    </div>
  );
}
