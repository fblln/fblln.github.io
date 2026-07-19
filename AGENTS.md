# AGENTS.md

Operating guide for AI agents (and humans) working in this repository. Read it
before making changes. It encodes non-negotiable engineering standards, not
suggestions — a change that doesn't meet them isn't done.

## What this repo is

A full **Rust → WebAssembly** engineering portfolio, plus its design system.

| Path | What it is | Stack |
|------|-----------|-------|
| `src/lib.rs` | Shared portfolio view — rendered natively at build time and hydrated in-browser | Rust, Leptos 0.8 (SSG/hydrate), `web-sys` |
| `src/main.rs` | Minimal browser entry point | Rust |
| `tools/site/` | Native Leptos → static portfolio HTML generator (Trunk post-build hook) | Rust |
| `styles.css` | Hand-authored portfolio layout styles | CSS |
| `shared/` | Navigation metadata plus typography/header CSS shared with Writing | Rust, CSS |
| `assets/fonts/` | Latin-subset woff2 (Archivo display, IBM Plex Mono labels) | — |
| `content/articles/` | Markdown articles | — |
| `tools/blog/` | Build-time Markdown → static HTML article generator (Trunk post-build hook) | Rust |
| `design-system/` | `@fblln/design-system` — React components mirroring the site 1:1 | TypeScript, React 18, tsup, Storybook |

The site is the **source of truth** for the visual language. See "Design system
sync" below — it is a hard invariant.

## Build & run

```bash
trunk serve            # dev server at http://127.0.0.1:8080
trunk build            # debug build into dist/
trunk build --release  # production build (opt-level=z, lto, panic=abort, strip)
cargo test             # unit tests (native CSR target)
cargo test --no-default-features --features ssr # static renderer tests
cargo fmt --check      # formatting gate (CI enforces)

cd design-system && npm run build       # emit dist/ (JS + d.ts + styles.css)
cd design-system && npm run storybook   # component workbench at :6006
```

CI (`.github/workflows/ci.yml`) gates every PR on: `cargo fmt --check`,
CSR + SSR + generator tests, a hydrate-mode wasm32 check, a release build, and
a **compressed-WASM budget of ≤ 512000 bytes**. Do not regress the budget; if a
change grows the bundle, justify it or pay it back elsewhere.

## Engineering standards

These are the point of this file. They apply to Rust and TypeScript alike.

### 1. Testing is the deliverable, not an afterthought

- **100% unit-test coverage.** Every unit of logic ships with tests that exercise
  it. Coverage is measured, not assumed — a line with no test is treated as a
  line that does not work. Untested logic does not merge.
  - Rust: `cargo llvm-cov --workspace` (install `cargo-llvm-cov`). Target 100%
    line + branch on all non-entry-point/non-view logic; assert it in CI once wired.
  - Design system: `vitest run --coverage` with the coverage threshold set to
    100 in the Vitest config.
- **Invariants covered by mutation testing.** Line coverage proves code ran, not
  that a test would fail if it broke. Every invariant — the properties that must
  hold (uniqueness, ordering, filter correctness, size budgets, parse/round-trip
  laws) — must be pinned by a test that *fails under mutation*. A surviving
  mutant is a missing assertion; kill it or document why it's equivalent.
  - Rust: `cargo mutants` (install `cargo-mutants`). No survivors on logic
    modules.
  - TypeScript: StrykerJS (`@stryker-mutator/core`) over the design system.
- **Real integration testing through a headless browser.** Because this app only
  exists once compiled to WASM and mounted in a real DOM, unit tests are not
  enough — the boot path, hydration, keyboard model (`J/K/Enter/S//Esc`), project
  filtering, the case-study/system panels, and the runtime-facts readout must be
  driven in an actual browser and asserted on observed behavior.
  - Use **Playwright** (headless Chromium/WebKit/Firefox) against a `trunk serve`
    or a served `dist/`. Prefer role/text queries over CSS selectors; assert what
    the user sees and what the DOM actually does, not implementation details.
  - Cover both surfaces: the site, and the design system (drive Storybook stories
    or a mounted component page). Test the responsive breakpoints (≤680px mobile
    nav, ≤900px) as real viewports, not media-query guesses.
  - Other tools are fine where they fit better (e.g. `wasm-bindgen-test` for
    in-browser Rust unit tests) — the requirement is *real browser execution*,
    not a specific vendor.

When you add a feature, you add: unit tests to 100%, an invariant test the
relevant mutant can't survive, and a browser-level check that the feature works
end to end. A PR without all three is incomplete.

### 2. File decomposition

Small, single-responsibility files with clear seams. `src/lib.rs` currently
holds most of the portfolio view — when you touch a cohesive area, pull it into
their own modules (e.g. `projects` data, `keyboard` handling, `runtime`
diagnostics, each `view` component) rather than growing the monolith. A file
should have one reason to change and be readable top to bottom without scrolling
past unrelated concerns. Same for the design system: one component per directory
(as it already is) — keep it that way.

### 3. Wide block comments — the why and the what

Every module, non-trivial function, and non-obvious block gets a block comment
that explains **what** it does and, more importantly, **why** it exists — the
constraint, the trade-off, the failure mode it guards against. The existing code
already sets the bar (see the comments on `wasm_transfer_size`, the Trunk
`wasm-opt` flags, the boot-timing capture): those explain *why*, not just *what*.
Match that density. A reader should never have to reverse-engineer intent from
mechanics. Comments that only restate the code are noise; comments that capture
the reasoning are the point.

## Design system sync (hard invariant)

`@fblln/design-system` exists so designs built from its components map 1:1 onto
the live site. **Any change to the site's design tokens must be mirrored in the
design system, in the same change.**

- Site layout tokens live in `styles.css`; shared font declarations and stacks
  live in `shared/typography.css`. The DS mirrors them in
  `design-system/src/styles/tokens.css` and self-hosts the identical fonts in
  `design-system/src/styles/fonts.css` (same woff2 as `assets/fonts/`, inlined as
  data URIs so `dist/index.css` stays self-contained).
- Change a color, `--font-sans`/`--font-mono`, `--line`, or spacing on the site →
  update the DS tokens (and regenerate `fonts.css` if the fonts changed) → rebuild
  the DS → verify a component renders identically.
- `dist/` and `ds-bundle/` are gitignored build artifacts; `src/` and durable
  `.design-sync/` state are the committed source of truth.

Treat drift here as a bug of the same severity as a failing test.

## House rules

- Keep the compressed WASM budget green (≤ 512 KB).
- `cargo fmt` before every commit; CI rejects unformatted code.
- Don't add a dependency for what a few lines of stdlib/native platform can do;
  don't add speculative abstraction. Shortest change that meets the standards
  above wins.
- Never leave logic untested to "come back later." Later is now.
