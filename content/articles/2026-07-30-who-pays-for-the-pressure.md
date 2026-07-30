+++
title = "Who Pays for the Pressure"
date = "2026-07-30"
description = "Bad API design is rarely bad taste. It is pressure correctly identified and incorrectly located — real forces from inside the implementation escaping into the contract, where every caller pays for them forever. Read through Google Cloud IoT Core, Datadog monitors, and the designs that got the same problems right."
tags = ["API Design", "Architecture", "Complexity"]
+++

Most bad APIs I have read were not designed carelessly. They were designed by
people who understood their system extremely well, and then wrote that
understanding down in public.

That is the failure. Not ignorance — fluency. The team knows which service
owns the routing table, knows the connection has to be live before a command
can land, knows the alert engine evaluates on ingestion time and not event
time. All of that knowledge is true, hard-won, and usually expensive. It then
leaks into the resource hierarchy, the URL, the status codes, and the field
names, and from that point on every client has to learn it too.

I have written twice about
[architecture following pressure](/articles/architecture-must-follow-pressure/)
and about
[what makes a boundary worth its cost](/articles/risk-complexity-and-pressure/).
This is the same argument pointed at the narrowest, most permanent surface a
system has.

> Pressure should shape the implementation. It should not shape the contract.

An interface is the one part of a system that cannot be refactored on a
Tuesday. Whatever complexity you put there, you have not solved — you have
distributed it, at n copies, to people who cannot fix it.

## The test

John Ousterhout's measure of a module is the ratio between what it hides and
what it charges. For an API the same question has a sharper form, because the
caller is a stranger:

> Does this field exist because the caller needs it, or because we do?

Every leak below survives that question badly. In each case the underlying
pressure is real — I want to be precise about that, because "this API is bad"
is a cheap sentence and usually a wrong one. Intermittent connectivity is
real. Ingestion lag is real. Multi-tenant isolation is real. The designs
below identified the force correctly. They just put the response in the
wrong place.

## Leak one: topology in the path

