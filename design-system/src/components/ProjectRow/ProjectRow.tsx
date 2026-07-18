import type { ReactNode } from "react";
import { Tag } from "../Tag/Tag";
import "./ProjectRow.css";

export interface ProjectRowProps {
  /** Row index; a number is zero-padded to two digits. */
  number: number | string;
  name: ReactNode;
  /** Monospace stack line, e.g. "RUST · WASM · GDAL". */
  stack?: ReactNode;
  /** Optional expanded summary paragraph. */
  summary?: ReactNode;
  /** Optional tag chips shown beside the summary. */
  tags?: string[];
  /** Inverts the row to the ink surface (selected state). */
  active?: boolean;
  onClick?: () => void;
}

/** A row in the work list: index, name, stack, and a disclosure arrow, with an
 *  optional summary + tags underneath. Inverts to ink when `active`. */
export function ProjectRow({
  number,
  name,
  stack,
  summary,
  tags,
  active = false,
  onClick,
}: ProjectRowProps) {
  const num = typeof number === "number" ? String(number).padStart(2, "0") : number;
  return (
    <article className={active ? "ds-project ds-project--active" : "ds-project"}>
      <button className="ds-project__open" type="button" onClick={onClick}>
        <span className="ds-project__number">{num}</span>
        <span className="ds-project__name">{name}</span>
        <span className="ds-project__stack">{stack}</span>
        <span className="ds-project__arrow" aria-hidden="true">
          ↗
        </span>
      </button>
      {(summary || (tags && tags.length > 0)) && (
        <div className="ds-project__summary">
          {summary && <p>{summary}</p>}
          {tags && tags.length > 0 && (
            <div className="ds-project__tags">
              {tags.map((t) => (
                <Tag key={t}>{t}</Tag>
              ))}
            </div>
          )}
        </div>
      )}
    </article>
  );
}
