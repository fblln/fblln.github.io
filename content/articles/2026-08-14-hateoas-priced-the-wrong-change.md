+++
title = "What Runtime Affordances Cost — and Buy"
date = "2026-08-14"
description = "Runtime controls keep the server authoritative about current transitions and invocation targets, but they move work from build time into execution. The relevant question is not whether HATEOAS won: it is whether applicability and binding change often enough to justify traversal, dynamic failure, and weaker static closure. Second of three on hypermedia and agents."
tags = ["API Design", "Protocols", "Architecture", "Complexity"]
+++

*Second of three. [Part I](/articles/the-browser-was-never-the-smart-client/)
separated operation semantics, current applicability and invocation binding.*

That distinction creates a temporal trade: compile stable knowledge once, or
discover current knowledge when it is needed. GitHub is a useful place to see
both choices operating in the same API.

GitHub's REST API is full of URLs: repositories point to commits, comments,
issues, assignees and archives, often through URI templates. GitHub also
publishes an OpenAPI description and uses it to generate the Octokit SDKs.

That coexistence is more interesting than either side admitting defeat. The
URLs are useful identifiers and navigation targets. The schema is useful as a
stable description from which a typed operation surface can be compiled. A
single API can publish both because they answer different questions.

It is tempting to turn GitHub into a verdict: even the publisher of all those
links chose generated methods, therefore traversal failed. The evidence does not
carry that much weight. URL fields are not necessarily typed state-transition
controls; an SDK does not prove that no consumer follows them; and a public API
with a stable route structure is one deployment, not the definition of all
distributed systems.

What GitHub does provide is a good place to ask the right question. If a client
can learn an operation from a description before execution or from a control
during execution, what changes when the information moves?

## The graph can live in two places

OpenAPI has carried a Link Object since 3.0. A response can declare that one
operation leads to another and bind parameters out of the preceding response:

```yaml
responses:
  '200':
    links:
      Cancel:
        operationId: cancelOrder
        parameters:
          orderId: $response.body#/id
```

That is a description of a possible relationship between operations. It lives
in an artifact a generator, validator or compatibility checker can inspect. It
does not necessarily assert that cancellation is available for every order
returned by this response, and the link information does not have to appear in
the response instance.

A runtime control makes a different assertion:

```json
{
  "id": "123",
  "_links": {
    "cancel": { "href": "/orders/123/cancel" }
  }
}
```

Its presence says something about this representation. In a sufficiently rich
media type it can also carry a method, input fields and content type. Ship the
order and the control can disappear without changing the class-level definition
of cancellation.

The design-time graph says what transitions the interface knows about. The
runtime graph says which transitions the server is offering here. Treating them
as substitutes hides the most useful information in each.

## What runtime binding buys

A server-provided control can carry at least four kinds of late-bound knowledge.

**Current applicability.** The server can derive the offered controls from
resource state, authorization, policy, inventory or any other fact it owns. A
client no longer has to reconstruct all of those predicates from response
fields. Absence may still be ambiguous — unauthorized, temporarily unavailable
and conceptually inapplicable are not the same — but presence is a useful
positive statement.

**An opaque target.** The client need not know how an identifier maps into a
path. More importantly, the target can encode a continuation, select a region,
route to an owning service, include a signature, or point at a temporary upload
location. Cosmetic route renaming is the least interesting version of this
benefit.

**Bound arguments.** The server can supply the order identifier, version token,
workflow handle or other values it already knows. The client provides only what
remains open. HTML forms have always done this with hidden fields; machine
formats can do the same without pretending the bound values are user input.

**Independent evolution of flow.** A server can insert a confirmation,
redirect a transition, or change the route through a workflow without asking
every client to reconstruct the path. This helps only if clients understand the
controls involved. Hypermedia can rearrange known concepts more readily than it
can introduce unknown ones.

These benefits share a condition: the late-bound fact must actually vary. If
every client release and server release move together, every order uses the
same target, and applicability is already required as a field, compiling the
operation may be simpler.

That is the benefit column. The cost column begins with the same condition in
reverse: a fact discovered at runtime has to be fetched, interpreted and
handled as uncertain even on days when it did not change.

## What runtime binding costs

The corresponding bill is real, but it is conditional rather than universal.

| Cost | When it appears | What can reduce it |
|---|---|---|
| Additional reads | the client does not already hold a fresh representation containing the control | bookmarks, events, caches, embedding controls in data already required |
| Weaker static closure | valid transitions depend on state that cannot be enumerated completely at build time | schemas for operation shape, compatibility checks for media types, contract tests |
| Runtime branching and failure | controls may be absent, stale or unfamiliar | explicit reason codes, version tokens, revalidation, fallback policy |
| Batch friction | applicability is exposed only one resource at a time | collection controls, bulk resources, query resources, job APIs |

