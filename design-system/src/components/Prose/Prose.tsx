import type { ReactNode } from "react";
import "./Prose.css";

export interface ProseProps {
  /** Rendered JSX content. Ignored if `html` is provided. */
  children?: ReactNode;
  /** Pre-rendered HTML string (e.g. from a Markdown renderer). */
  html?: string;
}

/** Long-form reading container. Applies the article typography scale to any
 *  nested headings, paragraphs, lists, code, quotes, tables, and images. */
export function Prose({ children, html }: ProseProps) {
  if (html !== undefined) {
    return <div className="ds-prose" dangerouslySetInnerHTML={{ __html: html }} />;
  }
  return <div className="ds-prose">{children}</div>;
}
