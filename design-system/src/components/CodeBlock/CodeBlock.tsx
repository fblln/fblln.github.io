import "./CodeBlock.css";

export interface CodeBlockProps {
  /** Raw source to display. Whitespace is preserved. */
  code: string;
  /** Optional language label, surfaced via the `data-lang` attribute. */
  lang?: string;
}

/** Dark, horizontally-scrolling code block. Highlighting is applied at build
 *  time on the site (syntect); here it renders plain, styled source. */
export function CodeBlock({ code, lang }: CodeBlockProps) {
  return (
    <pre className="ds-code" data-lang={lang}>
      <code>{code}</code>
    </pre>
  );
}
