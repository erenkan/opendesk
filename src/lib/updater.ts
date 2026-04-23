import { relaunch } from '@tauri-apps/plugin-process';
import { check, type Update } from '@tauri-apps/plugin-updater';

export type UpdateCheck =
  | { kind: 'none' }
  | { kind: 'available'; version: string; notes: string | undefined }
  | { kind: 'error'; message: string };

export async function checkForUpdate(): Promise<{ update: Update | null; result: UpdateCheck }> {
  try {
    const update = await check();
    if (!update) return { update: null, result: { kind: 'none' } };
    return {
      update,
      result: { kind: 'available', version: update.version, notes: update.body },
    };
  } catch (e) {
    return { update: null, result: { kind: 'error', message: String(e) } };
  }
}

export async function installAndRelaunch(update: Update): Promise<void> {
  await update.downloadAndInstall();
  await relaunch();
}
