import type { SVGProps } from 'react';

// Domain-specific glyphs without a clean lucide equivalent: the desk
// silhouette on the header plus the sit / stand / focus posture icons.
// Generic icons (plus/minus/edit/chevron/sun/moon/check/gear/contrast/
// trash) come from `lucide-react` directly at call sites — do not
// re-add them here.

export const DeskIcon = ({
  size = 18,
  className,
  ...rest
}: { size?: number } & SVGProps<SVGSVGElement>) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" className={className} {...rest}>
    <rect x="3" y="9" width="18" height="2.2" rx="0.6" fill="currentColor" />
    <rect x="7" y="11.2" width="2" height="8" rx="0.4" fill="currentColor" opacity="0.85" />
    <rect x="15" y="11.2" width="2" height="8" rx="0.4" fill="currentColor" opacity="0.85" />
    <path
      d="M12 7.5 L12 3.5 M10 5.2 L12 3.2 L14 5.2"
      stroke="currentColor"
      strokeWidth="1.4"
      strokeLinecap="round"
      strokeLinejoin="round"
    />
  </svg>
);

export const StandIcon = ({ size = 16 }: { size?: number }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 20 20"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.5"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <circle cx="10" cy="3.2" r="1.4" />
    <path d="M10 5v6M7 8l3-2 3 2M7 17l3-6 3 6" />
  </svg>
);

export const SitIcon = ({ size = 16 }: { size?: number }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 20 20"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.5"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <circle cx="10" cy="3.2" r="1.4" />
    <path d="M10 5v4l-3 3M10 9l3 3M7 12v5M13 12v5M5 17h10" />
  </svg>
);

export const FocusIcon = ({ size = 16 }: { size?: number }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 20 20"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.5"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <circle cx="10" cy="10" r="6" />
    <circle cx="10" cy="10" r="2.2" />
  </svg>
);
