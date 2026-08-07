+++
title = "Performance Is Making Gradle Less Gradle"
date = "2026-08-07"
description = "Gradle 9.7 promotes Isolated Projects to incubating, and the headline is parallel project configuration. The real story is that Gradle can only parallelize configuration because it has started making entire categories of previously legal build logic illegal. Performance pressure is quietly turning a programmable build system into a declarative one — and TypeScript is doing the same thing from the opposite direction."
tags = ["Architecture", "Build Systems", "Performance", "Complexity"]
+++

Gradle has always been interesting to me because the thing that made it good
is the same thing that made it expensive.

You could do almost anything.

A build file was never really a build description. It was a program that
happened to be about building. Projects could inspect each other, mutate each
other, reach into global state, configure things three levels of indirection
away, and generally perform tricks that look clever in a five-module demo and
look like an incident in a four-hundred-module repository.

At small scale that flexibility was the selling point. At large scale it
became rent.

Gradle 9.7 promotes [Isolated
Projects](https://docs.gradle.org/current/userguide/isolated_projects.html)
from experimental to incubating. The headline is a performance feature. The
interesting part is what Gradle had to take away to get it.

## The original flexibility was not a mistake

I do not mean that Gradle was badly designed. The flexibility *was* the
product.

If you arrived from a tool where a build was a rigid XML incantation, having a
real language and a real object model available during the build felt like
being let out of prison. Want to configure every subproject from the root?

```groovy
subprojects {
    tasks.withType(Test) {
        ...
    }
}
```

Done. Want project A to read project B? You could. Want a plugin to crawl the
entire project model and mutate whatever it finds interesting? Nothing stopped
you. Certainly not Gradle.

The trouble starts when Gradle tries to answer one deceptively boring
question:

> Can I configure these two projects at the same time?

At that point every piece of that freedom sends its invoice.

If configuring `:vehicle-api` might mutate `:telemetry`, Gradle cannot
configure them concurrently. If a plugin can read arbitrary global state,
Gradle cannot know what the configuration actually depends on. If configuring
one project has invisible side effects on another, Gradle cannot cache either
result independently.

The performance problem was never only a performance problem. It was an
architecture problem holding a stopwatch.

## Parallelism has a price, and the price is isolation

This is the part worth writing about.

With Isolated Projects enabled, build logic belonging to one project can no
longer freely access the mutable state of another. Gradle can therefore
configure projects concurrently without guessing.

The causal chain is almost suspiciously clean:

<figure class="diagram">
<svg viewBox="0 0 620 146" role="img" aria-label="A left-to-right chain of four stages. The first, filled solid, is performance pressure. Arrows lead through parallel configuration, safe concurrency and project isolation. An arrow drops from the last stage into a bar spanning the full width labelled stricter build architecture. A note beneath says nobody chose the boundary for its elegance; the stopwatch chose it.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">THE CHAIN &middot; PERFORMANCE ASKING FOR ARCHITECTURE</text>
  <g font-family="var(--font-mono)" font-size="9" text-anchor="middle">
    <rect x="0" y="26" width="140" height="44" fill="var(--signal)"/>
    <text x="70" y="48" fill="var(--paper)">PERFORMANCE</text>
    <text x="70" y="60" fill="var(--paper)">PRESSURE</text>
    <rect x="160" y="26" width="140" height="44" fill="none" stroke="var(--line)"/>
    <text x="230" y="48" fill="var(--ink)">PARALLEL</text>
    <text x="230" y="60" fill="var(--ink)">CONFIGURATION</text>
    <rect x="320" y="26" width="140" height="44" fill="none" stroke="var(--line)"/>
    <text x="390" y="48" fill="var(--ink)">SAFE</text>
    <text x="390" y="60" fill="var(--ink)">CONCURRENCY</text>
    <rect x="480" y="26" width="140" height="44" fill="none" stroke="var(--line)"/>
    <text x="550" y="48" fill="var(--ink)">PROJECT</text>
    <text x="550" y="60" fill="var(--ink)">ISOLATION</text>
  </g>
  <g stroke="var(--ink)" fill="none">
    <path d="M140 48 L154 48 M148 44 L154 48 L148 52"/>
    <path d="M300 48 L314 48 M308 44 L314 48 L308 52"/>
    <path d="M460 48 L474 48 M468 44 L474 48 L468 52"/>
    <path d="M550 70 L550 88 M546 82 L550 88 L554 82"/>
  </g>
  <rect x="0" y="92" width="620" height="26" fill="var(--ink)" opacity="0.08"/>
  <text x="310" y="109" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">STRICTER BUILD ARCHITECTURE</text>
  <text x="0" y="138" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">nobody chose the boundary for its elegance</text>
  <text x="620" y="138" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">the stopwatch chose it</text>
</svg>
<figcaption>Read left to right it is a performance roadmap; read the bar at the bottom and it is an architecture decision. Nobody at Gradle argued for the boundary on its merits — the schedule made it the only remaining move.</figcaption>
</figure>

Nobody at Gradle woke up one morning moved by the inherent beauty of
architectural boundaries. Gradle needs to configure large builds faster.
Parallelism is the obvious lever. Parallelism requires independence.
Independence requires removing shared mutable state.

And so something that used to be a polite suggestion in a best-practices
document becomes a constraint the tool enforces.

Gradle 9.7 leans into that. Violations fail fast by default, several accesses
to mutable build-scoped state are treated as incompatible, and the legacy
`org.gradle.unsafe.isolated-projects` property names are deprecated in favour
of `org.gradle.isolated-projects` — they still work as aliases, for now.

The `.unsafe` prefix is disappearing because the constraint is becoming the
architecture. It is worth being precise about the maturity, though: the
feature is not enabled by default and is not yet recommended for production.
Most builds and most plugins will need changes to satisfy it. That is not a
footnote — it is the whole point. The tool is asking the ecosystem to give
something up.

## Gradle has been moving this way for years

Isolated Projects did not appear from nowhere. Line up Gradle's performance
features in order and the pattern is hard to miss.

**Incremental builds.** Gradle needs to know `inputs → task → outputs`. If a
task secretly reads or writes something Gradle does not know about,
incrementality quietly lies. Freedom down, performance up.

**Build cache.** Now Gradle wants to reuse outputs on a different machine.
That demands a stronger contract, roughly `f(inputs) = outputs`. Hidden
dependencies stop being quirks and start being bugs. More freedom removed.

**Configuration cache.** Gradle asks the obvious follow-up: why configure the
build from scratch every time? But caching configuration means the
configuration phase can no longer behave like arbitrary application startup
code. Gradle has to understand and serialize the resulting model, and plugins
have to stop holding references to objects they do not own.

**Isolated Projects.** Now configuration itself should run concurrently:

<figure class="diagram">
<svg viewBox="0 0 620 112" role="img" aria-label="A wall-clock timeline. Two configuration bars, project A drawn solid and project B outlined, both start at the same moment and overlap for their whole duration. Notes to the right say there is no ordering constraint between them because neither can reach the other's state. The axis is labelled serialised before, overlapped now.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">CONFIGURATION &middot; TWO LANES ON ONE WALL CLOCK</text>
  <path d="M60 24 L60 92" stroke="var(--line)" stroke-dasharray="3 3" fill="none"/>
  <rect x="60" y="28" width="240" height="20" fill="var(--signal)"/>
  <text x="70" y="42" font-family="var(--font-mono)" font-size="9" fill="var(--paper)">PROJECT A CONFIGURATION</text>
  <rect x="60" y="56" width="262" height="20" fill="none" stroke="var(--line)"/>
  <text x="70" y="70" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">PROJECT B CONFIGURATION</text>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="344" y="42">no ordering constraint between them</text>
    <text x="344" y="70">because neither can reach the other</text>
  </g>
  <path d="M0 92 L614 92 M608 88 L614 92 L608 96" stroke="var(--ink)" fill="none"/>
  <text x="0" y="108" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">wall clock &rarr;</text>
  <text x="620" y="108" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">serialised before &middot; overlapped now</text>
</svg>
<figcaption>The overlap is the entire feature. Everything Isolated Projects takes away exists to make these two bars legal to draw starting at the same tick.</figcaption>
</figure>

A project is supposed to describe itself, not wander through the build
modifying its neighbours.

Every step is the same move:

<figure class="diagram">
<svg viewBox="0 0 620 190" role="img" aria-label="Four columns, one per Gradle optimization: incremental builds, build cache, configuration cache and isolated projects, each with the contract it demands. Beneath each column a pair of bars sits on a shared baseline. The pale bar, representing the freedom the build logic keeps, shrinks from left to right. The solid bar, representing what Gradle is allowed to assume, grows by the same steps.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">FOUR OPTIMIZATIONS &middot; EACH BUYS SPEED WITH FREEDOM</text>
  <g font-family="var(--font-mono)" font-size="9" text-anchor="middle">
    <rect x="0" y="24" width="140" height="24" fill="none" stroke="var(--line)"/>
    <text x="70" y="39" fill="var(--ink)">INCREMENTAL</text>
    <rect x="160" y="24" width="140" height="24" fill="none" stroke="var(--line)"/>
    <text x="230" y="39" fill="var(--ink)">BUILD CACHE</text>
    <rect x="320" y="24" width="140" height="24" fill="none" stroke="var(--line)"/>
    <text x="390" y="39" fill="var(--ink)">CONFIG CACHE</text>
    <rect x="480" y="24" width="140" height="24" fill="none" stroke="var(--line)"/>
    <text x="550" y="39" fill="var(--ink)">ISOLATED PROJECTS</text>
  </g>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">
    <text x="70" y="64">inputs &rarr; task &rarr; outputs</text>
    <text x="230" y="64">f(inputs) = outputs</text>
    <text x="390" y="64">the model must serialize</text>
    <text x="550" y="64">no shared mutable state</text>
  </g>
  <g fill="var(--ink)" opacity="0.12">
    <rect x="8" y="94" width="58" height="46"/>
    <rect x="168" y="106" width="58" height="34"/>
    <rect x="328" y="118" width="58" height="22"/>
    <rect x="488" y="130" width="58" height="10"/>
  </g>
  <g fill="var(--signal)">
    <rect x="74" y="130" width="58" height="10"/>
    <rect x="234" y="118" width="58" height="22"/>
    <rect x="394" y="106" width="58" height="34"/>
    <rect x="554" y="94" width="58" height="46"/>
  </g>
  <path d="M0 140 L620 140" stroke="var(--line)" fill="none"/>
  <rect x="0" y="152" width="10" height="10" fill="var(--ink)" opacity="0.12"/>
  <text x="16" y="161" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">freedom the build logic keeps</text>
  <rect x="320" y="152" width="10" height="10" fill="var(--signal)"/>
  <text x="336" y="161" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">what Gradle is allowed to assume</text>
  <text x="0" y="184" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">every column makes something illegal that used to be legal</text>
  <text x="620" y="184" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">that removal is the optimization</text>
</svg>
<figcaption>The two bars are the same quantity measured from opposite ends. Gradle never found a faster algorithm for configuration — it kept buying knowledge about the build with freedom taken from the build.</figcaption>
</figure>

The end state looks remarkably declarative for a tool whose original pitch was
that your build script is just code.

## Strictness is what buys the speed

The headline is: *Gradle can configure projects in parallel.*

The actual story is: *Gradle can configure projects in parallel because it
started making entire categories of previously legal behaviour illegal.*

That distinction gets lost constantly. Performance work is usually sold as
implementation wizardry — better algorithms, larger caches, more threads. But
often the largest optimization is making the system possible to reason about.

Remove hidden dependencies. Restrict mutation. Draw boundaries. Turn implicit
inputs into explicit ones.

Do that, and the machine suddenly has options. It can cache. It can reorder.
It can parallelize. It can skip work entirely.

> Strict architecture does not cost performance. It creates the space where
> performance becomes legal.

The numbers Gradle reports from migrating its own build are not subtle. An IDE
sync after adding a dependency dropped from roughly 45 seconds to 27. A sync
after changing build logic went from about 2 minutes 57 seconds to 1 minute
16. Across months of real developer usage, median sync time fell from roughly
1:24 to 0:47, and the 95th percentile dropped from nearly seven minutes to a
little over three.

<figure class="diagram">
<svg viewBox="0 0 620 200" role="img" aria-label="Four measurements from Gradle migrating its own build, drawn to scale at 1.35 pixels per second. Each row shows a pale bar for the time before and a solid bar for the time after, starting from the same origin. Sync after adding a dependency falls from 45 to 27 seconds. Sync after a build-logic change falls from 2 minutes 57 to 1 minute 16. Median sync falls from 1 minute 24 to 47 seconds. The 95th percentile falls from nearly 7 minutes to a little over 3.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">GRADLE&rsquo;S OWN BUILD &middot; IDE SYNC, BEFORE AND AFTER</text>
  <g font-family="var(--font-mono)" font-size="9">
    <text x="0" y="34" fill="var(--ink)">SYNC AFTER ADDING A DEPENDENCY</text>
    <text x="620" y="34" fill="var(--signal)" text-anchor="end">45s &rarr; 27s</text>
    <text x="0" y="72" fill="var(--ink)">SYNC AFTER A BUILD-LOGIC CHANGE</text>
    <text x="620" y="72" fill="var(--signal)" text-anchor="end">2:57 &rarr; 1:16</text>
    <text x="0" y="110" fill="var(--ink)">MEDIAN SYNC &middot; MONTHS OF REAL USE</text>
    <text x="620" y="110" fill="var(--signal)" text-anchor="end">1:24 &rarr; 0:47</text>
    <text x="0" y="148" fill="var(--ink)">95TH PERCENTILE</text>
    <text x="620" y="148" fill="var(--signal)" text-anchor="end">~6:50 &rarr; ~3:10</text>
  </g>
  <g fill="var(--ink)" opacity="0.12">
    <rect x="0" y="40" width="61" height="12"/>
    <rect x="0" y="78" width="239" height="12"/>
    <rect x="0" y="116" width="113" height="12"/>
    <rect x="0" y="154" width="553" height="12"/>
  </g>
  <g fill="var(--signal)">
    <rect x="0" y="40" width="36" height="12"/>
    <rect x="0" y="78" width="103" height="12"/>
    <rect x="0" y="116" width="63" height="12"/>
    <rect x="0" y="154" width="257" height="12"/>
  </g>
  <text x="0" y="190" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">drawn to scale &middot; one second = 1.35 px</text>
  <text x="620" y="190" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="end">pale bar: before &middot; solid bar: after</text>
</svg>
<figcaption>The bottom row is the one that matters. Medians move for many reasons; a 95th percentile falling by half is the tail of the distribution — the sync that made someone stop and check their phone — being cut off at the source.</figcaption>
</figure>

Seven minutes to three is a large amount of accumulated pressure finally
finding an exit.

## TypeScript is doing the same thing from the other end

There is a second example running on a parallel track, and it is almost too
neat.

TypeScript started from the opposite corner of the design space. Its genius
was gradual adoption: take JavaScript, add some types, keep going, do not
frighten anyone. For years it kept a deliberately strange relationship with
strictness — capable of very strong guarantees, and perfectly willing to let
you off the hook.

Then the defaults moved. TypeScript 6.0 makes `strict` true by default, and
TypeScript 7 carries those defaults into the native compiler. Individual
options can still be relaxed, and `"strict": false` remains something you can
write. Culturally, that is a footnote. What changed is the contract:

<figure class="diagram">
<svg viewBox="0 0 620 156" role="img" aria-label="Two panels compare TypeScript's defaults. On the left, labelled before, an outlined block reads permissive, with a dashed branch beneath it reading optionally become strict, and a note that a project ends up unchecked by accident. On the right, labelled now, a solid block reads strict, with a branch reading explicitly ask for exceptions, and a note that a project ends up unchecked by decision.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">THE DEFAULT MOVED &middot; TYPESCRIPT 6.0</text>
  <text x="0" y="32" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">BEFORE</text>
  <text x="320" y="32" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">NOW</text>
  <rect x="0" y="40" width="300" height="28" fill="none" stroke="var(--line)"/>
  <text x="150" y="58" font-family="var(--font-mono)" font-size="10" fill="var(--ink)" text-anchor="middle">PERMISSIVE</text>
  <rect x="320" y="40" width="300" height="28" fill="var(--signal)"/>
  <text x="470" y="58" font-family="var(--font-mono)" font-size="10" fill="var(--paper)" text-anchor="middle">STRICT</text>
  <path d="M20 68 L20 92 L36 92" stroke="var(--line)" stroke-dasharray="3 3" fill="none"/>
  <text x="42" y="95" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">optionally become strict</text>
  <path d="M340 68 L340 92 L356 92" stroke="var(--signal)" fill="none"/>
  <text x="362" y="95" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">explicitly ask for exceptions</text>
  <text x="20" y="120" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">a project ends up unchecked by accident</text>
  <text x="340" y="120" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">a project ends up unchecked by decision</text>
  <text x="0" y="148" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the capability never changed &middot; both columns can express both states</text>
  <text x="620" y="148" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">only who has to say so</text>
</svg>
<figcaption>Every configuration reachable on the right was reachable on the left. What moved is the cost of ending up unchecked: it used to be free and silent, and now it takes a line of config with your name on it.</figcaption>
</figure>

The team's own reasoning is the interesting bit: with `--strict` off, a
project that ends up unchecked usually got there by accident rather than by
decision. That is a statement about scale, not about taste. TypeScript needed
permissiveness to invade the JavaScript world. Once it became the
infrastructure underneath enormous codebases, permissiveness stopped being
purely an asset.

The pressure changed, so the default changed.

## Mature systems tend to get less permissive

I think this pattern is consistently underrated.

Young technologies win by being flexible. They let you escape. They do not
impose architecture. They tolerate the deeply unpleasant thing your existing
system already does, because tolerating it is how they get adopted at all.

Then people actually use them. Repositories become enormous. Build graphs
acquire thousands of nodes. Millions of developers start expecting IDE
feedback to be instantaneous. Plugins interact in combinations nobody
designed, tested, or imagined.

And flexibility acquires a measurable cost:

<figure class="diagram">
<svg viewBox="0 0 620 178" role="img" aria-label="Four pale bars grow longer down the figure: flexibility, implicit dependencies, larger state space, harder reasoning. Below a dividing rule, a single short solid bar labelled safe optimizations is a fraction of their width, annotated as what is left for the machine.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">WHAT FLEXIBILITY COSTS AT SCALE</text>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--ink)">
    <text x="0" y="36">FLEXIBILITY</text>
    <text x="0" y="60">IMPLICIT DEPENDENCIES</text>
    <text x="0" y="84">LARGER STATE SPACE</text>
    <text x="0" y="108">HARDER REASONING</text>
  </g>
  <g fill="var(--ink)" opacity="0.12">
    <rect x="230" y="27" width="70" height="12"/>
    <rect x="230" y="51" width="150" height="12"/>
    <rect x="230" y="75" width="250" height="12"/>
    <rect x="230" y="99" width="350" height="12"/>
  </g>
  <path d="M0 126 L620 126" stroke="var(--line)" fill="none"/>
  <text x="0" y="148" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">SAFE OPTIMIZATIONS</text>
  <rect x="230" y="139" width="46" height="12" fill="var(--signal)"/>
  <text x="286" y="148" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">&middot; what is left for the machine</text>
  <text x="0" y="172" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the cost is not the freedom &middot; it is the state space the freedom opens</text>
  <text x="620" y="172" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">and the optimizer pays it</text>
