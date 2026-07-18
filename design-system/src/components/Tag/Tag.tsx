import type { ReactNode } from "react";
import "./Tag.css";

export interface TagProps {
  children: ReactNode;
}

/** Small bordered monospace chip — used for stacks, topics, and article tags.
 *  Borders inherit the current text color, so it reads on both paper and ink. */
export function Tag({ children }: TagProps) {
  return <span className="ds-tag">{children}</span>;
}
