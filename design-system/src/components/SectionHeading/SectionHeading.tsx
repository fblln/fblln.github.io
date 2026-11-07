import type { ReactNode } from "react";
import "./SectionHeading.css";

export interface SectionHeadingProps {
  /** Small monospace kicker above the title, e.g. "SELECTED SYSTEMS". */
  eyebrow?: string;
  title: ReactNode;
  /** Optional supporting paragraph, right-aligned on wide screens. */
  description?: ReactNode;
  /** The page's lead section: one size step larger than the others. */
  lead?: boolean;
}

/** Oversized display heading with a monospace eyebrow and optional lede. */
export function SectionHeading({ eyebrow, title, description, lead }: SectionHeadingProps) {
  return (
    <div className={lead ? "ds-section-heading ds-section-heading--lead" : "ds-section-heading"}>
      <div>
        {eyebrow && <p className="ds-eyebrow">{eyebrow}</p>}
        <h2 className="ds-section-heading__title">{title}</h2>
      </div>
      {description && <p className="ds-section-heading__desc">{description}</p>}
    </div>
  );
}
