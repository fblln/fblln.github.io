+++
title = "The Environment Is Part of the Test"
date = "2026-08-08"
description = "A thousand system tests against a dozen containers, on eight cores. The decision that shapes everything downstream is not the assertion library, or whether the scenarios are written in Gherkin. It is who starts the containers — because whoever owns the environment also owns how many times it gets started, whether tests can be isolated without emptying a database, and whether a CI failure arrives with evidence or with a stack trace. Kafka answered that question a decade ago and most JVM suites still haven't."
tags = ["Testing", "Architecture", "Distributed Systems", "Kotlin"]
+++

The numbers came first, as they usually do. Roughly a thousand system tests.
A dozen containers per topology — several Spring Boot services, Kafka, MongoDB,
a pile of HTTP mocks standing in for carrier APIs, a warehouse simulator that
models pickers and conveyors. Eight cores and sixteen gigabytes
to run all of it, on a laptop and on a CI runner that is not meaningfully
larger.

None of that is a technology choice. It is a budget, and a budget has already
made several decisions on your behalf. The work is finding out which ones.

The decision that turned out to matter is not one that usually gets debated. It
is not the assertion library, not the mocking strategy, not whether the
scenarios should be readable by someone who will never read them. It is this:

> Who starts the containers?

Everything else in this article is a consequence of answering that once and then
refusing to answer it a second way somewhere else.

## Three answers, and what each one costs

**The tests own it.** A `@BeforeAll` somewhere spins up the world, usually
through Testcontainers, and the test class is the thing that knows what
infrastructure it needs. This is the default in most JVM codebases because it is
the shape that works beautifully one level down — and it does. For a single
service asking *do I speak to Kafka and Mongo correctly?*, a `KafkaContainer` in
the test class is exactly right: the dependency is small, the lifecycle is short,
and the test genuinely does own the thing it started.

At system level the same shape inverts. The environment is now nine containers
with a startup order and a readiness contract, it takes most of a minute to
become true, and it is shared by three hundred tests that have no relationship to
each other except that they need it. A test that owns its world can only be run
one way: alone, from the beginning.

**CI owns it.** The workflow file starts Compose, waits, runs a Gradle task,
uploads whatever it can find. This works until the day the failure is not
reproducible locally, at which point you discover that the environment a
developer runs and the environment CI runs are two implementations of the same
intent, drifting, and only one of them is under test.

**Nobody owns it.** There is a README with eleven commands in it. This is more
common than either of the above and it is not really a third option, it is the
first one having failed.

Kafka's own system tests take a fourth position, and have for a decade. Ducktape
is a Python runner that allocates cluster nodes, installs and starts services on
them, runs the test, collects every service's logs into a results directory, and
tears the cluster down. The test does not start Kafka. The runner starts Kafka,
and hands the test a handle to it.

That is the whole idea, and it survives the translation from remote hosts to
local containers intact:

> The test runner owns the environment lifecycle.

