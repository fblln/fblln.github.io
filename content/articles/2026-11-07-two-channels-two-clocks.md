+++
title = "Two Channels, Two Clocks"
date = "2026-11-07"
description = "A command acknowledgement does not prove a charger started, and comparing a device timestamp against your own cannot establish it either. What the reconciler decides, why a generation counter settles it, and why the numbers that judge a command belong to the vendor rather than the fleet."
tags = ["Architecture", "Protocol", "Reliability", "Trade-offs"]
+++

You ask a charger to start. Ninety seconds later the platform's telemetry still
says plugged in and stopped. One of three things is true. The command is
failing and nothing will say so. The command worked and the reading predates
it. Or the answer is not knowable yet, and acting on either guess is a mistake.

I have been building a desired-state reconciler for this problem. *Desired
state* is what the user wants: this charger should be charging. *Reported state*
is the latest account the system has received from the device. The reconciler
is the software that keeps comparing the two, sends a command when needed, and
tracks whether it took effect. It also has to know when to wait.

My first version compared timestamps without checking which clock each came
from. Before getting to that mistake, it helps to lay out what sits between the
software and the charger.

## What sits between a button and a charger

The system I am working on controls energy devices through an aggregator: a
service that gives an application one programming interface (API) for equipment
from several manufacturers. A fleet can include chargers, home batteries, solar inverters,
and heat pumps. They may all appear in one application, but that does not mean
they share a connection, a protocol, or an operator.

For a charger reached through its manufacturer's cloud, the path looks like
this. The arrows show the direction of the messages, not separate physical
cables:

```text
COMMAND
user or schedule → reconciler → aggregator → manufacturer cloud → charger

STATE REPORT
charger → manufacturer cloud → aggregator → reconciler → user interface
```

The reconciler remembers what was requested and which command is outstanding.
The aggregator translates its request into the manufacturer's API. The
manufacturer's service delivers it over the connection the charger maintains.
The charger runs firmware (the software inside the appliance) which decides
whether it can act and controls the hardware.

On the return path, the charger sends measurements and status reports. This is
*telemetry*: messages such as "plugged in", "charging", or "drawing 7 kW".
Reports may be sent periodically, when something changes, or in response to a
poll. The value in the application is therefore the latest report it has
received, not a direct view of the hardware at this instant.

This is one deployment shape, not a requirement of all connected hardware.
Some devices expose a local interface; some chargers connect directly to an
operator's system. Removing a cloud hop can reduce delay, but a command response
and a later state report still need to be tied together.

## Why one API does not make one platform

There are platforms that unify access. The aggregator is doing exactly that.
What it cannot do by translating requests is give all the equipment the same
behaviour.

Different manufacturers ship different firmware, support different operations,
and manage their own accounts and device connections. Installed equipment also
outlives individual API versions. Integrating an existing charger usually means
working with the interface its owner can authorise, rather than replacing its
firmware or moving it to a new backend. A common protocol helps where it is
supported; it does not retrofit every device already in someone's garage.

Even "start charging" crosses several decisions. The cloud can accept the
request while the charger is disconnected. The charger can receive it while
the car is not ready to draw power. A local schedule or power limit can affect
what happens next. An application must preserve those distinctions if it is to
explain why the user is still waiting.

