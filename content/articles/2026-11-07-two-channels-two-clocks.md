+++
title = "Two Channels, Two Clocks"
date = "2026-11-07"
description = "A command acknowledgement does not prove a charger started, and comparing a device timestamp against your own cannot establish it either. What a desired-state reconciler decides, why every fix turned out to be an identity rather than a timestamp, and why the numbers that judge a command belong to the vendor rather than the fleet."
tags = ["Architecture", "Protocol", "Reliability", "Trade-offs"]
+++

You ask a charger to start. Ninety seconds later the telemetry still says
plugged in and stopped. One of three things is true: the command is failing and
nothing will say so, the command worked and the reading predates it, or the
answer is not knowable yet.

Each wrong guess costs something different. Guess failing and the loop re-sends
into a rate-limited API that was already retrying on its behalf. Guess worked
and somebody is told their car is charging when it is not. Refuse to guess at
all and the car silently does not charge.

I have spent a while building a desired-state reconciler for this, and the
answer is not what I want to write down. I got the same thing wrong three times
in three different places, and each time it looked like a timing problem and
turned out to be an identity problem. This is that sequence, in the order I hit
it.

## What sits between a button and a charger

The devices are reached through an aggregator: one API covering equipment from
several manufacturers. Chargers, home batteries, inverters and heat pumps share
an application without sharing a connection, a protocol or an operator.

For a charger reached through its manufacturer's cloud, the path runs in two
directions that are not each other's reverse:

```text
COMMAND
user or schedule → reconciler → aggregator → manufacturer cloud → charger

STATE REPORT
charger → manufacturer cloud → aggregator → reconciler → user interface
```

The command channel carries an instruction and returns an acknowledgement,
stamped by the sender. The telemetry channel carries state on the device's own
schedule, stamped by the device. They have independent latency, independent
failure and independent ordering, and an aggregator unifies access to them
without unifying their behaviour.

Almost every bug in this domain comes from treating those as one channel.

An outside observer with a single clock sees a sequence the application never
sees in that order:

| Time | Event |
| --- | --- |
| 12:00:00 | The charger records "stopped"; the report waits in a queue. |
| 12:00:05 | The application requests charging. |
| 12:00:06 | The platform acknowledges the request. |
| 12:00:10 | The charger starts charging. |
| 12:00:20 | The old "stopped" report arrives. |

<figure class="diagram">
<svg viewBox="0 0 620 170" role="img" aria-label="Two horizontal time axes. The upper axis is the application's clock and carries the request at 12:00:05, its acknowledgement at 12:00:06, and the arrival of a telemetry report at 12:00:20. The lower axis is the charger's clock and carries a capture of the stopped state at 12:00:00 and the start of charging at 12:00:10. A long arrow runs from the early capture on the device axis up to the late arrival on the application axis, showing that the reading which arrives last was taken first.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">ONE SEQUENCE &middot; TWO CHANNELS &middot; TWO CLOCKS</text>
  <text x="0" y="32" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">OUR CLOCK</text>
  <line x1="0" y1="44" x2="620" y2="44" stroke="var(--ink)"/>
  <g stroke="var(--ink)">
    <path d="M170 38 L170 50 M205 38 L205 50 M500 38 L500 50"/>
  </g>
  <text x="160" y="64" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">12:00:05 request &middot; :06 acknowledged</text>
  <text x="620" y="64" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="end">12:00:20 the stopped report arrives</text>
  <path d="M74 112 C 210 110, 380 80, 495 52" fill="none" stroke="var(--signal)"/>
  <path d="M499 49 l-10 1 l5 7 z" fill="var(--signal)"/>
  <text x="290" y="84" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="middle">captured first &middot; delivered last</text>
  <text x="0" y="110" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">12:00:00 captures &ldquo;stopped&rdquo;</text>
  <text x="330" y="110" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">12:00:10 starts charging</text>
  <line x1="0" y1="122" x2="620" y2="122" stroke="var(--ink)"/>
  <g stroke="var(--ink)">
    <path d="M70 116 L70 128 M330 116 L330 128"/>
  </g>
  <text x="0" y="146" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">DEVICE CLOCK</text>
  <text x="620" y="146" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="end">nothing on this axis is stamped by us</text>
</svg>
<figcaption>The charger starts charging at 12:00:10 and never says so. What arrives at 12:00:20 is the reading from before the command.</figcaption>
</figure>

At 12:00:20 the application learns something new about an old state. Treating
"just received" as "just measured" would re-send a command that had already
worked. A receipt timestamp says when a server got a message; it cannot say
when the charger captured the reading.

## The ack is not the actuation

Before the clocks, the simpler trap. An *acknowledgement* is a reply confirming
some stage of receipt. *Actuation* is the physical act: the relay closing, the
current flowing. Which stage an ack confirms depends entirely on the API, and
OCPP 1.6 is explicit about it.

