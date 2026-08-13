+++
title = "The Affordance Strikes Back"
date = "2026-08-16"
description = "A tool description is a paragraph of English carrying purpose, preconditions, side effects, exclusions and interactions with every other tool — and a study of 856 tools across 103 servers found 97% of them defective in at least one of those jobs. That is not a protocol failure; it is one prose field carrying work a schema could do. HAL-FORMS, Hydra and the W3C Web of Things all encode part of that work as data, and a Hydra-to-MCP gateway has now shown the two models can be composed. Stable verbs, dynamic affordances. Fourth of four."
tags = ["API Design", "Protocols", "Architecture", "Complexity"]
+++

*Fourth of four. [Part I](/articles/the-browser-was-never-the-smart-client/)
argued that hypermedia's successful client was a browser plus a human.
[Part II](/articles/hateoas-priced-the-wrong-change/) itemized what traversal
costs even a team that understands the domain perfectly.
[Part III](/articles/hateoas-failed-the-agent-test/) argued that when the human
was finally replaced by something intelligent, the protocol built for it
separated operation description from application state rather than fusing them.
This part is about the half of the problem that separation left open, and about
the hypermedia work that was already solving it.*

An MCP tool description of the kind that ships in production:

```json
{
  "name": "cancel_order",
  "description": "Cancel an order."
}
```

What a model needs to know before calling it runs considerably longer. Whether
cancelling is possible after fulfilment has started. Whether it refunds
automatically or whether `issue_refund` is a separate step that must follow.
Whether it is idempotent, and what happens on the second call. Whether a
`reason` is required and whether the string is validated or free text. Whether
it emails the customer. Whether the right operation for a shipped order is
actually `create_return`. What the error looks like when the order is already
cancelled, and whether that error means stop or means retry differently.

Every one of those is a real question, the answer changes what the agent should
do, and there is exactly one field in the protocol where the answer can live:
`description`, a string of English prose.

That field is doing an enormous amount of work, and we now have a reasonable
idea of how well it is doing it. Hasan and colleagues analysed 856 tools across
103 MCP servers and found that 97.1% carried at least one description defect,
with 56% failing to state their purpose clearly. They then tried the obvious
fix of augmenting the descriptions, and the result is more interesting than the
diagnosis: task success improved by a median of a few percentage points and
partial goal completion by around fifteen, while execution steps rose by
roughly two thirds and a sixth of cases got *worse*. Better prose helped, cost
more, and did not help uniformly.

As a protocol indictment that is a misreading. As a statement about interface
boundaries it is exact:

> Tool semantics encoded primarily as prose are a weak interface boundary.

Weak, not useless. Prose is how you communicate purpose and nuance to a
language model, and no schema will replace the sentence explaining that
cancelling a shipped order is the wrong instinct. But a great deal of what
those descriptions are struggling to carry is not nuance. It is *structure*:
preconditions, current validity, required arguments for this case, which
operation supersedes which in which state. Structure is exactly what
hypermedia formats have been encoding as data for fifteen years.

Part III ended by separating two questions the fourth constraint had bundled
together. MCP answers operation identity well. It does not, on its own, answer
operation availability. This part is about who does.

## What an affordance actually is

The previous three parts used "affordance" loosely to mean roughly "a link with
a verb in it," which was fine for arguing against basic HAL and is not good
enough here.

The W3C WebAgents Community Group has been working on the question directly,
and their interoperability report gives the definition I will use: an
affordance is "a relation between an agent's capabilities and the capabilities
that the environment exposes: it specifies what the agent can do and how,"
formalised as a "machine-readable, discoverable description of an available
interaction." Note what that packs in. Not just what exists — what *this* agent
can do, *how* to do it, discoverable, and machine-readable. A bare `rel` and
`href` satisfies about a third of it.

The report's more useful contribution is a taxonomy. It identifies three
interaction paradigms, and once you have them, the argument of this whole
series snaps into focus:

**Hypermedia-driven**, where agents "navigate and discover affordances at
runtime by following hypermedia controls embedded in resource
representations, requiring no prior knowledge of the environment's structure."

**Description-driven**, where agents "consume machine-readable interface
specifications before invoking affordances, relying on out-of-band or
pre-fetched descriptions."

**Protocol-driven**, where agents "use a standardized invocation protocol that
manages tool enumeration and invocation through a dedicated server."

