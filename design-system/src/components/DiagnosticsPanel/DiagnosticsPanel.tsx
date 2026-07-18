import type { ReactNode } from "react";
import "./DiagnosticsPanel.css";

export interface DiagnosticItem {
  label: ReactNode;
  value: ReactNode;
}

export interface DiagnosticsPanelProps {
  /** Label/value pairs, laid out in a two-column grid on an ink surface. */
  items: DiagnosticItem[];
}

/** The runtime diagnostics grid: monospace labels with signal-colored values on ink. */
export function DiagnosticsPanel({ items }: DiagnosticsPanelProps) {
  return (
    <div className="ds-diagnostics">
      {items.map((it, i) => (
        <div key={i}>
          <span>{it.label}</span>
          <strong>{it.value}</strong>
        </div>
      ))}
    </div>
  );
}
