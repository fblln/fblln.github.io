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

```text
                  PERFORMANCE PRESSURE
                         │
                         ▼
                 parallel configuration
                         │
                         ▼
               requires safe concurrency
                         │
                         ▼
                    isolation
                         │
                         ▼
          no arbitrary cross-project mutation
                         │
                         ▼
              stricter build architecture
```

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

```text
project A configuration ──┐
                          ├── independently executable
project B configuration ──┘
```

A project is supposed to describe itself, not wander through the build
modifying its neighbours.

Every step is the same move:

```text
desired performance optimization
              ↓
new constraint
              ↓
better-defined architecture
```

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

```text
OLD

permissive
   │
   └── optionally become strict


NEW

strict
   │
   └── explicitly ask for exceptions
```

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

```text
flexibility
    ↓
implicit dependencies
    ↓
larger state space
    ↓
harder reasoning
    ↓
fewer safe optimizations
```

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

```text
huge multi-project builds
        ↓
slow configuration and IDE sync
        ↓
need more parallelism
        ↓
parallel configuration
        ↓
shared mutable project state is unsafe
        ↓
projects must become isolated
```

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
