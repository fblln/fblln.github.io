## Building with this design system

This is the design language of **fblln.github.io** — a brutalist engineering
portfolio: paper background, near-black ink, one hot orange accent, oversized
tight display type, monospace labels, hairline borders.

### Setup — no provider, just the stylesheet

There is **no** theme provider or root wrapper. Load the stylesheet once at the
app root and render components directly:

```jsx
import "@fblln/design-system/styles.css";
```

Without that import, components render unstyled. Nothing else is required — every
component is self-contained.

### Styling idiom — tokens, not classes

Style your own layout glue with the **CSS custom properties** the system defines
(never hardcode hexes, never invent utility classes — there are none). Real names:

- Color: `--ink` (text / dark surfaces), `--paper` (page bg), `--signal` (orange
  accent), `--muted` (secondary text on paper), `--muted-invert` (secondary text on
  ink), `--line` (hairline on paper), `--line-invert` (hairline on ink), `--code-bg`,
  `--ok` (healthy green).
- Type: `--font-sans` (Archivo — UI + display), `--font-mono` (IBM Plex Mono —
  labels, meta, values), `--font-reading` (Charter/Georgia serif — long-form body).
- Spacing: `--pad` (the standard responsive section padding).

Example glue: `<section style={{ background: "var(--paper)", padding: "var(--pad)" }}>`.

Compose by arranging components — do not override their internals.

### Two surfaces

Most components sit on **paper** (dark text on light). Three are **dark, self-styled**
— they carry their own `--ink` background and render correctly on any surface:
`DiagnosticsPanel`, `Metric`, `SiteFooter`. Put them anywhere; don't wrap them in a
dark box yourself.

### Where the truth lives

Read `styles.css` (and the `_ds_bundle.css` it imports) for the exact token values,
and each component's `.prompt.md` + `.d.ts` for its props before composing.

### One idiomatic snippet

```jsx
import { NavBar, StatusBadge, SectionHeading, ImpactStat, Button } from "@fblln/design-system";

<>
  <NavBar
    wordmark="FE/26"
    links={[{ label: "Work", href: "#work" }, { label: "Writing", href: "/articles/" }]}
    action={<StatusBadge active>WASM/ACTIVE</StatusBadge>}
  />
  <section style={{ padding: "var(--pad)" }}>
    <SectionHeading eyebrow="SELECTED IMPACT" title="THE WORK"
      description="Systems built around real data and explicit constraints." />
    <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", marginTop: "var(--pad)" }}>
      <ImpactStat value="10M+" label="connected vehicles" />
      <ImpactStat value="100+" label="APIs shaped" />
      <ImpactStat value="0→20+" label="org growth" />
      <ImpactStat value="100K+" label="IoT devices" />
    </div>
    <Button variant="solid" href="#contact">Start a conversation ↗</Button>
  </section>
</>
```
