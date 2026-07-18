import type { ReactNode } from "react";
import "./Button.css";

export interface ButtonProps {
  /** Visual weight. `outline` is bordered; `solid` is filled ink. */
  variant?: "outline" | "solid";
  /** Render as a link when set; otherwise a `<button>`. */
  href?: string;
  onClick?: () => void;
  type?: "button" | "submit" | "reset";
  disabled?: boolean;
  className?: string;
  children: ReactNode;
}

/** Uppercase, letter-spaced call-to-action. Fills with the signal color on hover. */
export function Button({
  variant = "outline",
  href,
  onClick,
  type = "button",
  disabled,
  className,
  children,
}: ButtonProps) {
  const cls = ["ds-button", variant === "solid" && "ds-button--solid", className]
    .filter(Boolean)
    .join(" ");
  if (href !== undefined) {
    return (
      <a className={cls} href={href} onClick={onClick}>
        {children}
      </a>
    );
  }
  return (
    <button className={cls} type={type} onClick={onClick} disabled={disabled}>
      {children}
    </button>
  );
}
