+++
title = "Stable Verbs, Dynamic Affordances"
date = "2026-08-15"
description = "A tool catalog can describe what cancelOrder means without saying whether it applies to this order now. A resource can report current applicability without redefining the operation on every response. The useful synthesis is a stable typed vocabulary plus contextual affordances, with execution-time revalidation and an experiment honest enough to test whether the extra layer earns its keep. Third of three."
tags = ["API Design", "Protocols", "Architecture", "Complexity"]
+++

*Third of three. [Part I](/articles/the-browser-was-never-the-smart-client/)
separated the contracts; [Part II](/articles/hateoas-priced-the-wrong-change/)
priced the choice of carrying them at runtime.*

That leaves a precise gap: MCP can discover a stable operation at runtime, but
its tool catalog is not a statement about whether that operation applies to the
resource in front of the agent. Start with the part MCP already does well.

An MCP server can describe cancellation well:

```json
{
  "name": "cancel_order",
  "description": "Cancel an order before fulfilment begins.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "orderId": { "type": "string" },
      "reason": { "type": "string" }
    },
    "required": ["orderId", "reason"]
  }
}
```

The entry gives the operation a stable identity, explains its purpose and makes
its arguments machine-checkable. It can be discovered once, cached, placed in a
model's context and invoked by name.

It does not answer whether order `123` can be cancelled now.

The model may inspect a `status` field and infer a state machine from prose. It
may call the operation and learn from an error. The server may expose a separate
eligibility tool. Each can work. Each also recreates, in a different place, the
question HATEOAS wanted the current representation to answer: which transitions
does this resource presently offer?

That does not require making the tool catalog itself stateful. The catalog and
the resource can speak on different timescales.

Before joining those timescales, it helps to separate the kinds of knowledge
already compressed into a tool definition.

## A schema cannot carry the whole operation

JSON Schema is good at shape. It can say that `reason` is required, constrain a
string to an enum, bound an amount and describe the output structure. It cannot
by itself explain why cancellation differs from a refund, when a warehouse will
ignore it, or what promise the business makes after accepting it.

Tool protocols therefore combine schema with prose. Production descriptions
often ask one string to carry purpose, preconditions, side effects, interaction
with other tools, retry policy and error recovery.

Those are not variations of one fact. They drive different client behavior:

- **Purpose:** when should this operation be considered at all?
- **Precondition:** which resource states make the call meaningful?
- **Interaction:** must another operation precede or follow it?
- **Idempotence:** what should the client do after an ambiguous failure?
- **Validation:** which candidate inputs can be rejected before execution?
- **Side effect:** what else changes if the call succeeds?
- **Error semantics:** should the client stop, wait, refresh state or try a
  different operation?

An input schema can answer much of validation and some mechanical preconditions.
Annotations can express coarse properties such as read-only or destructive.
Prose still has to explain purpose and consequences. Current resource state must
answer whether the general rule applies to the thing in front of the agent.

Hasan and colleagues analysed 856 tools across 103 MCP servers and reported that
97.1% had at least one description defect under their taxonomy. Augmenting the
descriptions improved some outcomes, increased execution steps substantially,
and made a minority of cases worse. The result should not be read as a protocol
verdict or as proof that a schema can replace prose. It supports a smaller
claim: one prose field is a weak boundary for several different kinds of
knowledge.

The response is not to encode every consequence in a formal language. Purpose
and nuance belong in prose. Shapes belong in schemas. Current applicability
belongs with the state from which it was derived. The mistake is asking any one
of those channels to carry all three.

That allocation tells us what to look for in hypermedia. A bare link is not
enough; the useful comparison is a format that makes a transition structured
without pretending that structure supplies its business meaning.

## The stronger hypermedia example

Basic HAL is too thin to demonstrate the alternative. A relation and an `href`
name a possibility and bind a target, but do not describe how to submit it.
HAL-FORMS adds the missing transition metadata:

```json
{
  "id": "123",
  "status": "pending",
  "_links": {
    "self": { "href": "/orders/123" }
  },
  "_templates": {
    "cancel": {
      "method": "POST",
      "target": "/orders/123/cancel",
      "properties": [
        { "name": "reason", "required": true }
      ]
    }
  }
}
```

This control says more than “a thing called cancel exists.” It says that
cancellation is offered on this representation, identifies the target, selects
the method and declares the remaining input. The server can omit the template
after fulfilment begins or for a caller who lacks permission.

HAL-FORMS proves that structured runtime affordances are possible. It does not
prove that redefining the complete transition on every resource is ideal. The
method and `reason` field are normally the same for thousands of pending orders.
Repeating them keeps each representation self-contained, but spends bytes and
context restating stable knowledge.

The tool definition has the opposite profile. It describes cancellation once
and says nothing contextual. The two artifacts are not competitors:

| Stable operation definition | Contextual affordance |
|---|---|
| what cancellation means | whether cancellation applies here |
| general input and output schemas | arguments already bound for this resource |
| general side effects and safety policy | state-specific constraints |
| stable operation identifier | current invocation target |
| cached and checked as a contract | refreshed with resource state |

The useful part of HAL-FORMS is the right column. A stable tool catalog already
has a good left column.

The anchoring difference matters more than the syntax. An MCP definition says
“this server has an operation called cancellation.” A HAL-FORMS template says
“this representation offers cancellation.” If ten thousand orders share the
same method and schema, the first statement should not be repeated ten thousand
times merely to keep the second current. If two orders differ in policy,
inventory or version, the second statement cannot safely be replaced with the
first.

This is also where current constraints become useful rather than decorative.
The stable operation might allow a general `reason` enum of `customer_request`,
`payment_failure` and `inventory_failure`. A particular order may permit only
the first two, or require a supervisor token above some value. The contextual
affordance narrows a stable schema; it should not silently redefine the meaning
of the operation.

Once the two anchors are visible, the hybrid almost writes itself: keep the
definition with the server-wide vocabulary and let the resource carry only the
facts that vary with this situation.

## The hybrid

Keep the operation vocabulary where MCP and OpenAPI put it: named, described,
typed and stable enough to cache.

```json
{
  "id": "urn:example:operation:cancel-order:v1",
  "name": "cancelOrder",
  "description": "Cancel an order before fulfilment begins.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "orderId": { "type": "string" },
      "reason": { "type": "string" }
    },
    "required": ["orderId", "reason"]
  }
}
```

Then let the resource refer to that operation instead of redefining it:

```json
{
  "id": "order-123",
  "status": "pending",
  "version": "7",
  "affordances": [
    {
      "operation": "urn:example:operation:cancel-order:v1",
      "target": "/orders/123/cancel",
      "arguments": { "orderId": "order-123" },
      "constraints": {
        "reason": { "required": true }
      },
      "validForVersion": "7"
    }
  ]
}
```

The operation URI is a cross-reference, not a promise that a model can infer
the concept from the identifier. The catalog supplies meaning and general
shape. The resource supplies applicability, a current target, bound arguments
and any constraints that differ for this situation.

After shipment, the affordance is absent. Where policy permits, the server may
say why:

```json
{
  "id": "order-123",
  "status": "shipped",
  "version": "8",
  "affordances": [],
  "unavailableAffordances": [
    {
      "operation": "urn:example:operation:cancel-order:v1",
      "reason": "already_shipped"
    }
  ]
}
```

The negative entry is optional. Listing every unavailable operation could bloat
the response, and telling an unauthorized caller that an operation exists may
leak capability or policy information. A server might return reasons only for
operations the caller is allowed to know about, or expose a separate
applicability query when explanation is needed.