OCPP is the Open Charge Point Protocol, used between charging stations and
their management systems. `RemoteStartTransaction.req` carries two fields: a
mandatory `idTag` and an optional `connectorId`. The charge point answers with
a `RemoteStartStopStatus`, and the specification glosses `Accepted` as "Command
will be executed." That is a promise about the command. It says nothing about
the connector.

Three questions arrive as three messages at three times. Did the instruction
reach the charger: `RemoteStartTransaction.conf`, immediately. Did a
transaction begin: a later `StartTransaction.req`, where a `transactionId` is
finally assigned. What is the connector doing: `StatusNotification`, on its own
schedule. Those can diverge. There are public reports of chargers answering `Accepted`
and then sending `StatusNotification` with `SuspendedEV` (a status for an EV
declining energy inside a transaction already running) with no
`StartTransaction` at all: a state that presupposes the thing never
confirmed.

Then the part that matters for everything below. A `StartTransaction.req` does
eventually arrive, and nothing in it says which remote start caused it. The
request carried an `idTag`, and an idTag is a credential rather than a request
identifier: the same token can be presented at the plug by a driver who walked
up while the command was in flight. You hold an acknowledgement and a
transaction with no way to prove they belong to each other.

OCPP 2.0.1 closes exactly this gap. `RequestStartTransaction` carries a
`remoteStartId`, the charger echoes it back in the `TransactionEvent` that
reports the session, and `TransactionEvent` replaces `StartTransaction`,
`StopTransaction` and the in-transaction meter values with one stream.

The protocol grew a correlation id, because no acknowledgement could be made to
carry the answer. That is the first time this pattern shows up, and I did not
recognise it as a pattern until the third.

I have written this failure before, one layer down. The Kafka producer in [Some
Forces Do Not Resolve](/articles/some-forces-do-not-resolve/) can be told a
write succeeded as soon as the partition leader has it, and the leader can fail
before any follower does. The producer is told yes about something that did not
happen. So is the operator who pressed Start.

## Why the clocks disagree

If the acknowledgement cannot confirm the actuation, the telemetry has to. That
means ordering a reading against a command, which means comparing a device
timestamp against the server's.

