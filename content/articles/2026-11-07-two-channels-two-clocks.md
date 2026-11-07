+++
title = "Two Channels, Two Clocks"
date = "2026-11-07"
description = "A remote command and the state report that should confirm it travel on different channels, stamped by different clocks. The comparison every naive reconciler makes — is this reading newer than my command? — is unsound, and the fix is not a better timestamp. Read through Kubernetes observedGeneration, AWS IoT Device Shadow versions, OCPP RemoteStartTransaction, and the three answers a correlation has."
tags = ["Architecture", "Protocol", "Reliability", "Trade-offs"]
+++

You ask a charger to start. Ninety seconds later the platform's telemetry still
says plugged in and stopped. One of three things is true. The command is failing
and nothing will say so. The command worked and the reading predates it. Or the
answer is not knowable yet, and acting on either guess is a mistake.

The third case is the interesting one, and most control loops do not have a way
to express it. I have been building a desired-state reconciler for this problem,
a loop that takes an intent (this charger should be charging) and converges a
device to it through a platform API. The first version was wrong. It was wrong in
a way that is close to universal, and the mistake is invisible in the code. It
becomes visible on one question: which clock did each of these numbers come
from?

## The ack is not the actuation

Consumer energy hardware is not reachable directly. A client talks to a
platform, which talks to a manufacturer's cloud, which talks to the device over
whatever link it has. That chain has two channels, and they share nothing.

The command channel carries an instruction outward and an acknowledgement back.
Both ends of it are stamped locally. The telemetry channel carries state inward,
captured on the device's schedule and stamped by the device's clock. Different
paths, different latencies, different failure modes, different clocks.

The first consequence is the one everybody has met somewhere. An ack means the
instruction arrived, not that the relay moved. OCPP 1.6 is explicit about it:
the charge point replies to `RemoteStartTransaction` with a status indicating
whether it has accepted the request and *will attempt* to start a transaction.
The transaction itself shows up later as a `StartTransaction`, and the
connector's actual state as a `StatusNotification`. Those can diverge. There are
public reports of chargers answering `Accepted` and then sending
`StatusNotification` with `SuspendedEV`, with no `StartTransaction` at all.

I have written this failure before, one layer down. The Kafka producer in [Some
Forces Do Not Resolve](/articles/some-forces-do-not-resolve/) can be told a
write succeeded as soon as the partition leader has it, and the leader can fail
before any follower does. The producer is told yes about something that did not
happen. A charger that acknowledges a start it never performs is the same
sentence with different hardware, and it is worse in one specific way: Kafka at
least hands you a single ordered log to reconcile against. Here the thing that
would confirm the command arrives somewhere else entirely, on a clock you do not
own.

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
storm, where a loop sees a device reading "stopped", concludes its start failed,
sends another, sees "stopped" again a tick later, and hammers a rate-limited API
that was already retrying on its behalf. Anyone who has written one of these
loops has written that bug.

The comparison is also nonsense. `state.since` is when I issued the command, by
my clock. `obs.lastSeen` is when the device recorded its state, by the device's
clock. The expression compiles, runs, passes every test that existed, and orders
two events which have never been in the same frame of reference. A device running
ten minutes fast makes every one of its readings look like evidence. A device
running ten minutes slow disqualifies all of them. Neither device is broken.

Nothing in the type system objected, because both values were `number`. That is
the first thing I changed:

```ts
export type Instant = number & { readonly __kind: 'Instant' };
export type DeviceInstant = number & { readonly __kind: 'DeviceInstant' };
export type Duration = number & { readonly __kind: 'Duration' };
```

Three branded aliases, no runtime cost, and the unsound comparison becomes a
compile error. Exactly one function in the codebase is permitted to subtract one
clock from the other, it is a single expression, and reviewing the soundness of
the whole program's time handling now means reading one line. Staring at the
original would not have found it. It surfaced because somebody asked which
channel each timestamp had come from.

## A minimum delay bounds the offset from one side only

Having admitted the two clocks are different, the obvious move is to estimate
the offset between them and convert. Each snapshot carries one sample: arrival
time, minus the capture time the device reports. That sample is the clock offset
plus the transit delay.

Transit delay is never negative, so the smallest sample observed is the tightest
estimate of the offset available. NTP's clock filter does the same thing, keeping
a window of samples and preferring the one with minimum delay. It also has a
property that did not get enough thought the first time round. The minimum
sample is an *upper* bound on the offset and only an upper bound:

```
sample  = offset + latency,  latency >= 0
offset <= sample
```

An upper bound on the offset gives an upper bound on when a capture happened in
local time. That is enough to prove a reading was taken *before* some instant.
It proves nothing about *after*. Concluding that a snapshot post-dates the
command requires a lower bound on the offset, which means an upper bound on
latency. No sample provides one. Somebody has to declare it.

None of this is new. Cristian's 1989 paper on probabilistic clock
synchronization is about exactly this problem, and it reaches the same shape:
reading a remote clock when message delay is unbounded yields an *interval*
containing its value, derived from the round-trip delay, with the midpoint
chosen because it minimises the maximum error. The useful part is the property
Cristian puts alongside it. When a read succeeds, the reader knows the precision
it actually achieved. The estimate carries its own error bar, and a system that
discards the error bar on the way in has thrown away the only thing that makes
the estimate safe to use.

Once both bounds are declared, converting a device timestamp into local time
produces an interval rather than a point:

<figure class="diagram">
<svg viewBox="0 0 640 190" role="img" aria-label="Two parallel timelines. The upper line is our clock, carrying a marker for the instant a command was issued. The lower line is the device clock, carrying a capture marker. A bracket projected from the capture up onto our clock spans an interval that straddles the command marker, annotated cannot tell.">
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="0" y="12">OUR CLOCK</text>
    <text x="0" y="128">DEVICE CLOCK</text>
  </g>
  <line x1="0" y1="46" x2="640" y2="46" stroke="currentColor" stroke-width="1"/>
  <line x1="0" y1="152" x2="640" y2="152" stroke="currentColor" stroke-width="1"/>
  <line x1="300" y1="34" x2="300" y2="58" stroke="currentColor" stroke-width="2"/>
  <text x="300" y="28" font-family="var(--font-mono)" font-size="9" fill="currentColor" text-anchor="middle">command issued</text>
  <line x1="430" y1="140" x2="430" y2="164" stroke="currentColor" stroke-width="2"/>
  <text x="430" y="178" font-family="var(--font-mono)" font-size="9" fill="currentColor" text-anchor="middle">capture, device time</text>
  <path d="M430 140 L430 100 L190 100 L190 76" stroke="var(--muted)" stroke-width="1" fill="none" stroke-dasharray="3 3"/>
  <path d="M430 140 L430 110 L410 110 L410 76" stroke="var(--muted)" stroke-width="1" fill="none" stroke-dasharray="3 3"/>
  <line x1="190" y1="66" x2="410" y2="66" stroke="currentColor" stroke-width="2"/>
  <line x1="190" y1="60" x2="190" y2="72" stroke="currentColor" stroke-width="2"/>
  <line x1="410" y1="60" x2="410" y2="72" stroke="currentColor" stroke-width="2"/>
  <text x="300" y="86" font-family="var(--font-mono)" font-size="9" fill="currentColor" text-anchor="middle">cannot tell</text>
</svg>
<figcaption>The interval is the honest answer, and its width is the sum of two
numbers somebody chose. Narrow it and the loop decides sooner and is wrong more
often.</figcaption>
</figure>

So the correlation has three outcomes, not two:

```
lower bound > t   →  after
upper bound <= t  →  before
otherwise         →  cannot tell
```