<figure class="diagram">
<svg viewBox="0 0 620 300" role="img" aria-label="A vertical hybrid architecture. A stable operation model containing identity, semantics and schemas is learned and cached above an agent. Current affordances containing applicable operations, bound arguments and versioned constraints are refreshed from resources below it. Execution revalidates both state and authorization.">
  <rect x="150" y="18" width="320" height="84" fill="none" stroke="var(--ink)" stroke-width="1.5"/>
  <g font-family="var(--font-mono)" font-size="9">
    <text x="164" y="36" fill="var(--muted)">STABLE OPERATION MODEL</text>
    <text x="164" y="56" fill="var(--ink)">identity · purpose</text>
    <text x="164" y="72" fill="var(--ink)">input / output schemas</text>
    <text x="164" y="88" fill="var(--ink)">general effects and safety policy</text>
  </g>
  <path d="M310 102 L310 132 M306 126 L310 132 L314 126" stroke="var(--line)" fill="none"/>
  <text x="320" y="122" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">learned once · cached</text>
  <rect x="240" y="136" width="140" height="34" fill="var(--signal)"/>
  <text x="310" y="157" font-family="var(--font-mono)" font-size="9" fill="var(--paper)" text-anchor="middle">AGENT</text>
  <path d="M310 202 L310 174 M306 180 L310 174 L314 180" stroke="var(--signal)" fill="none"/>
  <text x="320" y="194" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">refreshed with resource state</text>
  <rect x="150" y="206" width="320" height="74" fill="none" stroke="var(--signal)"/>
  <g font-family="var(--font-mono)" font-size="9">
    <text x="164" y="224" fill="var(--signal)">CURRENT AFFORDANCES</text>
    <text x="164" y="244" fill="var(--ink)">applicable operations · targets</text>
    <text x="164" y="260" fill="var(--ink)">bound arguments · current constraints</text>
    <text x="164" y="276" fill="var(--ink)">resource version · optional reasons</text>
  </g>
  <text x="0" y="296" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">stable verbs</text>
  <text x="620" y="296" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">dynamic affordances</text>
</svg>
<figcaption>Two channels, two update frequencies. The catalog is a dictionary the agent can cache. The resource is a situation report it must refresh. Neither makes execution-time validation optional.</figcaption>
</figure>

## Availability is advice, not authorization

A contextual affordance is a statement made at representation time. It can be
stale before the next request arrives. Two clients may act concurrently. A
policy may change. A warehouse may begin fulfilment one millisecond after the
order was read.

The action endpoint must therefore re-check resource version, authorization and
business invariants. `validForVersion` can let the client submit a conditional
request and distinguish “your view was stale” from “the operation does not
exist,” but it cannot eliminate contention.

This is important because “the server says what is possible now” can sound like
a correctness guarantee. It is better described as decision support:

- It reduces invalid choices made from duplicated client rules.
- It narrows the operation set the agent has to consider.
- It supplies current bindings and constraints.
- It never grants authority beyond what execution revalidates.

The same caution applies to security. Omitting a control can improve the user or
agent experience, but it is not access control. A malicious caller can construct
the request anyway. HATEOAS changes what cooperative clients are told, not what
adversarial clients are capable of sending.

That is the design in theory. The available experiments are small, but they are
useful because they fail and succeed along the same applicability boundary.

## Directional evidence

Robin Tegg implemented one order-management domain conventionally and with
Spring HATEOAS publishing HAL-FORMS affordances, then drove agent tasks through
the order lifecycle. The tasks included deliberately invalid transitions, which
is important: a benchmark made only of successful endpoint calls would test
argument construction and miss the applicability problem entirely. The
hypermedia version generally produced fewer unnecessary calls and fewer invalid
transitions in multi-step workflows.

The result points at a plausible mechanism. Models are already competent at
putting an order ID into a tool call. Their harder problem is selecting the
right operation after the world has changed. Removing inapplicable transitions
shrinks that decision rather than asking the model to remember a lifecycle from
prose. In the language of Part I, the intervention changes the applicability
contract while holding the business vocabulary roughly constant.

That fits the hypothesis, but remains directional evidence. One implementation,
one domain and one harness cannot establish that dynamic affordances beat
static tools in general. The descriptions, prompts, response shapes and amount
of guidance must also be controlled before the affordance layer can be isolated
as the cause.

