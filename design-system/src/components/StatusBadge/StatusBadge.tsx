import type { ReactNode } from "react";
import "./StatusBadge.css";

export interface StatusBadgeProps {
  /** Label text, e.g. "WASM/ACTIVE". */
  children: ReactNode;
  /** `true` colors the dot with the signal accent; otherwise it reads healthy green. */
  active?: boolean;
}

/** Monospace status pill with a glowing dot — the runtime indicator in the header. */
export function StatusBadge({ children, active = false }: StatusBadgeProps) {
  return (
    <span className={active ? "ds-badge ds-badge--active" : "ds-badge"}>
      <span className="ds-badge__dot" aria-hidden="true" />
      {children}
    </span>
  );
}
