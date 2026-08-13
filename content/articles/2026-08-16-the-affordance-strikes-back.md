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

What a model needs to know before calling it runs considerably longer, and every
one of those questions is a different *kind* of thing. The protocol gives them
all the same place to live: `description`, a string of English prose.

<figure class="diagram">
<svg viewBox="0 0 620 246" role="img" aria-label="A single filled box on the left holding a two-line MCP tool description reading cancel order. From it a spine fans out to seven separate outlined boxes on the right, each labelled with a different category of knowledge: precondition, interaction, idempotence, validation, side effect, exclusion and error semantics, each carrying the question it has to answer. A footer records that 97.1 percent of 856 surveyed tools carried at least one defect in this one field.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">ONE FIELD</text>
  <rect x="0" y="96" width="180" height="40" fill="var(--signal)"/>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--paper)">
    <text x="12" y="114">"description":</text>
    <text x="12" y="130">"Cancel an order."</text>
  </g>
  <text x="280" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">SEVEN KINDS OF ANSWER</text>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="280" y="18" width="340" height="24" fill="none" stroke="var(--line)"/>
    <text x="290" y="34" fill="var(--signal)">PRECONDITION</text>
    <text x="392" y="34" fill="var(--ink)">after fulfilment starts?</text>
    <rect x="280" y="48" width="340" height="24" fill="none" stroke="var(--line)"/>
    <text x="290" y="64" fill="var(--signal)">INTERACTION</text>
    <text x="392" y="64" fill="var(--ink)">must issue_refund follow?</text>
    <rect x="280" y="78" width="340" height="24" fill="none" stroke="var(--line)"/>
    <text x="290" y="94" fill="var(--signal)">IDEMPOTENCE</text>
    <text x="392" y="94" fill="var(--ink)">what does call two do?</text>
    <rect x="280" y="108" width="340" height="24" fill="none" stroke="var(--line)"/>
    <text x="290" y="124" fill="var(--signal)">VALIDATION</text>
    <text x="392" y="124" fill="var(--ink)">is reason free text?</text>
    <rect x="280" y="138" width="340" height="24" fill="none" stroke="var(--line)"/>
    <text x="290" y="154" fill="var(--signal)">SIDE EFFECT</text>
    <text x="392" y="154" fill="var(--ink)">does it email the customer?</text>
    <rect x="280" y="168" width="340" height="24" fill="none" stroke="var(--line)"/>
    <text x="290" y="184" fill="var(--signal)">EXCLUSION</text>
    <text x="392" y="184" fill="var(--ink)">is create_return the verb?</text>
    <rect x="280" y="198" width="340" height="24" fill="none" stroke="var(--line)"/>
    <text x="290" y="214" fill="var(--signal)">ERROR SEMANTICS</text>
    <text x="392" y="214" fill="var(--ink)">stop, or retry differently?</text>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M180 116 L240 116"/>
    <path d="M240 30 L240 210"/>
    <path d="M240 30 L274 30 M268 26 L274 30 L268 34"/>
    <path d="M240 60 L274 60 M268 56 L274 60 L268 64"/>
    <path d="M240 90 L274 90 M268 86 L274 90 L268 94"/>
    <path d="M240 120 L274 120 M268 116 L274 120 L268 124"/>
    <path d="M240 150 L274 150 M268 146 L274 150 L268 154"/>
    <path d="M240 180 L274 180 M268 176 L274 180 L268 184"/>
    <path d="M240 210 L274 210 M268 206 L274 210 L268 214"/>
  </g>
  <text x="0" y="240" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">97.1% of 856 surveyed tools carry at least one defect in this field</text>
</svg>
<figcaption>The seven boxes on the right are seven different shapes of knowledge, and six of them are the sort of thing a schema declares rather than a sentence implies. The protocol has one slot for all of them, and it is prose. Nothing here is a protocol failure; it is a field carrying more categories than any single field can keep straight.</figcaption>
</figure>

We now have a reasonable idea of how well it is going. Hasan and colleagues
analysed 856 tools across 103 MCP servers and found that 97.1% carried at least
one description defect, with 56% failing to state their purpose clearly.