</svg>
<figcaption>Nothing in the top four bars is a defect. Each is a capability someone wanted, and each one widens the space of programs the tool must assume it might be running — which is drawn here as the thing that squeezes the bar at the bottom.</figcaption>
</figure>

At that point restrictions become features. Not because someone finally
discovered the One True Architecture, but because the system is under load and
something has to give.

## The rule has to earn its existence

This is really why I like the Isolated Projects story. It is the clearest
recent example of the thing I keep writing about: [architecture must follow
pressure](/articles/architecture-must-follow-pressure/), and a boundary is
only [worth its cost](/articles/risk-complexity-and-pressure/) when you can
name the force that put it there.

"Projects should not mutate each other" reads like another clean-code
commandment when it appears in a style guide. It becomes considerably more
interesting when the causal chain is visible:

<figure class="diagram">
<svg viewBox="0 0 620 214" role="img" aria-label="A vertical load path of five stacked blocks, each with an annotation to its right. Huge multi-project builds, annotated measured rather than assumed. Slow configuration and IDE sync, annotated the complaint everyone files. Parallel configuration, annotated the only lever left. Shared mutable state is unsafe, annotated a fact about concurrency. The final block, drawn solid, reads projects must be isolated, annotated and only here does the rule appear.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">THE RULE, WITH ITS RECEIPT</text>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="0" y="24" width="300" height="24" fill="none" stroke="var(--line)"/>
    <text x="12" y="40" fill="var(--ink)">HUGE MULTI-PROJECT BUILDS</text>
    <rect x="0" y="60" width="300" height="24" fill="none" stroke="var(--line)"/>
    <text x="12" y="76" fill="var(--ink)">SLOW CONFIGURATION AND IDE SYNC</text>
    <rect x="0" y="96" width="300" height="24" fill="none" stroke="var(--line)"/>
    <text x="12" y="112" fill="var(--ink)">PARALLEL CONFIGURATION</text>
    <rect x="0" y="132" width="300" height="24" fill="none" stroke="var(--line)"/>
    <text x="12" y="148" fill="var(--ink)">SHARED MUTABLE STATE IS UNSAFE</text>
    <rect x="0" y="168" width="300" height="26" fill="var(--signal)"/>
    <text x="12" y="185" fill="var(--paper)">PROJECTS MUST BE ISOLATED</text>
  </g>
  <g stroke="var(--ink)" fill="none">
    <path d="M20 48 L20 58 M16 53 L20 58 L24 53"/>
    <path d="M20 84 L20 94 M16 89 L20 94 L24 89"/>
    <path d="M20 120 L20 130 M16 125 L20 130 L24 125"/>
    <path d="M20 156 L20 166 M16 161 L20 166 L24 161"/>
  </g>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="330" y="40">measured, not assumed</text>
    <text x="330" y="76">the complaint everyone files</text>
    <text x="330" y="112">the only lever left</text>
    <text x="330" y="148">a fact about concurrency</text>
  </g>
  <text x="330" y="185" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">&mdash; and only here does the rule appear</text>
  <text x="0" y="210" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">read it upward and it is a style guide</text>
  <text x="620" y="210" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">read it downward and it is a force</text>