<figure class="diagram">
<svg viewBox="0 0 620 246" role="img" aria-label="Four stacked rows. Hypermedia-driven: resource then affordance then action. Description-driven: description then known operation then action. Protocol-driven: tool list then tool call then result. A fourth filled row labelled hybrid combines a stable described vocabulary with resource-specific affordances feeding a single action. A footer notes the three paradigms are not mutually exclusive.">
  <g font-family="var(--font-mono)" font-size="9">
    <text x="0" y="12" fill="var(--muted)">HYPERMEDIA-DRIVEN</text>
    <rect x="0" y="20" width="150" height="22" fill="none" stroke="var(--line)"/>
    <text x="75" y="35" fill="var(--ink)" text-anchor="middle">resource</text>
    <rect x="180" y="20" width="150" height="22" fill="none" stroke="var(--line)"/>
    <text x="255" y="35" fill="var(--ink)" text-anchor="middle">affordance</text>
    <rect x="360" y="20" width="150" height="22" fill="none" stroke="var(--line)"/>
    <text x="435" y="35" fill="var(--ink)" text-anchor="middle">action</text>
    <text x="0" y="72" fill="var(--muted)">DESCRIPTION-DRIVEN</text>
    <rect x="0" y="80" width="150" height="22" fill="none" stroke="var(--line)"/>
    <text x="75" y="95" fill="var(--ink)" text-anchor="middle">description</text>
    <rect x="180" y="80" width="150" height="22" fill="none" stroke="var(--line)"/>
    <text x="255" y="95" fill="var(--ink)" text-anchor="middle">known operation</text>
    <rect x="360" y="80" width="150" height="22" fill="none" stroke="var(--line)"/>
    <text x="435" y="95" fill="var(--ink)" text-anchor="middle">action</text>
    <text x="0" y="132" fill="var(--muted)">PROTOCOL-DRIVEN</text>
    <rect x="0" y="140" width="150" height="22" fill="none" stroke="var(--line)"/>
    <text x="75" y="155" fill="var(--ink)" text-anchor="middle">tool list</text>
    <rect x="180" y="140" width="150" height="22" fill="none" stroke="var(--line)"/>
    <text x="255" y="155" fill="var(--ink)" text-anchor="middle">tool call</text>
    <rect x="360" y="140" width="150" height="22" fill="none" stroke="var(--line)"/>
    <text x="435" y="155" fill="var(--ink)" text-anchor="middle">result</text>
  </g>
  <g stroke="var(--line)" fill="none">
    <path d="M150 31 L174 31 M168 27 L174 31 L168 35"/>
    <path d="M330 31 L354 31 M348 27 L354 31 L348 35"/>
    <path d="M150 91 L174 91 M168 87 L174 91 L168 95"/>
    <path d="M330 91 L354 91 M348 87 L354 91 L348 95"/>
    <path d="M150 151 L174 151 M168 147 L174 151 L168 155"/>
    <path d="M330 151 L354 151 M348 147 L354 151 L348 155"/>
  </g>
  <g font-family="var(--font-mono)" font-size="9">
    <text x="0" y="192" fill="var(--signal)">HYBRID</text>
    <rect x="0" y="200" width="240" height="22" fill="var(--signal)"/>
    <text x="120" y="215" fill="var(--paper)" text-anchor="middle">described vocabulary</text>
    <rect x="270" y="200" width="240" height="22" fill="var(--signal)"/>
    <text x="390" y="215" fill="var(--paper)" text-anchor="middle">+ affordances for this resource</text>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M240 211 L264 211"/>
    <path d="M510 211 L534 211 M528 207 L534 211 L528 215"/>
  </g>
  <text x="545" y="215" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">action</text>
  <text x="0" y="240" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the report is explicit that these are not mutually exclusive</text>
</svg>
<figcaption>HAL and Hydra navigation sit in the first row, OpenAPI and Web of Things descriptions in the second, MCP in the third. Twenty years of argument assumed you had to pick one. The report states plainly that several surveyed initiatives already combine more than one.</figcaption>
</figure>

The report is explicit that "these paradigms are not mutually exclusive; several
of the initiatives surveyed below combine elements of more than one." That is a
standards working group saying out loud the thing this series has been circling
— which is corroboration, not proof, and the report is a living Community Group
document rather than a Recommendation. But it means the hybrid question is
being asked by people whose job is interoperability, not only by people writing
blog posts about REST.