Connectivity software such as [Eclipse Hono](https://eclipse.dev/hono/docs/)
provides adapters for different device protocols and separate APIs for commands
and telemetry. That makes integration easier. The application still needs a
contract for what an acknowledgement means and what proves completion.

The operational problems follow these boundaries. A manufacturer API may limit
how often the aggregator can poll. A connection can drop after a command was
sent but before its reply arrives. Retrying may deliver the same command twice.
A buffered state report can arrive after a newer one. Meanwhile, a person can
change the device locally without going through our application. The reconciler
has to retain enough history to avoid treating every mismatch as a reason to
send another command.

## Receipt, action, and observation are different events

Consider a possible sequence, described here by an outside observer with one
clock so the order is unambiguous:

| Time | Event |
| --- | --- |
| 12:00:00 | The charger records "stopped"; the report waits in a queue. |
| 12:00:05 | The application requests charging. |
| 12:00:06 | The platform acknowledges the request. |
| 12:00:10 | The charger starts charging. |
| 12:00:20 | The old "stopped" report reaches the application. |

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
<figcaption>The charger starts charging at 12:00:10 and never tells us. What reaches us at 12:00:20 is the reading from before we asked.</figcaption>
</figure>

At 12:00:20 the application receives new information about an old state. Acting
as though "just received" means "just measured" would make it retry a command
that already worked. A receipt timestamp can tell us when our server got the
message; it cannot tell us when the charger captured the reading.

The command and telemetry channels are separate logical flows, even when they
share a network connection. They can pass through different queues and retry
independently. Receiving a response on one does not establish how far the other
has progressed.

## The ack is not the actuation

An *acknowledgement*, often shortened to *ack*, is a reply confirming some stage
of receipt or acceptance. *Actuation* is the physical action: in this case,
starting the charging process. Which stage an ack confirms depends on the API.

An ack means the instruction arrived, not that the relay moved. OCPP 1.6 is
explicit about it. OCPP is the Open Charge Point Protocol, used for
communication between charging stations and their management systems.

`RemoteStartTransaction.req` carries two fields: a mandatory `idTag` and an
optional `connectorId`. The charge point answers with a
`RemoteStartStopStatus`, either `Rejected` or `Accepted`, and the
specification's own gloss on `Accepted` is "Command will be executed." That is
a promise about the command. It says nothing about the connector.

Three questions, three messages, three arrival times. Did the instruction reach
the charger: `RemoteStartTransaction.conf`, immediately. Did a transaction
begin: a later `StartTransaction.req`, which is where a `transactionId` is
finally assigned. What is the connector actually doing: `StatusNotification`,
on its own schedule. Those can diverge. There are public reports of chargers
answering `Accepted` and then sending `StatusNotification` with `SuspendedEV`
(a status for an EV declining energy inside a transaction that is already
running) with no `StartTransaction` at all. The charger reported a state that
presupposes the thing it never confirmed.

A `StartTransaction.req` does eventually arrive, and nothing in it says which
remote start caused it. The request carried an `idTag`, and an idTag is a
credential rather than a request identifier: the same token can be presented at
the plug by a driver who walked up while your command was in flight. You hold
an acknowledgement and a transaction with no way to prove they belong to each
other. OCPP 2.0.1 closes exactly this gap. `RequestStartTransaction` carries a
`remoteStartId`, the charger echoes it back in the `TransactionEvent` that
reports the session, and `TransactionEvent` replaces `StartTransaction`,
`StopTransaction` and the in-transaction meter values with one stream. The
protocol grew a correlation id, because no acknowledgement could be made to
carry the answer.

I have written this failure before, one layer down. The Kafka producer in [Some
Forces Do Not Resolve](/articles/some-forces-do-not-resolve/) can be told a
write succeeded as soon as the partition leader has it, and the leader can fail
before any follower does. The producer is told yes about something that did not
happen. So is the operator who pressed Start. Kafka at least hands you a single
ordered log to reconcile against. Here the thing that would confirm the command
arrives somewhere else entirely, on a clock you do not own.

## Why the clocks disagree

The application server and the charger each keep time locally. Writing both
timestamps in UTC gives them the same notation, but does not synchronise the
clocks that produced them. This is not a timezone problem.

A hardware clock counts ticks from an oscillator. Its rate is not perfect, so
its error accumulates between corrections. A device can also start with the
wrong time after losing power or rebooting before it has synchronised. Network
time services such as NTP correct clock offset and rate, but a device that cannot
reach its time source has to keep running without fresh corrections. These are
ordinary clock-management concerns described in
[RFC 5905](https://www.rfc-editor.org/rfc/rfc5905).

Clock error and delivery delay are separate problems. A perfectly timed reading
can sit in a queue for thirty seconds. An immediately delivered reading can
carry a timestamp thirty seconds ahead of our server. Subtracting the capture
time from the arrival time mixes those two effects.

For example, suppose the device clock is two minutes fast. A reading actually
taken at 12:00 is stamped 12:02. We send a command at 12:01 by our clock. A plain
timestamp comparison calls the reading newer than the command, although it was
taken a minute earlier. The device does not need to be wildly wrong to cause
trouble; the error only needs to exceed the gap between the two events.

There can be more than two clocks along the path. A vendor might timestamp a
measurement on the device, stamp it at cloud ingestion, or return the time its
cache was refreshed. Before comparing a field called `lastSeen`, we need to know
which event it records and who supplied the time. In my original model it was
the device's capture timestamp, while the command time came from my server.

## I compared two clocks and called it evidence

The rule at the centre of the first implementation was one line, and I was
pleased with it. A reading is evidence about a command only if the reading was
taken after the command:

```ts
function isEvidenceFor(obs: Observation, state: Phase): boolean {
  return obs.lastSeen > state.since;
}
```

That single comparison does real work. It is what stops the duplicate-command
storm, where a loop sees a device reading "stopped", concludes its start
failed, sends another, sees "stopped" again a tick later, and hammers a
rate-limited API that was already retrying on its behalf. The stale reading
keeps triggering the same command.

The comparison is also nonsense. `state.since` is when I issued the command, by
my clock. `obs.lastSeen` is when the device recorded its state, by the device's
clock. The expression compiles, runs, passes every test that existed, and
orders two events which have never been in the same frame of reference. A
device running ten minutes fast makes every one of its readings look like
evidence. A device running ten minutes slow disqualifies all of them. Either device may still execute commands correctly; its timestamp is not a
reliable basis for this comparison.

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

Separating local and device timestamps in the type system helps catch
accidental mixing, but it does not supply the missing evidence. The report
needs to identify which desired-state update the device acted on.

## What the reconciler actually decides

Everything so far is the problem. The reconciler is the answer to it, and it
is one pure function:

```text
(intent, observation, command report, clock model, current phase, now)
    → (next phase, a command or nothing, a sentence saying why)
```

No I/O, no clock of its own, no randomness, no mutation of its inputs. The
clock model is threaded in rather than read, which is why every pathology in
this article is reproducible in a unit test instead of requiring a car and a
patient afternoon.

The sentence is not decoration. A loop that moves a device from one state to
another without recording which evidence persuaded it produces incidents
nobody can close. The same sentence goes to the audit log and to the
user-facing fault string, so the two can never disagree about why a device is
where it is.

### Five phases, and most of them exist to stop you acting

IDLE and CONVERGED both mean the device agrees with intent, and they are not
interchangeable. CONVERGED remembers what it converged to and which generation
did it, so a later reading that disagrees is drift and can be named as such.
IDLE remembers nothing and treats any reading as informative, which is correct
at a cold start and wrong everywhere else.

ISSUED is the only phase carrying a deadline, and the only one that can be
confirmed. It holds the command id and the generation, because a report is
ours only if the id matches and a snapshot is evidence only if the generation
has moved.

SUSPENDED is what a dead feed looks like from inside the loop. Without it the
last snapshot stays current forever and the reconciler issues commands against
a frozen reading. It carries the watermark into the outage, so on recovery the
loop can tell a fresh capture from the stale one it went dark holding.

FAILED is terminal, deliberately. If neither channel could confirm inside a
generous window, the right move is to surface that and stop, rather than keep
poking a device that may be refusing for a reason.

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

### Three rules, and the third is the one implementations lack

**Evidence must post-date the action.** A reading taken before the command says
nothing about the command. This is the rule my first implementation thought it
was enforcing.

**The command channel outranks telemetry when it has something real to say.**
`REJECTED` and `APPLIED` are first-hand and need no correlation at all. They
are also the two things many platforms never send, which is why the telemetry
path cannot be skipped.

**Correlating telemetry with an action crosses two clocks, so it has three
answers.** After, before, and cannot tell. A reconciler that cannot say "I do
not know yet" will eventually tell someone their car is charging when it is
not.

### Why this is a loop and not a call

A charger can stop on its own. Somebody can press the button on the front
panel. A vendor cloud can drop a command without telling anyone. None of that
produces an event; it produces a disagreement visible only on the next
snapshot, which means the only way to notice is to keep looking.

Having a generation pays a second dividend here. A fresh reading that
disagrees with intent while reporting the same generation we last converged at
is positive evidence that something other than us moved the device: a command
advances the counter, and a human at the panel does not. That is exact,
same-channel, and needs no clock. Deciding what to do about it is a policy the
function deliberately does not hold, because fighting a human is right for a
charger on a tariff and wrong for someone who has just turned the heating up.

## Ordering state updates with a counter

The solution is to assign a generation to each desired-state update and carry
that generation through to the device's report. A *generation* is just the
revision number of the requested state for that device: 41, then 42, then 43. For a charger, the exchange
would look like this:

```text
platform → device: desired generation 42, charging = true
device → platform: accepted generation 42
device → platform: applied generation 42, charging = true
```

The second message says the request arrived. The third reports that the device
applied that particular update and reached the requested state. A delayed
report for generation 41 cannot confirm generation 42, even if it says
"charging". Neither clock participates in that decision.

The counter needs a contract: all participants must agree who assigns it and
what returning it means. The authority accepting desired-state changes
allocates generations per device and persists them across restarts. A retry of
the same update retains its generation. A new intent gets a new one. The device
must associate its reported result with the generation it actually processed;
copying the latest number onto an unrelated reading would recreate the original
bug with an integer in place of a timestamp.

A matching generation alone also does not establish success. The report must
say whether that update is pending, applied, or rejected, and what state
resulted. The following is the decision rule for this protocol, not a
replacement for the whole state machine:

```text
report generation < target generation  → stale for this intent
report generation > target generation  → refresh the target; this intent was superseded
report generation = target generation:
    applied, state matches target      → confirmed
    rejected                           → failed
    otherwise                          → keep waiting until the deadline
```

Kubernetes uses `metadata.generation` to identify a desired-state revision and
`status.observedGeneration` to identify the revision a controller has observed.
The status conditions still have to say whether the work succeeded. An observed
generation tells you which revision a condition describes; it does not turn an
unsuccessful condition into a successful one. The [API
conventions](https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md)
and
[kstatus](https://github.com/kubernetes-sigs/cli-utils/blob/master/pkg/kstatus/README.md)
make that separation useful to clients waiting for reconciliation.

AWS IoT Device Shadow uses a related versioning mechanism. Its document version
advances on updates, and a caller can supply the version it expects to prevent
a conflicting write. A `clientToken` correlates a response with its request.
These identify document revisions and requests; the shadow version alone does
not prove that hardware applied a desired state. That still needs a device
report with the appropriate application-level meaning. The distinction is
visible in the [shadow document
contract](https://docs.aws.amazon.com/iot/latest/developerguide/device-shadow-document.html).

Lamport's logical clocks provide the basis for ordering events without
wall-clock comparison. Here the narrower requirement is a sequence of
desired-state revisions with one allocation authority, carried into the
resulting reports.

If an older charger cannot report which update it applied, this protocol needs
firmware or platform support. A counter invented only at the client cannot fill
that gap. The client can display reported state and acknowledge that a command
was accepted, but it cannot claim that the report confirms this command.

## What you can actually download

I looked for an existing implementation. The projects below cover parts of the
problem, but I still needed to write the reconciliation loop.

[Eclipse Ditto](https://eclipse.dev/ditto/basic-feature.html) comes closest. It
is an open-source digital-twin framework that separates reported state, which
comes from the device, from desired state, which a backend sets as the target,
and persists both. Its documentation then says the reconciling logic "is up to
your application or device firmware". Ditto holds the two numbers and will not
drive the device toward one of them. That is a reasonable place to draw a line,
and it means the loop is yours.

[Azure IoT Plug and
Play](https://learn.microsoft.com/azure/iot-pnp/concepts-convention) specifies
the counter properly, and reading it was mildly deflating. The desired section
carries a `$version` the hub manages. The device confirms with a reported
property carrying `av`, an acknowledgment version matching that `$version`, and
`ac`, a status code: 200 accepted, 400 or 500 refused, and 202 for an update
still in progress that should later finish at 200. The hub mints the counter,
not the client, which is the correction I needed. And 202 is a state I did not
have: a device committing to "working on it" rather than leaving you to infer
from silence. That status does not confirm actuation, but it helps locate a
failure. A command that times out having only been acknowledged and one that
times out after the device said it was working are different faults with
different owners, and a timeout that cannot tell them apart sends somebody to
read the wrong logs.

Z-Wave solves the acknowledgement half in the wire format. A SET sent with the
[Supervision command
class](https://docs.silabs.com/z-wave/7.21.3/zwave-api/supervision) is answered
with a status saying whether the command was understood *and executed*, and
[zwave-js](https://github.com/zwave-js/node-zwave-js/discussions/6515)
separates never-sent from received-but-not-executed from invalid-value. It also
documents the staleness trap, in a library note: a GET issued immediately after
a SET returns a transitional value on some devices.

[Temporal](https://temporal.io/blog/ignition-flashing-2-300-conference-badges-with-temporal)
is the right shape for the durable half, running one state machine per physical
device and surviving devices that reboot mid-process. On the charger side
[EVerest](https://github.com/EVerest/everest) is the serious open stack.

The remaining work is to connect the desired generation to the device result,
handle stale reports and retries, and enforce a deadline.

## The counter does not choose the deadline

A generation tells the loop which update a report belongs to. It cannot make a
report arrive. The device may be offline, the command may be stuck in a queue,
or the actuation may have failed without producing a result.

The timeout therefore remains a policy choice. A longer timeout gives delayed
confirmation more time to arrive and makes users wait longer when it never
will. The loop should distinguish an explicit rejection from a deadline reached
without confirmation. Only the first establishes that the device refused the
request; the second leaves the physical outcome unresolved.

Retries must preserve that distinction too. Resending an update with the same
generation should let the receiver recognise a duplicate, rather than turn one
intent into several independently numbered operations.

## One fleet, several platforms

This is the part I got wrong for longer than the clock comparison, and it is
the harder of the two mistakes, because nothing about it looks like a bug.

A reconciler that talks to one platform can treat these numbers as constants.
A reconciler that talks to several is holding several different behaviours at
once, and the numbers stop being constants and become a property of the
vendor. Which vendor gives you what is not something you get to choose.

Sort the fleet by what each platform can actually tell you, and it falls into
four groups rather than two.

| | **echoes a generation** | **no generation** |
| --- | --- | --- |
| **reports applied** | confirmed twice over | the command channel confirms it |
| **no applied** | confirmed by integer comparison | **clock bounds, or nothing** |

Only the bottom-right group needs a clock at all. For everything else the
correlation short-circuits before any arithmetic happens. And the bottom-right
group is not a rare corner: it is Modbus and SunSpec, where a write to an
unimplemented register is ignored without an exception and the documented
idiom is to read the register back on the next scan. It is OCPP 1.6, whose
remote start carries an `idTag` and nothing that would let a later
`StartTransaction` be tied to it.

The top-left group is real too, and it comes from a specific place. Azure IoT
Plug and Play has the device echo `av`, the desired
version it is acknowledging, alongside a status code. AWS IoT Device Shadow
carries a `version` that advances on every update and will reject a write that
supplies the wrong one. Both are cloud-platform conventions rather than
properties of the hardware, which is the useful thing to know: the counter
lives wherever commands are serialised, and that place is usually software you
or your vendor already operate.

Two near-misses are easy to mistake for the real thing. EEBus SPINE has
`msgCounter` and `msgCounterReference`, which tie a reply to the request that
caused it. Matter gives every cluster a `DataVersion` that increments when
attribute data changes, which lets a client skip a report it already has. The
first is a correlation id and the second is cache invalidation. Neither says
which of your desired-state updates the device applied, and that is the only
question telemetry correlation asks.

### The cost of one set of numbers

A clock-only device cannot be confirmed until a capture is `maxLatency +
skewBound` of device-clock distance past the command, because proving *after*
needs a lower bound on the offset and the only way to get one is an upper
bound on latency. At the bounds I declared, 120 seconds of latency and 30 of
skew, that floor is 150 seconds. Whatever is left of the command timeout after
the floor is the entire window in which a snapshot has to be both captured and
delivered.

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

A device reporting every sixty seconds would usually miss that window, and the
loop would report `FAILED` for a command that worked. Widening the timeout
fixes it, and then every well-behaved vendor pays for the worst one: a
platform that echoes a generation and could answer in seconds now waits five
minutes before it will tell anyone a command failed.

So the three numbers belong to the vendor, not to the fleet. The odd part is
that my own system already believed this everywhere except here. The
backpressure layer keeps a rate limiter and a circuit breaker per vendor,
because a quota belongs to a vendor. The readout tallies correlation outcomes
per vendor, with a comment I had written myself saying that a fleet average
hides the one vendor that is the problem. Only the numbers deciding whether a
command succeeded were global, and that is the sort of inconsistency that
survives review because each half looks reasonable on its own.

The readout is also the instrument for setting them, which is the part that
makes this tractable rather than guesswork. A vendor sitting at a high
clock-fallback rate is telling you it needs room. A vendor at zero is telling
you its deadline can be cut. A high *unknown* rate on top of a high fallback
rate is a different message again: the bounds declared for that vendor are too
loose for its feed, and no amount of extra timeout will fix an interval that
always straddles the command.

The three numbers are now resolved per vendor, merged over the fleet defaults
so that a vendor states only what differs and an unconfigured one still
reconciles. A key that does not exist, or a value that is not a number, stops
the process at startup rather than reverting to a default, because a tuning
change that looks applied and is not is the hardest kind to debug. The window
check runs per vendor too: a hand-written override is the easiest place to put
a timeout under its own clock floor, and that failure is silent.

One consequence reads like a bug and is not. A vendor whose devices echo a
generation can legitimately run a *negative* window. It will never consult a
clock, so the floor it cannot clear is a floor it never stands on.

## The charger was charging and I reported that I could not tell

Two scenarios from the original clock-based implementation show why I want the
protocol to carry a generation. They use the same capture interval,
delivery latency, actuation delay, and command. The only difference is whether
the device reports the generation it last applied, and a test asserts they
differ in nothing else.

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

```
## unprovable-without-a-counter
  intent    #############################################################
  truth     ..###########################################################
  observed  ...............##############################################
  reported  .aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
  phase     >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
   5:00  FAILED - acknowledged but never observed to take effect

## generation-decides
  phase     >>>>>>>>>>>>>>>==============================================
   2:30  CONVERGED - device reports it applied generation 1
```

The `truth` lane is the simulator's ground truth, which the reconciler cannot
see. In the first scenario the charger starts charging twenty seconds in and
keeps charging for the rest of the run. The command worked. The loop reports
`FAILED` at five minutes, because under the declared bounds no snapshot can
ever be *proved* to post-date the command, and the acknowledgement it did get
proves only that the instruction arrived.

Here `FAILED` means the confirmation deadline expired, not that the charger
necessarily stayed stopped. The second scenario supplies the missing applied
generation and lets the loop confirm the command at 2:30. This is why I would
require generation reporting in the protocol rather than keep tuning clock
estimates.

In these scenarios, the generation counter supplies the evidence that the
timestamp comparison cannot.

## What to show someone who is waiting

While confirmation is pending, the interface still has to show a state.

Home Assistant has the vocabulary for this and arrived at it years ago. It
separates two ideas that look alike. `assumed_state` means no state report ever
arrives from this device, and the interface then shows separate on and off
buttons rather than a toggle, because a toggle is a claim that you know the
current state and are flipping it. `optimistic` means show the new state the
moment a command is sent, without waiting. A core developer's phrasing is
better than mine: "optimistic != assumed. Assumed: we never get a state
reported." Their MQTT entities run optimistically when no state topic exists
and wait for confirmation when one does, and the Homematic integration goes
further, reverting after a thirty-second timeout.

For the counter-based protocol, the interface can show the desired generation,
the last applied generation, and whether confirmation is pending. Receiving
telemetry and confirming a command remain separate facts: a device can report
its state without identifying the update that produced it.

`optimistic` stays configuration, because nothing derives it. It is a judgement
about which lie is worse, and the answer is not a property of the hardware:
showing "charging" a few seconds early is harmless on a charger, and showing
"locked" unconfirmed is not acceptable on a door at any latency. The display
policy never changes what the reconciler claims to have confirmed.

A deadline keeps the interface honest about how long confirmation can take. The
view can show the time remaining:

```
ui @0:10 (careful   )  not charging [unconfirmed, toggle], confirming start (290s left)
ui @0:10 (optimistic)  charging     [unconfirmed, toggle], confirming start (290s left)
ui @10:00 (final)      charging     [confirmed-exact, toggle]
```

The display policy changes the provisional state shown to the user; the
reconciler evaluates the same evidence in both cases. Generation evidence earns
`confirmed-exact`, so the interface can distinguish a confirmed result from a
provisional display.

## When the user presses Start twice

The user has a retry loop too. They press Start, see no change, and press it
again. The reconciler can keep waiting for one device command while the API
still treats those clicks as separate instructions. Refreshing the instruction's
timestamp or clearing a local override would change its meaning, even if no
second device command were sent.

The [implementation](https://github.com/fblln/device-reconciler) now gives the
user action its own durable identity. The client sends an `Idempotency-Key` with
its intent and reuses that key if the response is lost. PostgreSQL records the
key and the intent in one transaction. Concurrent copies return the same
operation; reusing a key with a different payload returns a conflict. The
response includes an operation ID and a URL for its current status, so receipt
can be shown immediately without claiming that the device has acted.

A second click may generate a different key. If the target and intent source
are unchanged, it joins the current operation anyway. It does not refresh the
operator timestamp, clear a local override, or extend a pending command's
deadline. If another action has superseded the original operation, replaying
its old key returns `SUPERSEDED` rather than restoring the old request.

Failure needs a different gesture. Sending Start again returns the existing
failure. An explicit retry names the failed operation and carries a new request
key; it starts a new evaluation while retaining the device's command counters
and clock evidence. Retrying that HTTP request returns the same new operation.
A request naming an operation that is no longer the current failure is rejected.

These identities serve different purposes. The request key identifies the
user action, the command ID deduplicates delivery, and the generation lets the
device identify which desired update it applied. The Hono path preserves the
command identity, but the device application still has to deduplicate physical
side effects. Disabling a button while confirmation is pending helps the user;
it cannot substitute for these guarantees across lost responses, multiple tabs,
or process restarts.

The repository exposes this through its API, without a control frontend. Request
keys are retained durably with no automatic expiry. The tests cover concurrent
submissions, persistence after reconnecting, payload conflicts, unchanged
override metadata, and explicit retry. A production service also needs a
retention policy and tenant-scoped authentication around that contract.

## A stamp on my own clock still could not order two reports

The three identities above are the design working. This is the part where it
nearly did not.

A report has to be stored before it can be matched, and storage needs an order.
The command channel's reports go into one row per device, last write wins, and
I was comfortable with that because `at` is on our clock. This article is about
two clocks and this is not that case: the platform stamps `at` on our side of
the boundary, so comparing two of them is sound arithmetic on one timeline.

It is still the wrong answer. `at` records when the platform acknowledged a
command, not which command it is about. A platform is free to acknowledge a
superseded command after it has acknowledged the one that replaced it, because
a queue drains unevenly and a retry lands when it lands. The stale report then
arrives carrying the newest stamp, last-write-wins evicts the confirmation for
the command actually in flight, and the device goes back to waiting for
telemetry or for its deadline to tell it something it had already been told.

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
<figcaption>Both stamps are ours. Neither crosses a clock boundary. The later one is still about the older command.</figcaption>
</figure>

It never produced a wrong answer, which is why it survived: the matcher checks
the command id, so a stale report is discarded rather than believed. What was
lost was the confirmation, and a lost confirmation looks exactly like a slow
device.

The fix is the shape of everything else here. Order by the sequence inside the
command id, which is our own counter, advanced once per minted command and
never reused. `at` survives as the tiebreak within a single command, so a
`PENDING` still gives way to an `APPLIED`. An id the system cannot parse orders
below everything, so it can fill an empty row and can never evict a real one.

The lesson is narrower than the one about clocks, and more irritating for it. A
timestamp can be in the right frame of reference and still not order the events
you care about, because it records when something was said rather than what it
was said about.

## Measuring a device without scoring it

Once the loop runs against a real fleet, the question stops being whether the
design is right and becomes which devices are bad. That turns out to be two
questions wearing one coat.

The decision log had the evidence and not the shape. A decision is written on
every wake, so counting rows by phase counts ticks: a device sitting in
`ISSUED` for ten wakes looks like ten of something. What a verdict actually
looks like in the log is a *transition* out of `ISSUED`, and that happens
exactly once per command that got one. Recording the phase each decision moved
away from is the whole change, and it turns an audit log into a denominator.

```text
device          vendor      cmds   confirmed   rejected   unconfirmed   drift   dark
dev-0012        tortoise      39       20.5%          0            31       0     17
dev-0007        hare          25       80.0%          5             0       0      0
dev-0000        hare         145       97.9%          0             3       0      0
dev-0031        tortoise       0         n/a          0             0       9      0
```

I did not combine these into a score, and that is the only interesting decision
in the table. The four outcomes have four different owners. A rejection is the
vendor answering no, which is the only crisp failure this system can observe.
An unconfirmed is nobody answering before the deadline, which on a clock-only
device may say more about the telemetry interval than about the charger. Drift
is somebody at the device disagreeing with us, detected exactly, because a
command advances the generation and a human at the panel does not. Dark is the
feed. A weighted average of those moves for reasons you cannot act on, which is
how a reliability dashboard ends up being watched by nobody.

`dev-0012` is the row that makes the case. Twenty per cent confirmed reads as a
broken charger until the next two columns say thirty-one unconfirmed, zero
rejected, seventeen periods dark. Nothing there is evidence about the hardware.
That device has a telemetry problem, and a composite score would have hidden it
behind a number that looked like a verdict on the charger.

Two absences are deliberate. A command superseded mid-flight is not counted
either way, because it leaves `ISSUED` for `ISSUED`, which is not a transition,
and it never got a verdict: counting it as a failure would blame a device for
an intent change. A device with no verdict yet shows `n/a` and sorts last
rather than first, because an absence of evidence is not a good score, and a
worst-first list that opens with devices nothing has been asked of sends
somebody to look at the wrong rack.

## What the invariants caught that I did not

The eight properties this loop must satisfy are written as executable
postconditions rather than as tests. Never issue into a channel that cannot be
reached. Never claim convergence without evidence. Never fail except at a
deadline or on a stated rejection. Never allocate the same generation to different intents. They run on every
decision of every scenario and every one of six hundred generated worlds, which
is a different thing from running on the dozen cases a unit test constructs.

Two findings matter particularly for the generation-based design.

The first was in a contract rather than in the code. "Claiming convergence
requires evidence" had been written as a check on the resulting phase, so a
loop sitting quietly in a converged state, with nothing new to look at, tripped
it. The check belongs on the transition that claims convergence, where the
loop consumes new evidence. Remaining in that phase does not make a new claim.

The second was in the state machine. Converging from the initial state, where
the device already happens to match the intent, stamped the phase with a
generation number that had never been issued to anything. The next genuine
command then appeared not to advance the counter. The absence of a generation
belonged in the type, not papered over with a zero.

The generated-world check tests whether the contracts hold across a sequence of
decisions. It is a loop over generated worlds asserting that the system's own
stated rules hold at every step, and it fails on whichever world lands on the
boundary first.

## References

**The two channels, in specifications**

1. Open Charge Alliance, OCPP 1.6 (released 2015) — `RemoteStartTransaction.req`
   carries `idTag` and an optional `connectorId` and nothing else;
   `RemoteStartStopStatus.Accepted` is glossed "Command will be executed".
   Actual state arrives as `StartTransaction` and `StatusNotification`. OCPP
   2.0.1 adds `RequestStartTransaction.remoteStartId`, echoed back in
   `TransactionEvent`, which also subsumes `StartTransaction`,
   `StopTransaction` and transaction meter values. Both specifications are
   behind the OCA's download page rather than a stable deep link.
   https://openchargealliance.org/download-ocpp/

2. Eclipse Hono — telemetry ingestion and command & control documented as
   separate APIs in opposite directions, with upstream volume expected to be
   orders of magnitude larger than downstream.
   https://github.com/eclipse/hono

**Ordering by counter rather than by clock**

3. Kubernetes API conventions — `metadata.generation`, "a sequence number
   representing a specific generation of the desired state", set by the system
   and monotonically increasing per resource; `status.observedGeneration`, "the
   `generation` most recently observed by the component responsible for acting
   upon changes to the desired state"; and a condition's own
   `observedGeneration`, "the .metadata.generation that the condition was set
   based upon".
   https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md

4. Kubernetes SIG CLI, kstatus — using `generation` and `observedGeneration`
   together to detect a status that does not yet reflect the current spec.
   https://github.com/kubernetes-sigs/cli-utils/blob/master/pkg/kstatus/README.md

5. Amazon Web Services, *Device Shadow service documents* — `desired`,
   `reported` and `delta` state, the `version` field incremented on every
   update and used for optimistic concurrency, and `clientToken` for
   associating a response with its request.
   https://docs.aws.amazon.com/iot/latest/developerguide/device-shadow-document.html

6. L. Lamport, *Time, Clocks, and the Ordering of Events in a Distributed
   System*, Communications of the ACM 21(7):558-565, 1978 — logical clocks as
   counters "with no actual timing mechanism", and happened-before as a partial
   order.
   https://doi.org/10.1145/359545.359563

**What platforms actually offer**

7. Azure IoT Plug and Play, writable-property convention — the device echoes
   `av`, the desired-property version it is acknowledging, alongside an `ac`
   status: 200 accepted, 202 accepted and in progress, 400 rejected, 500 threw.
   https://learn.microsoft.com/en-us/azure/iot/concepts-convention

8. SunSpec Information Model specification — a write to an unimplemented
   register is ignored and generates no exception, and the ACKNOWLEDGE
   exception for a slow write requires the model to carry a completion
   register. The remaining idiom is to read the value back.
   https://sunspec.org/wp-content/uploads/2019/08/SunSpec-Information-Models-12041.pdf

9. EEBus SPINE — `msgCounter` and `msgCounterReference` tie a reply to the
   request that caused it, which is a correlation id rather than a
   desired-state counter. evcc issue 33789 records an Elli Charger Pro 2
   answering a `LoadControlLimits` write successfully while charging does not
   start. https://github.com/evcc-io/evcc/issues/33789

10. Matter 1.4 core specification — every cluster carries a `DataVersion`
    incremented when its attribute data changes, used with `DataVersionFilter`
    to suppress reports a client already holds. Cache invalidation, not an
    account of which desired update was applied.
    https://csa-iot.org/wp-content/uploads/2024/11/24-27349-006_Matter-1.4-Core-Specification.pdf

**My own**

11. [Some Forces Do Not Resolve](/articles/some-forces-do-not-resolve/) — the
   force-and-tension distinction this article leans on, and the Kafka `acks`
   example of being told yes about something that did not happen.

12. [Who Pays for the Force](/articles/who-pays-for-the-force/) — on forces from
   inside an implementation escaping into a contract, including the deadline
   a user must wait for confirmation.
