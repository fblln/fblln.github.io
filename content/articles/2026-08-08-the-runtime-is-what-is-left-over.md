+++
title = "The Runtime Is What's Left Over"
date = "2026-08-08"
description = "This site is a Rust/WebAssembly portfolio, but the interesting decision was never the language. It was where to put the line between what a build can decide and what only a browser can. A static host with no server, a hard bundle budget, and a refusal to hand-write JavaScript pushed almost everything to the left of that line — and what remains on the right is a very short list. This is how the thing is actually built."
tags = ["Architecture", "Rust", "WebAssembly", "Build Systems"]
+++

Every personal site eventually becomes an argument about its own stack. I would
rather it be an argument about something else.

The stack here is easy to state and mostly uninteresting: Rust 2024, Leptos for
the view, Trunk for the build, `wasm-bindgen`/`web-sys` for the browser
boundary, GitHub Pages for hosting. You could reproduce the same result in a
dozen other stacks. The part I find worth writing down is not what the pieces
are, it is where the seam between them sits.

The whole design comes out of one question I kept re-asking: *does this have to
be decided in a browser?*

Almost nothing did.

## The system, before any of the details

Four moving parts, and it is worth holding all four in your head before we look
at any of them closely.

**One view library.** `src/lib.rs` holds the portfolio as a Leptos component
tree. It is not a browser file. It is a library, and it gets compiled twice, for
two different targets, with two different feature sets.

**A native renderer.** `tools/site` links that library on the host, asks it for
HTML, and writes the result into the page Trunk produced.

**A Markdown generator.** `tools/blog` is a completely separate binary that
never touches the portfolio view. It reads `content/articles/*.md` and writes
the entire Writing section — article pages, tag indexes, an Atom feed, a
sitemap.

**A React mirror.** `design-system/` reimplements the site's components in
TypeScript so that designs built from it map onto the live site exactly. It is
downstream by definition; the site is the source of truth.

Plus a `shared/` directory that belongs to none of them and is compiled or
concatenated into all of them: the design tokens, the type contract, the
navigation data, and the chrome markup. Nothing in there is a crate or a
package. They are files, included by path.

Everything those parts produce lands in one directory, and that directory is the
whole product. There is no runtime assembly step, because there is nothing at
runtime to assemble it.

<figure class="diagram">
<svg viewBox="0 0 620 186" role="img" aria-label="A system map. On the left, a solid block for the shared Leptos view library and an outlined block for the Markdown sources. Arrows from the view library fan out to two builds: a native ssr build and a WebAssembly hydrate build. An arrow from the Markdown sources feeds the blog generator. All three producers point right into a single wide block representing the dist directory that GitHub Pages serves. A note reads: one view, compiled for two different runtimes, and everything else is a consequence of that.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">THE SYSTEM &middot; ONE VIEW, TWO COMPILERS, ONE OUTPUT DIRECTORY</text>
  <g font-family="var(--font-mono)" font-size="9" text-anchor="middle">
    <rect x="0" y="30" width="140" height="48" fill="var(--signal)"/>
    <text x="70" y="48" fill="var(--paper)">src/lib.rs</text>
    <text x="70" y="62" fill="var(--paper)">LEPTOS VIEW</text>
    <rect x="0" y="105" width="140" height="46" fill="none" stroke="var(--line)"/>
    <text x="70" y="122" fill="var(--ink)">content/articles</text>
    <text x="70" y="136" fill="var(--muted)">MARKDOWN</text>
    <rect x="190" y="26" width="170" height="34" fill="none" stroke="var(--line)"/>
    <text x="275" y="47" fill="var(--ink)">NATIVE BUILD &middot; ssr</text>
    <rect x="190" y="70" width="170" height="34" fill="none" stroke="var(--line)"/>
    <text x="275" y="91" fill="var(--ink)">WASM BUILD &middot; hydrate</text>
    <rect x="190" y="111" width="170" height="34" fill="none" stroke="var(--line)"/>
    <text x="275" y="132" fill="var(--ink)">tools/blog &middot; MD &rarr; HTML</text>
    <rect x="430" y="26" width="190" height="119" fill="var(--ink)" opacity="0.08"/>
    <text x="525" y="80" fill="var(--ink)">dist/</text>
    <text x="525" y="94" fill="var(--muted)">WHAT PAGES SERVES</text>
  </g>
  <g stroke="var(--ink)" fill="none">
    <path d="M140 54 L165 54 L165 43 L184 43 M178 39 L184 43 L178 47"/>
    <path d="M140 54 L165 54 L165 87 L184 87 M178 83 L184 87 L178 91"/>
    <path d="M140 128 L184 128 M178 124 L184 128 L178 132"/>
    <path d="M360 43 L424 43 M418 39 L424 43 L418 47"/>
    <path d="M360 87 L424 87 M418 83 L424 87 L418 91"/>
    <path d="M360 128 L424 128 M418 124 L424 128 L418 132"/>
  </g>
  <text x="0" y="176" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">one view, compiled for two different runtimes</text>
  <text x="620" y="176" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">everything else is a consequence of that</text>