Then they tried the obvious fix. Augmenting the descriptions moved task success
by a median of a few percentage points and partial goal completion by around
fifteen, while execution steps rose by roughly two thirds and a sixth of cases
got *worse*. Better prose helped, cost more, and did not help uniformly.

As a protocol indictment that is a misreading. As a statement about interface
boundaries it is exact:

> Tool semantics encoded primarily as prose are a weak interface boundary.

Weak, not useless. Prose is how you communicate purpose and nuance to a language
model, and no schema will replace the sentence explaining that cancelling a
shipped order is the wrong instinct. But much of what those descriptions are
struggling to carry is not nuance. It is *structure*: preconditions, current
validity, required arguments for this case, which operation supersedes which in
which state. Structure is exactly what hypermedia formats have been encoding as
data for fifteen years.

Part III ended by separating two questions the fourth constraint had bundled
together. This part is about who answers the second one.

## What an affordance actually is

The previous three parts used "affordance" loosely, to mean "a link with a verb
in it." That was fine for arguing against basic HAL and is not good enough here.

The W3C WebAgents Community Group's interoperability report gives the definition
I will use: an affordance is "a relation between an agent's capabilities and the
capabilities that the environment exposes: it specifies what the agent can do
and how," formalised as a "machine-readable, discoverable description of an
available interaction." Not just what exists, but what *this* agent can do, how
to do it, discoverable, machine-readable. A bare `rel` and `href` satisfies about
a third of it.

The report's more useful contribution is a taxonomy of three interaction
paradigms:

**Hypermedia-driven**, where agents "navigate and discover affordances at
runtime by following hypermedia controls embedded in resource
representations, requiring no prior knowledge of the environment's structure."

**Description-driven**, where agents "consume machine-readable interface
specifications before invoking affordances, relying on out-of-band or
pre-fetched descriptions."

**Protocol-driven**, where agents "use a standardized invocation protocol that
manages tool enumeration and invocation through a dedicated server."

<figure class="diagram">
<svg viewBox="0 0 620 196" role="img" aria-label="Three routes converging on one shared action block. The first, hypermedia-driven, runs from a resource representation to an affordance for this state. The second, description-driven, runs from an interface description to an operation known in advance. The third, protocol-driven, runs from a tool catalog to a named callable. All three feed the same tall action block on the right. The first and third routes are drawn in the accent colour to mark the hybrid, which is a pair of existing routes running together rather than a fourth row.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">HOW THE AGENT LEARNS WHAT IT MAY DO</text>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="0" y="22" width="190" height="34" fill="none" stroke="var(--signal)"/>
    <text x="10" y="36" fill="var(--signal)">HYPERMEDIA-DRIVEN</text>
    <text x="10" y="50" fill="var(--ink)">resource representation</text>
    <rect x="220" y="22" width="190" height="34" fill="none" stroke="var(--signal)"/>
    <text x="230" y="44" fill="var(--ink)">affordance for this state</text>
    <rect x="0" y="72" width="190" height="34" fill="none" stroke="var(--line)"/>
    <text x="10" y="86" fill="var(--muted)">DESCRIPTION-DRIVEN</text>
    <text x="10" y="100" fill="var(--muted)">interface description</text>
    <rect x="220" y="72" width="190" height="34" fill="none" stroke="var(--line)"/>
    <text x="230" y="94" fill="var(--muted)">operation known in advance</text>
    <rect x="0" y="122" width="190" height="34" fill="none" stroke="var(--signal)"/>
    <text x="10" y="136" fill="var(--signal)">PROTOCOL-DRIVEN</text>
    <text x="10" y="150" fill="var(--ink)">tool catalog</text>
    <rect x="220" y="122" width="190" height="34" fill="none" stroke="var(--signal)"/>
    <text x="230" y="144" fill="var(--ink)">named callable</text>
    <rect x="440" y="22" width="180" height="134" fill="var(--signal)"/>
    <text x="530" y="93" fill="var(--paper)" text-anchor="middle">ACTION</text>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M190 39 L214 39 M208 35 L214 39 L208 43"/>
    <path d="M410 39 L434 39 M428 35 L434 39 L428 43"/>
    <path d="M190 139 L214 139 M208 135 L214 139 L208 143"/>
    <path d="M410 139 L434 139 M428 135 L434 139 L428 143"/>
  </g>
  <g stroke="var(--line)" fill="none">
    <path d="M190 89 L214 89 M208 85 L214 89 L208 93"/>
    <path d="M410 89 L434 89 M428 85 L434 89 L428 93"/>
  </g>
  <text x="0" y="186" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the hybrid is not a fourth row; it is the first and the third running at once</text>
