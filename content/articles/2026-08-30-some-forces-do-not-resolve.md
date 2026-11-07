+++
title = "Some Forces Do Not Resolve"
date = "2026-08-30"
description = "Three articles argued that architecture should follow force. The derivation stops working when two forces are both real and point in opposite directions: no mechanism resolves them, and what ships is a position on a dial. Read through Gradle's Isolated Projects, Kafka's acks default, ATAM's tradeoff points, and the timeouts nobody chose."
tags = ["Architecture", "Software Design", "Trade-offs"]
+++

I have written three times that architecture should follow force: that a
[pattern needs a reason](/articles/architecture-must-follow-force/), that the
reason [chains through risk into a
mechanism](/articles/risk-complexity-and-force/), and that [where you put the
response decides who pays for it](/articles/who-pays-for-the-force/).

The chain in the second of those runs force, risk, control, mechanism. It runs
forward cleanly whenever there is one force. Feed it two that point in opposite
directions and it produces two mechanisms that undo each other.

A conversation from a design review, compressed but not by much:

> "Why is this cached for five minutes?"
>
> "The pricing service cannot take the read volume."
>
> "Sales are seeing stale prices."
>
> "Yes."

Neither answer is wrong. The load is real and it is measurable in requests per
second. The staleness is real and it is measurable in the number of support
tickets that begin with "the site said". Nobody in that room has misidentified
anything, and there is no third design that makes both complaints go away.

That is not a force. That is two.

## A force ends in a mechanism; a tension ends in a number

A force makes the straightforward design insufficient, and the mechanism it
justifies makes the insufficiency go away. The dependency was slow, so there is
a timeout, and latency no longer propagates through the request path. Once the
mechanism exists, the force has somewhere to go.

A tension is a pair of forces where every mechanism that relieves one loads the
other. Cache lifetime. Batch size. Replication factor. Timeout budget. Schema
strictness. Retention window. In each case you can build the mechanism, and when
you are finished there is still a value sitting in a configuration file, and
that value is a decision nobody has finished making.

The test is cheap. Build the mechanism, then ask what is left. For a force,
nothing is left. For a tension there is a dial, and somebody is holding it
whether or not they know it.

<figure class="diagram">
<svg viewBox="0 0 620 150" role="img" aria-label="Two panels. On the left, labelled resolved, a single solid arrow of force runs into a mechanism block and stops, with a note reading nothing left pulling. On the right, labelled held, two solid arrows point inward from opposite ends of a track and meet a marker standing between them, with a note reading both still pulling.">
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="0" y="12">RESOLVED</text>
    <text x="340" y="12">HELD</text>
  </g>
  <g stroke="var(--signal)" stroke-width="8" fill="none">
    <path d="M0 66 L98 66"/>
  </g>
  <path d="M94 54 L114 66 L94 78" fill="var(--signal)"/>
  <text x="49" y="44" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="middle">force</text>
  <rect x="122" y="42" width="128" height="48" fill="none" stroke="var(--ink)" stroke-width="1.5"/>
  <text x="186" y="70" font-family="var(--font-mono)" font-size="10" fill="var(--ink)" text-anchor="middle">MECHANISM</text>
  <text x="125" y="112" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">nothing left pulling</text>
  <path d="M340 66 L620 66" stroke="var(--line)" fill="none"/>
  <g stroke="var(--signal)" stroke-width="8" fill="none">
    <path d="M340 66 L398 66"/>
    <path d="M620 66 L562 66"/>
  </g>
  <path d="M394 54 L414 66 L394 78" fill="var(--signal)"/>
  <path d="M566 54 L546 66 L566 78" fill="var(--signal)"/>
  <rect x="477" y="48" width="6" height="36" fill="var(--ink)"/>
  <g font-family="var(--font-mono)" font-size="9" text-anchor="middle">
    <text x="372" y="44" fill="var(--signal)">force</text>
    <text x="480" y="44" fill="var(--ink)">position</text>
    <text x="588" y="44" fill="var(--signal)">force</text>
  </g>
  <text x="480" y="112" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">both still pulling</text>
  <text x="310" y="140" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="middle">one ends in a mechanism &middot; the other ends in a number somebody owns</text>
</svg>
<figcaption>The dial on the right is not a design left unfinished. It is the finished design, and the only question remaining is whose name is on the number.</figcaption>
</figure>

## Three of them are already in this series

I have written about tensions several times without separating them from
everything else.