</svg>
<figcaption>The two arrows leaving the view library are the load-bearing part of the whole design. Everything difficult about this site — hydration markers, deterministic initial values, the feature-flag triad — exists because those two arrows must arrive at outputs that agree with each other.</figcaption>
</figure>

## What Leptos is actually doing here

Since the framework choice determines most of what follows, it's worth being
concrete about what it is, rather than filing it under "a Rust web framework."

Leptos is **fine-grained reactive**. There is no virtual DOM and no diffing pass.
The `view!` macro doesn't produce a description of a tree to be reconciled later;
it compiles into code that creates real DOM nodes, and reactive expressions
inside it compile into subscriptions that update exactly the node they belong to.
When a value changes, the framework doesn't ask what changed — the dependency
graph already knows which text node or attribute to touch.

That has a direct consequence for a byte budget: the runtime you ship is a
reactive graph, not a renderer. There is no reconciliation algorithm in the
bundle, because nothing reconciles.

State is held in signals, and the entire interactive portfolio is four of them:

```rust
let category = RwSignal::new("All".to_string());
let query = RwSignal::new(String::new());
let active = RwSignal::new(0usize);
let expanded = RwSignal::new(None::<usize>);
```

Category filter, search text, keyboard cursor, which case study is open. That is
the complete list of what this page can be in a state about.

The filtering shows the model best. Projects are never added or removed:

```rust
class:hidden=move || !projects::matches(project, &category.get(), &query.get())
```

Every project is in the server-rendered HTML permanently, and searching toggles a
class on it. No list reconciliation, no keyed children, no nodes created after
first paint. It also means the no-JavaScript version of this page contains every
project rather than an empty container waiting to be filled — the accessible
fallback isn't a fallback I built, it's the same DOM with nothing hiding
anything.

### The three modes are the whole trick

The same crate compiles under three mutually exclusive feature flags, and they
are the reason this architecture is possible at all:

- `ssr` — native target. The view renders to an HTML string, no DOM involved.
- `hydrate` — wasm target. The view expects its DOM to already exist and attaches to it.
- `csr` — wasm target. The view builds its DOM from nothing. Used by the tests.

`ssr` runs in `tools/site` at build time. `hydrate` is what ships. And hydration
is not "run the app again and hope" — the wasm build walks the existing document
expecting to find the exact structure its own view produces, including the marker
comments Leptos leaves for that purpose. Those stray `<!>` nodes in the served
HTML are not noise; they are how the browser build finds its place.

Which produces the one rule that governs everything the view is allowed to do:

> The static render and the first hydrated render must produce byte-identical
> output.

This is why the runtime diagnostics ship as lies. The page states its boot time,
the transfer size of its own WebAssembly, and the browser engine it's running
on — and a build has no way to know any of the three. So the build writes down
deterministic placeholders instead:

```rust
("fact-engine", "BROWSER ENGINE", "Browser VM"),
("fact-boot",   "BOOT TO WASM ENTRY", "STATIC"),
("fact-wasm",   "WASM RECEIVED", "80 KiB"),
```

and the browser overwrites them once it can actually measure:

```rust
if let Some(cell) = doc.get_element_by_id(&id) {
    cell.set_text_content(Some(&value));
}
```

Writing to a hydrated node from outside the framework sounds like exactly the
thing you're not supposed to do, and it's safe here for a precise reason: no
signal owns those nodes. Both builds emit the same placeholder, hydration
matches, and only then does the measurement land. Had those values been reactive
and initialised from the browser, the hydrating build would have produced
different text than the HTML it was attaching to — and Leptos would either
mismatch or silently corrupt the tree.

The pattern generalises: anything genuinely browser-dependent enters *after*
hydration, never during it. That constraint, arriving from the framework, turns
out to be the same line the deployment target was already drawing.

