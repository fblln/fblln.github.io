import type { ReactNode } from "react";
import "./ImpactStat.css";

export interface ImpactStatProps {
  /** The headline figure, e.g. "10M+". */
  value: ReactNode;
  /** Uppercase caption under the figure. */
  label: ReactNode;
}

/** A single oversized metric cell — the impact grid on the landing page is a row of these. */
export function ImpactStat({ value, label }: ImpactStatProps) {
  return (
    <div className="ds-impact">
      <strong>{value}</strong>
      <span>{label}</span>
    </div>
  );
}
