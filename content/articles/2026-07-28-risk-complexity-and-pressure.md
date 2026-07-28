+++
title = "Risk, Complexity, and Pressure"
date = "2026-07-28"
description = "Architecture Must Follow Pressure sits between George Fairbanks's risk-driven architecture and John Ousterhout's philosophy of software design."
tags = ["Architecture", "Software Design", "Risk", "Complexity"]
+++

After publishing
[Architecture Must Follow Pressure](/articles/architecture-must-follow-pressure/),
I started wondering whether I had accidentally given a new name to an old idea.
This is always a dangerous question. You spend years watching systems fail,
extract a principle from the wreckage, give it a memorable name, and then
discover that someone described most of it more rigorously fifteen years ago.

In this case, I found two close relatives:

* George Fairbanks's **risk-driven model of software architecture**;
* John Ousterhout's **philosophy of managing software complexity**.

Neither says exactly that architecture must follow pressure. But together they
clarify what I was trying to express. Fairbanks asks where architectural effort
is justified. Ousterhout asks whether a design actually hides complexity.
Pressure adds a third question:

> What force made this mechanism necessary?

## Preparing for the wrong disaster

There is a recurring scene in architecture reviews. Someone introduces an
interface in front of a database repository.

“Why do we need this interface?”

“We may replace the database one day.”

“Is that planned?”

“No.”

“Is there another implementation?”

“No, but there could be.”

The interface is approved. It is architecture, after all. Soon there is an
interface, an implementation, an adapter, a factory, and a configuration class.

Six months later, the application fails because an external service starts
taking eight seconds to respond. There is no aggressive timeout, no concurrency
isolation, no graceful degradation. The database remains exactly where it was.
The hypothetical replacement was beautifully abstracted; the actual failure was
allowed to propagate through the system.

The adapter survived.

The service did not.

This is the gap I was trying to describe with the word **pressure**.
Architecture should not be organised around every event that can be imagined.
It should respond to forces sufficiently real, costly, or dangerous that the
simpler design can no longer tolerate them.

## Fairbanks: risk must pay for architecture

In *Just Enough Software Architecture*, George Fairbanks argues against
applying the same architectural process and level of rigour to every system.
His model is simple: identify and prioritise risks, apply techniques that
reduce them, then check whether they were actually reduced.

> “There is no need for meticulous designs when risks are small, nor any
> excuse for sloppy designs when risks threaten your success.”

The important point is not merely that architects should think about risk. Risk
should determine where architectural effort is spent and how much of it is
justified — because architecture has a cost. Abstractions cost comprehension.
Boundaries cost coordination. Distributed components add latency,
infrastructure, observability, deployment machinery, testing, and new failure
modes. Those costs may be justified, but something must justify them.

This is close to the principle from my original article:

> No architectural pressure, no architectural pattern.

Neither idea is anti-architecture. Both reject architecture by ceremony. The
intervention should follow the threat.

## From pressure to mechanism

Fairbanks uses risk to direct architectural attention. I use pressure to
describe the force that makes a simpler design insufficient — and the two chain
together.

<figure class="diagram">
<svg viewBox="0 0 620 126" role="img" aria-label="A four-stage derivation read left to right: pressure, the force; risk, what it costs; control, the response; mechanism, what you build. A dashed arrow returns from mechanism to pressure, indicating the chain can be read backwards to audit an existing system.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">DERIVATION</text>
  <g font-family="var(--font-mono)" font-size="10">
    <rect x="0" y="26" width="130" height="30" fill="var(--signal)"/>
    <text x="65" y="46" fill="var(--paper)" text-anchor="middle">PRESSURE</text>
    <rect x="163" y="26" width="130" height="30" fill="none" stroke="var(--line)"/>
    <text x="228" y="46" fill="var(--ink)" text-anchor="middle">RISK</text>
    <rect x="326" y="26" width="130" height="30" fill="none" stroke="var(--line)"/>
    <text x="391" y="46" fill="var(--ink)" text-anchor="middle">CONTROL</text>
    <rect x="490" y="26" width="130" height="30" fill="none" stroke="var(--line)"/>
    <text x="555" y="46" fill="var(--ink)" text-anchor="middle">MECHANISM</text>
  </g>
  <g stroke="var(--ink)" stroke-width="1" fill="none">
    <path d="M130 41 L157 41 M151 37 L157 41 L151 45"/>
    <path d="M293 41 L320 41 M314 37 L320 41 L314 45"/>
    <path d="M456 41 L483 41 M477 37 L483 41 L477 45"/>
  </g>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">
    <text x="65" y="72">the force</text>
    <text x="228" y="72">what it costs</text>
    <text x="391" y="72">the response</text>
    <text x="555" y="72">what you build</text>
  </g>
  <g stroke="var(--line)" fill="none">
    <path d="M555 84 L555 100 L65 100 L65 90" stroke-dasharray="3 3"/>
    <path d="M61 96 L65 90 L69 96"/>
  </g>
  <text x="310" y="120" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">read it backwards to audit &middot; what force is this resisting?</text>