None of those rows means a conforming client must walk from `/` before every
action. REST starts with an initial URI, but clients can retain identifiers,
receive links in events, cache safe responses and begin later work from a known
resource. The unavoidable part is smaller: before relying on a contextual
affordance, the client needs a sufficiently current statement that the
affordance exists.

Sometimes that read is pure overhead. A command processor receiving an order ID
solely to cancel it may prefer one `cancelOrder` call and let the server reject
an invalid transition. Sometimes the read was needed anyway. A support agent
deciding between an address change, cancellation and refund needs the current
order before it can make a sensible choice; controls can arrive with data it was
already going to fetch.

The same distinction matters for batch work. Per-resource controls can be an
awful interface to a reconciliation job over forty thousand records. That does
not imply that hypermedia forbids bulk operations. A collection can expose a
bulk transition, a query can select eligible resources, and a job resource can
represent asynchronous progress. It means only that an API exposing
applicability *solely* inside individual representations has optimized for a
different access pattern.

### The same operation in three workloads

Consider three callers that all need cancellation.

A queue consumer receives an explicit command:

```text
Cancel order 123 because the payment expired.
```

It already knows its goal, the order and the reason. Fetching an order solely to
discover a control adds latency without improving the decision. The efficient
shape is one command with an idempotency key or version precondition, followed
by a structured rejection if cancellation is no longer legal.

A support agent receives a different problem:

```text
The customer wants the package sent to a different address.
```

Now cancellation is only one candidate among address change, replacement,
refund and “do nothing.” The agent needs current order state before choosing.
If that representation also carries the applicable operations, the affordance
cost is almost free and may prevent a wrong branch.

A reconciliation job has a third shape:

```text
Find every expired unpaid order and cancel it before midnight.
```

Neither forty thousand individual reads nor forty thousand operation schemas is
the right abstraction. The API needs a collection query and probably an
asynchronous bulk job with its own controls. Calling that “less RESTful” would
confuse a resource model with an access pattern.

The architecture should follow the workload. Runtime affordances are not a tax
every caller must pay in the same way, and a command endpoint is not evidence
that contextual controls never earn their keep.

Workload determines how often the runtime bill is paid. The representation
model determines how much assurance can be bought before any workload begins.

## Static assurance does not disappear, but it changes scope

Typed descriptions earn their place before the first request. A pipeline can
diff two OpenAPI documents, detect a removed required field, regenerate a
client, and fail a build. A relation that conditionally disappears from one
resource cannot be classified the same way, because disappearance may be the
application behaving correctly.

That is not the same as saying a hypermedia API has no artifact to test. Media
types can be versioned and described by schemas. Relation vocabularies can have
compatibility rules. Producers and consumers can run contract tests. Rich forms
can validate request shapes. What static tooling cannot prove is the complete
set of controls that will be present for every future resource state — and that
incompleteness is the mechanism, not an accidental tooling gap.

The clean division is:

- Use static artifacts to check stable operation semantics and representation
  structure.
- Use runtime representations to report contextual applicability and binding.
- Validate state and authorization again when the operation executes.

Trying to force the second row into a closed build-time enum duplicates a state
machine. Refusing a schema for the first row gives up useful assurance for no
corresponding gain.

Compatibility is only one reason stable operation identity matters. Production
systems also need to aggregate, retry and govern calls after deployment.

## Operations need identities, not necessarily routes

Monitoring and retry policy are sometimes presented as fatal problems for
followed links. They are better understood as requirements on the control.

OpenTelemetry discourages raw URI paths as low-cardinality span names. That does
not mean only compiled clients can be observed. The server still knows its
matched route; a client may know a URI template; and application instrumentation
can name a span by a stable relation or operation identifier. A bare `href`
offers less help than a described operation, but traversal does not make the
request intrinsically anonymous.

Retry has the same shape. Basic HAL does not say which method submits a domain
transition. A form-oriented control does. Even then, `POST` alone cannot tell a
client whether repeating `cancelOrder` is safe after an ambiguous failure.
Idempotency keys, conditional requests, method semantics and domain policy still
matter. The valid criticism is that a useful machine control must carry or
reference this metadata, not that no hypermedia control can.