[Google Cloud IoT Core](https://github.com/googleapis/googleapis/blob/master/google/cloud/iot/v1/resources.proto)
addressed a device like this:

```text
projects/{project}/locations/{location}/registries/{registry}/devices/{deviceId}
```

A registry is a grouping of devices sharing configuration: the MQTT and HTTP
bridge settings, the CA certificates used to verify device credentials, the
Pub/Sub topics that telemetry and state land on. It is a genuinely useful
internal construct — and by the letter of the schema it is also part of the
device's name, because `Device.id` is documented as unique *within a device
registry*, not within a project.

So the address is honest about the model. My objection is to the model.

<figure class="diagram">
<svg viewBox="0 0 620 132" role="img" aria-label="A resource path split into four segments: projects, locations, registries and devices. The projects and devices segments are outlined and labelled known to the caller. The locations and registries segments are filled and labelled must be looked up. A caption notes the device has one identity while the address requires four.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">ONE DEVICE, FOUR COORDINATES</text>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="0" y="28" width="140" height="30" fill="none" stroke="var(--line)"/>
    <text x="70" y="47" fill="var(--ink)" text-anchor="middle">projects/{project}</text>
    <rect x="150" y="28" width="150" height="30" fill="var(--signal)"/>
    <text x="225" y="47" fill="var(--paper)" text-anchor="middle">locations/{location}</text>
    <rect x="310" y="28" width="150" height="30" fill="var(--signal)"/>
    <text x="385" y="47" fill="var(--paper)" text-anchor="middle">registries/{registry}</text>
    <rect x="470" y="28" width="150" height="30" fill="none" stroke="var(--line)"/>
    <text x="545" y="47" fill="var(--ink)" text-anchor="middle">devices/{deviceId}</text>
  </g>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">
    <text x="70" y="76">the caller knows this</text>
    <text x="545" y="76">the caller knows this</text>
  </g>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="middle">
    <text x="305" y="76">the platform knows these</text>
  </g>
  <path d="M150 88 L460 88" stroke="var(--signal)" stroke-dasharray="3 3" fill="none"/>
  <text x="0" y="112" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">an operational decision &middot; welded into identity</text>
  <text x="620" y="112" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">and it is in every request forever</text>
</svg>
<figcaption>The two shaded segments describe where the platform put the device. Because a device id is only unique inside a registry, they are also part of its name — which is the objection, not the defence. A client holding a serial number still cannot construct this URL.</figcaption>
</figure>

Consider what a fleet operator actually has: a serial number, a VIN, an IMEI —
one identifier, printed on the hardware. To call the API it must also know
which region and which registry that device was provisioned into, and it must
keep knowing, correctly, forever. That mapping is a lookup table the platform
already owns and every client now has to keep a copy of.

And here is the detail that gives the game away. The same schema defines
`Device.num_id`, described as a server-defined unique numeric ID that is
globally unique. A flat identifier existed. It was minted for every device,
and it was read-only — you could observe it, but you could not address
anything with it. The platform kept the simple identity for itself and handed
callers the compound one.

Nor is the compound one correctable later. There is no move operation;
re-homing a device into a different registry means deleting it and creating it
again. An operational decision about grouping is therefore also a decision to
invalidate every path any client has stored.

The correction is not to delete registries — they contain something real, a
shared configuration and credential domain, which is exactly the bar a
boundary has to clear. The correction is to stop making them a *coordinate*.
Devices get one stable, project-scoped identifier — the platform was already
minting one — and the registry becomes a mutable field on the device rather
than a path segment you must resolve before you can ask a question.

The smell test I would apply: if part of an address encodes an operational
decision rather than a fact about the thing, expect to regret it. Regions,
shards, clusters, tenancy tiers and grouping constructs are all choices
somebody will eventually want to revise. Identity is the one thing that is not
supposed to be revisable — so whatever you weld into it stops being revisable
too.

## Leak two: a status the caller cannot act on

Same API, and now the more instructive failure. To send a command to a device:

```text
POST …/devices/{deviceId}:sendCommandToDevice
```

The
[method's own contract](https://github.com/googleapis/googleapis/blob/master/google/cloud/iot/v1/device_manager.proto)
states it: if the command could not be delivered, the method returns an error,
and in particular, if the device is not subscribed, it returns
`FAILED_PRECONDITION`. Being subscribed requires being connected over MQTT
right now.

The pressure behind that is completely legitimate: the device is on a cellular
link, in a basement, asleep, or driving through a tunnel. The platform cannot
promise delivery. Reporting that honestly is better than lying.

But look at what the contract does with it. The client asked to unlock a door.
It received an error whose only actionable content is *try again at some
unspecified later time, forever, until it works*. So every serious client
builds the same thing: a retry loop, a backoff policy, a queue of pending
commands, a decision about how long a command stays meaningful, and a way to
avoid double-executing when a retry races a late success.

That is a durable command queue. Every client writes one. Each writes it
slightly differently, and each gets the idempotency slightly wrong, because
the state that would make it correct lives on the server.

Compare
[AWS IoT Jobs](https://docs.aws.amazon.com/iot/latest/developerguide/iot-jobs-lifecycle.html),
which faces exactly the same physics and answers it differently. A job
execution is a resource with a lifecycle — `QUEUED`, `IN_PROGRESS`, terminal
states. Submitting one succeeds whether or not the device is awake, and the
device retrieves its pending executions when it reconnects, ordered with
in-progress work ahead of queued work. Offline is not an error; it is a
duration.

The difference is not effort. Both teams understood intermittent connectivity
perfectly. One of them turned that understanding into a resource; the other
turned it into a 400-class response and let the callers deal with it.

I made a similar argument about
[DTLS connection IDs](/articles/the-lookup-that-saves-the-handshake/): the fix
there was to stop anchoring state on an identifier the protocol did not own —
a NAT-assigned address — and anchor it on one it did. This is the same move at
a different layer. `FAILED_PRECONDITION` anchors the client's model on
*whether a socket happens to be open right now*, which is the single most
volatile fact in the system and the one the client has least ability to
observe or influence.

If a caller cannot repair, resume, or route around a state, exposing that state
is not an API feature. It is observability wearing an API's clothes.

## Leak three: the string that is really a schema

Now [Datadog](https://docs.datadoghq.com/monitors/configuration/), which is a
genuinely good product with an alerting API that illustrates the next two
leaks better than anything I could invent.

A metric monitor is defined by a query in a string DSL:

```text
avg(last_5m):avg:system.cpu.user{host:web-01} > 0.75
```

Compact, readable, and pleasant to type. It is also a complete little language
— aggregation, window, scope, filter, comparison, threshold — with no schema
that any tooling can see. An OpenAPI spec can say `type: string`. That is all
it can say.

The consequence shows up in a place you would not predict. The monitor object
*also* carries `options.thresholds`, and Datadog's
[API options guide](https://docs.datadoghq.com/monitors/guide/monitor_api_options/)
describes the arrangement plainly: the critical threshold is defined in the
query, but can also be specified in that option — while the warning threshold
can *only* be set there. One value has two homes; its sibling has one; keeping
them consistent is left to whoever is holding the JSON. Practitioners have
reported for years that a mismatch is rejected with `Alert threshold (X) does
not match that used in the query (Y)`, which is a cross-field invariant
somebody had to implement by hand, in the server, because no schema validator
can reach inside a string.

<figure class="diagram">
<svg viewBox="0 0 620 148" role="img" aria-label="A monitor definition with two fields. The query field contains a string ending in greater than 0.75. Below it the options thresholds critical field contains 0.75. A curved arrow connects the two values, labelled must agree, with a note that the API returns 400 when they disagree.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">ONE VALUE, TWO HOMES</text>
  <g font-family="var(--font-mono)" font-size="9">
    <text x="0" y="42" fill="var(--muted)">query</text>
    <rect x="88" y="28" width="532" height="24" fill="none" stroke="var(--line)"/>
    <text x="98" y="44" fill="var(--ink)">avg(last_5m):avg:system.cpu.user{host:web-01} &gt; </text>
    <rect x="356" y="30" width="34" height="20" fill="var(--signal)"/>
    <text x="373" y="44" fill="var(--paper)" text-anchor="middle">0.75</text>
    <text x="0" y="110" fill="var(--muted)">options</text>
    <rect x="88" y="96" width="532" height="24" fill="none" stroke="var(--line)"/>
    <text x="98" y="112" fill="var(--ink)">thresholds.critical = </text>
    <rect x="215" y="98" width="34" height="20" fill="var(--signal)"/>
    <text x="232" y="112" fill="var(--paper)" text-anchor="middle">0.75</text>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M373 56 L373 80 L232 80 L232 92"/>
    <path d="M228 86 L232 92 L236 86"/>
  </g>
  <text x="420" y="76" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">the same number, twice</text>
  <text x="0" y="142" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">a schema could have enforced this &middot; a string cannot</text>
</svg>
<figcaption>Two representations of one fact, kept in step by hand-written server-side validation. This is the recurring tax on stringly-typed contracts: invariants a type system would have carried for free become runtime errors somebody has to write, document, and explain.</figcaption>
</figure>

[Composite monitors](https://docs.datadoghq.com/monitors/types/composite/)
take it further. Their query references other monitors by numeric ID inside
the same string:

```text
1234 && 5678
```

Those are foreign keys in a text field. Nothing in the schema knows they are
references, so nothing can enforce referential integrity, cascade a rename, or
render the dependency in a UI without parsing the expression first. The documented limits — at most ten constituents, no nesting
of composites — read like parser and evaluator constraints surfaced as API
rules, which is what tends to happen when the expression language and the
resource model are the same field.

And there is a semantic gap directly caused by the split: silencing a
constituent monitor does not silence the composite, because a composite is
configured independently of the monitors it names. That is defensible
behaviour and it is documented. It is also exactly the kind of surprise you
get when a relationship is expressed as text rather than as structure —
the system cannot reason about a reference it never modelled.

The alternative is not exotic. A typed condition tree:

```json
{
  "all": [
    { "signal": "system.cpu.user",
      "scope": { "host": "web-01" },
      "aggregation": { "function": "AVG", "window": "PT5M" },
      "operator": "GREATER_THAN",
      "threshold": { "value": 0.75, "unit": "RATIO" } },
    { "monitorRef": "mon_5678", "state": "ALERT" }
  ]
}
```

Verbose, and that is a real cost — the DSL is genuinely nicer to type. But the
threshold exists once. The reference is a reference. Editors autocomplete it,
generated clients model it, a linter catches a deleted dependency, and
renaming a signal is a mechanical change rather than a string rewrite across
every monitor in the account.

A DSL is a fine thing to *offer*. It stops being fine when it is the only
representation, because then the structure exists — it always exists — but
only inside the server's parser, where no client can reach it.

## Leak four: one noun, six lifecycles

Stay with the Datadog monitor and read what it contains.

There is a condition. There is a `message`, a free-text field, inside which
[notification targets](https://docs.datadoghq.com/monitors/notify/) are
written as `@slack-team-channel` or `@pagerduty-service` — routing embedded in
prose. That same field carries conditional templating, `{{#is_alert}}` and
friends, so it is also a rendering layer. Then, from the
[options list](https://docs.datadoghq.com/monitors/guide/monitor_api_options/):
`renotify_interval` and `renotify_occurrences`, a delivery retry policy;
`notify_no_data` and `no_data_timeframe`, an absence policy; then
`new_group_delay`, `evaluation_delay`, `timeout_h`, auto-resolve.

One resource, six independently changing concerns, one lifecycle. Rotate the
on-call rotation and you edit a prose field on every monitor that mentions the
old team. Change the escalation policy and you edit the alerting rules.

<figure class="diagram">
<svg viewBox="0 0 620 210" role="img" aria-label="Left, a single block labelled monitor containing six stacked concerns: condition, notification targets in prose, message template, renotify policy, no-data policy and evaluation delay. Right, the same concerns split across two blocks: an alerting rule holding the condition, and Alertmanager holding routing, grouping, repeat interval and inhibition, with a labelled seam between them.">
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="0" y="12">ONE RESOURCE</text>
    <text x="340" y="12">TWO OWNERS</text>
  </g>
  <rect x="0" y="26" width="270" height="152" fill="none" stroke="var(--ink)" stroke-width="1.5"/>
  <text x="135" y="44" font-family="var(--font-mono)" font-size="10" fill="var(--ink)" text-anchor="middle">MONITOR</text>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">
    <text x="135" y="68">condition (string DSL)</text>
    <text x="135" y="88">@targets, inside prose</text>
    <text x="135" y="108">message template</text>
    <text x="135" y="128">renotify policy</text>
    <text x="135" y="148">no-data policy</text>
    <text x="135" y="168">evaluation delay</text>
  </g>
  <rect x="340" y="26" width="280" height="62" fill="none" stroke="var(--line)"/>
  <text x="480" y="44" font-family="var(--font-mono)" font-size="10" fill="var(--ink)" text-anchor="middle">ALERTING RULE</text>
  <text x="480" y="68" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">expr &middot; for &middot; labels</text>
  <g stroke="var(--signal)" fill="none">
    <path d="M480 88 L480 110 M476 104 L480 110 L484 104"/>
  </g>
  <text x="492" y="104" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">labels, not names</text>
  <rect x="340" y="116" width="280" height="62" fill="none" stroke="var(--line)"/>
  <text x="480" y="134" font-family="var(--font-mono)" font-size="10" fill="var(--ink)" text-anchor="middle">ALERTMANAGER</text>
  <text x="480" y="158" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">route &middot; group_by &middot; repeat_interval</text>
  <text x="0" y="200" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">change the rota &middot; edit every rule</text>
  <text x="620" y="200" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">change the rota &middot; edit one route</text>
</svg>
<figcaption>The same six concerns, cut along a different seam. On the right, a rule states a fact about the system and attaches labels; routing decides who hears about it. Neither side needs to know the other's contents.</figcaption>
</figure>

Prometheus splits it. An
[alerting rule](https://prometheus.io/docs/prometheus/latest/configuration/alerting_rules/)
holds `expr`, `for`, and labels — a statement that something is true, with no
opinion about who should hear it.
[Alertmanager](https://prometheus.io/docs/alerting/latest/configuration/) owns
`route`, `group_by`, `group_wait`, `group_interval`, `repeat_interval`,
receivers, and inhibition rules. The join is by label match, not by a name
written into a message body.

The result is that the two halves change on their own clocks and belong to
different people. An SRE tunes grouping without touching a threshold. A
service team tightens a threshold without knowing which Slack channel exists
this quarter.

This is the deep-module argument applied to composition rather than to
implementation. The question is not only "does this interface hide a lot?" but
"is what it hides one thing?" A noun that owns six lifecycles cannot be
versioned, permissioned, or reused along any of them.

## Leak five: the semantics nobody wrote down

The last one is the quietest, and the one that produces incidents rather than
irritation.

Datadog's `evaluation_delay` is a number of seconds to wait before evaluating
a window, and the
[configuration documentation](https://docs.datadoghq.com/monitors/configuration/)
recommends around a fifteen-minute delay for cloud metrics. It exists for a
completely real reason: metrics arrive late, and evaluating a window before
its data has landed produces false alarms.

But look at what the field is. It is an implementation timer, exposed raw. The
underlying question is semantic — *does this rule evaluate on the time the
event happened or the time we received it?* — and the API answers it with a
tuning knob and a suggested value. The caller is told how long to wait
without being told what they are waiting for.

Every event-evaluating API owes its callers answers to a specific list, and
most of them answer none of it in the schema:

* is a condition evaluated on event time or ingestion time;
* what happens to data that arrives after its window closed;
* is the rule edge-triggered on entering the matching state, or level-triggered
  while it remains true;
* what does a stale signal mean — matching, not matching, or unknown;
* is delivery at-least-once, and if so what field do I deduplicate on;
* does editing a rule affect events already evaluated under the old version.

None of these are exotic. All of them determine whether a correct-looking rule
produces correct-looking alerts. And each one that is missing from the schema
gets discovered the same way: at 3am, by someone reasoning backwards from a
notification that should not exist.

The remedy is to name the behaviour instead of the mechanism. Rather than a
delay in seconds:

```json
{ "evaluation": { "timeBase": "EVENT_TIME", "latenessTolerance": "PT15M" },
  "notificationPolicy": { "mode": "ON_TRANSITION_TO_MATCHING" } }
```

versus the level-triggered variant:

```json
{ "notificationPolicy": { "mode": "WHILE_MATCHING", "repeatInterval": "PT10M" } }
```

The second form is barely longer than a bare integer and it states a policy
rather than a timer. It also survives an implementation change: the day the
engine stops using a fixed delay, `latenessTolerance` still means what it
said, and `evaluation_delay` does not.

And when the event finally goes out, it should carry enough time to be
reconciled against:

```json
{ "id": "evt_01J…",
  "ruleId": "mon_5678",
  "ruleRevision": 4,
  "occurredAt": "2026-07-29T09:30:00Z",
  "observedAt": "2026-07-29T09:30:08Z",
  "state": "MATCHED" }
```

The gap between `occurredAt` and `observedAt` is the whole subject. Collapse
them into one timestamp and a perfectly correct rule will, sooner or later,
produce an operationally misleading alert that nobody can explain afterwards
because the evidence was never recorded.

## What the leaks have in common

Read the five together and they are one failure in five costumes.

| Leak | Real pressure behind it | Where it was put |
|---|---|---|
| Topology in the path | multi-tenant sharding and config | the address |
| Unactionable status | intermittent connectivity | an error code |
| Stringly-typed rules | expressiveness, ergonomics | a text field |
| One noun, six lifecycles | ergonomics: alerting in one call | a single resource |
| Unstated semantics | ingestion lag, late data | a tuning knob |

In every row the pressure is real and the engineering behind it is competent.
In every row the response was placed on the caller's side of the boundary,
where it is permanent and multiplied.

The recurring cause is worth naming, because it is not incompetence and it is
not laziness. It is that the fastest way to ship a capability is to expose the
mechanism that implements it. The mechanism already exists. It already has
names. Publishing it costs one afternoon; designing an interface that hides it
costs a week and an argument. The bill for that afternoon arrives later, in
somebody else's budget, which is precisely why it keeps getting signed.

## The questions I would ask in review

Not a checklist so much as five ways of asking the same thing:

1. **Can the caller construct this request from what they actually hold?** If
   the address needs facts only the platform knows, the address is wrong.
2. **For every state I expose — what do I expect the caller to *do* with it?**
   If the answer is "retry indefinitely" or "nothing", it is telemetry, not
   contract.
3. **Is there a schema inside any of my strings?** If a value has grammar, the
   grammar belongs in types, or at minimum in types *as well*.
4. **Does this resource have one reason to change?** Count the roles who would
   need write access to it. More than one is the count of resources you should
   have had.
5. **If two identical events arrive out of order, what does this contract
   promise?** If the schema cannot answer, the caller will find out
   empirically.

None of this is an argument for thin APIs. It is the opposite. Hiding the
registry means the platform has to resolve it. Accepting a command for a
sleeping device means the platform has to own a durable queue. Typed condition
trees mean writing a real evaluator instead of a parser. Every one of these
corrections moves work *inward* — that is the entire point. The interface gets
smaller precisely because the system got deeper.

The pressure never disappears. It only ever gets assigned.

Design the contract, and you decide who pays. Publish the mechanism, and you
have decided too — you have just decided it is not you.

## References

Google Cloud IoT Core was retired on 16 August 2023 and its documentation site
has since gone with it. The service definition survives in the public
`googleapis` repository, which is the authoritative source for everything I
quote about it below.

**The APIs**

1. Google Cloud IoT Core, `resources.proto` — the `Device` and `DeviceRegistry`
   messages, including the `id` uniqueness comment and `num_id`.
   https://github.com/googleapis/googleapis/blob/master/google/cloud/iot/v1/resources.proto

2. Google Cloud IoT Core, `device_manager.proto` — the resource paths and the
   `SendCommandToDevice` contract, including the `FAILED_PRECONDITION`
   condition.
   https://github.com/googleapis/googleapis/blob/master/google/cloud/iot/v1/device_manager.proto

3. AWS IoT, jobs and job execution states.
   https://docs.aws.amazon.com/iot/latest/developerguide/iot-jobs-lifecycle.html

4. Datadog, monitor configuration — thresholds, evaluation delay, no-data
   handling, new group delay.
   https://docs.datadoghq.com/monitors/configuration/

5. Datadog, monitor API options — the exact option names, and the note that
   critical is defined in the query but may also be given as an option while
   warning may not.
   https://docs.datadoghq.com/monitors/guide/monitor_api_options/

6. Datadog, composite monitor type — the boolean query over monitor IDs, the
   ten-monitor limit, no nesting, and independence from constituent downtimes.
   https://docs.datadoghq.com/monitors/types/composite/

7. Datadog, monitor notifications — `@` handles and message templating.
   https://docs.datadoghq.com/monitors/notify/

8. Prometheus, alerting rules.
   https://prometheus.io/docs/prometheus/latest/configuration/alerting_rules/

9. Prometheus Alertmanager, configuration — routing, grouping, and
   `repeat_interval`.
   https://prometheus.io/docs/alerting/latest/configuration/

10. The `Alert threshold does not match that used in the query` rejection is
    not described in Datadog's own documentation; it is reported by users
    against the Terraform provider, where it has been discussed since the
    provider's first issue.
    https://github.com/DataDog/terraform-provider-datadog/issues/1

**The ideas**

11. John Ousterhout, *A Philosophy of Software Design*, Second Edition.
    Yaknyam Press, 2021.
    https://web.stanford.edu/~ouster/cgi-bin/book.php

12. Fabio Ellena, “Architecture Must Follow Pressure,” 2026.
    https://fblln.github.io/articles/architecture-must-follow-pressure/

13. Fabio Ellena, “Risk, Complexity, and Pressure,” 2026.
    https://fblln.github.io/articles/risk-complexity-and-pressure/

14. Fabio Ellena, “The Lookup That Saves the Handshake,” 2026.
    https://fblln.github.io/articles/the-lookup-that-saves-the-handshake/