</svg>
<figcaption>Forwards, it derives a mechanism. Backwards, it interrogates one — which is the direction most existing systems need.</figcaption>
</figure>

Suppose an external dependency occasionally becomes slow. The slowness is the
pressure. Resource exhaustion, cascading latency, and unavailability are the
risks. Timeouts, concurrency limits, caching, or asynchronous processing are
possible controls. Those controls create architecture, and now the mechanism has
a reason to exist.

A queue does not exist because queues decouple things. It exists because the
producer and consumer must not share availability or throughput. An idempotency
key is not generic defensive programming; it exists because a timeout can make
the result ambiguous, and a retry may duplicate an irreversible operation. A
state machine is not an impressive diagram; it exists because partial progress
must survive restarts and delayed messages.

This is where Fairbanks and pressure are closest. Both demand traceability
between architecture and something that threatens success.

## Risk explains priority; pressure explains shape

I would not use risk and pressure interchangeably. Fairbanks provides a method
for deciding what deserves architectural attention. Pressure is more concerned
with the shape of the response. I want to walk through a system and ask: what
force is this resisting? Why is there a queue here? Why is this component
isolated? Why is this workflow represented as durable state? Why is there a
separate read model?

Risk tells us that something deserves attention. Pressure explains why the
response has this particular form. It may come from production behaviour,
supplier limitations, regulation, latency budgets, security constraints,
organisational boundaries, or incidents that reveal invalid assumptions. These
can all be translated into risks — but pressure keeps attention on the force
itself and on how it deforms the system.

Risk helps us prioritise.

Pressure helps us explain shape.

## Ousterhout: structure must earn its cost

The second close relative comes from John Ousterhout's
*A Philosophy of Software Design*. Ousterhout treats complexity as the central
problem in software design: anything about the structure of a system that makes
it difficult to understand or modify. He identifies two major causes,
**dependencies**, where one part cannot be understood or changed without
considering others, and **obscurity**, where important information is not
obvious. These appear as familiar symptoms — small changes require
modifications in many places, developers must hold too much information in
their heads, it is unclear which code must change.

You receive a harmless-looking ticket:

> Add one field to the response.

Two days later, you have changed three domain models, four transfer objects, a
mapper, a persistence entity, an event schema, and several fixtures. The field
is still a string. The structure has amplified the change.

Ousterhout's answer is not more layers. It is **deep modules**.

## Deep modules and shallow architecture

A deep module exposes a simple interface while hiding substantial complexity.
Ousterhout's formulation is that the best modules are those whose interfaces
are much simpler than their implementations. A filesystem is the classic
example: its interface is small — open, read, write, close — while behind it may
exist caching, allocation, permissions, buffering, concurrency, devices, and
crash recovery. The caller receives significant capability without understanding
most of the machinery. That is depth.

