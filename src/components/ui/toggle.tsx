import { cn } from '@/lib/cn';

type ToggleProps = {
  on: boolean;
  onChange: (v: boolean) => void;
  disabled?: boolean;
  className?: string;
};

export function Toggle({ on, onChange, disabled, className }: ToggleProps) {
  return (
    <button
      type="button"
      onClick={() => !disabled && onChange(!on)}
      disabled={disabled}
      className={cn(
        'relative h-[22px] w-9 shrink-0 rounded-full p-0 transition-colors duration-200',
        "shadow-[inset_0_0.5px_1px_rgba(0,0,0,0.12)]",
        on ? 'bg-accent-base' : 'bg-[rgba(120,120,128,0.3)]',
        disabled && 'opacity-50 cursor-not-allowed',
        className,
      )}
      aria-pressed={on}
    >
      <div
        className={cn(
          'absolute top-[2px] h-[18px] w-[18px] rounded-full bg-white transition-[left] duration-200',
          'shadow-[0_2px_4px_rgba(0,0,0,0.25),0_0_0_0.5px_rgba(0,0,0,0.08)]',
          on ? 'left-4' : 'left-[2px]',
        )}
      />
    </button>
  );
}
