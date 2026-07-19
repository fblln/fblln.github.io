+++
title = "Architecture Must Follow Pressure"
date = "2026-07-19"
description = "Not fashion, not diagrams, not whatever book the team read last quarter. Architecture should emerge as a response to real constraints — and the sharpest of those constraints is failure. A failure-mode-driven view of software design."
tags = ["Architecture", "Systems", "Reliability"]
+++

I recently wrote down a sentence that captures a lot of how I think about
software:

> Architecture must follow pressure.

It is not a famous quote — at least, not that I know of. It is simply the
shortest way I have found to express something I increasingly believe:
architecture should emerge as a response to real constraints. Not because a
framework recommends it. Not because a conference talk made it look elegant.
Not because every application apparently needs controllers, use cases, ports,
adapters, repositories, factories, domain services, application services, and
twelve interfaces before it is allowed to save a row in a database.

Architecture needs a reason.

## I am not against architecture

I am not a fanatic of hexagonal architecture, but I am not against it either.
The same applies to clean architecture, domain-driven design, CQRS, event
sourcing, functional programming, microservices, or almost any other serious
engineering approach. These tools exist because they solve real problems.

The trouble starts when the solution is applied before the problem exists.

A team sees a potentially replaceable database and creates an abstraction. Then
an abstraction for the abstraction. Then separate domain, application,
infrastructure, and delivery layers. Soon, changing a field requires touching
nine files. The architecture is technically clean, but the codebase has become
harder to understand than the problem it solves.

That is not simplicity. That is complexity wearing formal clothes.

## What creates architectural pressure?

Pressure is any force that makes the straightforward design insufficient. For
example:

- a dependency fails frequently;
- business rules change independently from infrastructure;
- two teams need to deploy separately;
- data must survive retries and partial failure;
- a component has strict latency or memory constraints;
- a security boundary must be enforced;
- multiple implementations genuinely exist;
- deterministic replay is required;
- a production incident exposes an invalid assumption.

These are real pressures. They justify boundaries. They justify isolation.
Sometimes they justify a port and an adapter. Sometimes a separate service.
Sometimes a queue, an event log, a cache, a state machine, or a carefully
designed type. But the pattern comes *after* the pressure. The pressure is what
explains why the pattern deserves to exist.

## "Best practice" is not pressure

"Best practice" is often used as a substitute for reasoning. It lets us skip
the uncomfortable part where we explain what can actually go wrong.

> Why do we need this interface? — Because it is best practice.
> Why four layers? — Separation of concerns.
> Which concerns are currently mixed? — None yet, but they might be one day.
> Why not introduce the separation when that day arrives? — Because then it
> might be harder.

This is how speculative complexity enters a codebase. Every individual decision
appears harmless: one interface, one wrapper, one additional layer, one generic
mechanism designed for a future use case. Eventually, developers spend more time
navigating architectural machinery than working on business behavior. The
architecture has stopped reducing complexity. It has become one of its primary
sources.

## Simplicity is not the absence of structure

There is an important distinction between simple code and unstructured code. I
care deeply about structure. Clear control flow, cohesive modules, deep
abstractions, separating calculations from effects, making state transitions
explicit, testing invariants, thinking about failures, retries, idempotency,
concurrency, and observability — all of it matters.

But structure should help a reader understand the system. It should not require
the reader to reconstruct a ceremony. The best modules often expose a very
small interface while hiding significant complexity behind it. The worst
abstractions expose almost no value while forcing every caller through another
file, another type, and another naming convention. One hides complexity. The
other relocates it.

## Begin with failure

The most useful architectural question I know is not *"which pattern belongs
here?"* It is *"what can go wrong, and what happens when it does?"* Begin with
failure:

1. What can fail?
2. What is the effect?
3. How severe is it?
4. What causes it?
5. How likely is it?
6. Can we detect it before it becomes dangerous?
7. Which control actually reduces the risk?

