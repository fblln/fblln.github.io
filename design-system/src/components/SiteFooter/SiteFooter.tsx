import type { ReactNode } from "react";
import "./SiteFooter.css";

export interface SiteFooterProps {
  /** Footer cells — plain text spans or links, spaced apart across the bar. */
  children: ReactNode;
}

/** Dark monospace footer bar. Its cells are spread with space-between. */
export function SiteFooter({ children }: SiteFooterProps) {
  return <footer className="ds-footer">{children}</footer>;
}
