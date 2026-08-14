+++
title = "The Browser Was Only Half the Client"
date = "2026-08-13"
description = "A browser can operate a site it has never seen because it shares the work with a person. That does not make hypermedia useless to machines, but it does reveal what the constraint can and cannot supply: controls can bind known operations to current resources without teaching a client what those operations mean. First of three on the contracts hidden inside HATEOAS."
tags = ["API Design", "Protocols", "Architecture", "Complexity"]
+++

*First of three. This part separates the three contracts that API designs keep
trying to make one mechanism carry. [Part II](/articles/hateoas-priced-the-wrong-change/)
asks when runtime controls are worth their cost.
[Part III](/articles/stable-verbs-dynamic-affordances/) builds the hybrid.*

One line of markup, barely changed since 1993, underwrites the most successful
distributed system anybody has shipped.

```html
<a href="/orders/123/cancel">Cancel order</a>
```

The browser understands a great deal about that line. It knows that this is an
anchor, that the value in quotes is a URI, that the URI can be resolved against
the current document, that activating it retrieves another representation, and
that the response will carry a status and a media type. None of that knowledge
is specific to this order or this server.

About *cancelling*, it understands nothing. It does not know whether cancellation
is free, whether it can be undone, how it differs from a refund, or whether it is
the right response to a customer asking to change an address.

Somebody supplies that judgment. On the Web it is normally a person who read
"Cancel order" and already understood orders. The browser supplies uniform
mechanics. The reader supplies a goal and domain knowledge. Together they make a
client that can enter an application it has never seen and still get useful work
done.

<figure class="diagram">
<svg viewBox="0 0 620 254" role="img" aria-label="Two four-stage chains. On the left a server sends HTML to a browser, a human supplies domain judgment, and an action follows. On the right a server sends a machine representation to a program, while domain knowledge arrives from a developer or model rather than from the representation itself.">
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="0" y="12">THE WEB</text>
    <text x="330" y="12">A MACHINE API</text>
  </g>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="0" y="26" width="270" height="24" fill="none" stroke="var(--line)"/>
    <text x="135" y="42" fill="var(--ink)" text-anchor="middle">SERVER · text/html</text>
    <rect x="0" y="72" width="270" height="40" fill="none" stroke="var(--line)"/>
    <text x="12" y="88" fill="var(--ink)">BROWSER</text>
    <text x="12" y="104" fill="var(--muted)">anchors, forms, methods, URIs</text>
    <rect x="0" y="134" width="270" height="40" fill="var(--signal)"/>
    <text x="12" y="150" fill="var(--paper)">HUMAN</text>
    <text x="12" y="166" fill="var(--paper)">goals, meaning, consequences</text>
    <rect x="0" y="196" width="270" height="24" fill="none" stroke="var(--ink)" stroke-width="1.5"/>
    <text x="135" y="212" fill="var(--ink)" text-anchor="middle">ACTION</text>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M135 50 L135 66 M131 60 L135 66 L139 60"/>
    <path d="M135 112 L135 128 M131 122 L135 128 L139 122"/>
    <path d="M135 174 L135 190 M131 184 L135 190 L139 184"/>
  </g>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="330" y="26" width="290" height="24" fill="none" stroke="var(--line)"/>
    <text x="475" y="42" fill="var(--ink)" text-anchor="middle">SERVER · machine representation</text>
    <rect x="330" y="72" width="290" height="40" fill="none" stroke="var(--line)"/>
    <text x="342" y="88" fill="var(--ink)">PROGRAM</text>
    <text x="342" y="104" fill="var(--muted)">media type, controls, transport</text>
    <rect x="330" y="134" width="290" height="40" fill="var(--signal)"/>
    <text x="342" y="150" fill="var(--paper)">DEVELOPER, POLICY OR MODEL</text>
    <text x="342" y="166" fill="var(--paper)">goals, meaning, consequences</text>
    <rect x="330" y="196" width="290" height="24" fill="none" stroke="var(--ink)" stroke-width="1.5"/>
    <text x="475" y="212" fill="var(--ink)" text-anchor="middle">ACTION</text>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M475 50 L475 66 M471 60 L475 66 L479 60"/>
    <path d="M475 112 L475 128 M471 122 L475 128 L479 122"/>
    <path d="M475 174 L475 190 M471 184 L475 190 L479 184"/>
  </g>
  <text x="0" y="244" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">hypermedia supplies the controls; something else supplies the purpose</text>