That is almost the opposite of pattern-first architecture. Instead of asking
*"should we use hexagonal architecture?"* you ask *"what failures must this
system survive, and where do we need controls?"* Then the architecture follows
naturally:

- retries appear because transient failures matter;
- idempotency appears because retries can duplicate effects;
- isolation appears because one dependency must not collapse the whole request;
- caching appears because a critical dependency cannot meet the latency target;
- observability appears because undetected failure is itself a risk;
- replay appears because some failed operations must be recoverable;
- stronger boundaries appear where failure propagation becomes dangerous.

If an external dependency is slow, the problem is not that we are missing a port
and an adapter. The problem is that latency can propagate through the request
path, consume resources, trigger retries, and eventually amplify into an outage.
*Now* we have pressure. That pressure may justify a timeout, a bulkhead, a
cache, asynchronous processing, or graceful degradation — each control mapped to
a real failure mode.

This is also why architecture without a credible failure model often feels
artificial to me. A pattern added without a failure mode is like a control with
no identified risk. It may look rigorous, but it is not connected to anything.

## Production creates the strongest pressure

Architecture discussions often focus on static structure: which layer owns this
class, where an interface should live, whether the repository belongs to the
domain or the infrastructure package. Production asks different questions.

What happens when the dependency takes eight seconds to reply? What happens when
it returns success *after* the caller has timed out? Can this operation be
retried safely? What happens if only half of the writes succeed? How much memory
does this design consume under load? Can we replay the operation? Can we see why
it failed? Can the system degrade without lying to the caller?

These questions create architecture more reliably than diagrams do. A slow
external service may justify a cache. A cache may justify invalidation events.
Invalidation events may justify idempotent consumers. Idempotent consumers may
justify persistent offsets or deduplication keys. Now the architecture has a
causal chain — every mechanism exists because the previous pressure demanded it.
That architecture is easier to defend, test, and evolve.

## Let the code earn its abstractions

I increasingly prefer a simple rule:

> No architectural pressure, no architectural pattern.

This does not mean waiting until a codebase becomes a disaster. It means
identifying the actual forces early and designing for *those*. We should still
think ahead — but thinking ahead is not the same as implementing every possible
future. A good design leaves room for change through clear responsibilities,
strong invariants, and small public APIs. It does not need placeholder
abstractions for imaginary requirements.

An abstraction should earn its existence by doing at least one useful thing:
hiding meaningful complexity, protecting an invariant, isolating a real
boundary, supporting actual variation, improving testability at an external
edge, or reducing cognitive load. If it does none of these, it is probably just
indirection.

## The architecture should tell a story

When I inspect a system, I want to understand why it has its current shape.

> This service exists because the workload needs independent scaling. This queue
> exists because the producer and consumer cannot fail together. This state
> machine exists because the workflow has recoverable intermediate states. This
> adapter exists because the external API is unstable and contaminating the
> domain would be expensive. This cache exists because the source system cannot
> meet the latency requirement.

That is a coherent story. Compare it with:

> This interface exists because repositories should always have interfaces. This
> layer exists because application logic belongs in the application layer. This
> factory exists because constructors are too concrete. This mapper exists
> because domain objects must never touch transport objects, even though they
> contain the same three fields.

That is not a story. It is a template.

## Follow pressure, not fashion

Software engineering has a recurring tendency to turn good ideas into universal
rules. A pattern solves a difficult problem. The industry notices. The pattern
becomes fashionable. Then it is applied to systems that do not have the original
problem. Eventually people become frustrated with the complexity and declare the
pattern useless. The pattern was not useless — it was simply separated from the
pressure that justified it.

So perhaps the better question is not *"which architecture should we use?"* but
*"what pressure does this system need to survive?"* Once that is clear — and the
pressure is usually failure: its probability, its impact, its propagation, and
our ability to detect and control it — the architecture becomes much less
mysterious. And usually much smaller.
