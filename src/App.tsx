import { useEffect, useState } from 'react';
import { Popover } from '@/components/popover/popover';
import { useResolvedTheme } from '@/hooks/useResolvedTheme';
import { type AccentId, applyAccent, DEFAULT_ACCENT, loadAccent, saveAccent } from '@/lib/accents';
import type { ThemeMode } from '@/lib/constants';
import { DEFAULT_UNIT, loadUnit, saveUnit, type UnitSystem } from '@/lib/units';

export default function App() {
  const [themeMode, setThemeMode] = useState<ThemeMode>('system');
  const [accent, setAccentState] = useState<AccentId>(DEFAULT_ACCENT);
  const [unit, setUnitState] = useState<UnitSystem>(DEFAULT_UNIT);
  const resolved = useResolvedTheme(themeMode);

  // Mirror dark mode to <html> so Tailwind's `dark:` + our CSS var swap fires.
  useEffect(() => {
    document.documentElement.classList.toggle('dark', resolved === 'dark');
  }, [resolved]);

  // Load + apply persisted accent once on mount, then any subsequent change
  // flows through setAccent below.
  useEffect(() => {
    const id = loadAccent();
    setAccentState(id);
    applyAccent(id);
    setUnitState(loadUnit());
  }, []);

  const setAccent = (id: AccentId) => {
    setAccentState(id);
    applyAccent(id);
    saveAccent(id);
  };

  const setUnit = (u: UnitSystem) => {
    setUnitState(u);
    saveUnit(u);
  };

  return (
    // 10px padding gives the popover's CSS shadow room to render on
    // transparent space and keeps the rounded corners from butting up
    // against the NSPanel rectangular frame (corner artifact source).
    <div className="fixed inset-0 overflow-hidden p-[10px]">
      <Popover
        themeMode={themeMode}
        setThemeMode={setThemeMode}
        accent={accent}
        setAccent={setAccent}
        unit={unit}
        setUnit={setUnit}
      />
    </div>
  );
}
