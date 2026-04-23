export const MIN_H = 68;
export const MAX_H = 127;

export const clampCm = (cm: number) => Math.max(MIN_H, Math.min(MAX_H, cm));

export type ThemeMode = 'light' | 'dark' | 'system';