<figure class="diagram">
<svg viewBox="0 0 620 206" role="img" aria-label="Two stacks side by side. On the left, labelled tests own it, three outlined boxes chain downward: JUnit BeforeAll, then Testcontainers, then the whole topology. On the right, labelled the runner owns it, a solid box for the systest CLI branches down into two boxes, Compose and JUnit Platform, and JUnit Platform in turn points down to a box labelled tests, no infrastructure. A note reads: a test that owns its world can only be run one way.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">WHO OWNS THE ENVIRONMENT</text>
  <g font-family="var(--font-mono)" font-size="9" text-anchor="middle">
    <text x="140" y="34" fill="var(--muted)">TESTS OWN IT</text>
    <text x="480" y="34" fill="var(--muted)">THE RUNNER OWNS IT</text>
    <rect x="0" y="44" width="280" height="34" fill="none" stroke="var(--line)"/>
    <text x="140" y="65" fill="var(--ink)">JUNIT &middot; @BeforeAll</text>
    <rect x="0" y="98" width="280" height="34" fill="none" stroke="var(--line)"/>
    <text x="140" y="119" fill="var(--ink)">TESTCONTAINERS</text>
    <rect x="0" y="152" width="280" height="34" fill="none" stroke="var(--line)"/>
    <text x="140" y="173" fill="var(--muted)">THE WHOLE TOPOLOGY</text>
    <rect x="340" y="44" width="280" height="34" fill="var(--signal)"/>
    <text x="480" y="65" fill="var(--paper)">SYSTEST CLI</text>
    <rect x="340" y="98" width="134" height="34" fill="none" stroke="var(--line)"/>
    <text x="407" y="119" fill="var(--ink)">COMPOSE</text>
    <rect x="486" y="98" width="134" height="34" fill="none" stroke="var(--line)"/>
    <text x="553" y="119" fill="var(--ink)">JUNIT PLATFORM</text>
    <rect x="340" y="152" width="280" height="34" fill="none" stroke="var(--line)"/>
    <text x="480" y="173" fill="var(--ink)">TESTS &middot; NO INFRASTRUCTURE</text>
  </g>
  <g stroke="var(--ink)" fill="none">
    <path d="M140 78 L140 92 M136 86 L140 92 L144 86"/>
    <path d="M140 132 L140 146 M136 140 L140 146 L144 140"/>
    <path d="M407 78 L407 92 M403 86 L407 92 L411 86"/>
    <path d="M553 78 L553 92 M549 86 L553 92 L557 86"/>
    <path d="M553 132 L553 146 M549 140 L553 146 L557 140"/>
  </g>
  <text x="0" y="200" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">a test that owns its world can only be run one way</text>
  <text x="620" y="200" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">alone, from the beginning</text>
</svg>
<figcaption>The left stack is not wrong, it is misapplied. It is the correct shape for a component test inside a service repository, where the container really is a detail of the test. At system level the containers outlive every individual test, and ownership has to move up with them.</figcaption>
</figure>

## The arithmetic that settles it

There is a version of this argument that is about taste, and a version that is
about a number. The number is more persuasive.

Take a topology that reaches readiness in forty seconds and tears down in ten.
If the environment belongs to the test, a thousand tests cost a thousand
lifecycles: fifty thousand seconds, near enough fourteen hours, spent before any
assertion has executed. If the environment belongs to the runner, the same
thousand tests fall into a handful of distinct topologies — fulfilment,
inventory, returns, the full system, a few fault environments — and cost
five lifecycles. Four minutes.

Those figures are arithmetic from a stated model, not a measurement of anything
I have run. The ratio is the part that is real, and the ratio is two hundred to
one.

<figure class="diagram">
<svg viewBox="0 0 620 128" role="img" aria-label="Two bars on a common left edge, comparing the cost of starting an environment. A long pale bar labelled per test reads one thousand times compose up, readiness and down. A short solid bar labelled per group reads five times the same, with a thousand tests inside them. A note says the widths understate it, the ratio is two hundred to one.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">THE COST OF STARTING &middot; THE SAME 1,000 TESTS, THE SAME TOPOLOGIES</text>
  <text x="0" y="45" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">PER TEST</text>
  <rect x="150" y="32" width="470" height="18" fill="var(--ink)" opacity="0.08"/>
  <text x="385" y="45" font-family="var(--font-mono)" font-size="8" fill="var(--ink)" text-anchor="middle">1,000 &times; (COMPOSE UP &middot; READINESS &middot; DOWN)</text>
  <text x="0" y="79" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">PER GROUP</text>
  <rect x="150" y="66" width="60" height="18" fill="var(--signal)"/>
  <text x="220" y="79" font-family="var(--font-mono)" font-size="8" fill="var(--muted)">5 &times; THE SAME &middot; AND ALL 1,000 TESTS RUN INSIDE THEM</text>
  <text x="0" y="120" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">nothing about the tests changed</text>
  <text x="620" y="120" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">the widths understate it &mdash; the ratio is 200 to 1</text>
</svg>
<figcaption>The bars are not to scale; drawn honestly the lower one would be two pixels wide. This is the only optimisation in the design that is worth an order of magnitude, and it is available exclusively to whoever owns the lifecycle — a test cannot decline to start an environment it is responsible for.</figcaption>
</figure>

