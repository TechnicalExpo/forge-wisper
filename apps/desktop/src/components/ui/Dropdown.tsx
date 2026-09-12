import React, { useState, useRef, useEffect } from "react";
import { ChevronDown } from "lucide-react";

export interface DropdownOption {
  value: string;
  label: string;
}

interface DropdownProps {
  options: DropdownOption[];
  value: string;
  onChange: (value: string) => void;
  label?: string;
  compact?: boolean;
  /** When false the trigger hugs its content (for inline layouts). Default true (block). */
  fullWidth?: boolean;
  className?: string;
  placeholder?: string;
}

export const Dropdown: React.FC<DropdownProps> = ({
  options,
  value,
  onChange,
  label,
  compact = false,
  fullWidth = true,
  className = "",
  placeholder,
}) => {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  // Close on outside click
  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open]);

  const selectedLabel =
    options.find((o) => o.value === value)?.label ??
    placeholder ??
    options[0]?.label ??
    "";

  // Size variants
  const baseSize = compact
    ? "px-2 py-0.5 text-[12px] rounded-[6px]"
    : "px-3 py-2 text-[13px] rounded-[7px]";

  const panelSize = compact ? "text-[12px]" : "text-[13px]";

  return (
    <div className="relative" ref={ref}>
      {label && (
        <label className="text-[11px] text-[var(--text-muted)] block mb-1 font-mono">
          {label}
        </label>
      )}
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className={`${fullWidth ? "w-full" : ""} flex items-center justify-between ${baseSize} bg-[var(--surface-primary)] border border-[var(--border)] text-[var(--text-primary)] font-mono cursor-pointer hover:border-[var(--accent)] transition-colors ${className}`}
      >
        <span className={fullWidth ? "truncate" : ""}>{selectedLabel}</span>
        <ChevronDown
          className={`shrink-0 ml-2 w-4 h-4 text-[var(--text-muted)] transition-transform ${open ? "rotate-180" : ""}`}
        />
      </button>
      {open && (
        <div
          className={`absolute z-50 top-full left-0 right-0 ${compact ? "mt-0.5" : "mt-1"} bg-[var(--surface-elevated)] border border-[var(--border)] ${compact ? "rounded-[6px]" : "rounded-[7px]"} shadow-lg overflow-hidden`}
        >
          {options.map((opt) => (
            <button
              key={opt.value}
              type="button"
              onClick={() => {
                onChange(opt.value);
                setOpen(false);
              }}
              className={`w-full text-left px-3 ${compact ? "py-1" : "py-2"} ${panelSize} font-mono cursor-pointer transition-colors ${
                value === opt.value
                  ? "bg-[var(--accent)]/10 text-[var(--accent)]"
                  : "text-[var(--text-primary)] hover:bg-[var(--surface-primary)]"
              }`}
            >
              {opt.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
};