<figure class="diagram">
<svg viewBox="0 0 620 176" role="img" aria-label="Two modules compared by proportion. The deep module has a narrow interface bar above a large implementation block containing caching, allocation, permissions, buffering, concurrency and crash recovery. The shallow module has an interface bar as wide as the diagram above an implementation only one line tall containing a null-and-blank check.">
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="0" y="12">DEEP</text>
    <text x="340" y="12">SHALLOW</text>
  </g>
  <text x="140" y="34" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">interface</text>
  <text x="480" y="34" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">interface</text>
  <rect x="100" y="40" width="80" height="16" fill="var(--signal)"/>
  <rect x="340" y="40" width="280" height="16" fill="var(--signal)"/>
  <rect x="20" y="56" width="240" height="90" fill="none" stroke="var(--line)"/>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">
    <text x="140" y="80">caching &middot; allocation</text>
    <text x="140" y="98">permissions &middot; buffering</text>
    <text x="140" y="116">concurrency &middot; devices</text>
    <text x="140" y="134">crash recovery</text>
  </g>
  <rect x="340" y="56" width="280" height="22" fill="none" stroke="var(--line)"/>
  <text x="480" y="71" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">userName != null &amp;&amp; !userName.isBlank()</text>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">
    <text x="140" y="168">hides far more than it charges</text>
    <text x="480" y="96">charges the same rent</text>
  </g>
</svg>
<figcaption>The ratio is the whole design. An interface is a fixed tax; only the volume beneath it decides whether the tax was worth paying.</figcaption>
</figure>

A shallow module does the opposite. It introduces a new interface, file, type,
and navigation step while hiding almost nothing.

```java
final class UserNameValidator {
    boolean validate(String userName) {
        return userName != null && !userName.isBlank();
    }
}
```

This abstraction has not removed complexity. It has charged an interface tax
for three boolean expressions. Which connects directly to a sentence from my
original article:

> The best modules hide meaningful complexity. The worst abstractions merely
> relocate it.

Every interface introduces complexity. A useful abstraction must hide
substantially more than it creates. It has to earn its rent.

## A boundary must contain something

A shallow architecture is like a house where every room has been divided into
smaller rooms. Each room has exactly one responsibility. There is a room for
sitting. A room for standing. A room containing the handle used to open the
next room. Technically, the separation is impeccable. Unfortunately, reaching
the kitchen requires passing through fourteen doors.

<figure class="diagram">
<svg viewBox="0 0 620 112" role="img" aria-label="A corridor drawn as a long rectangle divided by fourteen vertical door lines, with a dashed path running the full length from the entrance to a kitchen marked at the far right.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">ARCHITECTURAL INTERIOR DESIGN</text>
  <rect x="0" y="28" width="620" height="44" fill="none" stroke="var(--line)"/>
  <g stroke="var(--line)" stroke-width="1">
    <path d="M40 28 L40 72 M80 28 L80 72 M120 28 L120 72 M160 28 L160 72 M200 28 L200 72 M240 28 L240 72 M280 28 L280 72 M320 28 L320 72 M360 28 L360 72 M400 28 L400 72 M440 28 L440 72 M480 28 L480 72 M520 28 L520 72 M560 28 L560 72"/>
  </g>
  <rect x="560" y="28" width="60" height="44" fill="var(--ink)" opacity="0.08"/>
  <text x="590" y="54" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">kitchen</text>
  <g stroke="var(--signal)" stroke-width="1.5" fill="none">
    <path d="M6 50 L548 50" stroke-dasharray="4 4"/>
    <path d="M542 45 L550 50 L542 55"/>
  </g>
  <text x="0" y="94" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">14 boundaries &middot; 14 crossings &middot; 0 contained decisions</text>
  <text x="620" y="94" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">local simplicity, global complexity</text>
</svg>
<figcaption>Each separation is defensible on its own. The cost is only visible when you try to get somewhere.</figcaption>
</figure>

Many codebases suffer from this kind of architectural interior design. Each
class is small. Each layer is pure. The complete behaviour is distributed across
so many units that nobody can see it without opening half the project. Local
simplicity has created global complexity.

A boundary is useful when it contains something:

* a failure domain;
* a security policy;
* supplier-specific behaviour;
* a concurrency model;
* a set of invariants;
* an independently changing capability.

A boundary containing nothing is just a door in the middle of a corridor.

## Complexity can be pressure

This is where Ousterhout and pressure strongly overlap, because complexity can
itself make the simpler design insufficient. When every workflow duplicates the
same retry and concurrency rules, those rules create pressure for a shared
abstraction. When every caller must understand a supplier's strange error model,
that complexity creates pressure for a boundary. When idempotency, timeouts, and
state transitions are repeated across multiple components, the distributed
knowledge creates pressure for a deeper module.

A useful abstraction appears because complexity has accumulated in a form that
can be compressed. This is different from adding an interface because one day
there might be two implementations. The first responds to observed complexity;
the second responds to imagination.