To group tests by environment, the environment has to be a value rather than a
side effect. Discovery resolves each test's declared topology to a normalised
spec — the ordered Compose files, the variables, the resolved image tags — and
fingerprints it. Tests with the same fingerprint are the same bucket. The runner
starts each bucket once.

Which means the declaration has to be readable without executing anything:

```kotlin
@Environment("fulfilment")
class PickToShipTest
```

Not a `@BeforeAll` that starts containers, because you cannot bucket a side
effect. A name, resolved by the runner, to a list of files it can hash.

## Three owners, and the seams between them

Once the runner owns the lifecycle, the rest of the system has exactly three
responsibilities, and the discipline is in refusing to let them bleed.

**Compose defines the topology.** It is the canonical description of what runs.
It knows nothing about tests.

**The harness defines lifecycle and access.** It starts and stops topologies,
waits for readiness, hands tests a typed context, collects diagnostics.

**JUnit defines test semantics.** Discovery, tags, parameterisation,
`@BeforeEach`, assertion failures, IDE integration.

The useful form of that is the negative one, because the negative one is
checkable:

- Compose files contain no testing logic.
- Test classes start no containers.
- The CI workflow contains no orchestration.
- Individual tests manage no Kafka consumers, no Mongo clients, no log files.

Each of those is a boundary that will be crossed for a good local reason —
someone needs a topic created, someone needs an extra wait, someone needs the
gateway's log — and each crossing is how the suite becomes something only its
author can run.

## A runner, not a language

The temptation at this point is to build a test framework. Do not build a test
framework.

JUnit Jupiter already provides everything a system test needs from a test
model: `@Test`, `@BeforeEach`, tags, parameterisation, parameter injection,
failure semantics, and — not a small thing when a developer is debugging at four
in the afternoon — the ability to click a green triangle in an IDE. The JUnit
Platform Launcher exposes discovery, filtering and execution as a library, which
means a Kotlin CLI can drive all of it programmatically:

```text
systest CLI  →  JUnit Platform Launcher  →  Jupiter  →  system tests
```

The CLI is a custom *runner*. It is not a custom *testing language*. That
distinction is the entire difference between this and the Cucumber suites I have
watched teams pay for: Gherkin buys you a natural-language layer that nobody
outside the team reads, in exchange for a step-definition layer that everybody
inside the team maintains. The abstraction is real, the audience is imaginary.
Meanwhile the actual problem — nobody can start the environment — is untouched,
because it was never a language problem.

So tests stay as Kotlin:

```kotlin
@Test
fun `a picked order becomes dispatchable`(ctx: HarnessContext) {
    ctx.simulator.completePick(order = ctx.ids.order())

    eventually {
        ctx.api.shipment(ctx.ids.order()).state isEqualTo READY_TO_SHIP
    }
}
```

`ctx` arrives by parameter injection from an extension the runner installed.
There is no static `Harness.INSTANCE`, because global mutable state is how a
suite discovers, three hundred tests in, that it can never be parallelised.

## Isolation is a naming problem

The instinct inherited from single-service integration testing is to make the
world empty before each test. Truncate the collections, reset the topics, start
from zero.

You cannot make this world empty. It is nine containers with in-flight consumer
offsets, scheduled work and internal caches; the best you can do is make it
briefly quiet, and it will not be quiet in the shard running beside it. Worse,
a suite that depends on emptiness has permanently forfeited concurrency — that
is a much larger cost than it looks, paid later, by someone else.

The alternative is to stop needing emptiness. Every test gets identities that
nothing else in the run can collide with, derived from the run id and the test
id:

```text
run       8f30
test      reserve-0341
order     ORD-8f30-0341-001
sku       SKU-8f30-0341
group     systest-8f30-0341
```

Tests create their own resources under those names and assert on those names.
Shared state stops being contention and becomes background. And because the same
identifier travels through the system as a header —

```http
X-Test-Id: 8f30-reserve-0341
```

— a fifty-megabyte cluster log becomes greppable by the thing you actually care
about.

