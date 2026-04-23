import type { ButtonHTMLAttributes, ReactNode } from 'react';
import { cn } from '@/lib/cn';

export function NudgeButton({
  children,
  className,
  ...props
}: { children: ReactNode } & ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      type="button"
      {...props}
      className={cn(
        'flex h-[22px] w-[22px] items-center justify-center rounded-[5px] p-0',
        'bg-nudge-bg border border-nudge-border text-text-main',
        'disabled:opacity-40 disabled:cursor-not-allowed',
        className,
      )}
    >
      {children}
    </button>
  );
}

export function StepperButton({
  children,
  className,
  ...props
}: { children: ReactNode } & ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      type="button"
      {...props}
      className={cn(
        'flex w-8 items-center justify-center rounded-md p-0',
        'bg-nudge-bg border border-chip-border text-text-main cursor-pointer',
        className,
      )}
    >
      {children}
    </button>
  );
}
