import React from "react";

interface BadgeProps {
  children: React.ReactNode;
  icon?: React.ReactNode;
  /**
   * Visual variant.
   * `accent` → accent text on raised surface (default, "OS Keyring Secured")
   * `muted`  → muted text on raised surface
   */
  variant?: "accent" | "muted";
  className?: string;
}

export const Badge: React.FC<BadgeProps> = ({
  children,
  icon,
  variant = "accent",
  className = "",
}) => {
  const text =
    variant === "accent" ? "text-[var(--accent)]" : "text-[var(--text-muted)]";
  return (
    <span
      className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-[4px] text-[11px] font-mono bg-[var(--surface-elevated)] ${text} border border-[var(--border)] ${className}`}
    >
      {icon}
      {children}
    </span>
  );
};