There is a second observation in that report which is sharper than anything in
Parts I and II, and it reframes the problem. The report separates *perception*
from *action* — "perception is the process of sensing the current state of the
environment, while action is the process of modifying it" — and then notes that
current LLM tool protocols return environment state "exclusively as a
side-effect of action invocations," collapsing the distinction. The Web of
Things does not: it has a separate channel for observing state and another for
changing it.

That explains the shape of the gap. An MCP agent's model
of the world is assembled from the return values of things it did. There is no
first-class way to say *what is true right now* independently of having acted.
Fielding, who scoped his own design more carefully than most of his quoters,
described the uniform interface as optimized "for the common case of the Web"
and therefore "not optimal for other forms of architectural interaction." An
agent operating a stateful system is one of those other forms — and so, it
turns out, is a protocol that only lets you learn the world by poking it.

## HAL was never the strongest case

Part I made a comparison that was fair against the format most people mean by
hypermedia and unfair against the field. It said hypermedia trades a schema for
a relation name. Against basic HAL, that holds:

```json
{
  "_links": {
    "cancel": { "href": "/orders/123/cancel" }
  }
}
```

There is the affordance. It tells you a relation and a target. Not the method,
not the required inputs, not the response shape, not the consequences.
Everything a client needs beyond "something called cancel is possible here"
lives in documentation the client author read once.

HAL-FORMS is the same idea with the missing metadata put back:

```json
{
  "id": "123",
  "status": "CREATED",
  "_links": {
    "self":   { "href": "/orders/123" },
    "cancel": { "href": "/orders/123/cancel" }
  },
  "_templates": {
    "cancel": {
      "method": "POST",
      "properties": [
        { "name": "reason", "required": true }
      ]
    }
  }
}
```

The `_templates` block is doing four jobs at once. It says cancellation is
available **for this order, in this state, right now**. It says where to send
the request. It says which method. It says which inputs are required. Ship the
order and the template disappears along with the link.

Against the MCP tool for the same operation, the difference is not that one has
structure and the other does not. Both have structure. The
difference is *where the structure is anchored*.

<figure class="diagram">
<svg viewBox="0 0 620 234" role="img" aria-label="Three stacked panels comparing what each format carries. Basic HAL carries relation and target only. Richer affordance formats such as HAL-FORMS, Hydra and Web of Things carry contextual operation, method, input and output description and current state. MCP carries a stable named tool, a prose description and input and output schemas. A footer contrasts anchoring to this resource now versus anchoring to the server.">
  <g font-family="var(--font-mono)" font-size="9">
    <text x="0" y="12" fill="var(--muted)">BASIC HAL</text>
    <rect x="0" y="20" width="620" height="30" fill="none" stroke="var(--line)"/>
    <text x="12" y="39" fill="var(--ink)">relation  +  target</text>
    <text x="608" y="39" fill="var(--muted)" text-anchor="end">anchored to this resource, now</text>
    <text x="0" y="76" fill="var(--signal)">HAL-FORMS &middot; HYDRA &middot; WOT</text>
    <rect x="0" y="84" width="620" height="58" fill="var(--signal)"/>
    <text x="12" y="103" fill="var(--paper)">contextual operation  +  method  +  input schema  +  output schema</text>
    <text x="12" y="121" fill="var(--paper)">+  current availability  +  protocol binding</text>
    <text x="608" y="135" fill="var(--paper)" text-anchor="end">anchored to this resource, now</text>
    <text x="0" y="168" fill="var(--muted)">MCP TOOL</text>
    <rect x="0" y="176" width="620" height="46" fill="none" stroke="var(--ink)" stroke-width="1.5"/>
    <text x="12" y="195" fill="var(--ink)">stable name  +  prose description  +  input schema  +  output schema</text>
    <text x="608" y="213" fill="var(--muted)" text-anchor="end">anchored to the server, cacheable</text>
  </g>
</svg>
<figcaption>The middle row is the correction to Parts I and II. Richer hypermedia formats are not schema-free; they carry as much interaction metadata as a tool definition does. What they add is the anchor — the metadata describes an operation available on <em>this</em> resource in <em>this</em> state. What they give up is the stable, cacheable vocabulary the bottom row gets for free.</figcaption>
</figure>