## The constraints were doing the deciding

Three facts about the deployment target shaped everything downstream, and none
of them were aesthetic preferences.

**There is no server.** GitHub Pages serves files. No request-time rendering, no
edge function, no database. Anything that looks dynamic has to have been decided
earlier, by something running on my machine or in CI.

**There is a hard size budget.** CI gzips the release WebAssembly module and
fails the build above 512000 bytes. It is enforced, not aspirational:

```yaml
gzip -9 -c "$wasm_file" > /tmp/app.wasm.gz
bytes="$(wc -c < /tmp/app.wasm.gz)"
test "$bytes" -le 512000
```

A budget you can't fail is a wish. This one has teeth, so every runtime feature
has to justify its bytes against features that already exist.

**There is no hand-written application JavaScript.** Not as a purity stunt —
because two hand-maintained implementations of the same view drift, and I did
not want a JS layer whose only job is to re-describe what the Rust already knows.

Put those together and the architecture stops being a choice. Work migrates to
build time because build time is the only place with room for it.

<figure class="diagram">
<svg viewBox="0 0 620 136" role="img" aria-label="A left-to-right pipeline of five stages. Trunk emits the shell and WebAssembly, then a blog generator turns Markdown into HTML, then esbuild minifies, then a site generator renders the portfolio body, and the result is the dist directory drawn as a filled block. A bracket beneath the middle three stages is labelled: one post build hook, deliberately serial. A note says Trunk may run same-stage hooks concurrently, so the shell script runs them in order.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">THE BUILD &middot; EVERY STAGE RUNS BEFORE A READER EXISTS</text>
  <g font-family="var(--font-mono)" font-size="9" text-anchor="middle">
    <rect x="0" y="26" width="100" height="44" fill="none" stroke="var(--line)"/>
    <text x="50" y="44" fill="var(--ink)">TRUNK</text>
    <text x="50" y="56" fill="var(--muted)">SHELL + WASM</text>
    <rect x="120" y="26" width="100" height="44" fill="none" stroke="var(--line)"/>
    <text x="170" y="44" fill="var(--ink)">BLOG GEN</text>
    <text x="170" y="56" fill="var(--muted)">MD &rarr; HTML</text>
    <rect x="240" y="26" width="100" height="44" fill="none" stroke="var(--line)"/>
    <text x="290" y="44" fill="var(--ink)">ESBUILD</text>
    <text x="290" y="56" fill="var(--muted)">MINIFY</text>
    <rect x="360" y="26" width="100" height="44" fill="none" stroke="var(--line)"/>
    <text x="410" y="44" fill="var(--ink)">SITE GEN</text>
    <text x="410" y="56" fill="var(--muted)">SSR BODY</text>
    <rect x="480" y="26" width="140" height="44" fill="var(--signal)"/>
    <text x="550" y="51" fill="var(--paper)">DIST</text>
  </g>
  <g stroke="var(--ink)" fill="none">
    <path d="M100 48 L114 48 M108 44 L114 48 L108 52"/>
    <path d="M220 48 L234 48 M228 44 L234 48 L228 52"/>
    <path d="M340 48 L354 48 M348 44 L354 48 L348 52"/>
    <path d="M460 48 L474 48 M468 44 L474 48 L468 52"/>
  </g>
  <path d="M120 78 L120 86 L460 86 L460 78" fill="none" stroke="var(--line)"/>
  <text x="290" y="102" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">ONE post_build HOOK &middot; DELIBERATELY SERIAL</text>
  <text x="0" y="128" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">trunk may run same-stage hooks concurrently</text>
  <text x="620" y="128" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">so one shell script runs them in order</text>
</svg>
<figcaption>The ordering is a real dependency, not tidiness. The site generator inlines the CSS that minification just produced, so it has to run after it — and Trunk offers no ordering guarantee between hooks in the same stage, which is why they live in a single script instead of three hook entries.</figcaption>
</figure>

## Rendering the portfolio into its own shell

The native side of that fan-out is four lines. The generator links the library
and asks it for HTML:

```rust
pub fn render_static_app() -> String {
    view! { <App /> }.to_html()
}
```

Trunk has already emitted the document shell and the hydrate-mode bundle by that
point. The site generator then replaces the body, rather than appending to it,
so repeated watch builds converge instead of accumulating:

```rust
format!("{}<body>{topbar}{BOOT}{NO_SCRIPT}<div id=\"app\">{app}</div>{panel}</body>{}", …)
```

The app gets its own `#app` root so everything around it can live outside
Leptos' hydration cursor — anything inside it would be something the browser
build has to account for and doesn't know about. The boot screen and the
`<noscript>` notice are there for that reason. The header and the diagnostics
panel are there for a different one, which is the subject of a later section:
they are rendered from a builder the article generator also calls, and code the
browser build never runs has no business inside the tree it hydrates.

That generator also does one thing that looks like a micro-optimization and
isn't. Trunk emits a `<link rel="preload" as="fetch">` for the WebAssembly
module, and `wasm-bindgen`'s loader later fetches the same URL itself. The two
requests have different modes, so the browser can't reuse the preloaded
response, and the module is downloaded twice. The fix is to delete that one
preload tag from the final document and keep everything else. On a site whose
whole premise is a byte budget, shipping the payload twice would have been a
quiet joke at my own expense.

## Writing is a different pipeline with the same rule

Articles are Markdown with TOML front matter. A second native binary reads
`content/articles/*.md` and writes static HTML: `pulldown-cmark` for the
Markdown, `syntect` for syntax highlighting, plus a table of contents, per-tag
index pages, an Atom feed, a refreshed sitemap, and a reading-time estimate.

Syntax highlighting is the clearest case of the rule. The alternative — shipping
a highlighter to every reader so it can re-derive, on every page view, a
colouring that is identical every time — is the same computation performed a
few thousand times instead of once. So `syntect` runs during the build, emits
class names, and the theme ships as a stylesheet. Readers get coloured code with
zero bytes of highlighter.

The reading time is a rounding of `words / 200`, and it deliberately strips
inline HTML first so a large SVG diagram doesn't inflate the estimate into a
lie. Small thing. It has a test, because the interesting failure is silent.

One behaviour worth stealing: the generator deletes and recreates its own output
subtree on every run. Moving an article into `drafts/` should remove its route,
its feed entry, and its tag page — not leave a public URL behind that nothing
links to anymore and nothing will ever clean up. A generator that only ever adds
files is a generator that lies about what's published.

## What the runtime is actually allowed to do

Here is the entire list of things this site decides in a browser:

1. Which article routes get a copy button, and what's on the clipboard when you press one.
2. How far down the page you are.
3. Whether the runtime diagnostics panel is open.
4. Whether this is a cold or warm load.

That's it. Everything else — content, structure, highlighting, navigation,
metadata, feeds — was settled before the request.

<figure class="diagram">
<svg viewBox="0 0 620 116" role="img" aria-label="Two bars on a common left edge. A long solid bar labelled build time lists HTML, highlighting, table of contents, feed, sitemap and CSS. A much shorter outlined bar labelled runtime lists scroll position, clipboard and cache state. A note beneath says the widths are not a measurement, the asymmetry is the design.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">WHAT IS DECIDED WHEN</text>
  <text x="0" y="45" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">BUILD TIME</text>
  <rect x="110" y="32" width="510" height="18" fill="var(--signal)"/>
  <text x="365" y="45" font-family="var(--font-mono)" font-size="8" fill="var(--paper)" text-anchor="middle">HTML &middot; HIGHLIGHTING &middot; TOC &middot; FEED &middot; SITEMAP &middot; CSS &middot; ROUTES</text>
  <text x="0" y="79" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">RUNTIME</text>
  <rect x="110" y="66" width="86" height="18" fill="none" stroke="var(--line)"/>
  <text x="206" y="79" font-family="var(--font-mono)" font-size="8" fill="var(--muted)">SCROLL POSITION &middot; CLIPBOARD &middot; CACHE STATE</text>
  <text x="0" y="108" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the widths are not a measurement</text>
  <text x="620" y="108" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">the asymmetry is the design</text>
</svg>
<figcaption>Anything on the top row is computed once, by me, and served as bytes. Anything on the bottom row genuinely cannot be known until a specific person is looking at a specific viewport — which is the only admission ticket to that row.</figcaption>
</figure>

Article pages are fully static HTML that never needed the bundle. They load it
anyway, purely to add the two touches above, and the generator inlines that
loader itself:

```rust
let wasm = js.replace(".js", "_bg.wasm");
format!("<script type=\"module\">import init from \"/{js}\";init({{module_or_path:\"/{wasm}\"}})</script>")
```