Imagination has an unlimited budget.

Code does not.

## Complexity and resilience are different objectives

Ousterhout's main concern is complexity experienced by developers. Pressure also
includes forces acting on the running system and the organisation around it. A
component may be easy to understand and still require isolation because it fails
frequently. A module may have an elegant interface and still require separation
because two teams deploy independently. A security boundary may increase local
complexity but remain necessary because trust cannot cross it.

Ousterhout asks whether a design reduces complexity. Pressure asks whether it
survives the forces acting upon it. Usually these goals reinforce each other.
Sometimes they conflict: a distributed system is more complex than a monolith,
yet independent scaling, regulatory separation, team ownership, or failure
isolation may justify that cost.

Pressure does not seek the least complex architecture in absolute terms. It
seeks the least complex architecture capable of withstanding the actual forces.

## Fairbanks tells us when; Ousterhout tells us whether

Fairbanks asks whether a risk is significant enough to justify architectural
effort. Ousterhout asks whether the resulting structure reduces complexity or
merely creates more interfaces and dependencies. Pressure identifies the force
that put the question on the table in the first place.

### A worked example: remote vehicle operation

A client sends a command to unlock a door. The request times out and the client
closes the connection. From the client's side that is the entire event.

It is not the entire event. The provider had already accepted the command and
answered `202`, and that answer arrived on a socket nobody was holding. Dispatch,
execution, and the vehicle's acknowledgement all happen afterwards, to an
audience of nobody.

<figure class="diagram">
<svg viewBox="0 0 620 192" role="img" aria-label="A sequence diagram with three lifelines: client, provider and vehicle. The client sends unlock, then times out and closes the connection, at which point its lifeline becomes dashed. The provider's 202 response is drawn stopping short of the closed connection. The provider dispatches anyway, the vehicle unlocks, and the state is reported back. Only a later poll or webhook reaches the client, ending the shaded span labelled the client is in the dark.">
  <g stroke="var(--signal)" fill="none">
    <path d="M182 28 L182 34 L592 34 L592 28"/>
  </g>
  <text x="387" y="22" font-family="var(--font-mono)" font-size="10" fill="var(--ink)" text-anchor="middle">the client is in the dark</text>
  <rect x="182" y="44" width="410" height="128" fill="var(--signal)" opacity="0.1"/>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="0" y="68">CLIENT</text>
    <text x="0" y="116">PROVIDER</text>
    <text x="0" y="164">VEHICLE</text>
  </g>
  <g stroke="var(--line)" fill="none">
    <path d="M90 64 L182 64"/>
    <path d="M182 64 L592 64" stroke-dasharray="3 5"/>
    <path d="M592 64 L610 64"/>
    <path d="M90 112 L610 112"/>
    <path d="M90 160 L610 160"/>
  </g>
  <path d="M96 70 L172 108" stroke="var(--ink)" fill="none"/>
  <polygon points="172,108 162,107 166,101" fill="var(--ink)"/>
  <text x="94" y="58" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">unlock</text>
  <path d="M178 60 L186 68 M186 60 L178 68" stroke="var(--signal)" stroke-width="1.5" fill="none"/>
  <text x="194" y="56" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">timeout &middot; connection closed</text>
  <path d="M196 106 L242 83" stroke="var(--muted)" stroke-dasharray="3 3" fill="none"/>
  <path d="M240 78 L245 88" stroke="var(--muted)" stroke-width="1.5" fill="none"/>
  <text x="250" y="88" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">202</text>
  <path d="M280 118 L356 156" stroke="var(--muted)" fill="none"/>
  <polygon points="356,156 346,155 350,149" fill="var(--muted)"/>
  <rect x="376" y="154" width="50" height="12" fill="var(--signal)"/>
  <text x="401" y="184" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="middle">unlocked</text>
  <path d="M426 154 L502 116" stroke="var(--muted)" fill="none"/>
  <polygon points="502,116 496,123 492,117" fill="var(--muted)"/>
  <path d="M516 106 L592 68" stroke="var(--ink)" fill="none"/>
  <polygon points="592,68 586,75 582,69" fill="var(--ink)"/>
  <text x="586" y="56" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="end">poll / webhook</text>
