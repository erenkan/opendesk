import { useState } from 'react';
import { cn } from '@/lib/cn';
import { Pencil, Trash2 } from 'lucide-react';

type Props = {
  label: string;
  value: string;
  numberLabel: string;
  active: boolean;
  disabled?: boolean;
  onClick: () => void;
  onEdit?: () => void;
  onDelete?: () => void;
  editing?: boolean;
};

export function PresetItem({
  label,
  value,
  numberLabel,
  active,
  disabled,
  onClick,
  onEdit,
  onDelete,
  editing,
}: Props) {
  const [hover, setHover] = useState(false);

  return (
    <div
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      className={cn(
        'flex items-stretch overflow-hidden rounded-lg border text-[13px] font-medium transition-colors',
        active
          ? 'border-accent-ink text-white [background:linear-gradient(180deg,var(--accent-base),var(--accent-ink))]'
          : 'border-chip-border text-text-main bg-chip-bg',
        !active && hover && 'bg-[rgba(20,18,16,0.06)] dark:bg-white/[0.09]',
        active && 'shadow-[0_1px_0_rgba(255,255,255,0.3)_inset,0_2px_6px_var(--accent-ink)44]',
        disabled && 'opacity-50',
      )}
    >
      <button
        type="button"
        disabled={disabled}
        onClick={onClick}
        className={cn(
          'flex flex-1 items-center gap-[9px] px-2.5 py-2 text-left text-[13px] font-medium min-w-0',
          'bg-transparent border-none',
          disabled ? 'cursor-not-allowed' : 'cursor-pointer',
        )}
        style={{ color: 'inherit', fontFamily: 'inherit' }}
      >
        <span
          className={cn(
            'flex h-[22px] w-[22px] shrink-0 items-center justify-center rounded-[5px]',
            'text-[11px] font-semibold tabular-nums',
            active
              ? 'bg-white/20 text-white'
              : 'border border-chip-border bg-[rgba(20,18,16,0.045)] dark:bg-white/[0.05] text-text-dim',
          )}
        >
          {numberLabel}
        </span>
        <span className="flex-1 overflow-hidden text-ellipsis whitespace-nowrap">
          {label}
        </span>
        <span
          className={cn(
            'shrink-0 text-[11px] font-medium [font-feature-settings:"tnum"]',
            active ? 'text-white/80' : 'text-text-dim',
          )}
        >
          {value}
        </span>
      </button>

      {onEdit && (
        <button
          type="button"
          aria-label="Edit preset"
          onClick={onEdit}
          className={cn(
            'flex w-[30px] items-center justify-center border-none cursor-pointer',
            active
              ? 'border-l border-white/20 text-white/85'
              : 'border-l border-chip-border text-text-dim',
            editing && (active ? 'bg-white/25' : 'bg-[rgba(20,18,16,0.06)] dark:bg-white/[0.09]'),
            !editing && 'bg-transparent',
          )}
        >
          <Pencil size={12} strokeWidth={1.6} />
        </button>
      )}

      {onDelete && (
        <button
          type="button"
          aria-label="Delete preset"
          onClick={onDelete}
          className={cn(
            'flex w-[30px] items-center justify-center border-none cursor-pointer',
            active
              ? 'border-l border-white/20 text-white/85 hover:bg-white/15'
              : 'border-l border-chip-border text-text-dim hover:bg-red-500/10 hover:text-red-500',
          )}
        >
          <Trash2 size={12} strokeWidth={1.6} />
        </button>
      )}
    </div>
  );
}