An MCP tool is anchored to the server: here is `cancel_order`, it exists, it
takes these arguments, it means this. A HAL-FORMS template is anchored to the
resource: here is cancellation, available on this order, right now, needing a
reason. The first is a dictionary entry. The second is a sentence about a
specific situation. Neither one is a substitute for the other, which is the
entire point and took me three articles to get to.

## Directional evidence that the anchor matters

Robin Tegg ran the experiment the argument needs. He implemented the same
order-management domain twice (a conventional endpoint-driven API and a Spring
HATEOAS implementation publishing HAL-FORMS affordances) with equivalent
business capabilities on both sides, then drove both with agent tasks covering
the order lifecycle: create, add and update items, progress, cancel, and
deliberately attempt invalid transitions. He measured service-side call volume,
invalid attempts and completion behaviour.

His finding is that the hypermedia implementation generally produced fewer
unnecessary calls and fewer invalid state transitions in multi-step workflows,
and that the conventional implementation was more likely to drift into
irrelevant calls before converging. His diagnosis of *why* is the sentence I
would put on the wall: most agent mistakes in multi-step workflows are
transition mistakes. The model can usually call an endpoint. The hard part is
deciding whether that call is valid for the current state.

He is careful, and the caution should be preserved rather than sanded off: he
describes the findings as directional rather than definitive. One domain, one
author, one agent harness, no adversarial replication. What it supports is the
weaker of the two hypotheses this series has been separating, that
state-specific affordances reduce transition ambiguity, and it supports it in
exactly the place theory said the win would be.

It does not show that dynamic hypermedia beats typed tools in general, and
nobody should cite it for that.

## Hydra: the operation as a first-class thing

HAL-FORMS solves availability and invocation. Hydra goes further on semantics.

A Hydra `Operation` describes what a client needs in order to construct a valid
request, and it carries `method`, `expects` — what information the server
expects — `returns` — what information the server returns — and
`possibleStatus`, the status codes you might get back and what they mean.
Operations attach in two places: to a supported class, where all instances
share them, and inline in a representation, where they describe what is
possible for that particular resource.

Conceptually:

```text
Hydra resource
    |
    +-- operation: cancel
          method:  POST
          expects: CancellationRequest
          returns: Order
          status:  409 if already fulfilled
```

That is not far off an MCP tool definition. Name, method, input type, output
type, error semantics — the same information, in a different serialization,
minted by a community group a decade earlier. Anyone who tells you hypermedia
cannot express schemas has looked at HAL and stopped.

The interesting difference is the two attachment points, because they are
precisely the two halves this series has been trying to separate.
`supportedOperation` on a class is a stable vocabulary: all orders can, in
principle, be cancelled. An inline operation on a representation is contextual:
*this* order, in its current state, offers cancellation. Hydra already
distinguishes operation identity from operation availability. It just does not
make the first one cacheable in the way a tool catalog is, because the class
description is fetched from the API documentation rather than enumerated up
front.

## Web of Things: the model that took perception seriously

The strongest conceptual work is not in HTTP API land at all.

The W3C Web of Things models a Thing with three kinds of Interaction Affordance
— defined as "metadata of a Thing that shows the possible choices to Consumers"
— and the three-way split is the part to steal:

**Properties** expose state. They can be read, written, and crucially
*observed*; `readOnly`, `writeOnly` and `observable` are declared in the
description itself.

**Actions** invoke a function, with optional `input` and `output` data schemas,
plus `safe`, `idempotent` and `synchronous` flags stating whether the operation
changes anything, what a repeat call does, and whether it completes inline.

**Events** are an asynchronous push channel — the environment telling the agent
something happened, with no call required.

And underneath all three sit **Forms**, which the specification describes as
serializations of Protocol Bindings: the abstract affordance says *what* can be
done, the form says how to do it over HTTP, or MQTT, or CoAP. The description
of the capability is decoupled from the mechanics of invoking it.

A simplified conceptual example — this is illustrative shape, not normative
WoT, and a real Thing Description carries considerably more:

```json
{
  "title": "Vehicle",
  "properties": {
    "batteryLevel": { "type": "number", "readOnly": true, "observable": true },
    "doorsLocked":  { "type": "boolean", "observable": true }
  },
  "actions": {
    "unlock": { "safe": false, "idempotent": true },
    "setChargeTarget": {
      "input": {
        "type": "object",
        "properties": {
          "target": { "type": "integer", "minimum": 50, "maximum": 100 }
        }
      }
    }
  }
}
```