Kafka needs one extra idea, because a topic is not addressable by name the way a
document is. Before the test acts, the harness records the current end offsets;
afterwards it reads only forward from that mark.

```kotlin
val mark = ctx.kafka.checkpoint("shipments")
ctx.api.reserve(order)
val event = ctx.kafka.awaitAfter<ShipmentReady>(mark, "shipments") { it.order == order }
```

Nothing is deleted. Nothing is truncated. Two tests can be publishing to the same
topic at the same moment and neither can see the other's events, because each is
reading a different window of the same log and filtering by an order id only it
knows.

<figure class="diagram">
<svg viewBox="0 0 620 148" role="img" aria-label="A single horizontal rail representing one Kafka topic partition. Faint marks sit along the left half, representing other tests' events which remain in place. A solid vertical line labelled checkpoint stands in the middle. To the right of it further marks appear, two of them highlighted as this test's events. A note reads: nothing is truncated, nothing is deleted.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">ISOLATION WITHOUT A RESET &middot; ONE TOPIC, ONE PARTITION</text>
  <line x1="0" y1="80" x2="620" y2="80" stroke="var(--line)"/>
  <g fill="var(--ink)" opacity="0.22">
    <rect x="30" y="73" width="5" height="14"/>
    <rect x="72" y="73" width="5" height="14"/>
    <rect x="114" y="73" width="5" height="14"/>
    <rect x="156" y="73" width="5" height="14"/>
    <rect x="198" y="73" width="5" height="14"/>
    <rect x="280" y="73" width="5" height="14"/>
    <rect x="322" y="73" width="5" height="14"/>
    <rect x="448" y="73" width="5" height="14"/>
    <rect x="532" y="73" width="5" height="14"/>
    <rect x="574" y="73" width="5" height="14"/>
  </g>
  <g fill="var(--signal)">
    <rect x="364" y="70" width="6" height="20"/>
    <rect x="490" y="70" width="6" height="20"/>
  </g>
  <line x1="240" y1="54" x2="240" y2="106" stroke="var(--signal)"/>
  <text x="240" y="46" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="middle">CHECKPOINT</text>
  <text x="115" y="122" font-family="var(--font-mono)" font-size="8" fill="var(--muted)" text-anchor="middle">EVERY OTHER TEST &middot; STILL THERE</text>
  <text x="430" y="122" font-family="var(--font-mono)" font-size="8" fill="var(--ink)" text-anchor="middle">READ FORWARD, FILTER BY AN ID ONLY THIS TEST KNOWS</text>
  <text x="0" y="142" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">nothing is truncated, nothing is deleted</text>
  <text x="620" y="142" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">isolation stops being a cleanup problem</text>
</svg>
<figcaption>The checkpoint is what lets a shared environment behave like a private one. It also removes the last structural reason tests must run one at a time — after which the limit on concurrency is CPU, which is an honest limit you can go and buy more of.</figcaption>
</figure>

### The tests that cannot share, and the refusal that saves you

Unique names solve the tests that only *read* the environment, which is most of
them. They do nothing for the minority that change it: restart a broker, flip a
feature flag, saturate a queue, inject a fault. Those tests are not badly
written. They are the ones testing the things that matter most, and they need
the environment to themselves.

The instinct is to serialise the whole suite so that the awkward ten percent
can't hurt anyone. JUnit already has the smaller answer:

```kotlin
@ResourceLock(ENVIRONMENT, mode = READ)        // shares, runs concurrently
@ResourceLock(ENVIRONMENT, mode = READ_WRITE)  // runs alone
```

Read-mode tests run together; a write-mode test never overlaps one. Two meta-
annotations — call them `@SharedEnvTest` and `@ExclusiveEnvTest` — and the
distinction is declared at each test rather than assumed for all of them. Both
Temporal and Strimzi arrived at this same split independently, as a pool of
shared and dedicated clusters in one and an `@IsolatedTest` annotation in the
other, which is usually a sign the distinction is in the problem rather than in
anyone's taste.