The application server and the charger each keep time locally, and writing both
timestamps in UTC gives them the same notation without synchronising the clocks
that produced them. A hardware clock counts ticks from an oscillator whose rate
is not perfect, so its error accumulates between corrections, and a device can
start with the wrong time after a reboot. Network time services correct offset
and rate, as [RFC 5905](https://www.rfc-editor.org/rfc/rfc5905) describes, and
a device that cannot reach its time source runs without fresh corrections.

Clock error and delivery delay are separate problems that look alike in the
data. A perfectly timed reading can sit in a queue for thirty seconds; an
immediately delivered reading can carry a timestamp thirty seconds ahead.

Say the device clock is two minutes fast. A reading taken at 12:00 is stamped
12:02, a command goes out at 12:01, and a plain comparison calls the reading
newer than the command though it was taken a minute earlier. The error does not
have to be large. It only has to exceed the gap between the two events.

## I compared two clocks and called it evidence

The rule at the centre of my first implementation was one line, and I was
pleased with it:

```ts
function isEvidenceFor(obs: Observation, state: Phase): boolean {
  return obs.lastSeen > state.since;
}
```

That comparison does real work. It is what stops the duplicate-command storm,
where a loop sees "stopped", concludes its start failed, sends another, sees
"stopped" again and hammers an API that was already retrying.

It is also nonsense. `state.since` is when I issued the command, by my clock.
`obs.lastSeen` is when the device recorded its state, by the device's clock.
The expression compiles, runs, passes every test that existed, and orders two
events that have never been in the same frame of reference. A device running
ten minutes fast makes every reading look like evidence; a device running ten
minutes slow disqualifies all of them.

Branding the two timestamps as different types stops the compiler accepting
that line, which is a real improvement and supplies no missing evidence.

What it does supply is a defensible answer to a narrower question. Given a bound
on how far a device clock may have drifted and a bound on how long a reading
may sit in transit, converting a capture time into the server's frame yields an
interval, not a point, and the interval has three possible relationships with
the instant the command went out.

<figure class="diagram">
<svg viewBox="0 0 620 196" role="img" aria-label="A vertical line marks the instant the command was sent. Three horizontal intervals represent one telemetry capture converted into the application's clock frame under different clock estimates. The first interval lies entirely to the right of the command line and is labelled after. The second lies entirely to the left and is labelled before. The third straddles the command line and is labelled cannot tell.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">ONE CAPTURE, CONVERTED TO OUR CLOCK &middot; THREE HONEST ANSWERS</text>
  <text x="0" y="28" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the offset is bounded, not known, so the converted capture is an interval and not a point</text>
  <line x1="300" y1="42" x2="300" y2="166" stroke="var(--signal)"/>
  <text x="300" y="186" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="middle">the instant we sent the command</text>
  <g stroke="var(--ink)">
    <path d="M335 60 L480 60 M335 54 L335 66 M480 54 L480 66"/>
    <path d="M120 102 L262 102 M120 96 L120 108 M262 96 L262 108"/>
    <path d="M222 144 L392 144 M222 138 L222 150 M392 138 L392 150"/>
  </g>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="335" y="78">every instant in it is later than the command</text>
    <text x="120" y="120">every instant in it is earlier</text>
    <text x="222" y="162">the interval contains the command</text>
  </g>
  <g font-family="var(--font-mono)" font-size="9" text-anchor="end">
    <text x="620" y="63" fill="var(--ink)">after</text>
    <text x="620" y="105" fill="var(--ink)">before</text>
    <text x="620" y="147" fill="var(--signal)">cannot tell</text>
  </g>
</svg>
<figcaption>Two of these are evidence. The third is the one most implementations do not have a value for, so they round it to whichever of the other two is cheaper to code.</figcaption>
</figure>

Two of those are evidence. The third is the one most implementations have no
value for, so they round it to whichever neighbour is cheaper to code, and that
is where this class of bug lives.

## What a reconciler actually is

The loop that consumes those answers deserves a proper description, because
everything after this depends on its shape.

A *desired-state reconciler* holds two pictures of each device. **Desired
state** is what somebody wants: this charger should be charging. **Reported
state** is the most recent account the device has sent, which is always a
statement about the past. The loop compares them, acts when they differ, and
then has to decide what the next reading means.

The important property is that it is *level-triggered* rather than
*edge-triggered*. An edge-triggered system reacts to events: the user clicked,
so send a command, and the job is done when the send returns. A level-triggered
system reacts to a condition: the device does not match intent, so keep working
until it does, no matter how many wakes that takes. Edge-triggered logic loses
a command the moment a message is dropped, and nothing is left to notice.

Level-triggered is also why this is a loop and not a call. A charger can stop
on its own, a human can press the button on its front panel, a vendor cloud can
drop a command without telling anyone. None of those produces an event. All of
them produce a disagreement visible only on the next snapshot, so the only way
to notice is to keep looking.

*Convergence* is the loop reaching a state where reported matches desired and
it has evidence for saying so. *Drift* is that state coming apart afterwards
without any command from us.

The hard part is not deciding to act. It is deciding what a reading means while
an action is outstanding, and that needs the loop to remember what it is
waiting for.

## Five phases, and most of them exist to stop you acting

So the loop keeps an explicit phase per device rather than a pair of booleans.
A boolean can say "a command is outstanding"; it cannot say which command, what
it targeted, when its deadline expires, or which snapshot has already been
accounted for.

<figure class="diagram">
<svg viewBox="0 0 620 215" role="img" aria-label="A state diagram with five boxes. IDLE moves to ISSUED when the device differs from intent. ISSUED moves to CONVERGED when confirmed, and down to FAILED on a deadline or a rejection. FAILED returns to ISSUED on a new intent or an explicit retry. A curve returns from CONVERGED to ISSUED on drift or a third-party change. IDLE drops to SUSPENDED when telemetry goes dark.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">ONE DEVICE &middot; FIVE PHASES</text>
  <g fill="none" stroke="var(--ink)">
    <rect x="20" y="60" width="110" height="30"/>
    <rect x="250" y="60" width="110" height="30"/>
    <rect x="480" y="60" width="110" height="30"/>
    <rect x="250" y="160" width="110" height="30"/>
    <rect x="20" y="160" width="110" height="30"/>
  </g>
  <g font-family="var(--font-mono)" font-size="9" font-weight="700" fill="var(--ink)" text-anchor="middle">
    <text x="75" y="79">IDLE</text>
    <text x="305" y="79">ISSUED</text>
    <text x="535" y="79">CONVERGED</text>
    <text x="305" y="179">FAILED</text>
    <text x="75" y="179">SUSPENDED</text>
  </g>
  <g fill="none" stroke="var(--ink)">
    <path d="M130 75 L244 75"/>
    <path d="M360 75 L474 75"/>
    <path d="M285 90 L285 154"/>
    <path d="M330 160 L330 96"/>
    <path d="M75 90 L75 154"/>
  </g>
  <path d="M535 60 C 535 32, 305 32, 305 60" fill="none" stroke="var(--signal)"/>
  <g fill="var(--ink)">
    <path d="M250 75 l-6 -3 l0 6 z"/>
    <path d="M480 75 l-6 -3 l0 6 z"/>
    <path d="M285 160 l-3 -6 l6 0 z"/>
    <path d="M330 90 l-3 6 l6 0 z"/>
    <path d="M75 160 l-3 -6 l6 0 z"/>
  </g>
  <path d="M305 60 l-3 -6 l6 0 z" fill="var(--signal)"/>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="187" y="68" text-anchor="middle">differs from intent</text>
    <text x="417" y="68" text-anchor="middle">confirmed</text>
    <text x="277" y="128" text-anchor="end">deadline, or rejected</text>
    <text x="338" y="128">new intent, or explicit retry</text>
    <text x="420" y="28" text-anchor="middle" fill="var(--signal)">drift, or someone else moved it</text>
  </g>
  <text x="0" y="208" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">SUSPENDED is entered from a resting phase only &middot; an in-flight command keeps its deadline and is never abandoned</text>
</svg>
<figcaption>The arrow that is missing is the one from ISSUED to SUSPENDED. Dropping an outstanding command because the feed went quiet is how a loop sends the same instruction twice.</figcaption>
</figure>

**IDLE** and **CONVERGED** both mean the device agrees with intent, and they
are not interchangeable. CONVERGED remembers what it converged to and which
generation did it, so a later disagreement is drift and can be named as such.
IDLE remembers nothing and treats any reading as informative, which is correct
at a cold start and wrong everywhere after it.

**ISSUED** is the only phase carrying a deadline, and the only one that can be
confirmed. It holds the command id and the generation, because a report belongs
to it only if the id matches, and a snapshot is evidence only if the generation
has moved.

**SUSPENDED** is what a dead feed looks like from inside the loop. Without it
the last snapshot stays current forever and the reconciler issues commands
against a frozen reading. It carries a *watermark* into the outage: the device
time of the newest capture already accounted for. That comparison is the one
place in the system where both sides come from the same clock, so asking "is
there anything new here" is exact, and the lossy cross-clock estimate is needed
only for the single question of whether a capture post-dates a command.

**FAILED** is terminal, deliberately. If neither channel could confirm inside a
generous window, the honest move is to surface that and stop rather than keep
poking a device that may be refusing for a reason.

Three rules govern the transitions. Evidence must post-date the action, because
a reading taken before the command says nothing about it. The command channel
outranks telemetry when it has something real to say, because `REJECTED` and
`APPLIED` are first-hand and need no correlation at all. And correlating
telemetry with an action crosses two clocks, so it has three answers rather
than two.

A reconciler that cannot say "I do not know yet" will eventually tell someone
their car is charging when it is not.

The whole decision is one pure function, with no I/O, no clock of its own and
no randomness: it takes the intent, the observation, the latest command report,
the clock model, the current phase and the time, and returns the next phase, a
command or nothing, and a sentence saying why. The clock model is threaded in
rather than read, which is why every pathology in this article is reproducible
in a unit test instead of requiring a car and a patient afternoon. The sentence
is not decoration either: the same string goes to the audit log and to the
user-facing fault text, so the two cannot disagree about why a device is where
it is.

## Ordering state updates with a counter

The missing piece is the one OCPP 2.0.1 added. Assign a *generation* to each
desired-state update and carry it through to the device's report. It is the
revision number of the requested state: 41, then 42, then 43.

```text
platform → device: desired generation 42, charging = true
device → platform: accepted generation 42
device → platform: applied generation 42, charging = true
```

The second message says the request arrived. The third says the device applied
that particular update and reached the requested state. A delayed report for
generation 41 cannot confirm 42, even if it says "charging", and no clock
participates in that decision.

The counter is only as good as the contract around it. One authority allocates
generations per device and persists them across restarts, a retry of the same
update keeps its generation, and the device must associate its result with the
generation it actually processed. Copying the latest number onto an unrelated
reading recreates the original bug with an integer in place of a timestamp.

The counter also answers a question I had not thought to ask it. A fresh
reading that disagrees with intent while reporting the *same* generation the
loop last converged at is positive evidence that something other than the
reconciler moved the device: a command advances the counter and a human at the
front panel does not. That is exact, same-channel and clock-free, and it
separates "our command finally landed" from "somebody else turned it off",
which state alone cannot do.

What to do about it is a policy the decision function deliberately does not
hold. Fighting a human is right for a charger on a tariff and wrong for someone
who has just turned the heating up.

A property test caught the subtler half of the contract. Converging from the initial
state, where the device already matched intent, stamped the phase with a
generation that had never been issued to anything, so the next genuine command
appeared not to advance it. The absence of a generation belonged in the type
rather than papered over with a zero.

The other finding was in the contract rather than the code. "Claiming
convergence requires evidence" had been written as a check on the resulting
phase, so a loop sitting quietly in an already-converged state tripped it. The
check belongs on the transition that makes the claim, not on staying put.

A matching generation is still not success on its own. The report has to say
whether that update is pending, applied or rejected, and what state resulted,
which makes the rule for one report five lines:

```text
report generation < target generation  → stale for this intent
report generation > target generation  → refresh the target; this was superseded
report generation = target generation:
    applied, state matches target      → confirmed
    rejected                           → failed
    otherwise                          → keep waiting until the deadline
```

Kubernetes arrived here first. `metadata.generation` identifies a desired-state
revision and `status.observedGeneration` identifies the revision a controller
has observed, while the status conditions still say whether the work succeeded.
An observed generation tells you which revision a condition describes; it does
not turn an unsuccessful condition into a successful one.

I did look before writing one. [Eclipse
Ditto](https://eclipse.dev/ditto/basic-feature.html) persists both states and
then says the reconciling logic "is up to your application or device firmware".
[Azure IoT Plug and
Play](https://learn.microsoft.com/azure/iot-pnp/concepts-convention) specifies
the counter properly, with the hub minting `$version` and the device echoing it
as `av`. Z-Wave's [Supervision command
class](https://docs.silabs.com/z-wave/7.21.3/zwave-api/supervision) answers a
SET with whether the command was understood *and executed*. Each covers a part;
none closes the loop.

## Three identities, because one was never enough

With a generation in hand I thought the identity question was settled. It was
settled for exactly one of the three things that need identifying.

Start with delivery. Dispatch to a vendor API is **at-least-once**: the request
may be sent twice by a retry, by a second worker, or by a process that died
after the HTTP call and before recording success. Exactly-once delivery is not
available to buy, so the usual move is to make the duplicate harmless instead.
Every command carries an id the platform treats as an idempotency key, and a
repeat under the same key returns the original response rather than actuating a
second time. That guarantee belongs to the platform rather than to this code,
and it is the first thing to check when choosing one.

The id is derived from the device and a durable counter rather than drawn at
random, and that distinction matters more than it looks: a reconcile
transaction that gets retried re-derives the *same* id. A random id would mint
a new one, insert a second outbox row, and send a second command for the same
intent, which is the duplicate actuation the design exists to prevent.

Now the user. Someone presses Start, sees no change, and presses again. The
reconciler is still waiting on one device command, but the API treats those
clicks as two separate instructions, and the second one refreshes the
instruction's timestamp and clears any local override even if no second device
command is ever sent. The device is fine. The user's intent has been quietly
rewritten.

So each user action carries its own `Idempotency-Key`, recorded with the intent
in a single transaction. Concurrent copies return the same operation; reusing a
key with a different payload is a conflict. A *different* key carrying
identical intent joins the current operation rather than refreshing the
operator timestamp, clearing a local override, or extending a pending deadline.
Retrying a failure is a different gesture, and names the failed operation
explicitly, because pressing Start again should not silently retry something
that already failed.

That makes three identities doing three jobs. The request key identifies the
user action, the command id deduplicates delivery, and the generation says
which desired update the device applied. None substitutes for another, and
greying out a button substitutes for none of them: it cannot survive a lost
response, a second tab, or a restart.

I learned the third one by getting it wrong a second time.

A report has to be stored before it can be matched, and storage needs an order.
Reports went into one row per device, last write wins, which felt safe because
their `at` stamp is on the server's clock and crosses no boundary. It is still
the wrong
key. `at` records when the platform acknowledged a command, not which command
it is about, and a platform may acknowledge a superseded command after the one
that replaced it.

<figure class="diagram">
<svg viewBox="0 0 620 160" role="img" aria-label="A single timeline on the application's clock. A first command is issued, then superseded by a second. The acknowledgement for the second command arrives stamped at 2000, and the acknowledgement for the first arrives later stamped at 3000. Ordering the two by stamp keeps the superseded command and discards the confirmation for the one in flight. Ordering by command sequence keeps the right one.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">TWO REPORTS, ONE ROW &middot; THE STAMP SAYS THE WRONG ONE IS NEWER</text>
  <text x="0" y="32" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">OUR CLOCK &middot; the platform stamps `at`, so no clock boundary is crossed here</text>
  <line x1="0" y1="50" x2="620" y2="50" stroke="var(--ink)"/>
  <g stroke="var(--ink)">
    <path d="M60 44 L60 56 M180 44 L180 56 M320 44 L320 56 M470 44 L470 56"/>
  </g>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">
    <text x="60" y="70">issue /1</text>
    <text x="180" y="70">supersede, issue /2</text>
    <text x="320" y="70">report /2 &middot; at 2000</text>
    <text x="470" y="70">report /1 &middot; at 3000</text>
  </g>
  <text x="0" y="104" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">BY `at`</text>
  <text x="108" y="104" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">3000 &gt; 2000, so the row becomes /1 &middot; the confirmation for /2 is gone</text>
  <text x="0" y="130" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">BY SEQUENCE</text>
  <text x="108" y="130" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">1 &lt; 2, so the row stays /2, and the late arrival changes nothing</text>
  <text x="0" y="154" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">`at` still decides between two reports for the SAME command, where PENDING must give way to APPLIED</text>
</svg>
<figcaption>Both stamps come from the same clock. Neither crosses a boundary. The later one is still about the older command.</figcaption>
</figure>

The stale arrival then carries the newest stamp, and last-write-wins evicts the
confirmation for the command actually in flight. It never produced a wrong
answer, which is why it survived: the matcher checks the command id, so a stale
report is discarded rather than believed. What was lost was a confirmation, and
a lost confirmation looks exactly like a slow device.

Ordering by the sequence inside the command id fixes it, with `at` surviving as
the tiebreak within a single command. A timestamp can be in the right frame of
reference and still not order the events you care about, because it records
when something was said rather than what it was said about.

## One fleet, several platforms

A generation tells the loop which update a report belongs to. It cannot make a
report arrive, so a deadline is still needed, and a deadline is a policy choice:
longer gives delayed confirmation more time to arrive and makes users wait
longer when it never comes. Only an explicit rejection establishes that the
device refused. A deadline reached in silence leaves the physical outcome
unresolved, and the two must not be reported as one thing.

Choosing that number is where I got it wrong for longest, because nothing about
it looks like a bug.

Sort the fleet by what each platform can actually tell you, and it falls into
four groups rather than two.

| | **echoes a generation** | **no generation** |
| --- | --- | --- |
| **reports applied** | confirmed twice over | the command channel confirms it |
| **no applied** | confirmed by integer comparison | **clock bounds, or nothing** |

Only the bottom-right group needs a clock at all; for the rest, correlation
short-circuits before any arithmetic happens. That group is not a rare corner.
It is Modbus and SunSpec, where a write to an unimplemented register is ignored
without an exception and the documented idiom is to read the register back. It
is OCPP 1.6, whose remote start carries an `idTag` and nothing that would tie a
later `StartTransaction` to it.

Two near-misses are easy to mistake for the real thing. EEBus SPINE has
`msgCounterReference`, which ties a reply to its request, and Matter gives every
cluster a `DataVersion` that increments when attribute data changes. The first
is a correlation id and the second is cache invalidation. Neither says which
desired-state update the device applied, and that is the only question
telemetry correlation asks.

### The cost of one set of numbers

A clock-only device cannot be confirmed until a capture is `maxLatency +
skewBound` of device-clock distance past the command, because proving *after*
needs a lower bound on the clock offset and the only way to get one is an upper
bound on latency. At the bounds I declared, that floor is 150 seconds.

My timeout was 180 seconds.

<figure class="diagram">
<svg viewBox="0 0 620 190" role="img" aria-label="Three horizontal bars on a shared axis of seconds after the command. The first is a vendor that echoes a generation, confirmed at the first capture with no floor. The second is a clock-only vendor under a three hundred second timeout: the first hundred and fifty seconds are a floor in which no answer is possible, leaving a hundred and fifty second window. The third is the same vendor under the old hundred and eighty second timeout, leaving a window of thirty seconds.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">SECONDS AFTER THE COMMAND &middot; 1 PX = 1 S</text>
  <text x="0" y="32" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">A VENDOR THAT ECHOES A GENERATION</text>
  <rect x="0" y="38" width="600" height="22" fill="none" stroke="var(--line)"/>
  <rect x="28" y="38" width="4" height="22" fill="var(--signal)"/>
  <text x="40" y="53" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">confirmed at the first capture that carries the counter &middot; no floor, no clock</text>
  <text x="0" y="84" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">CLOCK-ONLY &middot; 300 S TIMEOUT</text>
  <rect x="0" y="90" width="150" height="22" fill="var(--ink)" opacity="0.07"/>
  <rect x="150" y="90" width="150" height="22" fill="var(--signal)" opacity="0.18"/>
  <rect x="0" y="90" width="300" height="22" fill="none" stroke="var(--line)"/>
  <text x="75" y="105" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">no answer possible</text>
  <text x="225" y="105" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="middle">150 s of window</text>
  <text x="0" y="136" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">CLOCK-ONLY &middot; THE 180 S TIMEOUT IT HAD</text>
  <rect x="0" y="142" width="150" height="22" fill="var(--ink)" opacity="0.07"/>
  <rect x="150" y="142" width="30" height="22" fill="var(--signal)" opacity="0.18"/>
  <rect x="0" y="142" width="180" height="22" fill="none" stroke="var(--line)"/>
  <text x="75" y="157" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">no answer possible</text>
  <text x="190" y="157" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">30 s</text>
  <g stroke="var(--line)">
    <path d="M0 170 L0 176 M150 170 L150 176 M180 170 L180 176 M300 170 L300 176"/>
  </g>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">
    <text x="0" y="187">0</text>
    <text x="150" y="187">150</text>
    <text x="180" y="187">180</text>
    <text x="300" y="187">300</text>
  </g>
</svg>
<figcaption>Thirty seconds in which a snapshot had to be both taken and delivered. A fleet reporting once a minute fails commands it applied, and the reason string blames the clock bounds rather than the deadline that was too short to use them.</figcaption>
</figure>

A device reporting once a minute usually misses that window, and the loop
reports `FAILED` for a command that worked. Widening the timeout fixes that
vendor and taxes every other: a platform that echoes a generation and could
answer in seconds then waits five minutes to report a failure.

So the three numbers belong to the vendor, not to the fleet.

The odd part is that my own system already believed this everywhere else. Rate
limiters and circuit breakers are per vendor; the readout tallies correlation
outcomes per vendor, under a comment I had written myself saying that a fleet
average hides the one vendor that is the problem. Only the numbers deciding
whether a command succeeded were global, which is the kind of inconsistency
that survives review because each half looks reasonable alone.

They are resolved per vendor now, merged over fleet defaults so a vendor states
only what differs. An unknown key or a non-numeric value stops the process at
startup rather than quietly reverting, because a tuning change that looks
applied and is not is the hardest kind to debug. One consequence reads like a
bug and is not: a vendor whose devices echo a generation can legitimately run a
*negative* window, because it never consults a clock and so never stands on the
floor it cannot clear.

## The charger was charging and I reported that I could not tell

All of this is easier to believe from two runs of the same scenario, with the
same capture interval, delivery latency, actuation delay and command. The only
difference is whether the device reports the generation it last applied, and a
test asserts they differ in nothing else.

<figure class="diagram">
<svg viewBox="0 0 620 232" role="img" aria-label="The same five-minute run drawn twice. In both, the ground truth is that the charger starts charging twenty seconds after the command and keeps charging. Without a counter the phase stays ISSUED for the whole run and ends in FAILED at five minutes. With a counter the phase becomes CONVERGED at two minutes thirty, when the device reports the generation it applied.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">THE SAME RUN, TWICE &middot; THE ONLY DIFFERENCE IS WHETHER THE DEVICE REPORTS A GENERATION</text>
  <text x="0" y="34" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">WITHOUT A COUNTER</text>
  <text x="0" y="52" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">truth</text>
  <rect x="97" y="42" width="523" height="14" fill="var(--ink)" opacity="0.14"/>
  <rect x="60" y="42" width="560" height="14" fill="none" stroke="var(--line)"/>
  <text x="104" y="52" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">charging, and stays charging</text>
  <text x="0" y="74" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">phase</text>
  <rect x="60" y="64" width="560" height="14" fill="none" stroke="var(--line)"/>
  <text x="66" y="74" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">ISSUED &middot; no snapshot can be proved to post-date the command</text>
  <rect x="616" y="64" width="4" height="14" fill="var(--signal)"/>
  <text x="620" y="94" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">5:00 FAILED &middot; acknowledged, never provably observed</text>
  <text x="0" y="128" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">WITH A COUNTER</text>
  <text x="0" y="146" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">truth</text>
  <rect x="97" y="136" width="523" height="14" fill="var(--ink)" opacity="0.14"/>
  <rect x="60" y="136" width="560" height="14" fill="none" stroke="var(--line)"/>
  <text x="104" y="146" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">charging, and stays charging</text>
  <text x="0" y="168" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">phase</text>
  <rect x="340" y="158" width="280" height="14" fill="var(--signal)" opacity="0.18"/>
  <rect x="60" y="158" width="560" height="14" fill="none" stroke="var(--line)"/>
  <rect x="338" y="158" width="4" height="14" fill="var(--signal)"/>
  <text x="66" y="168" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">ISSUED</text>
  <text x="348" y="168" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">CONVERGED</text>
  <text x="620" y="188" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">2:30 &middot; device reports it applied generation 1</text>
  <g stroke="var(--line)">
    <path d="M60 202 L60 208 M340 202 L340 208 M620 202 L620 208"/>
  </g>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="60" y="219" text-anchor="middle">0:00</text>
    <text x="340" y="219" text-anchor="middle">2:30</text>
    <text x="620" y="219" text-anchor="end">5:00</text>
  </g>
  <text x="0" y="231" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">same capture interval &middot; same delivery latency &middot; same actuation delay &middot; same command</text>
</svg>
<figcaption>The charger behaved identically in both runs. Only the reporting changed, and with it everything the operator was told.</figcaption>
</figure>

In both runs the charger starts charging twenty seconds in and keeps charging.
The command worked. `FAILED` there means the confirmation deadline expired, not
that the charger stayed stopped: the acknowledgement the loop received said
only that the instruction arrived, and under the declared bounds no snapshot
could ever be *proved* to post-date the command.

One run is confirmed at 2:30 and the other cannot be confirmed at all, and the
hardware did the same thing in both.

## Measuring a device without scoring it

Once the loop runs against a real fleet the question becomes which devices are
bad, and answering it needs the same care about what a number means.

The decision log had the evidence and not the shape. A decision is written on
every wake, so counting rows by phase counts ticks: a device sitting in
`ISSUED` for ten wakes looks like ten of something. A verdict is a *transition*
out of `ISSUED`, and that happens exactly once per command that got one.

```text
device          vendor      cmds   confirmed   rejected   unconfirmed   drift   dark
dev-0012        tortoise      39       20.5%          0            31       0     17
dev-0007        hare          25       80.0%          5             0       0      0
dev-0000        hare         145       97.9%          0             3       0      0
dev-0031        tortoise       0         n/a          0             0       9      0
```

These are not combined into a score, which is the only interesting decision in
the table. The four outcomes have four owners: a rejection is the vendor
answering no, an unconfirmed is nobody answering, drift is somebody at the
device disagreeing with us, dark is the feed.

`dev-0012` is the row that makes the case. Twenty per cent confirmed reads as a
broken charger until the next columns say thirty-one unconfirmed and seventeen
periods dark. None of that is evidence about the hardware, and a composite
score would have hidden a telemetry problem behind a number that looked like a
verdict on the charger.

A command superseded mid-flight is counted neither way, because it never got a
verdict. A device with no verdict shows `n/a` and sorts last: an absence of
evidence is not a good score.

## The pattern I did not see for a long time

Every failure in this article presents as a timing problem and resolves as a
naming problem.

The acknowledgement that cannot confirm the actuation is missing a correlation
id, which OCPP 2.0.1 added. The reading that cannot be ordered against the
command is missing a generation, which Kubernetes had all along. The second
click that rewrites the first is missing a request key. The stale report that
evicts a fresh one is missing a sequence, even though its timestamp was already
in the right frame of reference.

Four times I reached for a clock, and four times what the system actually
needed was for something to say which thing it was about.

A clock tells you when an event happened, assuming you own the clock, which
across a device boundary you do not. An identity tells you what an event refers
to, and it keeps working when the clocks are wrong, when the queue reorders,
and when the same instruction arrives twice.

That is the cheaper thing to build, and it is the one the protocols keep adding
after the fact.

## References

**The two channels, in specifications**

1. Open Charge Alliance, OCPP 1.6 — `RemoteStartTransaction.req` carries
   `idTag` and an optional `connectorId` and nothing else;
   `RemoteStartStopStatus.Accepted` is glossed "Command will be executed".
   OCPP 2.0.1 adds `RequestStartTransaction.remoteStartId`, echoed back in
   `TransactionEvent`. https://openchargealliance.org/download-ocpp/

2. Eclipse Hono — command and telemetry as separate APIs in opposite
   directions. https://github.com/eclipse/hono

**Ordering by counter rather than by clock**

3. Kubernetes API conventions — `metadata.generation` and
   `status.observedGeneration`, with conditions still reporting success
   separately. https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md

4. AWS IoT Device Shadow — a `version` used for optimistic concurrency, and
   `clientToken` tying a response to its request.
   https://docs.aws.amazon.com/iot/latest/developerguide/device-shadow-document.html

5. L. Lamport, *Time, Clocks, and the Ordering of Events in a Distributed
   System*, CACM 21(7):558-565, 1978 — counters "with no actual timing
   mechanism". https://doi.org/10.1145/359545.359563

**What platforms actually offer**

6. Azure IoT Plug and Play — the device echoes `av` with an `ac` status: 200
   accepted, 202 in progress, 400 rejected.
   https://learn.microsoft.com/en-us/azure/iot/concepts-convention

7. SunSpec Information Model — a write to an unimplemented register is ignored
   and generates no exception.
   https://sunspec.org/wp-content/uploads/2019/08/SunSpec-Information-Models-12041.pdf

8. EEBus SPINE — `msgCounterReference` ties a reply to its request. evcc issue
   33789 records a charger answering a `LoadControlLimits` write successfully
   while charging never starts. https://github.com/evcc-io/evcc/issues/33789

9. Matter 1.4 — a cluster `DataVersion` for suppressing reports a client
   already holds.
   https://csa-iot.org/wp-content/uploads/2024/11/24-27349-006_Matter-1.4-Core-Specification.pdf

**My own**

10. [Some Forces Do Not Resolve](/articles/some-forces-do-not-resolve/) — the
    Kafka `acks` example of being told yes about something that did not happen.

11. [Who Pays for the Force](/articles/who-pays-for-the-force/) — forces from
    inside an implementation escaping into a contract.