An MCP server in front of that vehicle translates naturally into
`get_battery_level`, `unlock`, `set_charge_target`: a flat catalog of
functions. What is lost in translation is everything structural: that battery
level is a continuously observable property rather than a thing you ask about,
that unlocking is idempotent, that the charge target has a range the agent
should respect before calling rather than after being rejected, that the
vehicle can *tell you* when something changes instead of waiting to be polled.

All of that would have to be reconstituted in prose, in the `description`
field, for a model to know it. Which returns us to the 97.1%.

WoT Thing Description 2.0 is a First Public Working Draft rather than a settled
standard, and the ecosystem is device-shaped rather than business-API-shaped.
Its value here is conceptual: it is an existence proof that you can describe an
environment an agent perceives and acts on, with typed affordances and
pluggable protocol bindings, without flattening everything into a function
catalog.

## Somebody already built the bridge

The strongest counterexample to Part III is not an argument but a working
gateway with a benchmark attached.

Antoine Bluchet and Kévin Dunglas set out the tension in almost the same terms
this series has: MCP separates application-controlled resources from
model-controlled tools, and REST/HATEOAS APIs break that assumption because
available actions are discovered at runtime inside resource content. Their
observation is that neither obvious workaround preserves hypermedia discovery —
mapping every endpoint to a tool statically loses the state-dependence, and
exposing one generic "call the API" tool loses the typing.

Their answer is a Dynamic Gateway Architecture with three discovery phases,
and the mechanism is more subtle than "expose the links as tools":

1. **Entrypoint.** The gateway exposes a single static bootstrap tool,
   `read_hydra_resource`, whose description carries the navigational guidance.
   No domain tools are registered yet. No system prompt required.
2. **Collection visit.** When the agent reads a Hydra collection, the gateway
   resolves the managed class, fetches and caches the API documentation,
   extracts the supported operations, and *registers typed tools* — `create_`,
   `read_`, `update_`, `replace_`, `delete_`, `search_` — for that class.
3. **Inline overrides.** Operations appearing inline in any resource response
   override or add tools, "reflecting per-resource state."

Phase two and phase three are not doing the same thing. Phase two turns a
*class-level* hypermedia description into a stable typed vocabulary —
description-driven, cacheable, exactly the shape MCP wants. Phase three lets
the *state of an individual resource* modify what is available —
hypermedia-driven, contextual, exactly the shape MCP's non-variance rule pushes
away from. The gateway is not choosing between the paradigms in that figure. It
is running two of them at once and letting them meet at the tool surface.

The authors report a comparative evaluation against a static OpenAPI baseline
(19 tasks across four tiers, two models, three runs per condition) in which
agents reach 100% completion on all 15 positive-capability tasks while the
gateway reduces tool-schema overhead from 77% to 2% of input context.

Those are the authors' own numbers, from a self-run benchmark on their own
architecture, in a preprint, with the prototype and the evaluation harness
published so that somebody else can check. That is the appropriate amount of
weight to put on them: it is a strong existence proof and a weak generalization.
Nobody has independently reproduced it, the task set is theirs, and 19 tasks in
one domain is a pilot, not a verdict.

But note which claim it lands on. Part III's argument was that MCP's enumerated
tool surface cannot act as an engine of application state. This gateway does not
contradict that — it *works within* it, keeping a small stable bootstrap surface
and growing the vocabulary lazily as the agent navigates. And the headline
result is not about correctness at all. It is about context: 77% of the input
window spent on tool schemas, down to 2%, because you stop shipping the entire
catalog to describe a world the agent is only touching one corner of.

That reframes the whole trade. Part III said hypermedia's dynamism is a cache
invalidation event on every call. This says a static catalog is a context tax on
every call. Both are true. Which one dominates depends on how large the
vocabulary is relative to the part of it any single task needs — and for a large
API, that ratio is brutal.

## What the trilogy actually proposes

So here is the synthesis, and it is not "HATEOAS was right."

Keep the operation vocabulary stable and described. Let availability be
discovered from state. Separate operation identity from operation availability
and stop making one mechanism carry both.