</svg>
<figcaption>Closing the connection ends the client's knowledge, not the operation. Everything inside the shaded span happens anyway, and only the poll or webhook — a second, separate conversation — ever closes it.</figcaption>
</figure>

**Pressure.** The client's knowledge and the system's state stop being the same
thing the instant a connection ends, and nothing about that looks like a failure
from either side.

**Risk.** Duplicate execution, incorrect user feedback, and divergence between
the command state and the observed vehicle state.

**Control.** A durable operation identifier, idempotency, explicit command
states, and reconciliation.

**Mechanism.** A durable command state machine with idempotent submission, and a
second channel to read the result back.

Fairbanks tells us that the consequences make this worthy of architectural
attention. Ousterhout tells us not to expose every retry rule, supplier state,
and network oddity to every caller — that complexity belongs behind a deep
capability. Pressure explains why that capability exists.

The architecture is no longer selected from a catalogue. It is derived.

## When pressure disappears

A mechanism that once reduced risk can later become the main source of
complexity. A supplier becomes reliable, but the translation layer still
contains six internal models. Two teams merge, but their systems still
communicate through a queue and reconcile eventual consistency. A second
implementation is deleted, but every operation still crosses an interface
designed for interchangeable providers.

<figure class="diagram">
<svg viewBox="0 0 620 146" role="img" aria-label="Two panels. On the left, labelled then, a solid arrow of pressure pushes into a mechanism block. On the right, labelled now, the arrow is gone and only its dashed outline remains, while the mechanism block stands unchanged and unloaded.">
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="0" y="12">THEN</text>
    <text x="340" y="12">NOW</text>
  </g>
  <g stroke="var(--signal)" stroke-width="8" fill="none">
    <path d="M0 62 L108 62"/>
  </g>
  <path d="M104 50 L124 62 L104 74" fill="var(--signal)"/>
  <text x="52" y="42" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="middle">pressure</text>
  <rect x="130" y="38" width="130" height="48" fill="none" stroke="var(--ink)" stroke-width="1.5"/>
  <text x="195" y="66" font-family="var(--font-mono)" font-size="10" fill="var(--ink)" text-anchor="middle">MECHANISM</text>
  <g stroke="var(--line)" stroke-dasharray="3 3" fill="none">
    <path d="M340 62 L448 62 M444 50 L464 62 L444 74"/>
  </g>
  <text x="392" y="42" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">gone</text>
  <rect x="470" y="38" width="130" height="48" fill="none" stroke="var(--ink)" stroke-width="1.5"/>
  <text x="535" y="66" font-family="var(--font-mono)" font-size="10" fill="var(--ink)" text-anchor="middle">MECHANISM</text>
  <text x="195" y="110" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">load justifies structure</text>
  <text x="535" y="110" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">structure justifies itself</text>
  <text x="310" y="138" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="middle">the only thing that changed is the thing nobody was tracking</text>
</svg>
<figcaption>Mechanisms are load-bearing until they are not. Nothing in the code records the difference, which is why the check has to be deliberate.</figcaption>
</figure>

The abstraction may have had a legitimate birth. That does not grant it
immortality. Fairbanks asks whether the original risk still exists. Ousterhout
asks whether the mechanism now costs more understanding than it saves. Pressure
asks:

> Is the force this structure was built to resist still present?

When the answer is no, deleting architecture may be the most architectural
decision available.

## Putting the three ideas together

Fairbanks approaches architecture through engineering risk. Ousterhout
approaches design through complexity. I arrived at pressure through production
systems, failure analysis, and watching useful patterns become harmful when
separated from the conditions that justified them.

The three ideas fit together:

> Use risk to decide what deserves architectural attention.
> Use complexity to judge whether the design is helping.
> Use pressure to explain why the system must have that shape.

The pattern still comes last.

First, find the force.

## References

1. George Fairbanks, *Just Enough Software Architecture: A Risk-Driven
   Approach*. Marshall & Brainerd, 2010.
   https://www.georgefairbanks.com/book/

2. John Ousterhout, *A Philosophy of Software Design*, Second Edition.
   Yaknyam Press, 2021.
   https://web.stanford.edu/~ouster/cgi-bin/book.php

3. Fabio Ellena, “Architecture Must Follow Pressure,” 2026.
   https://fblln.github.io/articles/architecture-must-follow-pressure/