</svg>
<figcaption>HAL and Hydra navigation take the first route, OpenAPI and Web of Things descriptions the second, MCP the third, and all three end in the same place. Twenty years of argument assumed you had to pick one. The report states plainly that several surveyed initiatives already combine more than one, which is why the accent colour here marks a pair of existing routes rather than a new row.</figcaption>
</figure>

The report is explicit that "these paradigms are not mutually exclusive; several
of the initiatives surveyed below combine elements of more than one." It is a
living Community Group document rather than a Recommendation, so that is
corroboration and not proof. But it means the hybrid question is being asked by
people whose job is interoperability, not only by people writing blog posts
about REST.

A second observation in the report is sharper than anything in Parts I and II.
It separates *perception* from *action* ("perception is the process of sensing
the current state of the environment, while action is the process of modifying
it") and then notes that current LLM tool protocols return environment state
"exclusively as a side-effect of action invocations," collapsing the
distinction. The Web of Things keeps a separate channel for each.

That explains the shape of the gap. An MCP agent's model of the world is
assembled from the return values of things it did, with no first-class way to
ask *what is true right now*. Fielding, who scoped his own design more carefully
than most of his quoters, described the uniform interface as optimized "for the
common case of the Web" and therefore "not optimal for other forms of
architectural interaction." An agent operating a stateful system is one of those
other forms, and so is a protocol that only lets you learn the world by poking
it.

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

A relation and a target. Not the method, not the required inputs, not the
response shape, not the consequences. Everything a client needs beyond
"something called cancel is possible here" lives in documentation the client
author read once.

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

The `_templates` block does four jobs at once: cancellation is available **for
this order, in this state, right now**, here is where to send the request, here
is the method, here are the required inputs. Ship the order and the template
disappears along with the link.

Against the MCP tool for the same operation, the difference is not that one has
structure and the other does not. Both do. The difference is *where the
structure is anchored*.

<figure class="diagram">
<svg viewBox="0 0 620 250" role="img" aria-label="Two payloads side by side, each braced beneath and labelled. On the left, an MCP tool definition carrying a name, a prose description and an input schema, braced and labelled operation identity, anchored to the server and cacheable. On the right, a HAL-FORMS resource carrying a status field and a templates block with a method and a required property, braced and labelled operation availability, anchored to this order in this state. Below both runs a dashed full-width strip holding a basic HAL link with only a relation and an href, annotated that it names the operation but neither describes nor binds it.">
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="0" y="12">MCP TOOL DEFINITION</text>
    <text x="330" y="12">HAL-FORMS RESOURCE</text>
  </g>
  <rect x="0" y="20" width="290" height="94" fill="none" stroke="var(--ink)" stroke-width="1.5"/>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--ink)">
    <text x="12" y="38">"name": "cancelOrder",</text>
    <text x="12" y="54">"description": "Cancel an</text>
    <text x="12" y="70">&nbsp;&nbsp;order before fulfilment.",</text>
    <text x="12" y="86">"inputSchema": {</text>
    <text x="12" y="102">&nbsp;&nbsp;orderId, reason* }</text>
  </g>
  <rect x="330" y="20" width="290" height="94" fill="var(--signal)"/>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--paper)">
    <text x="342" y="38">"status": "CREATED",</text>
    <text x="342" y="54">"_templates": {</text>
    <text x="342" y="70">&nbsp;&nbsp;"cancel": {</text>
    <text x="342" y="86">&nbsp;&nbsp;&nbsp;&nbsp;"method": "POST",</text>
    <text x="342" y="102">&nbsp;&nbsp;&nbsp;&nbsp;"properties": [ reason* ] } }</text>
  </g>
  <path d="M0 120 L0 128 L290 128 L290 120 M145 128 L145 136" stroke="var(--ink)" fill="none"/>
  <path d="M330 120 L330 128 L620 128 L620 120 M475 128 L475 136" stroke="var(--signal)" fill="none"/>
  <g font-family="var(--font-mono)" font-size="9" text-anchor="middle">
    <text x="145" y="150" fill="var(--ink)">OPERATION IDENTITY</text>
    <text x="145" y="164" fill="var(--muted)">anchored to the server &middot; cacheable</text>
    <text x="475" y="150" fill="var(--signal)">OPERATION AVAILABILITY</text>
    <text x="475" y="164" fill="var(--muted)">anchored to this order, in this state</text>
  </g>
  <text x="0" y="196" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">BASIC HAL &middot; NEITHER</text>
  <rect x="0" y="204" width="620" height="26" fill="none" stroke="var(--line)" stroke-dasharray="3 3"/>
  <text x="12" y="221" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">"cancel": { "href": "/orders/123/cancel" }</text>
  <text x="608" y="221" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="end">names it &middot; does not describe or bind it</text>
  <text x="0" y="246" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the two panels are not rivals; they answer different questions about the same verb</text>