Concretely, the vocabulary stays where MCP put it — a catalog entry that is the
same on Tuesday as it was on Monday, cacheable, prompt-stable, understood once:

```json
{
  "name": "cancelOrder",
  "description": "Cancel an order before fulfilment begins.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "orderId": { "type": "string" },
      "reason":  { "type": "string" }
    },
    "required": ["orderId", "reason"]
  }
}
```

And the resource says which of those operations apply to it, right now, with
the arguments it can already bind — this is illustrative, not a proposal for
wire format:

```json
{
  "id": "order-123",
  "status": "pending",
  "affordances": [
    {
      "operation": "cancelOrder",
      "arguments":   { "orderId": "order-123" },
      "constraints": { "reason": { "required": true } }
    }
  ]
}
```

After shipment:

```json
{
  "id": "order-123",
  "status": "shipped",
  "affordances": [],
  "unavailable": [
    { "operation": "cancelOrder", "reason": "already_shipped" }
  ]
}
```

That last field is the one I would fight for. A hypermedia response says an
affordance is absent; it does not say why, and an agent that cannot tell "not
allowed right now" from "does not exist here" will either give up too early or
retry forever. A machine-readable reason code turns a missing link into
actionable information, which is precisely the thing MCP's execution-error
channel does well and hypermedia's absent-link convention does badly.

<figure class="diagram">
<svg viewBox="0 0 620 300" role="img" aria-label="A vertical hybrid architecture. At the top a box titled stable operation model listing name, semantic description, input and output schema, and general side effects, connected downward with the annotation understood once and cached. In the middle a box labelled agent. Below it a box titled current affordances listing valid operations now, bound resource identifiers, state-specific constraints and reason codes when unavailable, connected upward with the annotation discovered continuously. A footer reads stable verbs, dynamic affordances.">
  <rect x="150" y="20" width="320" height="88" fill="none" stroke="var(--ink)" stroke-width="1.5"/>
  <g font-family="var(--font-mono)" font-size="9">
    <text x="164" y="38" fill="var(--muted)">STABLE OPERATION MODEL</text>
    <text x="164" y="58" fill="var(--ink)">name</text>
    <text x="164" y="72" fill="var(--ink)">semantic description</text>
    <text x="164" y="86" fill="var(--ink)">input / output schema</text>
    <text x="164" y="100" fill="var(--ink)">general side effects</text>
  </g>
  <g stroke="var(--line)" fill="none">
    <path d="M310 108 L310 134 M306 128 L310 134 L314 128"/>
  </g>
  <text x="320" y="126" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">understood once &middot; cached</text>
  <rect x="240" y="138" width="140" height="34" fill="var(--signal)"/>
  <text x="310" y="159" font-family="var(--font-mono)" font-size="9" fill="var(--paper)" text-anchor="middle">AGENT</text>
  <g stroke="var(--signal)" fill="none">
    <path d="M310 202 L310 176 M306 182 L310 176 L314 182"/>
  </g>
  <text x="320" y="196" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">discovered continuously</text>
  <rect x="150" y="206" width="320" height="74" fill="none" stroke="var(--signal)"/>
  <g font-family="var(--font-mono)" font-size="9">
    <text x="164" y="224" fill="var(--signal)">CURRENT AFFORDANCES</text>
    <text x="164" y="244" fill="var(--ink)">valid operations now</text>
    <text x="164" y="258" fill="var(--ink)">bound resource identifiers</text>
    <text x="164" y="272" fill="var(--ink)">constraints &middot; reason codes when not</text>
  </g>
  <text x="0" y="296" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">stable verbs</text>
  <text x="620" y="296" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">dynamic affordances</text>
</svg>
<figcaption>Two channels, two update frequencies. The top one is a dictionary the agent learns once and caches; the bottom one is a situation report the agent re-reads constantly. HATEOAS fused them and paid for it in vocabulary discovery. MCP kept only the top one and pays for it in prose.</figcaption>
</figure>

Three architectures, honestly compared:

| Model | Operation semantics | Availability | Main strength | Main weakness |
|---|---|---|---|---|
| Static tool catalog | catalog entry: prose + schema | inferred from fields, or learned from errors | cacheable, typed, prompt-stable | prose carries the semantic load; whole catalog in context |
| Dynamic hypermedia | affordance in the representation | always contextual, always current | state is never stale | vocabulary must be rediscovered; schema churn in context |
| Hybrid | stable typed operations | resource-specific affordances | typed semantics *and* current applicability | needs a cross-reference model nobody has standardized |

