import type { ReactNode } from "react";
import "./Metric.css";

export interface MetricProps {
  /** The oversized signal-colored figure, e.g. "42.7×". */
  value: ReactNode;
  /** Monospace caption describing the figure. */
  label: ReactNode;
}

/** Headline metric — a huge accent number with a caption, framed by hairlines.
 *  Designed for the ink case-study surface. */
export function Metric({ value, label }: MetricProps) {
  return (
    <div className="ds-metric">
      <strong>{value}</strong>
      <span>{label}</span>
    </div>
  );
}
