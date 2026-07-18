import type { ReactNode } from "react";
import "./Callout.css";

export interface CalloutProps {
  children: ReactNode;
}

/** Pull-quote / aside with a signal-colored left rule, in the reading serif. */
export function Callout({ children }: CalloutProps) {
  return <blockquote className="ds-callout">{children}</blockquote>;
}
