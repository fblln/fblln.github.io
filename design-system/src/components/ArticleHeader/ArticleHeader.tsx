import type { ReactNode } from "react";
import "./ArticleHeader.css";

export interface ArticleHeaderProps {
  /** Monospace kicker, e.g. "2026-07-17 · 4 min read". */
  eyebrow?: string;
  title: ReactNode;
  /** Topic tags shown under the title. */
  tags?: string[];
}

/** The masthead for a reading page: mono eyebrow, sans display title, tag row. */
export function ArticleHeader({ eyebrow, title, tags }: ArticleHeaderProps) {
  return (
    <header className="ds-article-header">
      {eyebrow && <p className="ds-article-header__eyebrow">{eyebrow}</p>}
      <h1 className="ds-article-header__title">{title}</h1>
      {tags && tags.length > 0 && (
        <div className="ds-article-header__meta">
          {tags.map((t) => (
            <span key={t} className="ds-article-header__tag">
              {t}
            </span>
          ))}
        </div>
      )}
    </header>
  );
}