The rule that makes it hold is a refusal. When an exclusive test is asked to run
against an environment it cannot have exclusively — a reused one, a shared CI
lane — the runner must **fail with the command that would fix it**, not run
anyway. Temporal's cross-SDK runner does exactly this, rejecting configuration-
mutating variants against an external server rather than executing them wrong.
A harness that degrades quietly under a condition it can detect is worse than
one that has no opinion at all, because it converts a clear error into twenty
confusing ones somewhere else.

## `eventually`, and what a timeout owes you

`Thread.sleep(5000)` is a bet that the system is slower than nothing and faster
than five seconds, placed by someone who did not know either number. It is also
the single largest source of wall-clock time in most suites, because the number
only ever goes up: each flake adds a second, and no one ever takes one away.

Awaitility, or a thin wrapper over it, with the poll interval and timeout
defined centrally rather than per assertion:

```kotlin
eventually { ctx.mongo.orders.get(order).state isEqualTo RESERVED }
```

The wrapper earns its existence on failure, not on success. An eventual
assertion that times out with `expected RESERVED but was PENDING` has thrown away
the one moment when it knew everything: it had a Mongo connection, a Kafka
consumer positioned at the right offset, and the container logs for the window
it was waiting through. All of that belongs in the failure. A polling helper
that only reports the last comparison it made is a polling helper that made you
reproduce the bug to learn what it already saw.

## You cannot collect a log retroactively

The harness starts following container output the moment the environment is up,
not after something goes wrong:

```bash
docker compose logs --follow --timestamps --no-color
docker compose events --json
```

Both stream to disk for the lifetime of the environment. This is the piece
teams skip, and it is the piece that determines whether a CI failure is a
diagnosis or a ticket.

Record the start and end timestamp of every test. On failure, cut the cluster log
to that window plus a couple of seconds either side, split it by service, and
write it beside the result. Then the artifact directory answers the question
directly instead of inviting you to reproduce it:

```text
tests/inventory/ReserveStockTest/reservesTheLastUnit/
    result.json
    failure.txt
    logs/gateway.log  logs/inventory-service.log
    kafka/consumed.jsonl
    mongo/order.json
```

The Docker event stream is the one people leave out, and it is the one that
converts confusion into a sentence. An assertion can only ever tell you what did
not arrive. The event log tells you why.

<figure class="diagram">
<svg viewBox="0 0 620 158" role="img" aria-label="A four-second timeline with two rails. Along the top, four moments: test starts, the inventory service dies, the container restarts, the test times out. The upper rail, labelled collected always, has a mark at all four moments. The lower rail, labelled collected after failure, is dashed and empty until the final moment. A note contrasts the assertion message with the event log.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">FOUR SECONDS THAT EXPLAIN A FAILURE</text>
  <g font-family="var(--font-mono)" font-size="8" text-anchor="middle" fill="var(--muted)">
    <text x="160" y="32">13:42:31</text>
    <text x="310" y="32">13:42:32</text>
    <text x="450" y="32">13:42:33</text>
    <text x="578" y="32">13:42:35</text>
  </g>
  <g font-family="var(--font-mono)" font-size="8" text-anchor="middle" fill="var(--ink)">
    <text x="160" y="44">TEST STARTS</text>
    <text x="310" y="44">SERVICE DIES</text>
    <text x="450" y="44">RESTARTS</text>
    <text x="578" y="44">TIMES OUT</text>
  </g>
  <text x="0" y="66" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">ALWAYS ON</text>
  <line x1="110" y1="62" x2="620" y2="62" stroke="var(--line)"/>
  <g fill="var(--signal)">
    <rect x="158" y="56" width="4" height="12"/>
    <rect x="308" y="56" width="4" height="12"/>
    <rect x="448" y="56" width="4" height="12"/>
    <rect x="576" y="56" width="4" height="12"/>
  </g>
  <text x="0" y="110" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">ON FAILURE</text>
  <line x1="110" y1="106" x2="560" y2="106" stroke="var(--line)" stroke-dasharray="3 5"/>
  <line x1="560" y1="106" x2="620" y2="106" stroke="var(--line)"/>
  <rect x="576" y="100" width="4" height="12" fill="var(--ink)" opacity="0.35"/>
  <text x="335" y="124" font-family="var(--font-mono)" font-size="8" fill="var(--muted)" text-anchor="middle">THE THREE SECONDS THAT EXPLAIN IT ARE ALREADY GONE</text>
  <text x="0" y="150" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the assertion says: expected a shipment event, none arrived</text>
  <text x="620" y="150" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">the event log says: it died at :32</text>
