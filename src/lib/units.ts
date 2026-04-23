export type UnitSystem = 'cm' | 'in';

export const UNIT_KEY = 'opendesk:unit';
const CM_PER_INCH = 2.54;

function detectDefaultUnit(): UnitSystem {
  try {
    const locale = navigator.language || 'en-US';
    const region = new Intl.Locale(locale).maximize().region;
    return region === 'US' || region === 'LR' || region === 'MM' ? 'in' : 'cm';
  } catch {
    return 'cm';
  }
}

export const DEFAULT_UNIT: UnitSystem = detectDefaultUnit();

export function loadUnit(): UnitSystem {
  try {
    const raw = localStorage.getItem(UNIT_KEY);
    if (raw === 'cm' || raw === 'in') return raw;
  } catch {
    /* private mode */
  }
  return DEFAULT_UNIT;
}

export function saveUnit(u: UnitSystem): void {
  try {
    localStorage.setItem(UNIT_KEY, u);
  } catch {
    /* private mode */
  }
}

export function cmToInches(cm: number): number {
  return cm / CM_PER_INCH;
}

export function inchesToCm(inches: number): number {
  return inches * CM_PER_INCH;
}

export function formatHeight(cm: number, unit: UnitSystem): string {
  if (unit === 'cm') return `${Math.round(cm)} cm`;
  return `${(cm / CM_PER_INCH).toFixed(1)}"`;
}

export function formatUnitSuffix(unit: UnitSystem): string {
  return unit === 'cm' ? 'cm' : '"';
}

export function displayNumber(cm: number, unit: UnitSystem): number {
  return unit === 'cm' ? Math.round(cm) : Number((cm / CM_PER_INCH).toFixed(1));
}

export function stepCm(unit: UnitSystem): number {
  return unit === 'in' ? CM_PER_INCH : 1;
}
