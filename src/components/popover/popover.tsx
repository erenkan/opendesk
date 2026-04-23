import { useState } from 'react';
import { useDesk } from '@/hooks/useDesk';
import { useAutoSession } from '@/hooks/useAutoSession';
import { MIN_H, MAX_H, type ThemeMode } from '@/lib/constants';
import type { AccentId } from '@/lib/accents';
import type { UnitSystem } from '@/lib/units';
import { HeightHeader } from './height-header';
import { SliderColumn } from './slider-column';
import { PresetList } from './preset-list';
import { Reminder } from './reminder';
import { Footer } from './footer';
import { SettingsView } from '@/components/settings/settings-view';

type Props = {
  themeMode: ThemeMode;
  setThemeMode: (m: ThemeMode) => void;
  accent: AccentId;
  setAccent: (id: AccentId) => void;
  unit: UnitSystem;
  setUnit: (u: UnitSystem) => void;
};

export function Popover({
  themeMode,
  setThemeMode,
  accent,
  setAccent,
  unit,
  setUnit,
}: Props) {
  const d = useDesk();
  useAutoSession(d.connection);
  const [view, setView] = useState<'main' | 'settings'>('main');
  const [activePresetId, setActivePresetId] = useState<string | null>(null);
  const [activePresetLabel, setActivePresetLabel] = useState<string | null>(null);

  const liveHeight = d.height?.cm ?? MIN_H;
  const moving = !!d.height?.moving;
  const connected = d.connection.state === 'connected';

  const moveTo = (cm: number) => {
    const clamped = Math.max(MIN_H, Math.min(MAX_H, Math.round(cm)));
    d.moveTo(clamped);
  };

  return (
    <div className="flex h-full flex-col overflow-hidden rounded-[14px] border border-pop-border bg-pop-bg text-[13px] text-text-main shadow-[0_1px_0_rgba(255,255,255,0.3)_inset,0_4px_10px_rgba(0,0,0,0.22),0_1px_3px_rgba(0,0,0,0.12)] backdrop-blur-[40px] backdrop-saturate-[180%]">
      {view === 'settings' ? (
        <SettingsView
          onBack={() => setView('main')}
          themeMode={themeMode}
          setThemeMode={setThemeMode}
          accent={accent}
          setAccent={setAccent}
          unit={unit}
          setUnit={setUnit}
        />
      ) : (
        <>
          <div className="flex min-h-0 flex-1 flex-col overflow-y-auto">
            <HeightHeader
              heightCm={liveHeight}
              moving={moving}
              activePreset={activePresetLabel}
              unit={unit}
            />

            {d.error && (
              <div className="mx-4 mb-1 rounded-md border border-red-500/30 bg-red-500/10 px-3 py-2 text-[11px] text-red-400">
                <b className="mr-1 font-mono">{d.error.code}</b>
                {d.error.message}
              </div>
            )}

            <div className="flex gap-3.5 px-4 pb-3.5">
              <SliderColumn
                heightCm={liveHeight}
                disabled={!connected}
                moving={moving}
                onManualMove={() => {
                  setActivePresetId(null);
                  setActivePresetLabel(null);
                }}
              />
              <PresetList
                activePresetId={activePresetId}
                disabled={!connected}
                liveHeight={liveHeight}
                unit={unit}
                onPresetGo={(p) => {
                  setActivePresetId(p.id);
                  setActivePresetLabel(p.name);
                  moveTo(p.height);
                }}
                onPresetCleared={() => {
                  setActivePresetId(null);
                  setActivePresetLabel(null);
                }}
              />
            </div>

            <div className="mx-3.5 h-px bg-divider" />

            <Reminder />
          </div>

          <Footer
            connection={d.connection}
            onDisconnect={d.disconnect}
            onSettings={() => setView('settings')}
          />
        </>
      )}
    </div>
  );
}
