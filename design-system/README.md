# @fblln/design-system

A React + TypeScript component library that mirrors the visual language of
[fblln.github.io](https://fblln.github.io) — Fabio Ellena's portfolio. Same
tokens, same typography, same brutalist grid. Built so designs made with these
components map 1:1 onto the real site.

## Use

```bash
npm install          # from design-system/
npm run storybook    # browse components at http://localhost:6006
npm run build        # emit dist/ (JS + d.ts + styles.css)
```

Consume:

```tsx
import { NavBar, StatusBadge, SectionHeading, Button } from "@fblln/design-system";
import "@fblln/design-system/styles.css"; // load once, at the app root

<NavBar wordmark="FE/26" links={[{ label: "Work", href: "#work" }]}
  action={<StatusBadge active>WASM/ACTIVE</StatusBadge>} />
```

## Styling idiom

- **Tokens, not hardcoded values.** Colors, type, and spacing are CSS variables
  on `:root` — `var(--ink)`, `var(--paper)`, `var(--signal)`, `var(--muted)`,
  `var(--line)`, `var(--code-bg)`; fonts `var(--font-sans)`, `var(--font-mono)`,
  `var(--font-reading)`; spacing `var(--pad)`. Use these for any layout glue you
  write around the components. See [`src/styles/tokens.css`](src/styles/tokens.css).
- **Compose components; don't restyle them.** Each component owns its look via a
  `ds-`-prefixed class. Build layouts by arranging components, not overriding
  their internals.
- **Two surfaces.** Most components sit on paper (`--paper`/`--ink`); a few are
  built for the ink surface (`DiagnosticsPanel`, `Metric`, `SiteFooter`).

## Components

| Group | Components |
| --- | --- |
| Core UI | `Button`, `Tag`, `StatusBadge`, `NavBar`, `SiteFooter` |
| Site patterns | `SectionHeading`, `ImpactStat`, `ProjectRow`, `DiagnosticsPanel`, `Metric` |
| Article / reading | `ArticleHeader`, `TableOfContents`, `CodeBlock`, `Callout`, `Prose` |

Foundations (color + type tokens) are browsable under **Foundations/Tokens** in
Storybook.
