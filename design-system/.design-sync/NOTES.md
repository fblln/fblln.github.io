# design-sync notes — @fblln/design-system

First sync: 2026-07-18. Shape: storybook. 15 components, all graded `match`.
Small DS — solo phase + one roster compare covered it; no fan-out needed.

## General learnings

- **[GENERAL] CSS entry.** The compiled stylesheet is `dist/index.css` (tsup bundles
  every component's CSS + tokens + global into it). `cfg.cssEntry = "dist/index.css"`
  is required — without it the converter finds no static CSS and designs render
  unstyled.
- **[GENERAL] Fonts are system stacks by design.** sans = Arial/Helvetica, mono =
  Courier New, reading = Charter → Iowan → Georgia → serif. This mirrors the site's
  no-webfont philosophy. `[FONT_MISSING]` warns for Charter/Iowan Old Style/Cambria —
  **expected and accepted**: the stack falls through to Georgia (near-universal), and
  there is no webfont to ship. Do not chase it.
- **[GENERAL] Preview surface param isn't threaded.** The `.storybook/preview`
  decorator switches background on a custom `surface: "ink"` story param; the
  design-sync preview cards do NOT receive that param, so they render on paper.
  Consequence: any dark-surface component must **self-style its own background**.
  DiagnosticsPanel, SiteFooter, and Metric all do.
- **ImpactStat / ProjectRow** are wider than a grid cell → `cfg.overrides.*.cardMode
  = "column"` (full-width card per story). Presentation-only; grades carry.
- **Metric fix.** Metric originally used invert (light) colors for its label + rules
  with no background — invisible on paper (the preview surface). Fixed by giving it
  its own ink surface (`background: var(--ink)` + padding), consistent with
  DiagnosticsPanel/SiteFooter. See Re-sync risks.

## Re-sync risks (watch-list for the next run)

- **Metric diverges from the site deliberately.** The site's `.case-metric` is flush
  inside an already-ink case panel (no own background). The DS `Metric` carries its
  own ink surface so it's visible/portable anywhere. If someone rebuilds the site's
  case panel from the DS, Metric brings its own dark box — visually near-identical on
  ink, but not a byte-match to the site element.
- **`[FONT_MISSING]` is permanent and accepted** (system-font stacks). A future run
  will see it every time — it is not a regression.
- **cardMode "column"** on ImpactStat/ProjectRow is presentation-only; if either gains
  a portal/overlay story later, revisit (`escape` would need `single`).
- All 15 graded from images on a first sync; every component has a single primary
  story except Button (4), StatusBadge/Tag/ImpactStat (2), ProjectRow (3) — all graded
  exhaustively, no sibling-trust shortcuts taken.