Tegg's experiment addresses the correctness side of the trade. A second
experiment, by Antoine Bluchet and Kévin Dunglas, addresses the context side by
building a gateway from Hydra descriptions to MCP tools. Hydra matters here only
because it can express class-level operations and operations attached to
individual resources. The gateway discovers a small part of an API at a time
and reports dramatically less tool-schema context than a static OpenAPI
baseline in the authors' pilot benchmark.

The bridge has three stages:

1. It begins with one bootstrap tool that reads a Hydra resource.
2. Visiting a collection resolves its documented class and turns class-level
   operations into typed tools.
3. Operations attached directly to a resource can add or override what was
   learned from the class.

The first two stages are lazy vocabulary discovery. The third is contextual
applicability. Keeping them distinct matters because only the third makes the
visible tool surface a function of previous navigation.

The authors evaluated 19 tasks across four tiers, two models and three runs per
condition. They report full completion on the 15 positive-capability tasks and
a reduction in tool-schema share of input context from 77% for the static
baseline to 2% for the gateway. Those are striking numbers and deliberately
limited evidence: the authors designed the gateway, domain, harness and tasks,
and the result has not been independently reproduced.

The implementation exposes a compatibility tension, not a solved standard. Its
tool set grows after the agent navigates a resource. Under MCP's current rule,
the server's enumerated tool set must not vary as a side effect of prior
requests. The gateway therefore demonstrates a useful host or extension design,
and perhaps a reason to revisit that boundary; it is not evidence that
state-dependent tool registration straightforwardly conforms to the present
protocol.

The context result still matters. A complete static catalog pays for every tool
on every context assembly unless the host filters it. Fully dynamic discovery
pays in navigation and invalidation. The hybrid proposed here avoids changing
the catalog per resource: the stable tools remain fixed, while representations
refer to the subset that applies. Whether a model host can exploit that subset
without duplicating schemas in context is an implementation question worth
measuring.

Taken together, the studies expose two different pressures: transition mistakes
when applicability is implicit, and context cost when the whole vocabulary is
eagerly exposed. The proposed hybrid is an attempt to address both, not a result
either study has already proved.

## The alternatives on one page

The three candidates now have clear shapes:

| Model | Stable semantics | Current applicability | Main advantage | Main exposure |
|---|---|---|---|---|
| Static tool catalog | catalog description and schema | inferred from fields or errors | simple, typed, cacheable | transition mistakes and whole-catalog context |
| Dynamic HAL-FORMS | repeated in each template | control presence | self-contained and current | repeated schemas, traversal and context churn |
| Hybrid references | stable catalog | resource-level operation references | stable meaning plus contextual choice | new cross-reference and cache rules |

The hybrid does not dominate automatically. For a five-tool server with a
two-state resource, status fields and errors may be clearer than an affordance
layer. For a hundred-operation domain where policy changes the valid subset per
resource, asking a model to infer applicability may be the expensive design.
The break-even point depends on catalog size, state complexity, concurrency,
latency and how often clients already read the resource before acting.

There is also a deployment question. A resource may be served by one API while
the stable operation catalog comes from another tool server. A portable design
needs globally meaningful operation identifiers, rules for resolving them,
version compatibility and behavior when the client recognizes the reference but
does not currently have the tool. That is a small data model and a substantial
interoperability commitment.

That uncertainty is exactly why the series should end with an experiment rather
than a victory declaration.

## What would settle the argument

Build the same stateful domain behind three interfaces.

**A — Static catalog.** All operations are listed up front. The agent infers
applicability from fields and execution errors.

**B — Stable catalog plus contextual references.** The same operation names,
descriptions and schemas are listed up front. Resource responses additionally
name the operations that currently apply, bind known arguments and carry
versioned constraints.

**C — Fully dynamic forms.** Operations, methods and input shapes are discovered
from HAL-FORMS controls as the agent navigates.

Keep the backend, model, prompts, operation prose, schemas and authorization
rules equivalent. Randomize task order and repeat across several models and at
least two domains: one with a small obvious state machine, another with
policy-dependent transitions that cannot be inferred from a status enum.

