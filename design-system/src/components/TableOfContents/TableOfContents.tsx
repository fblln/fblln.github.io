import type { ReactNode } from "react";
import "./TableOfContents.css";

export interface TocEntry {
  /** Heading depth — 2 (top level) or 3 (indented). */
  level: 2 | 3;
  text: ReactNode;
  href: string;
}

export interface TableOfContentsProps {
  entries: TocEntry[];
  title?: string;
}

/** Bordered contents box for long reads. h3 entries indent under their h2. */
export function TableOfContents({ entries, title = "Contents" }: TableOfContentsProps) {
  return (
    <nav className="ds-toc" aria-label={title}>
      <p className="ds-toc__title">{title}</p>
      <ul>
        {entries.map((e, i) => (
          <li key={i} className={e.level === 3 ? "ds-toc__l3" : undefined}>
            <a href={e.href}>{e.text}</a>
          </li>
        ))}
      </ul>
    </nav>
  );
}