That last cell is the honest cost. The hybrid needs a resource to name an
operation that lives in a different document, and nothing standardizes that
link today. The Hydra gateway does it by convention inside one implementation.
Doing it across servers means agreeing on how a representation cites a tool —
which is a small specification and a large amount of politics.

## What would actually settle it

The trilogy started with a natural experiment and should end by asking for a
controlled one. Build the same stateful domain, orders is fine and it has carried
three articles already, three ways:

**A. Static catalog.** Every tool listed up front, availability inferred from
resource fields and execution errors.

**B. Stable catalog plus resource affordances.** Identical tool schemas, but
each resource states which operations are valid now, with bound arguments and
reason codes for the ones that are not.

**C. Dynamic discovery.** Operations discovered lazily from Hydra or HAL-FORMS
controls as the agent navigates, in the shape the gateway demonstrates.

Then measure the things that actually distinguish them. Task completion rate.
Invalid state transitions attempted. Unnecessary calls. Argument-validation
failures and retries. Number of tool schemas resident in context, total input
and output tokens, and prompt-cache hit rate — because B and C are making
opposite bets about the context window and the bet should be settled with
numbers rather than intuition. Latency, including the extra round trips
discovery costs.

And two more that matter more than any of the above, because they test the
thing the whole argument has been about:

**Can the server change the workflow without changing the client?** Add a
mandatory approval step between order and fulfilment. B and C should absorb it
silently. A requires a description edit at minimum.

**Can the server introduce an operation the agent has never heard of?** Not a
new argument on a known verb — a genuinely new business concept. This is the
claim hypermedia has always made and Part I argued it cannot cash, because a
new URL does not teach a new concept. It would be extremely interesting to be
wrong about that, and a language model is the first client for which the
question can even be posed.

## Where this leaves the argument

Part III's punchline stands: we spent two decades waiting for a client good
enough to navigate the graph, it arrived, and it read the schema instead.

What I would add, having now read what the richer formats actually do, is that
the schema and the graph were never competing for the same job. A schema says
what an operation is. A graph — a good one, with methods and inputs and
statuses on it, not a bare `rel` — says which operations this thing supports
right now and how to bind them. MCP made the first one stable, cacheable and
machine-checkable, which is the reason it worked. Hypermedia made the second one
authoritative, which is the reason people keep coming back to it after being
told it lost.

The mistake was never picking one. It was assuming they were the same
mechanism, which HATEOAS asserted and MCP's non-variance rule denied, and which
the WebAgents report and a working Hydra gateway both suggest was a false
choice all along.

MCP may have been right to keep operation identity stable. HATEOAS may have
been right that the server should say what is possible *now*. Those are not
competing claims, and the next agent protocol will probably look less like
hypermedia returning from the dead than like us finally unbundling the two ideas
HATEOAS shipped together.

Stable verbs. Dynamic affordances.

## References

Standards move; the versions cited here were current in August 2026. WoT Thing
Description 2.0 is a First Public Working Draft and the WebAgents
interoperability report is a living Community Group document — neither is a W3C
Recommendation, and both should be re-read before being relied on.

**The evidence about descriptions**

1. Mohammed Mehedi Hasan, Hao Li, Gopi Krishnan Rajbahadur, Bram Adams, Ahmed
   E. Hassan, “Model Context Protocol (MCP) Tool Descriptions Are Smelly!
   Towards Improving AI Agent Efficiency with Augmented MCP Tool Descriptions,”
   2026 — 856 tools across 103 servers, 97.1% with at least one description
   defect, 56% not clearly stating purpose, and the cost/benefit of augmenting
   descriptions.
   https://arxiv.org/abs/2602.14878

2. “Smell-Aware Evaluation of MCP Server Descriptions,” 2026 — corroborating
   evidence on description quality.
   https://arxiv.org/abs/2602.18914

**The taxonomy**

3. W3C WebAgents Community Group, *Report on Interoperability for Agents on the
   Web* — affordances as machine-readable discoverable descriptions of an
   available interaction; hypermedia-driven, description-driven and
   protocol-driven paradigms and their combinability; perception and action as
   distinct channels; the observation that LLM tool protocols return
   environment state as a side effect of invocation. Living CG report, not a
   Recommendation.
   https://w3c-cg.github.io/webagents/TaskForces/Interoperability/Reports/report-interoperability.html

