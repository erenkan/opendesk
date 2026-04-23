#!/usr/bin/env node
/**
 * Render the two master SVGs in src-tauri/icons/ to PNGs the Tauri bundler
 * and the tray code consume.
 *
 *   icon.svg       → icon-1024.png (for `pnpm tauri icon` to fan out)
 *   tray-icon.svg  → tray-icon.png (22×22 + @2x 44×44)
 *
 * Uses @resvg/resvg-js so we don't need librsvg installed system-wide.
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { Resvg } from '@resvg/resvg-js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const iconsDir = path.resolve(__dirname, '..', 'src-tauri', 'icons');

function render(svgPath, outPath, size) {
  const svg = fs.readFileSync(svgPath, 'utf8');
  const resvg = new Resvg(svg, {
    fitTo: { mode: 'width', value: size },
    background: 'rgba(0,0,0,0)',
  });
  const data = resvg.render().asPng();
  fs.writeFileSync(outPath, data);
  console.log(`rendered ${path.basename(outPath)} @ ${size}px`);
}

// Master app icon — 1024 is what `tauri icon` wants as source.
render(
  path.join(iconsDir, 'icon.svg'),
  path.join(iconsDir, 'icon-1024.png'),
  1024,
);

// Tray icon — template-friendly 22×22 plus retina 44×44.
render(
  path.join(iconsDir, 'tray-icon.svg'),
  path.join(iconsDir, 'tray-icon.png'),
  22,
);
render(
  path.join(iconsDir, 'tray-icon.svg'),
  path.join(iconsDir, 'tray-icon@2x.png'),
  44,
);