Measure:

- successful task completion;
- invalid transitions attempted;
- unnecessary reads and calls;
- end-to-end latency;
- schema and response tokens placed in model context;
- prompt-cache hit rate;
- stale-affordance conflicts;
- recovery after state changes concurrently;
- leakage of operations an unauthorized caller should not learn about.

Then add the hardest test: introduce a genuinely new operation after the agent
was built. Give B and C the same description and schema. Does either let the
model use the new concept correctly, and what prior domain knowledge is doing
the work? A language model makes that question testable for the first time. It
does not make the answer automatic.

## Where the argument lands

HATEOAS is not a substitute for a schema. A schema is not a current account of
resource state. A tool catalog is not an authorization decision. A link is not
an explanation of a business concept.

Each becomes more useful when assigned the contract it can actually carry:

- stable descriptions define operation identity, semantics and general shape;
- current representations report applicability, bindings and situational
  constraints;
- execution revalidates authorization and state.

That architecture is not “HATEOAS was right” or “MCP replaced REST.” It is a
claim about update frequency. Definitions and situations change on different
clocks. Making one artifact authoritative for both forces either stable
knowledge to churn or contextual knowledge to go stale.

The hybrid has a real cost: it needs a standard cross-reference between a
resource and an operation catalog, client behavior for unknown references,
cache rules, security rules for negative affordances and a strategy for races.
Those costs should be tested against simpler status fields and ordinary errors,
not waved away because the design looks elegant.

But the proposition is now narrow enough to test and useful enough to matter:

> Keep the verbs stable. Let the server report which of them apply to the
> situation the client is actually in.

Stable verbs. Dynamic affordances.

## References

**Tool descriptions and protocol behavior**

1. Mohammed Mehedi Hasan, Hao Li, Gopi Krishnan Rajbahadur, Bram Adams, Ahmed
   E. Hassan, “Model Context Protocol (MCP) Tool Descriptions Are Smelly!
   Towards Improving AI Agent Efficiency with Augmented MCP Tool Descriptions,”
   2026 — description-defect taxonomy and augmentation results.
   https://arxiv.org/abs/2602.14878

2. Model Context Protocol, specification `2026-07-28`, *Tools* — stable tool
   discovery, schemas, list changes and the prohibition on per-connection or
   request-side-effect variance.
   https://modelcontextprotocol.io/specification/2026-07-28/server/tools

**Contextual affordances**

3. The HAL-FORMS Media Type — `_templates`, methods, content types and
   transition properties.
   https://rwcbook.github.io/hal-forms/

4. Spring HATEOAS reference documentation — affordances and HAL-FORMS output.
   https://docs.spring.io/spring-hateoas/docs/current/reference/html/

5. Hydra Core Vocabulary — class-level `supportedOperation`, resource-level
   operations, methods, expected inputs and possible statuses. Used here only
   for the gateway evidence rather than as a second explanation of the model.
   https://www.hydra-cg.com/spec/latest/core/

**Experiments**

6. Robin Tegg, “Hypermedia APIs for Autonomous Agents,” 3 June 2026 — the
   order-domain comparison and directional results on transition mistakes.
   https://robintegg.com/2026/06/03/hypermedia-apis-for-autonomous-agents.html

7. Antoine Bluchet, Kévin Dunglas, “APIs We Built Are Meant for Computers: How
   Do We Expose Hypermedia APIs to LLMs?” — the Hydra-to-MCP gateway and pilot
   evaluation.
   https://hal.science/hal-05630480

8. Hydra MCP Bridge — prototype and benchmark accompanying the paper.
   https://github.com/coopTilleuls/hydra-mcp-brigde

**In this series**

9. Fabio Ellena, “The Browser Was Only Half the Client,” 2026 — Part I.
   https://fblln.github.io/articles/the-browser-was-never-the-smart-client/

10. Fabio Ellena, “What Runtime Affordances Cost — and Buy,” 2026 — Part II.
    https://fblln.github.io/articles/hateoas-priced-the-wrong-change/
