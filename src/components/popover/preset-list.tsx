import { useEffect, useState } from 'react';
import { Plus } from 'lucide-react';
import { PresetItem } from './preset-item';
import { CustomEditor } from './custom-editor';
import {
  loadPresets,
  savePresets,
  makePreset,
  MAX_PRESETS,
  type Preset,
} from '@/lib/presets';
import { formatHeight, type UnitSystem } from '@/lib/units';

type Props = {
  activePresetId: string | null;
  disabled: boolean;
  liveHeight: number;
  unit: UnitSystem;
  onPresetGo: (preset: Preset) => void;
  onPresetCleared: () => void;
};

export function PresetList({
  activePresetId,
  disabled,
  liveHeight,
  unit,
  onPresetGo,
  onPresetCleared,
}: Props) {
  const [presets, setPresets] = useState<Preset[]>([]);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [draftId, setDraftId] = useState<string | null>(null);

  useEffect(() => {
    setPresets(loadPresets());
  }, []);

  const persist = (next: Preset[]) => {
    setPresets(next);
    savePresets(next);
  };

  const updatePreset = (id: string, patch: Partial<Preset>) => {
    persist(presets.map((p) => (p.id === id ? { ...p, ...patch } : p)));
  };

  const addPreset = () => {
    if (presets.length >= MAX_PRESETS) return;
    const next = makePreset(presets.length + 1, Math.round(liveHeight));
    persist([...presets, next]);
    setEditingId(next.id);
    setDraftId(next.id);
  };

  const deletePreset = (id: string) => {
    persist(presets.filter((p) => p.id !== id));
    if (activePresetId === id) onPresetCleared();
    setEditingId(null);
    if (draftId === id) setDraftId(null);
  };

  const cancelEdit = () => {
    if (editingId && draftId === editingId) {
      // Cancelling a freshly-added preset: drop it.
      persist(presets.filter((p) => p.id !== editingId));
    }
    setEditingId(null);
    setDraftId(null);
  };

  const commitEdit = () => {
    setEditingId(null);
    setDraftId(null);
  };

  const editingPreset = presets.find((p) => p.id === editingId) ?? null;

  return (
    <div className="flex flex-1 flex-col gap-1.5">
      <div className="mb-0.5 flex items-center gap-1">
        <span className="text-[10.5px] font-semibold uppercase tracking-[0.08em] text-text-faint">
          Presets
        </span>
        {presets.length > 0 && (
          <span className="text-[10.5px] font-medium tabular-nums text-text-faint">
            · {presets.length}/{MAX_PRESETS}
          </span>
        )}
        <div className="flex-1" />
        {presets.length > 0 && presets.length < MAX_PRESETS && (
          <button
            type="button"
            onClick={addPreset}
            aria-label="Add preset"
            title={`Save current (${formatHeight(liveHeight, unit)}) as preset`}
            className="flex items-center gap-[3px] rounded-[5px] border border-chip-border bg-transparent px-1.5 py-[2px] text-[10.5px] font-medium text-text-dim cursor-pointer hover:bg-chip-bg hover:text-text-main"
          >
            <Plus size={10} strokeWidth={1.8} />
            Add
          </button>
        )}
      </div>

      {presets.length === 0 ? (
        <button
          type="button"
          onClick={addPreset}
          disabled={disabled}
          className="flex flex-col items-center gap-1.5 rounded-[9px] border border-dashed border-text-faint/30 px-3 py-[18px] text-text-dim hover:border-accent-base hover:bg-chip-bg hover:text-text-main disabled:cursor-not-allowed disabled:opacity-50"
          style={{ fontFamily: 'inherit' }}
        >
          <span className="flex h-[26px] w-[26px] items-center justify-center rounded-full border border-current opacity-70">
            <Plus size={13} strokeWidth={1.8} />
          </span>
          <span className="text-[11.5px] font-medium leading-tight">
            Save a preset
          </span>
          <span className="max-w-[160px] text-center text-[10.5px] leading-snug text-text-faint">
            Move the desk where you like it, then add it here.
          </span>
        </button>
      ) : (
        presets.map((p, idx) => (
          <div key={p.id}>
            <PresetItem
              label={p.name}
              value={formatHeight(p.height, unit)}
              numberLabel={String(idx + 1)}
              active={activePresetId === p.id}
              disabled={disabled}
              onClick={() => {
                setEditingId(null);
                onPresetGo(p);
              }}
              onEdit={() =>
                setEditingId((cur) => (cur === p.id ? null : p.id))
              }
              onDelete={() => deletePreset(p.id)}
              editing={editingId === p.id}
            />
            {editingId === p.id && editingPreset && (
              <CustomEditor
                height={editingPreset.height}
                label={editingPreset.name}
                unit={unit}
                onHeight={(h) => updatePreset(p.id, { height: h })}
                onLabel={(l) =>
                  updatePreset(p.id, {
                    name: (l ?? '').trim() || `Preset ${idx + 1}`,
                  })
                }
                currentHeight={Math.round(liveHeight)}
                onSave={commitEdit}
                onCancel={cancelEdit}
              />
            )}
          </div>
        ))
      )}
    </div>
  );
}
