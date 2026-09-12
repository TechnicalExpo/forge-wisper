import React from "react";

interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  title?: string;
  /** Visual size. `md` is the default (w-12 h-6). `sm` is smaller (w-10 h-5). */
  size?: "sm" | "md";
  className?: string;
}

export const Toggle: React.FC<ToggleProps> = ({
  checked,
  onChange,
  title,
  size = "md",
  className = "",
}) => {
  const track =
    size === "sm"
      ? "w-10 h-5"
      : "w-12 h-6";
  const thumb =
    size === "sm"
      ? "w-4 h-4"
      : "w-5 h-5";
  const onShift =
    size === "sm"
      ? "translate-x-5"
      : "translate-x-6";

  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      title={title}
      className={`relative shrink-0 ${track} rounded-full transition-colors cursor-pointer ${
        checked ? "bg-[var(--accent)]" : "bg-[var(--surface-elevated)]"
      } ${className}`}
    >
      <span
        className={`absolute top-1/2 left-0.5 ${thumb} rounded-full bg-white shadow-sm -translate-y-1/2 transition-transform ${
          checked ? onShift : "translate-x-0"
        }`}
      />
    </button>
  );
};