</svg>
<figcaption>Collecting on failure is collecting too late — by the time the assertion gives up, the interesting seconds have scrolled past and the only surviving copy was in a stream nobody was reading. The cost of always-on collection is a subprocess and some disk. The cost of the alternative is measured in reproductions.</figcaption>
</figure>

### A collected log is evidence; an asserted log is a test

Once the stream exists, there is a second use for it that costs almost nothing.
Strimzi runs a matcher over the operator's log after **every** system test,
failing the test on any error or exception that isn't on a short allowlist of
known-benign messages.

That reframes the log from evidence into an assertion every test carries without
writing one. A test that satisfies its own expectations while the gateway threw
fifty null pointers in the background is not a passing test; it is a test that
wasn't looking. The allowlist is the interesting part of the design — it is a
small, reviewable file, and every entry in it is a decision someone had to
defend in a pull request. Compare that with the usual arrangement, where the
same fifty exceptions are also tolerated, but silently, by nobody, forever.

## The assertions nobody wrote

Tests assert what their author thought to assert. That is a much smaller set
than the behaviour a distributed system actually has, and the gap is where
compatibility regressions live: a field that changed type, an event emitted
twice, a new message inserted between two others, a retry that used to be
invisible.

Temporal's cross-SDK suite handles this by committing **the execution history
itself** as a fixture, then checking three things on every run: that the current
history replays, that every stored history from earlier versions still replays,
and that the current run's scrubbed events match what was recorded.

The fulfilment equivalent is a golden file of the events a scenario emits, with
ids and timestamps scrubbed:

```text
tests/inventory/ReserveStockST/reservesTheLastUnit/golden/events.jsonl
```

`--update-golden` rewrites it; code review catches the diff. It is approval
testing pointed at a message bus, and it is the only mechanism in the whole
harness that tests compatibility *between versions of your own services* rather
than the behaviour of one version. Everything else in this article asks "does it
work?". This one asks "did it change?" — and for a system where other teams
consume your events, that is the question with the longer tail.

## Sharding is the only parallelism you can afford

The obvious way to speed up a thousand tests is to run them concurrently inside
one environment. It is also the way that pays worst here, because the environment
is not a passive backdrop — those nine containers are the tenant on those eight
cores. Every test thread you add competes with the system under test, and a
slower system under test makes every timeout closer to the edge. You buy
throughput with flakiness, at a bad exchange rate.

So: two to four concurrent tests inside an environment, and scale horizontally
instead. Each CI shard gets its own complete environment, and shard assignment is
computed, never curated:

```text
shard(testId) = hash(testId) % shardCount
```

Stable for a given shard count, requiring no list, surviving renames of
everything except the test itself. Curated shard lists exist in a lot of CI
configurations and they all have the same defect: they are a manual index of a
thing that changes daily, so they are wrong within a week and nobody notices
until the coverage gap becomes an incident.

Which makes the entire CI integration one line:

```yaml
- name: System tests
  run: ./systest run fulfilment --shard ${{ matrix.shard }}
```

Anything more in that file is orchestration a developer cannot run.

## It still has to be startable by hand

Two properties are worth protecting even when they cost something.

The first is that the topology remains inspectable without the harness. Compose
overlays — a `base.yml`, a stack per domain, a feature per optional component, a
fault injection file — compose in the literal sense, and the environment the
runner starts is the environment you get from typing the same files yourself:

```bash
docker compose -f compose/base.yml -f compose/stacks/fulfilment.yml up
```

A harness that is the only way to start the system is a harness you will be
debugging instead of debugging the system.