Gradle 9.7 promotes Isolated Projects to incubating, and I described that as
[performance turning a programmable build system into a declarative
one](/articles/performance-is-making-gradle-less-gradle/). Programmability and
configuration speed are both real, and Gradle did not resolve them. It moved its
position along them, made previously legal build logic illegal, and paid for the
move with a measurement: a 95th percentile falling from nearly seven minutes to
about three. TypeScript 6.0 flipping `strict` to true by default is the same
move made from the opposite end of the same dial.

[Stable Verbs, Dynamic Affordances](/articles/stable-verbs-dynamic-affordances/)
ends on two costs that do not share a solution: transition mistakes when
applicability is implicit, and context cost when the whole vocabulary is exposed
eagerly. I called those tensions at the time, before I had a reason for why they
were a different kind of thing from the rest of the series.

The third is the one I like least, because it is not technical.
[Who Pays for the Force](/articles/who-pays-for-the-force/) reduced it to a
sentence: publishing the mechanism costs one afternoon, and designing an
interface that hides it costs a week and an argument. Both costs are real. The
asymmetry is that they land in different budgets, in different quarters, and
only one of them has a name attached at the moment of the decision.

## The literature got here first, and was more careful

Christopher Alexander's patterns were never the resolution of *a* force. Each
one names several, and the pattern is the balance struck among them rather than
their removal. Pattern-Oriented Software Architecture inherited that directly
and made *Forces* a section of the template, which is a quiet admission that a
pattern normally arrives with more than one. I have been using the singular for
three articles. It was convenient and it was slightly wrong.

At the SEI the vocabulary is sharper. ATAM is built around *sensitivity points*,
where one quality attribute is unusually responsive to a single decision, and
*tradeoff points*, which are sensitivity points that affect more than one
attribute at once. The method's output is not the correct architecture. It is a
list of the places where no answer is correct for everything, each one attached
to the scenario that made it matter.

CAP is the famous degenerate case. It gets cited as though it settled something,
and Brewer spent a 2012 retrospective explaining that the two-of-three
formulation misleads: partitions are rare, the real choice is what the system
does during one, and the interesting engineering is in the recovery afterwards.
A theorem became a slogan, and the slogan won, because a slogan does not require
anybody to pick a number.

## What goes wrong is not picking wrong

The failure mode of an unstated tension is not a bad decision. It is the same
decision, made again, quietly, in every file that has to make it.

Give a hundred services no deadline policy and every one of them picks a
timeout. The effective deadline of the whole system becomes whatever the
shallowest one is, discovered during the first incident where it matters. Give a
platform no freshness policy and every team picks a TTL. What users experience
as consistency is then an emergent property of forty independent guesses, none
of which was written down as a guess.

The tension does get resolved. It gets resolved forty times, by whoever happened
to be editing that file that week, and none of the forty know about the other
thirty-nine.

I wrote that a force never disappears and only ever gets assigned. A tension is
worse than that. It gets assigned repeatedly, and every assignment is invisible.

An unresolved force shows up as an incident.

An unstated tension shows up as drift.

## The record that is actually useful

Not a decision document. Four facts:

1. **Both forces, named separately, each with the measurement that makes it
   real.** Requests per second on one side, stale-price tickets on the other. If
   only one side has a number, this is not a tension. It is a preference with a
   rationalisation attached.
2. **The current position**, as a value or a rule. Five minutes. Two retries.
   `strict: true`.
3. **Who owns it.** A dial with no owner is a dial that moves whenever somebody
   is annoyed enough to move it.
4. **The observation that would move it.** A threshold, a ratio, a class of
   incident. Something that can be true later without anybody arguing about
   whether it is true.

The fourth is the one that gets skipped, and it is the only one that makes the
record survive contact with next year. It is also the same audit as [when the
force disappears](/articles/risk-complexity-and-force/), turned sideways. There,
the question is whether the force that justified a mechanism still exists. Here,
the question is whether the balance still points the same way.

## A worked example: the acknowledgement you are willing to wait for

A Kafka producer sends a record. It can be told the write succeeded as soon as
the partition leader has it, or only once every in-sync replica has it.

**Force.** Waiting for the full in-sync set adds a replication round trip to
every write, and the throughput ceiling drops with it.

**Opposing force.** Not waiting means a write can be acknowledged and then lost,
because the leader can fail after replying and before any follower has the
record. The producer is told "yes" about something that did not happen.

**Position.** `acks`, read together with the topic's `min.insync.replicas`.
Neither setting means anything alone: `acks=all` against a topic with
`min.insync.replicas=1` is a durability guarantee that guarantees one copy.

**Owner.** Whoever owns the topic's durability contract, which in practice is
nobody until the first loss.