</svg>
<figcaption>The browser is the generic half of the Web client, not the whole of it. Machine clients need the same second row. Sometimes it is compiled by a developer, sometimes imposed by policy, and sometimes supplied at runtime by a model. A representation can expose a choice without supplying the goal by which that choice should be judged.</figcaption>
</figure>

That observation is often turned into a verdict: HATEOAS works for humans and
is wasted on machines. The verdict is too broad. It correctly identifies a job
hypermedia does not do, then treats that job as the only one worth doing.

The better question is which contract a control actually carries.

## What the constraint promises

Fielding's fourth uniform-interface constraint concentrates control state in the
representations a client receives. The server presents possible transitions;
the client chooses one. In a JSON format, the familiar minimal example looks
like this:

```json
{
  "id": "123",
  "status": "pending",
  "_links": {
    "self":   { "href": "/orders/123" },
    "cancel": { "href": "/orders/123/cancel" }
  }
}
```

Ship the order and the `cancel` control disappears. The useful claim is not that
the client learns the meaning of cancellation from six letters. It is that the
server remains authoritative about whether this particular order currently
offers that transition and where the transition leads.

That is narrower than the claim usually made for HATEOAS, but it is not small.
Every team has shipped some version of the bug where a client inferred
"cancellable" from a stale status mapping and the server rejected the call. A
control derived from the state and policy the server actually holds can remove
that duplicated inference.

It cannot remove the final race. The order may ship after the representation is
read and before the action is submitted. The server must still authorize and
validate every transition. Hypermedia makes the client's information more
current; it does not make distributed state stop changing.

Fielding is also less magical about prior knowledge than some summaries of REST.
Clients know protocols, media types, relation types and vocabularies. Some of
that vocabulary may be domain-specific. The constraint does not eliminate the
need for a shared language; it tries to concentrate that language in reusable
processing models and put the service-specific choices in-band.

The argument therefore is not *coupling versus no coupling*. It is what kind of
coupling exists, where it is recorded, and how often it changes.

## Three contracts hiding in one control

`cancel` appears to be one fact. It is at least three.

| Contract | Question it answers | Typical change rate |
|---|---|---|
| Operation semantics | What does cancellation mean, what inputs does it take, and what are its consequences? | slow |
| Current applicability | May this caller cancel this order in its present state? | fast |
| Invocation binding | Where and how is that transition invoked, and which arguments are already known? | medium or fast |

The first is a dictionary entry. The second is a statement about a situation.
The third connects the two to a transport.

A relation and an `href` compress all three into a small control, but they do not
describe all three equally well. `cancel` names a concept only a domain-aware
consumer can interpret. Its presence is good evidence of current applicability.
The `href` binds that applicability to an invocation target. Basic HAL says
nothing about a submission method or request fields; form-oriented hypermedia
formats do, and Part III takes them seriously.

OpenAPI and RPC systems usually arrange the contracts differently. They define
the operation and its invocation in a description that a client can compile,
then leave current applicability to status fields, prose, policy checks or
execution errors. Neither arrangement makes a contract disappear. Each makes a
different one authoritative at a different time.

This distinction is the spine of the series:

> Operation semantics, current applicability and invocation binding are
> different contracts with different update frequencies.

Once they are separated, the old choice between a typed API and HATEOAS stops
looking binary.

## The generated client drew one contract and omitted another

The ordinary machine integration makes the split visible. You are handed an
OpenAPI document, run a generator, and receive a client with one method per
`operationId`, typed parameters and typed responses. Then you write:

```ts
const order = await api.getOrder({ orderId });
if (order.status === "pending") {
  await api.cancelOrder({ orderId, reason });
}
```

The generated client knows that `cancelOrder` exists, where to send it and what
arguments it accepts. It does not know why the `pending` branch is correct. The
generator gave `cancelOrder`, `refundOrder` and `createReturn` the same standing;
the developer supplied the lifecycle from documentation and memory.