The second is that the versions under test are recorded rather than implied.
`image: service-a:latest` is not a version, it is a timestamp you forgot to
write down; it makes a CI failure from Tuesday unreproducible on Wednesday for
reasons that have nothing to do with the code. Resolve the tags up front and
write the manifest into the results directory next to the logs, so a failure
always carries the answer to *what exactly was running?*

Strimzi does one better with that manifest, and it is worth copying exactly:
every resolved variable is written into the results directory in **the same
format the harness accepts as input**. The artifact you download from a failed
CI job is not a record of the run, it is the run — hand it back with
`--config` and you get the same environment, the same image versions, the same
flags. A record you can only read is a record you will misread.

## The tool has to be pleasant on a Tuesday

Everything above is about correctness and cost. The thing that actually decides
whether a suite gets used is smaller: how long it takes to try again.

If every attempt pays for a full lifecycle, the loop is a minute long and people
stop iterating locally — they push, wait for CI, and read a log. That is the
failure mode where a good harness is abandoned quietly. So the lifecycle
commands are separate from the run command:

```bash
./systest up fulfilment
./systest run --reuse-environment fulfilment --test ReserveStockTest
./systest down fulfilment
```

and a failing CI-style run can retain what it built:

```text
Environment retained because tests failed.
Compose project: systest-8f30
  ./systest status   ./systest logs   ./systest down
```

The first form makes the inner loop a second long. The second means the moment a
failure happens is not the moment its evidence is destroyed. Both are small
features, and between them they are most of the difference between a harness
people run and a harness people route around.

### The green triangle in the IDE

There is a harder version of the same requirement, and it is the one that
decides whether anybody adopts this. When a developer clicks the run gutter next
to a single test, the IDE does not invoke the CLI. It builds its own JUnit
configuration, forks a JVM, and calls the Platform Launcher directly. Nothing in
`main()` runs. If the environment lifecycle lives only in the CLI, the green
triangle is broken, and a test harness whose tests can't be run from the IDE is
a test harness people will work around rather than with.

The fix is to stop *starting* environments and start *acquiring* them:

```kotlin
val project = "systest-$fingerprint"          // deterministic, never random
composePs(project)?.let { return attach(it) } // already up: connect and go
compose(project, "up", "-d", "--wait")        // not up: start it, and remember that we did
```

registered from a `LauncherSessionListener`, which fires once per JVM session
under the IDE, under Gradle and under the CLI alike. `./systest up` stops being a
prerequisite and becomes a way to pre-pay the forty seconds in a terminal. The
teardown rule is one boolean: **tear down only what you started.**

Two details make it safe rather than merely convenient. Ports must be read back
from the running project rather than pinned, or two environments can never
coexist and the whole scheme collapses. And the fingerprint must include the
resolved image digests, not just the compose file paths — then rebuilding a
service changes the fingerprint, changes the project name, and leaves nothing to
attach to. Testing against a stale container stops being a thing you remember to
check and becomes a thing that cannot happen.

Arquillian drew this line fifteen years ago as *managed* versus *remote*
containers, and Testcontainers redrew it as reusable containers matched by a hash
of their configuration. It is a well-worn distinction and there is no credit for
rediscovering it slowly.

One last thing, small enough to be embarrassing and large enough to matter: the
moment someone sets a breakpoint, every readiness wait and every `eventually`
starts failing for reasons unrelated to the bug. Temporal ships an environment
variable for this. The JVM will tell you for free:

```kotlin
val debugging = ManagementFactory.getRuntimeMXBean()
    .inputArguments.any { it.startsWith("-agentlib:jdwp") }
```

Scale every timeout by twenty when that is true. A suite you cannot pause is a
suite you can only debug from the outside.

## What I would not build yet

The design is only as good as the things it declines to include. Not in the
first version, and possibly not ever: a Gherkin dialect, a bespoke assertion
syntax, custom test discovery, a Kubernetes orchestrator, a dynamic dependency
graph between services, container-per-test isolation, duration-weighted shard
balancing, a scheduler, a pool manager for environments you do not have the
cores to run two of.