This is why link-only formats are a weak target for the larger architectural
argument. They make invocation dynamic while leaving too much of the operation
in prose. A richer control can carry or reference the missing identity and
method metadata. Whether that additional machinery is worth deploying depends
on the kind of change it protects the client from.

## Which change are you insuring against?

The original sales pitch for hypermedia often concentrates on route freedom:
clients follow server-provided targets, so the server may reorganize its URI
space. Mature public APIs make that particular claim look overpriced. Stripe,
for example, evolves aggressively through dated versions while treating stable
routes as part of the integration surface. Its documented compatible changes
are dominated by payloads, optional parameters, resources, event types and
opaque values — the places product evolution actually applies pressure.

That observation is useful, but the conclusion must remain scoped. It shows
that route relocation is not the main risk in a Stripe-shaped API. It does not
show that all invocation binding is stable. A server-selected target may carry
more than a prettier path:

- an upload location can be signed and short-lived;
- a continuation can encode partition position the client must not parse;
- a device can be routed to the region that currently owns it;
- a workflow can insert approval or confirmation without changing the business
  name of the final operation;
- a capability URL can intentionally combine an address with delegated access.

The last case is a security design, not a property of URLs in general. Most
links are names, not permissions. The point is that “the server might rename
`/orders`” is the weakest version of late binding, and measuring the whole idea
against that one event understates the claim.

The economic test is frequency multiplied by consequence. How often does the
binding change independently of the client? How damaging is a stale binding?
How much does discovering it cost? A decade of stable routes pushes toward
compiled clients. Per-request signatures or ownership routing push the other
way. State-specific applicability may change on every representation even when
the route never moves at all.

This also explains why a hybrid is not a compromise for its own sake. It lets
the stable route-shaped part remain compiled while retaining only the
late-bound facts that have demonstrated a reason to be late-bound.

MCP sharpens that separation because it was designed for clients that really can
discover and interpret operations at runtime, yet still makes the operation
catalog deliberately stable.

## MCP chose a different kind of runtime discovery

The Model Context Protocol is useful here, but not as a referendum on REST. MCP
is an agent-tool protocol, while HATEOAS is an architectural constraint on how
an application represents transitions. One can be layered on the other.

MCP nevertheless demonstrates something important: runtime discovery does not
require the operation vocabulary to be derived from application state.
`tools/list` returns named callables with descriptions and JSON Schemas;
`tools/call` invokes one by name. A client can meet a server at runtime, learn
its vocabulary, and still receive a catalog stable enough to validate and
cache.

MCP also exposes resources, but keeps them deliberately separate from tools.
`resources/read` returns identified content. It does not make the resource a
container of transitions. The conceptual split is stark:

| Channel | Unit | Answers |
|---|---|---|
| `tools/list` | described callable | what operations this server exposes |
| `tools/call` | name plus arguments | invoke one operation |
| `resources/read` | URI-identified content | retrieve some current data |

This is not “static API versus discovery.” Both sides discover at runtime. The
difference is whether the vocabulary is a property of the server or a property
of the client's current position in a resource graph.

The 2026-07-28 specification makes that separation normative. The tool set may
change over time and may differ according to authorization presented on a
request, but it must not vary per connection or as a side effect of earlier
requests on that connection. Deterministic ordering is recommended partly to
improve caching and prompt-cache hit rates.

That cache concern is unusually important for language models. Tool names,
descriptions and schemas are often serialized into model context. A stable,
deterministically ordered prefix can be reused; a tool surface that changes
after every action can invalidate that prefix. The 2026 revision reinforces the
choice with cache hints on list results. Hypermedia normally optimizes network
representations. MCP also has to optimize the prompt assembled from them.

The stateless core makes the same choice about ongoing work. If a server needs
state across calls, the guidance is to return an explicit handle and accept it
as an argument later:

```text
create_basket()             → basket_id = bsk_a1b2c3
add_item(bsk_a1b2c3, item)  → updated basket
checkout(bsk_a1b2c3)        → receipt
```

The handle lets the model carry identity from one call to the next. It still
does not say which of `add_item`, `checkout` or `abandon_basket` applies to this
basket now. State has become visible data while the operation catalog remains
global.

This is not HATEOAS failing an agent test. MCP was optimized for named tool
invocation, interoperability with existing function-calling systems, stateless
deployment and model context. Its adoption does not isolate any one of those
causes. It is evidence for a narrower proposition:

> Capability discovery and state-dependent affordance discovery are independent
> design choices.

MCP keeps the first and generally leaves the second to ordinary data or errors.
A tool list can say:

```text
cancel_order exists, means this, and accepts these arguments
```

It does not, by itself, say:

```text
this caller may cancel order 123 in its current state
```

An agent can infer that from fields, call and interpret an error, or use a
separate applicability mechanism. The tool schema is excellent at operation
identity and input shape. It is not a current statement about every resource to
which the operation might apply.

Put beside hypermedia, MCP completes the ledger. One design keeps the vocabulary
stable and makes applicability the client's problem. The other can make
applicability explicit but may repeat or rediscover the vocabulary. Which
pressure dominates depends on the deployment.

## Put the choices on one ledger

The choice is not between modern schemas and obsolete links. It depends on the
deployment.

Runtime affordances become more valuable when:

- clients and servers are released independently;
- workflows or targets change without changing business semantics;
- authorization and state strongly affect which transitions apply;
- clients already need a fresh representation to decide what to do;
- targets are opaque, temporary, signed or server-selected;
- many heterogeneous consumers share a stable media type or vocabulary.

Stable described operations become more valuable when:

- callers are batch-oriented or latency-sensitive;
- the operation set and routing are stable;
- clients and servers can coordinate releases;
- build-time compatibility gates prevent expensive incidents;
- callers already know exactly which command they intend to issue;
- the operation catalog is small enough to enumerate cheaply.

Many systems occupy both columns. A generated client can know that
`cancelOrder` exists while an order representation states whether it applies
now and binds the current order ID. That design does not require the tool list
to mutate, nor does it require the client to reverse-engineer eligibility from a
status enum.

[Part III](/articles/stable-verbs-dynamic-affordances/) gives that split a
concrete shape. It uses one rich hypermedia format, HAL-FORMS, to show what
contextual controls can carry, then asks whether a stable operation catalog and
a dynamic affordance layer perform better together than either does alone.

## References

**Descriptions and runtime links**

1. GitHub, *About the OpenAPI description for the REST API* — GitHub's OpenAPI
   description and generated Octokit SDKs.
   https://docs.github.com/en/rest/about-the-rest-api/about-the-openapi-description-for-the-rest-api

2. GitHub REST API description repository.
   https://github.com/github/rest-api-description

3. OpenAPI Specification — the Link Object, `operationId`, `operationRef`,
   runtime expressions and parameter binding.
   https://spec.openapis.org/oas/latest.html

4. Swagger, *Links* — OpenAPI links as design-time relationships whose
   information need not appear in response instances.
   https://swagger.io/docs/specification/v3_0/links/

**Costs and assurance**

5. Roy T. Fielding, *Architectural Styles and the Design of Network-based
   Software Architectures*, 2000 — the efficiency cost of the uniform
   interface and REST's optimization for the Web's common case.
   https://ics.uci.edu/~fielding/pubs/dissertation/rest_arch_style.htm

6. oasdiff — OpenAPI compatibility and breaking-change detection.
   https://github.com/oasdiff/oasdiff

7. OpenTelemetry, *Semantic Conventions for HTTP Spans* — low-cardinality span
   targets, `http.route` for servers, `url.template` for clients, and the
   prohibition on substituting raw paths.
   https://opentelemetry.io/docs/specs/semconv/http/http-spans/

8. RFC 9110, *HTTP Semantics* — safe and idempotent methods and retry after
   connection failure.
   https://www.rfc-editor.org/rfc/rfc9110

9. Stripe, *Versioning* and *Upgrades* — dated API versions and the published
   categories of backwards-compatible change.
   https://docs.stripe.com/api/versioning
   https://docs.stripe.com/upgrades

**MCP**

10. Model Context Protocol, specification `2026-07-28`, *Tools* — tool discovery,
   descriptions and schemas, the non-variance rule, authorization carve-out and
   deterministic ordering for caching.
   https://modelcontextprotocol.io/specification/2026-07-28/server/tools

11. Model Context Protocol, specification `2026-07-28`, *Resources* — resource
    discovery and reading as a channel separate from tool enumeration.
    https://modelcontextprotocol.io/specification/2026-07-28/server/resources

12. Model Context Protocol, specification `2026-07-28`, *Caching* — cacheable
    list results, `ttlMs` and `cacheScope`.
    https://modelcontextprotocol.io/specification/2026-07-28/server/utilities/caching

13. Model Context Protocol, “The 2026-07-28 Specification” — the stateless core,
    explicit handles and list-result caching.
    https://blog.modelcontextprotocol.io/posts/2026-07-28/

**In this series**

14. Fabio Ellena, “The Browser Was Only Half the Client,” 2026 — Part I.
    https://fblln.github.io/articles/the-browser-was-never-the-smart-client/
