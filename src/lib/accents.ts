export type AccentId =
  | 'coral'
  | 'ocean'
  | 'forest'
  | 'grape'
  | 'sunset'
  | 'graphite';

export type Accent = {
  id: AccentId;
  label: string;
  base: string;
  ink: string;
  soft: string;
};

export const ACCENTS: Accent[] = [
  {
    id: 'coral',
    label: 'Coral',
    base: 'oklch(0.70 0.16 32)',
    ink: 'oklch(0.45 0.14 32)',
    soft: 'oklch(0.93 0.05 32)',
  },
  {
    id: 'ocean',
    label: 'Ocean',
    base: 'oklch(0.65 0.14 220)',
    ink: 'oklch(0.40 0.12 220)',
    soft: 'oklch(0.93 0.05 220)',
  },
  {
    id: 'forest',
    label: 'Forest',
    base: 'oklch(0.60 0.14 145)',
    ink: 'oklch(0.38 0.12 145)',
    soft: 'oklch(0.93 0.05 145)',
  },
  {
    id: 'grape',
    label: 'Grape',
    base: 'oklch(0.60 0.18 300)',
    ink: 'oklch(0.38 0.14 300)',
    soft: 'oklch(0.93 0.06 300)',
  },
  {
    id: 'sunset',
    label: 'Sunset',
    base: 'oklch(0.75 0.16 60)',
    ink: 'oklch(0.48 0.13 60)',
    soft: 'oklch(0.94 0.06 60)',
  },
  {
    id: 'graphite',
    label: 'Graphite',
    base: 'oklch(0.50 0.02 260)',
    ink: 'oklch(0.30 0.02 260)',
    soft: 'oklch(0.92 0.01 260)',
  },
];

export const DEFAULT_ACCENT: AccentId = 'coral';
export const ACCENT_KEY = 'opendesk:accent';

export function loadAccent(): AccentId {
  try {
    const raw = localStorage.getItem(ACCENT_KEY);
    if (raw && ACCENTS.some((a) => a.id === raw)) {
      return raw as AccentId;
    }
  } catch {
    /* private mode */
  }
  return DEFAULT_ACCENT;
}

export function saveAccent(id: AccentId): void {
  try {
    localStorage.setItem(ACCENT_KEY, id);
  } catch {
    /* private mode */
  }
}

export function applyAccent(id: AccentId): void {
  const accent = ACCENTS.find((a) => a.id === id) ?? ACCENTS[0];
  const root = document.documentElement;
  root.style.setProperty('--accent-base', accent.base);
  root.style.setProperty('--accent-ink', accent.ink);
  root.style.setProperty('--accent-soft', accent.soft);
}
