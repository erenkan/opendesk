import { clampCm, MIN_H } from './constants';

export type Preset = {
  id: string;
  name: string;
  height: number;
};

export const PRESET_KEY = 'opendesk:presets';
export const MAX_PRESETS = 6;

export function loadPresets(): Preset[] {
  try {
    const raw = localStorage.getItem(PRESET_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as Preset[];
    if (!Array.isArray(parsed)) return [];
    return parsed
      .filter((p) => p && typeof p.id === 'string')
      .slice(0, MAX_PRESETS)
      .map((p, i) => ({
        id: p.id,
        name: typeof p.name === 'string' && p.name.trim() ? p.name.slice(0, 24) : `Preset ${i + 1}`,
        height: clampCm(Math.round(typeof p.height === 'number' ? p.height : MIN_H)),
      }));
  } catch {
    return [];
  }
}

export function savePresets(presets: Preset[]): void {
  try {
    localStorage.setItem(PRESET_KEY, JSON.stringify(presets));
  } catch {
    /* private mode */
  }
}

export function makePreset(index: number, heightCm: number): Preset {
  return {
    id: `p${Date.now()}-${index}`,
    name: `Preset ${index}`,
    height: clampCm(Math.round(heightCm)),
  };
}