Spanner does the same thing at the other end of the reliability spectrum.
TrueTime's `TT.now()` does not return a timestamp; it returns `[earliest,
latest]`, and Google spends GPS receivers and atomic clocks on keeping that
interval narrow rather than on pretending it is a point. A read-write
transaction then takes its commit timestamp from the top of the interval and
*waits out* the uncertainty before making its writes visible. The uncertainty is
exposed in the type and paid for in latency, which is the opposite of hiding it
behind a single number. If a system with atomic clocks in its datacentres
declines to collapse the interval, a reconciler talking to a charger over a
consumer broadband link has no business collapsing it either.

The third outcome is the whole point, and it is the one a boolean API cannot
express. `cannot tell` is not a failure and not a maybe-treat-as-false. It means
wait for a newer snapshot, or wait for the command channel to say something
first-hand. Collapsing it forces a guess at the exact moment the system has the
least information available, which is where this class of bug is born. A
reconciler that cannot say "I do not know yet" will eventually tell somebody
their car is charging when it is not.

## The industry chose a counter

The interval arithmetic was satisfying enough to sit with for a while before
checking what everyone else does. Everyone else does not use timestamps at all.

Kubernetes puts a sequence number on desired state. The API conventions define
`metadata.generation` as "a sequence number representing a specific generation of
the desired state", set by the system and monotonically increasing per resource,
and `status.observedGeneration` as "the `generation` most recently observed by
the component responsible for acting upon changes to the desired state". A status
whose `observedGeneration` is behind the object's `generation` is *known* to be
stale, not estimated to be. Conditions carry their own copy, defined as the
generation the condition "was set based upon", so staleness is tracked per
condition rather than per object. The Gateway API conformance suite asserts the
counter advances after a spec change, which says how much weight it carries: it
is what they test to prove reconciliation happened at all.

AWS IoT Device Shadow does the same job with a `version` on the shadow document,
incremented on every update, and processes an update only if the version supplied
matches the current one. Optimistic concurrency, not clock comparison. It also
carries a `clientToken` so a response can be tied back to the request that
caused it, which is the same idea applied to the command channel rather than the
state.

Both arrive at one rule. Do not order distributed events by wall clock when a
counter is available. A counter is written by one party, cannot go backwards,
cannot collide, and survives a clock correction.

Which is Lamport, in 1978, and it is slightly embarrassing to arrive at it from
the other direction. The paper defines a clock as a function assigning a number
to each event in a process, notes that these are logical rather than physical
clocks, and says they "may be implemented by counters with no actual timing
mechanism". A generation is a logical clock with one writer. The reason it orders
events that a wall clock cannot is the reason Lamport gave: ordering is a
property of the causal structure, and a physical clock is only ever evidence
about it. All that interval arithmetic is
what remains when nobody gives you one.

And mostly nobody does. Kubernetes controls software it ships; a charger vendor
ships firmware to a garage in 2019 and never touches it again. So the fallback
is not academic, it is the common path, and the right structure is both
strategies ranked:

```ts
if (observedGeneration !== null) {
  return { correlation: byGeneration(observedGeneration, target), source: 'generation' };
}
return { correlation: capturedAfter(deviceTime, actedAt, clock, bounds), source: 'clock' };
```

The `source` field is there so every decision can say which kind of answer it
got. An operator reading the log should be able to see when the system is
relying on an estimate, because that is when the declared bounds matter and
nobody remembers declaring them.

## What you can actually download

Having worked out the shape, the honest next question is whether somebody has
already built it. Mostly, no. The pieces exist and each is only part of it.

[Eclipse Ditto](https://eclipse.dev/ditto/basic-feature.html) comes closest. It
is an open-source digital-twin framework that separates reported state, which
comes from the device, from desired state, which a backend sets as the target,
and persists both. Its documentation then says the reconciling logic "is up to
your application or device firmware". Ditto holds the two numbers and will not
drive the device toward one of them. That is a reasonable place to draw a line,
and it means the loop is yours.

[Azure IoT Plug and Play](https://learn.microsoft.com/azure/iot-pnp/concepts-convention)
specifies the counter properly, and reading it was mildly deflating. The desired
section carries a `$version` the hub manages. The device confirms with a
reported property carrying `av`, an acknowledgment version matching that
`$version`, and `ac`, a status code: 200 accepted, 400 or 500 refused, and 202
for an update still in progress that should later finish at 200. The hub mints
the counter, not the client, which is the correction I needed. And 202 is a
state I did not have: a device committing to "working on it" rather than leaving
you to infer from silence. It confirms nothing, and that is not its value. A
command that times out having only been acknowledged and one that times out
after the device said it was working are different faults with different owners,
and a timeout that cannot tell them apart sends somebody to read the wrong logs.

Z-Wave solves the acknowledgement half in the wire format. A SET sent with the
[Supervision command class](https://docs.silabs.com/z-wave/7.21.3/zwave-api/supervision)
is answered with a status saying whether the command was understood *and
executed*, and
[zwave-js](https://github.com/zwave-js/node-zwave-js/discussions/6515)
separates never-sent from received-but-not-executed from invalid-value. It also
documents the staleness trap, in a library note: a GET issued immediately after
a SET returns a transitional value on some devices.

[Temporal](https://temporal.io/blog/ignition-flashing-2-300-conference-badges-with-temporal)
is the right shape for the durable half, running one state machine per physical
device and surviving devices that reboot mid-process. On the charger side
[EVerest](https://github.com/EVerest/everest) is the serious open stack.

What none of them ship is the loop: ranking evidence sources, holding a
deadline, and treating "cannot tell" as a first-class outcome rather than a
boolean somebody guesses at.

## Within one channel, timestamps order perfectly

The cross-clock estimate is lossy, so the next question is how little of the
system has to depend on it. The answer turned out to be: one place.

"Is there anything new from the device?" is a question entirely inside the
telemetry channel. Both timestamps being compared come from the device's clock,
so the comparison is exact, with no skew term and no third outcome. That only
works if you keep the device's own timestamp around instead of normalising
everything into local time on the way in, which is what the first version did.

So every phase of the state machine carries a watermark: the device timestamp of
the newest capture it has already accounted for. The term is borrowed from stream
processing, where a watermark is a monotonic high-water mark of event time used
to decide when enough has arrived to act, and the reason for borrowing it is the
same. Out-of-order delivery is normal. Feeds retry, and an older capture can
land after a newer one. Without a monotonic watermark that late arrival looks
like news, and a reconciler reads it as the device having drifted back and sends
a command to fix a state that was already fixed.

The division ends up clean. Within the telemetry channel, exact comparisons.
Within the command channel, exact matching by command id. Across the two, one
audited estimate, in one function, with its assumptions in its signature.

## This is a tension, and the bounds are the number

[Some Forces Do Not Resolve](/articles/some-forces-do-not-resolve/) argued that
a force ends in a mechanism and a tension ends in a number. This is a tension,
and it took me a while to stop looking for the mechanism.

The force is that you must know whether a command took effect, because a user is
waiting and a silent failure is the worst outcome. The opposing force is that
concluding it took effect requires evidence you may not have, and concluding it
failed sends another command into a system that was already retrying. No design
makes both complaints go away. What ships is a position on two dials: the skew
and latency bounds, which decide how often the loop is willing to conclude
anything, and the command timeout, which decides how long a user waits before
being told the truth.

Tighter bounds decide more often and are wrong more often. A longer timeout
reports fewer false failures and makes the user wait longer for a real one.
Neither belongs in a constant inside the decision function, because neither is a
technical fact. The timeout is a promise about how long somebody stares at a
spinner. The bounds are a claim about a fleet. Both belong to whoever knows the
fleet, so both are parameters, and the only honest thing the code can do is name
them and show which one it used.

## The charger was charging and I reported that I could not tell

The hardest part of this to argue with is a pair of simulated scenarios whose
worlds are identical. Same capture interval, same delivery latency, same
actuation delay, same command. The only difference is whether the device reports
the generation it last applied, and a test asserts they differ in nothing else.

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
`FAILED` at five minutes, because under the declared bounds no snapshot can ever
be *proved* to post-date the command, and the acknowledgement it did get proves
only that the instruction arrived.

That is a false negative, and it is the correct output. The alternative is a
system that reports success whenever the evidence is ambiguous, which is a lie
that looks like a feature until the first time somebody's car did not charge
overnight. The second scenario is the same world with a device that counts, and
the ambiguity disappears entirely.

The honest summary of the whole exercise is that the counter is not an
optimisation. It is the difference between a system that can verify its own
commands and a system that can only hope.

## What to show someone who is waiting

All of which raises a question the state machine does not answer. If the system
genuinely does not know, what does the interface say?

Home Assistant has the vocabulary for this and arrived at it years ago. It
separates two ideas that look alike. `assumed_state` means no state report ever
arrives from this device, and the interface then shows separate on and off
buttons rather than a toggle, because a toggle is a claim that you know the
current state and are flipping it. `optimistic` means show the new state the
moment a command is sent, without waiting. A core developer's phrasing is
better than mine: "optimistic != assumed. Assumed: we never get a state
reported." Their MQTT entities run optimistically when no state topic exists and
wait for confirmation when one does, and the Homematic integration goes further,
reverting after a thirty-second timeout.

Two things change when this sits on a reconciler that already models its own
uncertainty. `assumed_state` stops being configuration. The loop knows whether
anything from a device has ever been placeable in time, and a device that has
never produced a correlatable reading is exactly one whose state you never
really get, so the flag is derived and cannot drift out of date the way a
hand-written one does. `optimistic` stays configuration, because nothing derives
it. It is a judgement about which lie is worse, and the answer is not a property
of the hardware: showing "charging" a few seconds early is harmless on a
charger, and showing "locked" unconfirmed is not acceptable on a door at any
latency.

The deadline is what makes the interface honest rather than merely cautious.
Because the loop holds one, the view carries a countdown instead of a spinner
with no end:

```
ui @0:10 (careful   )  not charging [unconfirmed, toggle], confirming start (290s left)
ui @0:10 (optimistic)  charging     [unconfirmed, toggle], confirming start (290s left)
ui @10:00 (final)      charging     [confirmed-exact, toggle]
```

Same reconciler, same evidence, two interfaces saying different things, which is
exactly where a configuration knob belongs. And only a device that counts earns
`confirmed-exact`, which puts the value of the counter in front of a user rather
than leaving it in a log nobody reads.

## What the invariants caught that I did not

The eight properties this loop must satisfy are written as executable
postconditions rather than as tests. Never issue into a channel that cannot be
reached. Never claim convergence without evidence. Never fail except at a
deadline or on a stated rejection. Never reuse a generation. They run on every
decision of every scenario and every one of six hundred generated worlds, which
is a different thing from running on the dozen cases a unit test constructs.

Three bugs came out of that. None were the ones I was looking for.

The first was in a contract rather than in the code. "Claiming convergence
requires evidence" had been written as a check on the resulting phase, so a loop
sitting quietly in a converged state, with nothing new to look at, tripped it.
The property is about the *claim*, not the phase. A claim is exactly a
transition that moves the watermark.

The second was a contract that had drifted into being stricter than the decision
it checked. It applied the cross-clock rule to a re-confirmation the decision
had correctly made from the exact same-clock watermark, and rejected a sound
answer. There is a comment in that file saying a contract which reimplements the
rule it checks will drift from it and then check nothing. The comment was right.
I had done it anyway, in the file where the warning is written.

The third was real. Converging from the initial state, where the device already
happens to match the intent, stamped the phase with a generation number that had
never been issued to anything. The next genuine command then appeared not to
advance the counter. The absence of a generation belonged in the type, not
papered over with a zero.

The property that found the first two is the one to keep if only one could be
kept. It tests no output at all. It is a loop over generated worlds asserting
that the system's own stated rules hold at every step, and it fails on whichever
world lands on the boundary first.

## References

**The two channels, in specifications**

1. Open Charge Alliance, OCPP 1.6 (released 2015) — `RemoteStartTransaction.req`
   / `.conf`: the charge point replies with a status indicating whether it has
   accepted the request and will attempt to start a transaction. Actual state
   arrives as `StartTransaction` and `StatusNotification`. The specification is
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

**Reading a clock you do not own**

6. L. Lamport, *Time, Clocks, and the Ordering of Events in a Distributed
   System*, Communications of the ACM 21(7):558-565, 1978 — logical clocks as
   counters "with no actual timing mechanism", and happened-before as a partial
   order.
   https://doi.org/10.1145/359545.359563

7. F. Cristian, *Probabilistic clock synchronization*, Distributed Computing
   3:146-158, 1989 — reading a remote clock across unbounded message delay
   yields an interval, the midpoint minimises the maximum error, and a
   successful read tells you the precision it achieved.
   https://doi.org/10.1007/BF01784024

8. J. C. Corbett et al., *Spanner: Google's Globally-Distributed Database*,
   OSDI 2012 — TrueTime returns `[earliest, latest]` rather than an instant, and
   commit wait pays for the uncertainty in latency instead of hiding it.
   https://research.google/pubs/spanner-googles-globally-distributed-database-2/

9. D. L. Mills et al., RFC 5905, *Network Time Protocol Version 4: Protocol and
   Algorithms Specification*, section 10, *Clock Filter Algorithm* — the window
   of samples from which one is selected. NTPsec's documentation states the
   selection plainly: the sample with minimum delay "generally represents the
   most accurate offset statistic".
   https://www.rfc-editor.org/rfc/rfc5905 and
   https://docs.ntpsec.org/latest/stats.html

10. T. Akidau et al., *The Dataflow Model: A Practical Approach to Balancing
    Correctness, Latency, and Cost in Massive-Scale, Unbounded, Out-of-Order
    Data Processing*, PVLDB 8(12):1792-1803, 2015 — watermarks as a bound on
    event-time completeness, and the cost of deciding too early or too late.
    https://www.vldb.org/pvldb/vol8/p1792-Akidau.pdf

**My own**

11. [Some Forces Do Not Resolve](/articles/some-forces-do-not-resolve/) — the
   force-and-tension distinction this article leans on, and the Kafka `acks`
   example of being told yes about something that did not happen.

12. [Who Pays for the Force](/articles/who-pays-for-the-force/) — on forces from
   inside an implementation escaping into a contract, which is what a skew bound
   does if you hide it in a constant.