<figure class="diagram">
<svg viewBox="0 0 620 212" role="img" aria-label="An order lifecycle drawn left to right as created, paid, shipped and delivered. Beneath it, overlapping bars show where cancelOrder, refundOrder and createReturn apply. The generated methods do not contain this applicability row.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">THE ORDER LIFECYCLE</text>
  <g font-family="var(--font-mono)" font-size="9" text-anchor="middle">
    <rect x="0" y="22" width="140" height="26" fill="none" stroke="var(--line)"/>
    <text x="70" y="39" fill="var(--ink)">CREATED</text>
    <rect x="160" y="22" width="140" height="26" fill="none" stroke="var(--line)"/>
    <text x="230" y="39" fill="var(--ink)">PAID</text>
    <rect x="320" y="22" width="140" height="26" fill="none" stroke="var(--line)"/>
    <text x="390" y="39" fill="var(--ink)">SHIPPED</text>
    <rect x="480" y="22" width="140" height="26" fill="none" stroke="var(--line)"/>
    <text x="550" y="39" fill="var(--ink)">DELIVERED</text>
  </g>
  <g stroke="var(--line)" fill="none">
    <path d="M140 35 L156 35 M150 31 L156 35 L150 39"/>
    <path d="M300 35 L316 35 M310 31 L316 35 L310 39"/>
    <path d="M460 35 L476 35 M470 31 L476 35 L470 39"/>
  </g>
  <text x="0" y="72" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">WHERE EACH GENERATED METHOD APPLIES</text>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--paper)">
    <rect x="0" y="82" width="300" height="26" fill="var(--signal)"/>
    <text x="12" y="99">cancelOrder</text>
    <rect x="160" y="118" width="300" height="26" fill="var(--signal)"/>
    <text x="172" y="135">refundOrder</text>
    <rect x="320" y="154" width="300" height="26" fill="var(--signal)"/>
    <text x="332" y="171">createReturn</text>
  </g>
  <text x="0" y="202" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the schema defined the methods; the developer supplied this row</text>
</svg>
<figcaption>A typed description does an excellent job on operation identity and shape. Applicability is a different contract. Unless the API describes it explicitly, the client reconstructs it from fields, prose and failed requests.</figcaption>
</figure>

A bare `cancel` link would not teach the client the lifecycle either. What it can
do is replace the hand-drawn applicability row with a current statement: this
order offers cancellation now. Richer controls can also bind the order ID and
submission target. That is already enough to matter, even though the meaning of
cancellation remains shared knowledge.

The missing row therefore does not show that clients should know nothing. It
shows that they carry two very different kinds of knowledge, and only one of
them is a plausible target for architectural decoupling.

## Semantic coupling is not a design failure

An invoicing client coupled to the meaning of invoices is not broken. That
coupling is why the client exists. A refund processor has to know what a refund
does regardless of whether it calls `refundPayment(id)`, follows a
`refund-payment` relation, or submits an HTML form labelled "Refund."

What architecture can remove is accidental knowledge: which internal service
owns a route, how identifiers are interpolated, whether a target is regional or
signed, which transition is currently legal, or which fields the server can
already bind. Those are implementation or situation facts rather than the
business purpose of the client.

This gives hypermedia a more defensible ambition. It does not decouple the
consumer from the domain. It can decouple the consumer from parts of the
domain's current arrangement.

The value depends on how expensive those parts are. Interpolating an order ID
into a stable path is cheap accidental knowledge. Reproducing an authorization
policy that depends on fulfilment state, warehouse ownership and account risk
is not. “Hypermedia hides URL construction” is therefore both true and too
small: a useful control hides or reports whatever its media type is capable of
carrying. A link-only format carries little. A form can carry considerably
more.

The distinction also explains why a new relation is not automatically useful.
An adaptable client may be able to read a description of a relation it has
never seen, and a language model may reason about that description. But a new
URI alone does not teach a new business concept. Whether descriptions are
sufficient for genuinely novel operations is now an empirical question, not a
property delivered by the link syntax.

That leaves a practical question: if the link does not supply meaning, where
does the shared meaning come from? The answer is the same place it comes from
in every interface — a vocabulary — but hypermedia makes the shape of that
vocabulary especially visible.

## Shared vocabulary is still a contract

The IANA link-relation registry contains a compact vocabulary that transfers
well between domains: `next`, `prev`, `first`, `last`, `up`, `alternate`,
`describedby`, `edit`, `search`. A generic client can attach useful behavior to
those relations because their semantics were standardized elsewhere and stay
roughly the same everywhere.

