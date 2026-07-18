import type { ReactNode } from "react";
import "./NavBar.css";

export interface NavBarLink {
  label: string;
  href: string;
}

export interface NavBarProps {
  /** Short wordmark, e.g. "FE/26". */
  wordmark: string;
  href?: string;
  links?: NavBarLink[];
  /** Right-aligned slot — typically a <StatusBadge>. */
  action?: ReactNode;
}

/** Sticky top bar: wordmark left, centered nav, action slot right.
 *  The nav collapses (hidden) under ~680px, matching the site. */
export function NavBar({ wordmark, href = "#", links = [], action }: NavBarProps) {
  return (
    <header className="ds-topbar">
      <a className="ds-wordmark" href={href}>
        {wordmark}
      </a>
      <nav className="ds-nav" aria-label="Primary">
        {links.map((l) => (
          <a key={l.href + l.label} href={l.href}>
            {l.label}
          </a>
        ))}
      </nav>
      <div className="ds-topbar__action">{action}</div>
    </header>
  );
}
