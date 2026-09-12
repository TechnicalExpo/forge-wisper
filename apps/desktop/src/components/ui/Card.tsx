import React from "react";

interface CardProps {
  children: React.ReactNode;
  /** Padding level. Default `p-4`. */
  padding?: "none" | "sm" | "md" | "lg";
  /** Extra tailwind classes (e.g. "space-y-3", "flex items-center justify-between"). */
  className?: string;
  /** Add the subtle top divider between sections inside the card. */
  divide?: boolean;
}

const PADS: Record<NonNullable<CardProps["padding"]>, string> = {
  none: "",
  sm: "p-3",
  md: "p-4",
  lg: "p-5",
};

export const Card: React.FC<CardProps> = ({
  children,
  padding = "md",
  className = "",
  divide = false,
}) => (
  <div
    className={`forge-card rounded-[8px] border border-[var(--border)] bg-[var(--surface-primary)] ${PADS[padding]} ${
      divide ? "divide-y divide-[var(--border-subtle)]" : ""
    } ${className}`}
  >
    {children}
  </div>
);
