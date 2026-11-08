+++
title = "Two Channels, Two Clocks"
date = "2026-11-07"
description = "A command acknowledgement does not prove a charger started. Device telemetry and command timestamps cannot establish that link either. A generation counter carried from desired state into the device report gives the reconciler the evidence it needs."
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
The charger runs firmware—the software inside the appliance—which decides
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
explicit about it. OCPP is the Open Charge Point Protocol, used for communication
between charging stations and their management systems. The charge point replies
to `RemoteStartTransaction` with a
status indicating whether it has accepted the request and *will attempt* to
start a transaction. The transaction itself shows up later as a
`StartTransaction`, and the connector's actual state as a `StatusNotification`.
Those can diverge. There are public reports of chargers answering `Accepted`
and then sending `StatusNotification` with `SuspendedEV`, with no
`StartTransaction` at all.

I have written this failure before, one layer down. The Kafka producer in [Some
Forces Do Not Resolve](/articles/some-forces-do-not-resolve/) can be told a
write succeeded as soon as the partition leader has it, and the leader can fail
before any follower does. The producer is told yes about something that did not
happen. A charger that acknowledges a start it never performs is the same
sentence with different hardware, and it is worse in one specific way: Kafka at
least hands you a single ordered log to reconcile against. Here the thing that
would confirm the command arrives somewhere else entirely, on a clock you do
not own.

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

Separating local and device timestamps in the type system helps catch
accidental mixing, but it does not supply the missing evidence. The report
needs to identify which desired-state update the device acted on.

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

## The charger was charging and I reported that I could not tell

Two scenarios from the original clock-based implementation show why I want the
protocol to carry a generation. They use the same capture interval,
delivery latency, actuation delay, and command. The only difference is whether
the device reports the generation it last applied, and a test asserts they
differ in nothing else.

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

6. L. Lamport, *Time, Clocks, and the Ordering of Events in a Distributed
   System*, Communications of the ACM 21(7):558-565, 1978 — logical clocks as
   counters "with no actual timing mechanism", and happened-before as a partial
   order.
   https://doi.org/10.1145/359545.359563

**My own**

7. [Some Forces Do Not Resolve](/articles/some-forces-do-not-resolve/) — the
   force-and-tension distinction this article leans on, and the Kafka `acks`
   example of being told yes about something that did not happen.

8. [Who Pays for the Force](/articles/who-pays-for-the-force/) — on forces from
   inside an implementation escaping into a contract, including the deadline
   a user must wait for confirmation.