</svg>
<figcaption>This is the correction to Parts I and II. Richer hypermedia formats are not schema-free: a <code>_templates</code> block carries as much interaction metadata as a tool definition does. What it adds is the anchor, since the metadata describes an operation available on <em>this</em> order in <em>this</em> state. What it gives up is the stable, cacheable vocabulary the left panel gets for free. Basic HAL, in the dashed strip, has neither, which is the format Parts I and II were arguing with.</figcaption>
</figure>

An MCP tool is anchored to the server: `cancel_order` exists, takes these
arguments, means this. A HAL-FORMS template is anchored to the resource:
cancellation is available on this order, right now, needing a reason. The first
is a dictionary entry, the second a sentence about a specific situation, and
neither is a substitute for the other. That took me three articles to get to.

## Directional evidence that the anchor matters

Robin Tegg ran the experiment the argument needs. He implemented the same
order-management domain twice (a conventional endpoint-driven API and a Spring
HATEOAS implementation publishing HAL-FORMS affordances) with equivalent
business capabilities on both sides, then drove both with agent tasks over the
order lifecycle, including deliberately invalid transitions.

The hypermedia implementation generally produced fewer unnecessary calls and
fewer invalid state transitions in multi-step workflows; the conventional one
was more likely to drift into irrelevant calls before converging. His diagnosis
of *why* is the sentence I would put on the wall: most agent mistakes in
multi-step workflows are transition mistakes. The model can usually call an
endpoint. The hard part is deciding whether that call is valid for the current
state.

He describes the findings as directional rather than definitive, and that
caution should be preserved rather than sanded off. One domain, one author, one
agent harness, no adversarial replication. It supports the weaker of the two
hypotheses this series has been separating, that state-specific affordances
reduce transition ambiguity, in exactly the place theory said the win would be.
It does not show that dynamic hypermedia beats typed tools in general, and
nobody should cite it for that.

## Hydra: the operation as a first-class thing

HAL-FORMS solves availability and invocation. Hydra goes further on semantics.

A Hydra `Operation` describes what a client needs in order to construct a valid
request, and it carries `method`, `expects` (what the server wants sent),
`returns` (what it sends back), and `possibleStatus`, the status codes you might
get and what each one means:

```text
Hydra resource
    |
    +-- operation: cancel
          method:  POST
          expects: CancellationRequest
          returns: Order
          status:  409 if already fulfilled
```