4. Roy T. Fielding, *Architectural Styles and the Design of Network-based
   Software Architectures*, 2000 — section 5.1.5 on the uniform interface being
   optimized for the common case of the Web and not optimal for other forms of
   interaction.
   https://ics.uci.edu/~fielding/pubs/dissertation/rest_arch_style.htm

**Richer affordance formats**

5. HAL-FORMS — `_templates`, `method`, and `properties` as state-transition
   metadata.
   https://rwcbook.com/hal-forms/

6. Spring HATEOAS reference documentation — the practical HAL-FORMS
   implementation, affordances, and the media type.
   https://docs.spring.io/spring-hateoas/docs/current/reference/html/

7. Hydra Core Vocabulary — `Operation` with `method`, `expects`, `returns` and
   `possibleStatus`; `supportedOperation` on a class versus operations inline
   in a representation.
   https://www.hydra-cg.com/spec/latest/core/

8. W3C, *Web of Things (WoT) Thing Description 2.0*, First Public Working
   Draft, November 2025 — Interaction Affordances; Properties with `readOnly`,
   `writeOnly` and `observable`; Actions with `input`/`output` schemas and
   `safe`/`idempotent`/`synchronous`; Events; Forms as serializations of
   Protocol Bindings.
   https://www.w3.org/TR/wot-thing-description-2.0/

9. W3C, *Web of Things (WoT) Architecture 1.1*.
   https://www.w3.org/TR/wot-architecture11/

10. W3C, *Web of Things (WoT) Binding Templates / Registry* — the separation of
    abstract affordance from concrete protocol operation.
    https://www.w3.org/TR/wot-binding-registry/

11. JSON Hypertext Application Language (HAL) — the baseline the middle row
    improves on.
    https://www.ietf.org/archive/id/draft-kelly-json-hal-11.html

**Experiments**

12. Robin Tegg, “Hypermedia APIs for Autonomous Agents,” 3 June 2026 — the same
    order domain implemented conventionally and with Spring HATEOAS /
    HAL-FORMS, agent-driven tasks over the order lifecycle, fewer unnecessary
    calls and fewer invalid transitions on the hypermedia side. The author
    describes the findings as directional rather than definitive.
    https://robintegg.com/2026/06/03/hypermedia-apis-for-autonomous-agents.html

13. Antoine Bluchet, Kévin Dunglas, “APIs We Built Are Meant for Computers: How
    Do We Expose Hypermedia APIs to LLMs?” — the Dynamic Gateway Architecture,
    three-phase lazy discovery, and the authors' reported evaluation: 19 tasks,
    four tiers, two models, three runs, 100% completion on the 15
    positive-capability tasks, tool-schema overhead from 77% to 2% of input
    context.
    https://hal.science/hal-05630480

14. Hydra MCP Bridge — the prototype gateway and reproducible benchmark
    accompanying the paper: one static `read_hydra_resource` bootstrap tool,
    class-level operations registered as typed tools on collection visit, and
    inline operations overriding them per resource.
    https://github.com/coopTilleuls/hydra-mcp-brigde

**The protocol**

15. Model Context Protocol, specification `2026-07-28`, *Tools* — tool
    `name`/`description`/`inputSchema`/`outputSchema`, and the
    protocol-versus-execution error split.
    https://modelcontextprotocol.io/specification/2026-07-28/server/tools

16. OpenAPI Arazzo Specification — describing multi-step API workflows outside
    resource payloads; another sign that description-driven systems are
    reaching for workflow and state.
    https://spec.openapis.org/arazzo/latest.html

**In this series**

17. Fabio Ellena, “The Browser Was Never the Smart Client,” 2026 — Part I.
    https://fblln.github.io/articles/the-browser-was-never-the-smart-client/

18. Fabio Ellena, “HATEOAS Priced the Wrong Change,” 2026 — Part II.
    https://fblln.github.io/articles/hateoas-priced-the-wrong-change/

19. Fabio Ellena, “HATEOAS Failed the Agent Test,” 2026 — Part III.
    https://fblln.github.io/articles/hateoas-failed-the-agent-test/