Business verbs do not transfer as easily. `cancel-order`, `approve-loan` and
`rotate-credentials` carry rules specific to their domains. RFC 8288 permits
extension relation types identified by URIs, and Fielding explicitly allows
domain-specific vocabulary. You can publish
`https://example.com/rels/cancel-order`, give it a definition, and let clients
learn it.

The URI solves identity, not comprehension. A client still needs the relation's
definition, a domain model or a description it can reason about. That is not a
failure of RFC 8288; every protocol eventually rests on shared meaning. It does
mean that replacing `cancelOrder` with a relation URI has not removed the
semantic contract. It has changed how that contract is named and distributed.

The comparison becomes more useful when stated without absolutes:

| Interface style | Stable meaning lives in | Runtime representation can add |
|---|---|---|
| Generated RPC or OpenAPI client | operation description and client code | result data and errors |
| Basic link-style hypermedia | media type, relation definition and prose | relation presence and target |
| Form-oriented hypermedia | media type, relation definition and form rules | presence, target, method and current inputs |

None is contract-free. The important difference is which parts tooling can
check before deployment and which parts remain authoritative only at runtime.

At one end of that spectrum, the shared vocabulary is so small and settled that
the runtime control feels effortless. That is why the most successful machine
hypermedia example is also the least dramatic one.

## The relation everybody follows

Pagination marks the easy end of the spectrum:

```json
{
  "items": [ … ],
  "_links": { "next": { "href": "/orders?cursor=eyJvIjoxMjAwfQ" } }
}
```

Clients follow `next` without treating the choice as an architectural
commitment. The relation is broadly standardized, the target is deliberately
opaque, and the decision rule is tiny: follow while present, stop when absent.
No client has to weigh cancellation against a refund or understand a warehouse.

That success is not an exception to explain away. It reveals the boundary:

> Runtime controls are easiest to reuse when their semantics are already shared
> and choosing them requires little domain judgment.

Opaque continuation tokens, signed download links and server-selected upload
targets often share that shape. Their invocation binding changes independently;
their meaning does not.

Domain transitions sit further along the same continuum. A `cancel` control can
still be valuable, but not because it teaches cancellation. Its value comes
from the facts that *do* vary at runtime: that cancellation is applicable here,
that the server selected this target, and that some arguments may already be
bound.

We can now state the whole result of the browser example. Meaning must be shared;
applicability and binding need not be compiled. The remaining question is
whether learning those two facts at runtime costs less than the coupling it
removes.

## From distinction to price

The distinction creates a temporal trade. A stable operation description is
cheap to compile, validate, cache and monitor. A contextual control is current
and authoritative,
but has to be read, interpreted and revalidated at runtime. If applicability or
invocation changes independently of client releases, that price can be worth
paying. If neither changes, the control may be ceremony around a method the
client already knows.

[Part II](/articles/hateoas-priced-the-wrong-change/) puts both sides of that
ledger on the same page. It starts with an API that publishes both links and a
schema, follows the consequences into latency, compatibility and operations,
and ends with MCP — a useful counterexample showing that runtime discovery does
not have to make the operation vocabulary depend on application state.

## References

1. Roy T. Fielding, *Architectural Styles and the Design of Network-based
   Software Architectures*, University of California, Irvine, 2000 — chapter 5,
   the uniform-interface constraints, client-chosen transitions, and the
   efficiency trade-off of a uniform interface.
   https://ics.uci.edu/~fielding/pubs/dissertation/rest_arch_style.htm

2. Roy T. Fielding, “REST APIs must be hypertext-driven,” 2008 — initial URIs,
   media types, relation names, in-band controls, and the explicit acknowledgement
   that clients always need shared vocabulary.
   https://roy.gbiv.com/untangled/2008/rest-apis-must-be-hypertext-driven

3. Carson Gross, “HATEOAS is for Humans,” 2016 — the browser, agency and the
   human component of a uniform client.
   https://intercoolerjs.org/2016/05/08/hatoeas-is-for-humans.html

4. Carson Gross, “Hypermedia Clients” — why generic mechanics do not supply
   domain purpose, and why a link-only JSON format is not a complete client
   protocol.
   https://four.htmx.org/essays/hypermedia-clients

5. RFC 8288, *Web Linking* — registered and extension relation types.
   https://www.rfc-editor.org/rfc/rfc8288

6. IANA Link Relations registry — the standardized relation vocabulary.
   https://www.iana.org/assignments/link-relations/link-relations.xhtml