Name, method, input type, output type, error semantics: not far off an MCP tool
definition, in a different serialization, minted by a community group a decade
earlier. Anyone who tells you hypermedia cannot express schemas has looked at
HAL and stopped.

The interesting part is that operations attach in two places, and they are
precisely the two halves this series has been trying to separate.
`supportedOperation` on a class is a stable vocabulary: all orders can, in
principle, be cancelled. An inline operation on a representation is contextual:
*this* order, in its current state, offers cancellation. Hydra already
distinguishes operation identity from operation availability. It just does not
make the first cacheable the way a tool catalog is, because the class
description is fetched from the API documentation rather than enumerated up
front.

## Web of Things: the model that took perception seriously

The strongest conceptual work is not in HTTP API land at all. The W3C Web of
Things models a Thing with three kinds of Interaction Affordance, defined as
"metadata of a Thing that shows the possible choices to Consumers," and the
three-way split is the part to steal:

**Properties** expose state. They can be read, written, and crucially
*observed*; `readOnly`, `writeOnly` and `observable` are declared in the
description itself.

**Actions** invoke a function, with optional `input` and `output` data schemas,
plus `safe`, `idempotent` and `synchronous` flags stating whether the operation
changes anything, what a repeat call does, and whether it completes inline.

**Events** are an asynchronous push channel, the environment telling the agent
something happened with no call required.

Underneath all three sit **Forms**, which the specification describes as
serializations of Protocol Bindings: the abstract affordance says *what* can be
done, the form says how to do it over HTTP, or MQTT, or CoAP. The capability is
decoupled from the mechanics of invoking it.

A simplified example, illustrative rather than normative:

```json
{
  "title": "Vehicle",
  "properties": {
    "batteryLevel": { "type": "number", "readOnly": true, "observable": true }
  },
  "actions": {
    "unlock": { "safe": false, "idempotent": true },
    "setChargeTarget": {
      "input": { "target": { "type": "integer", "minimum": 50, "maximum": 100 } }
    }
  }
}
```

An MCP server in front of that vehicle translates naturally into
`get_battery_level`, `unlock`, `set_charge_target`: a flat catalog of functions.
What is lost is everything structural. That battery level is continuously
observable rather than something you ask about. That unlocking is idempotent.
That the charge target has a range the agent should respect before calling
rather than after being rejected. That the vehicle can *tell you* when something
changes instead of waiting to be polled. All of it would have to be
reconstituted in prose, in the `description` field, which returns us to the
97.1%.

WoT Thing Description 2.0 is a First Public Working Draft, and the ecosystem is
device-shaped rather than business-API-shaped. Its value here is conceptual: an
existence proof that you can describe an environment an agent perceives and acts
on without flattening everything into a function catalog.

## Somebody already built the bridge

The strongest counterexample to Part III is not an argument but a working
gateway with a benchmark attached.

Antoine Bluchet and Kévin Dunglas set out the tension in almost the same terms
this series has: MCP separates application-controlled resources from
model-controlled tools, and REST/HATEOAS APIs break that assumption because
available actions are discovered at runtime inside resource content. Neither
obvious workaround preserves hypermedia discovery. Mapping every endpoint to a
tool statically loses the state-dependence; exposing one generic "call the API"
tool loses the typing.

Their answer is a Dynamic Gateway Architecture with three discovery phases, and
the mechanism is subtler than "expose the links as tools":

1. **Entrypoint.** The gateway exposes a single static bootstrap tool,
   `read_hydra_resource`, whose description carries the navigational guidance.
   No domain tools are registered yet. No system prompt required.
2. **Collection visit.** When the agent reads a Hydra collection, the gateway
   resolves the managed class, fetches and caches the API documentation,
   extracts the supported operations, and *registers typed tools* — `create_`,
   `read_`, `update_`, `replace_`, `delete_`, `search_` — for that class.
3. **Inline overrides.** Operations appearing inline in any resource response
   override or add tools, "reflecting per-resource state."