Every one of those is a real technique that solves a real problem somewhere. None
of them is a problem this system has, and the cost of the wrong one is not the
code — it is that a suite with a bespoke test language has an owner, and a suite
with an owner has a bus factor.

The novelty here is meant to be in composition and lifecycle ownership. Compose,
JUnit, Awaitility, `kafka-clients` and the Mongo driver are all mature and all
someone else's maintenance burden. The part worth writing is the two hundred
lines that decide when the environment starts, which tests share it, and what
gets written to disk while it runs.

## What a system test actually is

A unit test is a function and an assertion. That definition is so comfortable
that it gets carried upward unexamined, and at system level it is missing two
thirds of the subject.

A system test is a function, plus the topology it ran against, plus the evidence
of what that topology did while it ran. In most suites the second is implicit,
the third does not exist, and both gaps show up in the same place — an
intermittent CI failure that nobody can reproduce and everybody has learned to
re-run.

Making all three explicit requires a single decision, which is the one at the
top of this article. Once the runner owns the environment, the rest stops being a
series of preferences: grouping becomes possible because topologies are values,
isolation becomes naming because resets are unavailable, evidence becomes
automatic because something is already watching, and CI becomes one line because
there is nothing left in it to configure.

It is also, reassuringly, not an original decision. Kafka's suite has worked
this way for a decade, Strimzi's does it on Kubernetes with a JUnit 5 suite that
looks a lot like the one described here, and Temporal does it by making the
server embeddable so the test process can own the topology outright. Three
projects, three substrates, the same answer about who holds the keys.

The environment was always part of the test. The only question is whether
anything in your system is responsible for it.

## References

**The prior art**

1. Ducktape — the system test framework behind Apache Kafka's integration
   suite; runner-owned service lifecycle, cluster allocation, per-service log
   collection.
   https://github.com/confluentinc/ducktape

2. Apache Kafka — `tests/`, a decade of system tests written against that model.
   https://github.com/apache/kafka/tree/trunk/tests

3. Strimzi — `TESTING.md` and the `systemtest` module: the same ownership model
   on Kubernetes, in JUnit 5. Source of the isolated/parallel annotations, the
   operator log matcher, and the re-runnable run configuration.
   https://github.com/strimzi/strimzi-kafka-operator/blob/main/development-docs/TESTING.md

4. Temporal — `docs/development/testing.md`, and the cross-SDK `features`
   runner. Shared and dedicated cluster pools, capability declarations, and a
   runner that refuses rather than degrades.
   https://github.com/temporalio/temporal/blob/main/docs/development/testing.md

5. Antithesis — where this idea ends up: your Compose topology, run
   deterministically under a fault-injecting hypervisor.
   https://antithesis.com/docs/getting_started/setup_guide/docker_compose/

**The tools**

6. JUnit Platform Launcher — programmatic discovery, filtering and execution;
   `LauncherSessionListener` for once-per-JVM environment lifecycle.
   https://junit.org/junit5/docs/current/user-guide/#launcher-api

7. JUnit 5 — parallel execution and `@ResourceLock`, which is the whole of the
   shared-versus-exclusive mechanism.
   https://docs.junit.org/current/user-guide/#writing-tests-parallel-execution

8. Testcontainers — the right tool one level down, at component scope; and
   reusable containers, which is attach-or-start by another name.
   https://java.testcontainers.org/features/reuse/

9. Awaitility — polling assertions with explicit timeouts.
   https://github.com/awaitility/awaitility

10. Docker Compose — merging multiple Compose files.
    https://docs.docker.com/compose/multiple-compose-files/

11. skodjob/test-metadata-generator — the annotations behind Strimzi's
    generated system-test documentation.
    https://github.com/skodjob/test-metadata-generator

**The ideas**

12. Fabio Ellena, "Architecture Must Follow Pressure," 2026.
    https://fblln.github.io/articles/architecture-must-follow-pressure/

13. Fabio Ellena, "The Runtime Is What's Left Over," 2026.
    https://fblln.github.io/articles/the-runtime-is-what-is-left-over/