</svg>
<figcaption>The same sentence sits at the bottom of this figure and inside every clean-code checklist. The difference is the four blocks above it, and whether you can point at them when someone asks why the rule is there.</figcaption>
</figure>

Now the rule has earned its keep.

That is a healthier way to think about strictness in general. I do not want
isolation because isolation is elegant. I do not want declarative builds
because declarative programming is fashionable. I do not want strict types
because strict types are morally superior to loose ones.

I want to know what pressure made the restriction necessary.

Gradle is a particularly good specimen because the pressure accumulated in
public, over years. It started as an extremely programmable build system.
Builds became huge. Performance started hurting. And piece by piece, every
optimization required Gradle to understand more about what the build was
doing.

To understand it, Gradle had to constrain it. To parallelize it, Gradle had to
isolate it.

Gradle is becoming stricter because reality is slowly taking away its options.
In my experience, that is where most of the interesting architecture comes
from.

## References

**Gradle**

1. Gradle 9.7.0 Release Notes — Isolated Projects graduates from experimental
   to incubating; legacy `org.gradle.unsafe.*` property names deprecated.
   https://docs.gradle.org/current/release-notes.html

2. Gradle User Manual, *Isolated Projects* — the feature, its requirements,
   and the Isolated Projects Constraints chapter.
   https://docs.gradle.org/current/userguide/isolated_projects.html

3. Gradle, *How the Gradle Team Adopted Isolated Projects* — the IDE sync
   measurements quoted above.
   https://blog.gradle.org/

**TypeScript**

4. Announcing TypeScript 6.0 — `strict` enabled by default, `esnext` module
   default, floating `target`.
   https://devblogs.microsoft.com/typescript/announcing-typescript-6-0/

5. microsoft/TypeScript issue #62333, *Enable `--strict` by default* — the
   design discussion behind the change.
   https://github.com/microsoft/TypeScript/issues/62333

**The ideas**

6. Fabio Ellena, "Architecture Must Follow Pressure," 2026.
   https://fblln.github.io/articles/architecture-must-follow-pressure/

7. Fabio Ellena, "Risk, Complexity, and Pressure," 2026.
   https://fblln.github.io/articles/risk-complexity-and-pressure/

8. Fabio Ellena, "Who Pays for the Pressure," 2026.
   https://fblln.github.io/articles/who-pays-for-the-pressure/