If no bundle is found — a manual generator run outside a Trunk build — the
string is empty and the pages simply stay static. That is the honest test of
progressive enhancement: the degraded path is not a fallback I wrote, it is what
happens when I write nothing.

The reading-progress bar is the whole model in thirty lines. It doesn't exist in
the served HTML; the wasm creates it, then updates one CSS property per scroll
event:

```rust
let pct = (scrolled / (full - viewport).max(1.0) * 100.0).clamp(0.0, 100.0);
bar.style().set_property("width", &format!("{pct}%"));
```

`full - viewport` is the real scrollable distance, so the bar reaches 100% at
the bottom instead of short of it. `.clamp` absorbs iOS rubber-banding.

And the smoothness people notice is not in that code at all. It's one line of
CSS:

```css
.reading-progress { transition: width 0.08s linear; }
```

Scroll events arrive in irregular bursts. An 80ms linear transition interpolates
across the gaps, so the bar glides between samples instead of stepping. The Rust
is a sampler; the smoothing is the browser's job, done in a line, for free. I
mention it because it is the shape of most good decisions in this codebase — the
cheapest layer that can own a problem should own it.

<figure class="diagram">
<svg viewBox="0 0 620 128" role="img" aria-label="Two horizontal rails labelled JS and WASM. Five arrows cross between them in sequence: a scroll event descending into WASM, then three ascending reads for scrollY, innerHeight and scrollHeight, then one descending write setting the width. A note says five crossings per event, and that the CSS transition hides the gaps between them.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">ONE SCROLL EVENT, CROSSING THE BOUNDARY</text>
  <text x="0" y="33" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">JS</text>
  <text x="0" y="97" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">WASM</text>
  <line x1="46" y1="30" x2="620" y2="30" stroke="var(--line)"/>
  <line x1="46" y1="94" x2="620" y2="94" stroke="var(--line)"/>
  <g stroke="var(--signal)" fill="none">
    <path d="M80 30 L80 94 M76 88 L80 94 L84 88"/>
    <path d="M520 30 L520 94 M516 88 L520 94 L524 88"/>
  </g>
  <g stroke="var(--ink)" fill="none">
    <path d="M200 94 L200 30 M196 36 L200 30 L204 36"/>
    <path d="M300 94 L300 30 M296 36 L300 30 L304 36"/>
    <path d="M400 94 L400 30 M396 36 L400 30 L404 36"/>
  </g>
  <g font-family="var(--font-mono)" font-size="8" fill="var(--muted)">
    <text x="86" y="66">scroll event</text>
    <text x="206" y="66">scrollY</text>
    <text x="306" y="66">innerHeight</text>
    <text x="406" y="66">scrollHeight</text>
    <text x="526" y="66">width = n%</text>
  </g>
  <text x="0" y="120" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">five crossings per event</text>
  <text x="620" y="120" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">the css transition hides the gaps between them</text>
</svg>
<figcaption>WebAssembly cannot touch the DOM. Every one of these arrows is a generated <code>web-sys</code> shim, and DOM objects are integer handles into a JavaScript-side table rather than anything living in linear memory — which is why every call in that module returns a <code>Result</code> or an <code>Option</code>.</figcaption>
</figure>

## The two scripts I did write

"No hand-written JavaScript" is true of the application and false of the
document, and the exceptions are informative.

The first is four lines in the shell that run before anything else:

```html
<script>
  try { if (localStorage.getItem('fe:booted')) document.documentElement.setAttribute('data-warm', ''); } catch (e) {}
</script>
```

A cold load gets a boot screen while the WebAssembly downloads. A warm load
shouldn't — the module is cached, the wait is gone, and showing a loading
sequence anyway is theatre. But the decision has to be made *before first paint*,
which is strictly before any wasm exists to make it. There is no Rust that can
run early enough. So there is a blocking inline script, and it stays as small as
a thing in that position deserves to be.

The second is the module loader above. Same category: the code that bootstraps
the runtime cannot itself be the runtime.

Both are the boundary announcing itself. If a rule has no exceptions you have
probably not tested it against anything real.

## The budget as a design instrument

The release profile is tuned for size rather than speed — `opt-level = "z"`,
fat LTO, one codegen unit, `panic = "abort"`, symbols stripped. `panic = "abort"`
in particular deletes the unwinding machinery, which a browser page has no use
for; a panic here is a bug I need to fix, not a condition I intend to recover
from.