**Trip-wire.** Kafka answered this itself. Through the 2.x line the producer
default was `acks=1`, chosen when the typical user was assembling a pipeline and
measuring throughput. KIP-679 flipped the 3.0 default to `acks=all` with
idempotence enabled. The two forces did not change. What changed was the
population: most people configuring a producer had stopped being pipeline
builders and had started being people who assumed durability was the default,
and were wrong about it in a way that only showed up during a broker failure.

Three ecosystems, three default flips, one shape. Gradle made legal code
illegal. TypeScript made loose types strict. Kafka made a fast acknowledgement
a slow one. In none of the three did anybody discover a design that removed the
opposing force, and in all three the move was paid for with a measurement.

## Not everything is a tension

"Everything is a trade-off" is what people say when they do not want to choose.
It is usually deployed to end a conversation, and it works, which is why it
survives.

A real tension has two forces you can measure, and moving toward one measurably
costs the other. A fake one has a force on one side and a taste on the other.
The interface in front of the database repository from *Risk, Complexity, and
Force* is the fake kind. "We may replace the database one day" cannot be
measured, had never been measured, and was still not measured in the six months
before the service fell over for a completely unrelated reason. There was one
force in that room. It was pointed at the timeout nobody had written.

Half of what looks like a tension in a design review is a preference standing up
straight. The measurement is what tells them apart, and asking for it costs one
sentence.

## Force, then the pair it belongs to

Force explains why a mechanism exists. Tension explains why you cannot have all
of them at once, and why the last part of a design is a set of numbers rather
than a diagram.

The mechanism is the part you can point at during a review. The number is the
part that decides what the system actually does, and it is almost always written
by whoever arrived at that file first.

First, find the force.

Then find out whether it has an opposite.

## References

**The ideas**

1. Christopher Alexander, Sara Ishikawa, and Murray Silverstein, *A Pattern
   Language: Towns, Buildings, Construction*. Oxford University Press, 1977.
   Each pattern states the forces it balances rather than a force it removes.

2. Frank Buschmann, Regine Meunier, Hans Rohnert, Peter Sommerlad, and Michael
   Stal, *Pattern-Oriented Software Architecture, Volume 1: A System of
   Patterns*. Wiley, 1996. The pattern template carries an explicit *Forces*
   section.

3. Rick Kazman, Mark Klein, and Paul Clements, *ATAM: Method for Architecture
   Evaluation*. Software Engineering Institute, CMU/SEI-2000-TR-004, 2000. The
   source of the sensitivity point and tradeoff point distinction.
   https://insights.sei.cmu.edu/library/atam-method-for-architecture-evaluation/

4. Len Bass, Paul Clements, and Rick Kazman, *Software Architecture in
   Practice*, Fourth Edition. Addison-Wesley, 2021.

5. Eric Brewer, "CAP Twelve Years Later: How the 'Rules' Have Changed," *IEEE
   Computer* 45(2), 2012.
   https://www.infoq.com/articles/cap-twelve-years-later-how-the-rules-have-changed/

6. George Fairbanks, *Just Enough Software Architecture: A Risk-Driven
   Approach*. Marshall & Brainerd, 2010.

7. John Ousterhout, *A Philosophy of Software Design*, Second Edition. Yaknyam
   Press, 2021.
   https://web.stanford.edu/~ouster/cgi-bin/book.php

**The systems**

8. KIP-679, *Producer will enable the strongest delivery guarantee by default* —
   the change that made `acks=all` and idempotence the Kafka 3.0 producer
   defaults.
   https://cwiki.apache.org/confluence/display/KAFKA/KIP-679%3A+Producer+will+enable+the+strongest+delivery+guarantee+by+default

9. Apache Kafka producer configuration — `acks`, `enable.idempotence`, and the
   interaction with `min.insync.replicas`.
   https://kafka.apache.org/documentation/#producerconfigs

10. Gradle, Isolated Projects.
    https://docs.gradle.org/current/userguide/isolated_projects.html

11. TypeScript, `strict`.
    https://www.typescriptlang.org/tsconfig/#strict

**Earlier articles in this series**

12. Fabio Ellena, "Architecture Must Follow Force," 2026.
    https://fblln.github.io/articles/architecture-must-follow-force/

13. Fabio Ellena, "Risk, Complexity, and Force," 2026.
    https://fblln.github.io/articles/risk-complexity-and-force/

14. Fabio Ellena, "Who Pays for the Force," 2026.
    https://fblln.github.io/articles/who-pays-for-the-force/

15. Fabio Ellena, "Performance Is Making Gradle Less Gradle," 2026.
    https://fblln.github.io/articles/performance-is-making-gradle-less-gradle/

16. Fabio Ellena, "Stable Verbs, Dynamic Affordances," 2026.
    https://fblln.github.io/articles/stable-verbs-dynamic-affordances/
