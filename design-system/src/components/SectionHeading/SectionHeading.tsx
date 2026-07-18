import type { ReactNode } from "react";
import "./SectionHeading.css";

export interface SectionHeadingProps {
  /** Small monospace kicker above the title, e.g. "SELECTED SYSTEMS". */
  eyebrow?: string;
  title: ReactNode;
  /** Optional supporting paragraph, right-aligned on wide screens. */
  description?: ReactNode;
}

/** Oversized display heading with a monospace eyebrow and optional lede. */
export function SectionHeading({ eyebrow, title, description }: SectionHeadingProps) {
  return (
    <div className="ds-section-heading">
      <div>
        {eyebrow && <p className="ds-eyebrow">{eyebrow}</p>}
        <h2 className="ds-section-heading__title">{title}</h2>
      </div>
      {description && <p className="ds-section-heading__desc">{description}</p>}
    </div>
  );
}