`web-sys` is also opted into a binding at a time. The crate covers essentially
all of the web platform, and the `Cargo.toml` lists only what's reachable:
`Clipboard`, `CssStyleDeclaration`, `NodeList`, `PerformanceResourceTiming`, and
a dozen more. Adding a feature is a visible line in a diff, which is exactly the
friction it should have.

The value of the CI gate is not the number. It's that a size regression fails a
pull request the same way a broken test does, so "I'll trim it later" is not an
available move. Later is a place where nobody trims anything.

## The seams that would drift, and what holds them

Two independently rendered surfaces — an interactive portfolio and a static
article generator — will disagree about their own navigation eventually. So the
navigation isn't in either of them:

```rust
pub const PRIMARY_NAV: [NavItem; 5] = [ … ];
```

Shared source, with a test pinning the properties that actually matter: unique
destinations, and a `Writing` link that stays absolute so it doesn't resolve
relative to a generated article route. That test doesn't check that navigation
looks right. It checks the two ways it silently goes wrong.

The chrome around it went the same way, though it took a detour first. The
header and the diagnostics panel used to exist twice — once as a Leptos `view!`
with a signal behind it, once as a `format!` string in the article generator —
and both surfaces loaded the same stylesheet, which made them look identical
enough that the duplication never announced itself. Now one builder emits the
markup for both, and one imperative module attaches the behaviour, because the
article side already had to do it that way and the general case was sitting
there the whole time. It was a net deletion: a component, two view blocks, four
signals, two effects, and a copy of the panel built with `innerHTML`.

### The bug that made the argument for me

The tokens were the last thing still duplicated, and they are where it actually
broke. Each surface declared its own `:root`:

```css
/* styles.css   */ --line: rgba(10, 10, 10, 0.3);
/* article.css  */ --line: rgba(10, 10, 10, 0.18);
```

Same name, different value, because a task to strengthen the portfolio's grid
hairlines moved one of them and not the other. Nothing looked wrong on either
page in isolation. But `shared/header.css` — one file, one rule — styles the
mobile navigation with `var(--line)`, so a single line of CSS rendered two
different hairlines depending on which document it landed in.

The design system had already solved this properly, with `--line` and a separate
`--line-soft` for exactly the lighter case Writing wanted. The site had simply
never adopted the second name, so `--line` quietly meant two things.

One `shared/tokens.css` now, and a test that fails if any surface redefines a
shared token or if the design system drifts from it. That's the shape of every
fix in this section: not "be careful," but "make the careless version fail."

Which is the point of the last seam, the design system itself. `@fblln/design-system`
mirrors the site's components in React so a design built from it maps onto the
live site one-to-one. The invariant is one-directional and unforgiving: the site
is the source of truth, and any token change here lands there in the same commit.
Two systems claiming to be the same design, updated at different times, are just
one system and one rumour about it.

## What I'd say the architecture actually is

Not "Rust and WebAssembly." That's the material, not the shape.

The shape is a line drawn through the middle of the system, with a single
question deciding which side each piece lands on: *can this be known before a
reader arrives?* Almost everything can. Content, structure, highlighting,
routing, metadata, styling — all of it collapses to bytes on disk. What's left
over is a genuinely short list of things that depend on a particular person, on
a particular viewport, at a particular moment.

The runtime is not the point of the system. The runtime is the residue after
the build has taken everything it can.

I did not arrive at that by taste. I arrived at it because a host with no server
and a budget with real teeth removed the alternatives one at a time, which in my
experience is where most durable architecture comes from. The constraint decides;
you just find out what it decided.

## References

**The stack**

1. Leptos — server-side rendering and hydration model.
   https://leptos.dev/

2. Trunk — WASM web application bundler, build hooks, and asset pipeline.
   https://trunkrs.dev/

3. `wasm-bindgen` and `web-sys` — the generated JavaScript/WebAssembly boundary.
   https://rustwasm.github.io/wasm-bindgen/

4. `pulldown-cmark` and `syntect` — build-time Markdown parsing and syntax
   highlighting.
   https://github.com/raphlinus/pulldown-cmark

**The ideas**

5. Fabio Ellena, "Architecture Must Follow Pressure," 2026.
   https://fblln.github.io/articles/architecture-must-follow-pressure/

6. Fabio Ellena, "Performance Is Making Gradle Less Gradle," 2026.
   https://fblln.github.io/articles/performance-is-making-gradle-less-gradle/