Phases two and three are not doing the same thing. Phase two turns a
*class-level* hypermedia description into a stable typed vocabulary:
description-driven, cacheable, exactly the shape MCP wants. Phase three lets the
*state of an individual resource* modify what is available: hypermedia-driven,
contextual, exactly the shape MCP's non-variance rule pushes away from. The
gateway is not choosing between the paradigms in that figure. It runs two of
them at once and lets them meet at the tool surface.

The authors report a comparative evaluation against a static OpenAPI baseline
(19 tasks across four tiers, two models, three runs per condition) in which
agents reach 100% completion on all 15 positive-capability tasks while the
gateway reduces tool-schema overhead from 77% to 2% of input context.

Those are the authors' own numbers, from a self-run benchmark on their own
architecture, in a preprint, with the prototype and the harness published so
somebody else can check. A strong existence proof and a weak generalization:
nobody has independently reproduced it, the task set is theirs, and 19 tasks in
one domain is a pilot rather than a verdict.

But note which claim it lands on. Part III argued that MCP's enumerated tool
surface cannot act as an engine of application state. This gateway does not
contradict that; it *works within* it, keeping a small stable bootstrap surface
and growing the vocabulary lazily as the agent navigates. And the headline
result is not about correctness. It is about context: 77% of the input window
spent on tool schemas, down to 2%, because you stop shipping the entire catalog
to describe a world the agent is touching one corner of.

That reframes the whole trade. Part III said hypermedia's dynamism is a cache
invalidation event on every call. This says a static catalog is a context tax on
every call. Both are true, and which one dominates depends on how large the
vocabulary is relative to the part any single task needs. For a large API, that
ratio is brutal.

## What the series actually proposes

The synthesis is not "HATEOAS was right." Keep the operation vocabulary stable
and described. Let availability be discovered from state. Separate operation
identity from operation availability and stop making one mechanism carry both.

Concretely, the vocabulary stays where MCP put it, a catalog entry that is the
same on Tuesday as it was on Monday, cacheable and understood once:

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
affordance is absent without saying why, and an agent that cannot tell "not
allowed right now" from "does not exist here" will either give up too early or
retry forever. A machine-readable reason code turns a missing link into
actionable information.

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
operation that lives in a different document, and nothing standardizes that link
today. The Hydra gateway does it by convention inside one implementation. Doing
it across servers means agreeing on how a representation cites a tool, which is
a small specification and a large amount of politics.

## What would actually settle it

The series started with a natural experiment and should end by asking for a
controlled one. Build the same stateful domain three ways. **A**: every tool
listed up front, availability inferred from fields and errors. **B**: identical
tool schemas, but each resource states which operations are valid now, with
bound arguments and reason codes for the ones that are not. **C**: operations
discovered lazily from Hydra or HAL-FORMS controls as the agent navigates. Then
count invalid transitions attempted, unnecessary calls, and tool schemas
resident in context, because B and C are making opposite bets about the context
window.

One question matters more than any of that: **can the server introduce an
operation the agent has never heard of?** Not a new argument on a known verb, a
genuinely new business concept. This is the claim hypermedia has always made and
Part I argued it cannot cash, because a new URL does not teach a new concept. It
would be extremely interesting to be wrong about that, and a language model is
the first client for which the question can even be posed.

## Where this leaves the argument

Part III's punchline stands: we spent two decades waiting for a client good
enough to navigate the graph, it arrived, and it read the schema instead.

What I would add, having read what the richer formats actually do, is that the
schema and the graph were never competing for the same job. A schema says what
an operation is. A good graph, with methods and inputs and statuses on it rather
than a bare `rel`, says which operations this thing supports right now and how
to bind them. MCP made the first stable, cacheable and machine-checkable, which
is why it worked. Hypermedia made the second authoritative, which is why people
keep coming back to it after being told it lost.

The mistake was never picking one. It was assuming they were the same mechanism,
which HATEOAS asserted and MCP's non-variance rule denied.

MCP may have been right to keep operation identity stable. HATEOAS may have been
right that the server should say what is possible *now*. Those are not competing
claims, and the next agent protocol will probably look less like hypermedia
returning from the dead than like us finally unbundling the two ideas HATEOAS
shipped together.

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
