+++
title = "HATEOAS Failed the Agent Test"
date = "2026-08-08"
description = "For twenty years hypermedia's defenders had one answer to every objection: clients are too dumb to follow the links. Then we built clients that aren't. LLM agents can read arbitrary JSON, interpret unfamiliar relations, and plan several moves ahead — and the protocol we handed them puts capabilities in a flat list, forbids that list from changing in response to what the client just did, and carries application state in an opaque argument. Read what the Model Context Protocol actually specifies and it is a point-by-point inversion of Fielding's fourth constraint. Then look at how anybody writes an HTTP client from an OpenAPI document, links and all, and notice they had already made the same decision."
tags = ["API Design", "Protocols", "Architecture", "Complexity"]
+++

Hypermedia as the engine of application state has always had one very good
answer to one very good objection.

The objection: nobody programs this way. The answer: because the clients were
not good enough yet. Real clients are compiled, brittle, and stupid; they
cannot look at a relation named `cancel` and work out what to do with it, so
their authors hard-code routes instead and the constraint never gets a fair
trial.

It was a fair answer for a long time. It is not available any more.

We now have clients that read arbitrary JSON, interpret relation names they
have never seen, hold a goal across several steps, and decide at runtime which
affordance is worth taking. Every property the defence asked for, we built.
And the protocol those clients actually got — the one that spread across the
industry in about eighteen months — puts the operations in a flat list, states
that the list **must not** change in response to what the client just did, and
passes application state as an opaque string argument.

That is not a lukewarm result. It is the inverse of the constraint, specified
in normative language, in the protocol written for the smartest clients we
have ever deployed.

## The constraint, stated properly

REST's fourth interface constraint is not a style preference. Fielding lists
four: identification of resources, manipulation of resources through
representations, self-descriptive messages, and hypermedia as the engine of
application state. The fourth one has a mechanism attached to it. REST
"concentrates all of the control state into the representations received in
response to interactions" — the client's model of what it may do next comes
from the last thing the server sent it, and from nowhere else.

In practice that produces the familiar shape:

```json
{
  "id": "123",
  "status": "pending",
  "_links": {
    "self":    { "href": "/orders/123" },
    "cancel":  { "href": "/orders/123/cancel" },
    "payment": { "href": "/orders/123/payment" }
  }
}
```

The order is pending, so `cancel` is present. Ship it and `cancel` disappears.
The client never encodes the state machine; it re-reads the available
transitions from each representation. That is the promise, and it is a real
one — the affordance set is always current because it is always freshly
derived from state the server actually holds.

Fielding's 2008 restatement is stricter than most people who quote it
remember. A REST API "must not define fixed resource names or hierarchies,"
should "spend almost all of its descriptive effort in defining the media
type(s)," should be entered "with no prior knowledge beyond the initial URI
(bookmark) and set of standardized media types," and should "never have
'typed' resources that are significant to the client." Any processing rule
that lives outside the media type is out-of-band information driving the
interaction, which is the failure mode the constraint exists to prevent.

Hold onto that last clause. It is the one the rest of this article keeps
returning to, because it is the clause that decides whether the constraint is
about *coupling* or about *where the coupling is written down*.

## What we actually shipped for agents

The Model Context Protocol is the interesting case because it is not a
compromise made under legacy pressure. It was designed recently, for LLM
clients specifically, by people with no REST installed base to protect.

It has nouns. `resources/list` returns URI-identified resources;
`resources/read` returns their contents. There are URI templates, custom
schemes, subscriptions for change notification. If you wanted to build
something hypermedia-shaped, the raw material is there.

Read what a resource read returns:

```json
{
  "contents": [
    { "uri": "file:///project/src/main.rs",
      "mimeType": "text/x-rust",
      "text": "fn main() {\n    println!(\"Hello world!\");\n}" }
  ]
}
```

Text or a base64 blob. A URI, a MIME type, an optional size, optional display
annotations. No links. No affordances. No transitions. Reading a resource
tells you what it contains and tells you nothing about what you may do next.
The noun half of the protocol is deliberately inert.

The verbs live somewhere else entirely. `tools/list` returns a flat array,
each entry carrying a name, a description, a JSON Schema for its input and
optionally one for its output. `tools/call` invokes one by name. Tools are
"model-controlled" by design — the spec's own phrase — meaning the model
discovers and selects them from that metadata.

<figure class="diagram">
<svg viewBox="0 0 620 214" role="img" aria-label="Two discovery shapes side by side. On the left, labelled navigate nouns, a vertical chain: fetch order 123, read the links object, follow the cancel link. Each step is connected by an arrow and annotated that the next step is unknowable until the previous response arrives. On the right, labelled list verbs, a single filled block for tools slash list feeds a stack of three named operations, and one arrow goes from the chosen operation to tools slash call. A note reads: the left column's shape depends on state, the right column's does not.">
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="0" y="12">HATEOAS &middot; NAVIGATE NOUNS</text>
    <text x="330" y="12">MCP &middot; LIST VERBS</text>
  </g>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="0" y="26" width="250" height="26" fill="none" stroke="var(--line)"/>
    <text x="125" y="43" fill="var(--ink)" text-anchor="middle">GET /orders/123</text>
    <rect x="0" y="82" width="250" height="26" fill="none" stroke="var(--line)"/>
    <text x="125" y="99" fill="var(--ink)" text-anchor="middle">read _links in the response</text>
    <rect x="0" y="138" width="250" height="26" fill="var(--signal)"/>
    <text x="125" y="155" fill="var(--paper)" text-anchor="middle">POST the href behind "cancel"</text>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M125 52 L125 76 M121 70 L125 76 L129 70"/>
    <path d="M125 108 L125 132 M121 126 L125 132 L129 126"/>
  </g>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="136" y="68">unknowable until</text>
    <text x="136" y="124">the previous reply lands</text>
  </g>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="330" y="26" width="290" height="26" fill="var(--signal)"/>
    <text x="475" y="43" fill="var(--paper)" text-anchor="middle">tools/list</text>
    <rect x="330" y="66" width="290" height="22" fill="none" stroke="var(--line)"/>
    <text x="340" y="81" fill="var(--ink)">cancel_order(order_id)</text>
    <rect x="330" y="94" width="290" height="22" fill="none" stroke="var(--line)"/>
    <text x="340" y="109" fill="var(--ink)">issue_refund(payment_id, amount)</text>
    <rect x="330" y="122" width="290" height="22" fill="none" stroke="var(--line)"/>
    <text x="340" y="137" fill="var(--ink)">get_account_balance(account_id)</text>
    <rect x="330" y="158" width="290" height="26" fill="none" stroke="var(--ink)" stroke-width="1.5"/>
    <text x="475" y="175" fill="var(--ink)" text-anchor="middle">tools/call</text>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M475 144 L475 152 M471 146 L475 152 L479 146"/>
  </g>
  <text x="0" y="206" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the left column's shape depends on state</text>
  <text x="620" y="206" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">the right column's is forbidden to</text>
</svg>
<figcaption>Both columns are dynamic — the tool list can change, and the spec has a notification for saying so. The difference is what it is allowed to change <em>in response to</em>. On the left that is the point of the design; on the right it is prohibited.</figcaption>
</figure>

So far this is only a shape difference, and shape differences are cheap to
argue about. The normative text is not.

## The sentence that settles it

Here is the clause, from the tools page of the current revision:

> Servers **MUST** respond to `tools/list` requests with the set of tools
> currently available to the requesting client. This set **MAY** be empty and
> **MAY** change over time … but **MUST NOT** vary per-connection or as a side
> effect of other requests on the connection.

Read it twice, because the first half is a distraction. Yes, the set may
change over time. There is a `notifications/tools/list_changed` message
specifically so servers can announce it. MCP is genuinely, deliberately
dynamic; a client is not entitled to assume the surface it saw last week.

It is the second half that matters. The tool set must not change *as a side
effect of other requests on the connection.* Which is to say: what you may do
next must not depend on what you just did.

That is the fourth constraint, negated, in a MUST NOT.

Under HATEOAS, `cancel` appearing in an order representation and vanishing
after shipment is not a quirk of the encoding — it is the mechanism. The
affordance set is a function of application state. MCP forbids exactly that
function. The verb list may vary with who is asking, and it may vary because
the server's own world changed. It may not vary because of where the client
has got to.

The same sentence appears, word for word, on the resources page. Neither half
of the protocol is permitted to be an engine of application state.

And once you notice that, the design reads as intentional rather than
accidental. A tool list that mutated as a consequence of prior calls could not
be cached, could not be given a `ttlMs` and a `cacheScope`, could not be
returned in a deterministic order for prompt-cache stability — all things this
revision explicitly asks for. Stable capability surfaces are what make the
client's context reusable. Hypermedia's central mechanism is, for this class
of client, a cache invalidation event on every single call.

## Application state became an argument

If the representations do not carry control state, something else must. The
spec addresses this directly, in a section titled *Stateful Tools*, and the
answer is worth quoting because it is the precise inverse of the constraint:

> MCP has no protocol-level session, so a server cannot rely on implicit
> per-connection state to relate one tool call to the next. Servers that need
> to maintain state across calls … should do so by returning an explicit
> handle from a creation tool and accepting that handle as an argument on
> subsequent calls.

And then, flatly: "The model is responsible for carrying `basket_id`
forward."

<figure class="diagram">
<svg viewBox="0 0 620 200" role="img" aria-label="Two ways of carrying application state. On the left, a representation block whose links object contains a cancel href, labelled control state inside the representation, with an arrow looping back into the client. On the right, a tool result block containing a basket identifier, an arrow carrying that identifier down into the arguments of the next tools slash call, labelled control state as an argument. A footer contrasts the server choosing the next move with the model choosing it.">
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="0" y="12">IN THE REPRESENTATION</text>
    <text x="330" y="12">IN THE ARGUMENTS</text>
  </g>
  <rect x="0" y="26" width="270" height="70" fill="none" stroke="var(--line)"/>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--ink)">
    <text x="12" y="44">{ "status": "pending",</text>
    <text x="12" y="60">&nbsp;&nbsp;"_links": {</text>
    <text x="12" y="76">&nbsp;&nbsp;&nbsp;&nbsp;"cancel": { "href": … } } }</text>
  </g>
  <rect x="0" y="122" width="270" height="30" fill="var(--signal)"/>
  <text x="135" y="141" font-family="var(--font-mono)" font-size="9" fill="var(--paper)" text-anchor="middle">the server picked the next move</text>
  <g stroke="var(--signal)" fill="none">
    <path d="M135 96 L135 116 M131 110 L135 116 L139 110"/>
  </g>
  <rect x="330" y="26" width="290" height="34" fill="none" stroke="var(--line)"/>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--ink)">
    <text x="342" y="47">result &rarr; basket_id: </text>
    <rect x="470" y="33" width="86" height="20" fill="var(--signal)"/>
    <text x="513" y="47" fill="var(--paper)" text-anchor="middle">bsk_a1b2c3</text>
  </g>
  <rect x="330" y="86" width="290" height="34" fill="none" stroke="var(--line)"/>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--ink)">
    <text x="342" y="107">add_item(basket_id: </text>
    <rect x="470" y="93" width="86" height="20" fill="var(--signal)"/>
    <text x="513" y="107" fill="var(--paper)" text-anchor="middle">bsk_a1b2c3</text>
    <text x="560" y="107">, …)</text>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M513 53 L513 62 L600 62 L600 74 L513 74 L513 90 M509 84 L513 90 L517 84"/>
  </g>
  <rect x="330" y="122" width="290" height="30" fill="none" stroke="var(--ink)" stroke-width="1.5"/>
  <text x="475" y="141" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">the model picked the next move</text>
  <text x="0" y="182" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">control state travels in the response</text>
  <text x="620" y="182" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">control state travels in the request</text>
  <text x="0" y="196" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">&mdash; and the transition set comes with it</text>
</svg>
<figcaption>An opaque handle is application state, and MCP puts it in the argument list rather than in a link. The handle names a cart; it does not say what may be done to the cart. That stays in the tool list, which is not allowed to change because a cart now exists.</figcaption>
</figure>

The guidance around handles makes the intent unambiguous. Handles should be
opaque, because ones "that encode internal structure invite parsing or
guessing." They should be authorization-checked on every call, because "a
handle is a name, not a capability." That is a designer looking straight at
the hypermedia idea — a URL you were handed, which is both the address and the
permission — and declining it.

## Discoverability was never the coupling

The strongest defence of HATEOAS is decoupling: put URLs in responses and the
client stops hard-coding them, so the server can restructure freely.

Look again at what a client does with `"cancel"`. To use it, it must know
whether cancelling serves its goal, what cancelling costs, what it requires,
what errors it can raise, whether it is reversible, and how it differs from
deleting, voiding, refunding, and closing. The `href` was the cheap part. It
was always the cheap part.

That is not a rhetorical claim; it is visible in the registry. Fielding's
answer to this objection is precise — relation names must be standardized, and
descriptive effort belongs in the media type. So go and look at what has
actually been standardized. The IANA link relations registry holds on the
order of 180 entries. `next`, `prev`, `first`, `last`, `up`. `alternate`,
`stylesheet`, `icon`. `author`, `describedby`, `edit`, `search`.
`predecessor-version`. And, yes, `payment`, registered against RFC 8288, whose
entire published meaning is: indicates a resource where payment is accepted.

<figure class="diagram">
<svg viewBox="0 0 620 176" role="img" aria-label="Two rows of link relations. The top row shows five outlined registered relations: next, edit, describedby, search and payment, annotated that a generic client can resolve these because they are registered and mean the same everywhere. The bottom row shows three filled domain operations: refund_payment, approve_loan and rotate_credentials, annotated that these are not registered, cannot usefully be, and must be defined in prose that the client reads out of band.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">WHAT A GENERIC CLIENT CAN RESOLVE ON ITS OWN</text>
  <g font-family="var(--font-mono)" font-size="9" text-anchor="middle">
    <rect x="0" y="28" width="96" height="24" fill="none" stroke="var(--line)"/>
    <text x="48" y="44" fill="var(--ink)">next</text>
    <rect x="106" y="28" width="96" height="24" fill="none" stroke="var(--line)"/>
    <text x="154" y="44" fill="var(--ink)">edit</text>
    <rect x="212" y="28" width="130" height="24" fill="none" stroke="var(--line)"/>
    <text x="277" y="44" fill="var(--ink)">describedby</text>
    <rect x="352" y="28" width="120" height="24" fill="none" stroke="var(--line)"/>
    <text x="412" y="44" fill="var(--ink)">search</text>
    <rect x="482" y="28" width="138" height="24" fill="none" stroke="var(--line)"/>
    <text x="551" y="44" fill="var(--ink)">payment</text>
  </g>
  <text x="0" y="70" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">registered &middot; ~180 of them &middot; the same everywhere</text>
  <g font-family="var(--font-mono)" font-size="9" text-anchor="middle">
    <rect x="0" y="94" width="196" height="24" fill="var(--signal)"/>
    <text x="98" y="110" fill="var(--paper)">refund_payment</text>
    <rect x="206" y="94" width="186" height="24" fill="var(--signal)"/>
    <text x="299" y="110" fill="var(--paper)">approve_loan</text>
    <rect x="402" y="94" width="218" height="24" fill="var(--signal)"/>
    <text x="511" y="110" fill="var(--paper)">rotate_credentials</text>
  </g>
  <text x="0" y="136" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">not registered &middot; not registerable &middot; documented in your prose</text>
  <path d="M0 150 L620 150" stroke="var(--line)" fill="none"/>
  <text x="0" y="170" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the standard vocabulary describes documents</text>
  <text x="620" y="170" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">your API describes a business</text>
</svg>
<figcaption>The registry is real, useful, and almost entirely about navigating and describing documents. Every verb that makes an API worth calling lives in the second row, where the standardized-relation argument runs out and the contract has to be written down somewhere a client can read it.</figcaption>
</figure>

There is no `cancel`. There is no `refund`. There will not be, and it is not
an oversight — RFC 8288 handles this case by saying extension relation types
are URIs. Which is the correction, and also the concession: you mint
`https://example.com/rels/refund`, you publish a page explaining what it
means, and every client author reads that page before writing a line. The
processing rules now live outside the media type, which by Fielding's own 2008
criterion is out-of-band information driving the interaction.

So the honest comparison is not *contract versus no contract*. It is:

| | Where the meaning is written | What tooling can check |
|---|---|---|
| Hard-coded RPC | prose docs, client source | nothing |
| Hypermedia + custom rels | prose docs, per-relation | that a link is present |
| gRPC / typed IDL | a `.proto` the client compiles | types, at build time |
| MCP tool | `description` + JSON Schema, on the wire | types, at call time, by the client |

Hypermedia moves the contract. It does not remove it. What it removes is the
machine-readable part — you trade a schema a validator can enforce for a
relation name a human has to look up. For a client that could not read a
schema anyway, that trade was nearly free. For a client that can, it is a
straight loss.

## How anybody actually writes an HTTP client

Put the agents aside for a moment, because none of this started with them. It
is worth watching an ordinary integration get built, by an ordinary team, on a
Tuesday.

You are given an OpenAPI document. You do not read it. You run a generator —
`openapi-generator`, `oapi-codegen`, `openapi-typescript`, NSwag, whichever one
your language uses — and out comes a client with one method per `operationId`,
typed parameters, typed responses, typed errors. Then you write this:

```ts
const order = await api.getOrder({ orderId });
if (order.status === "pending") await api.cancelOrder({ orderId });
```

And inside the generated method, compiled into your build artifact, is the
string `/orders/{orderId}/cancel`.

Fielding is unambiguous about what that is: a REST API "must not define fixed
resource names or hierarchies (an obvious coupling of client and server)." The
generated client's entire purpose is to define fixed resource names. Nobody on
the team experiences this as a violation of anything. They experience it as
having finished by lunchtime.

And it is not a tooling gap. This is the part worth sitting with, because the
usual explanation — hypermedia is fine, the tooling just never caught up —
does not survive contact with the spec. OpenAPI has carried a Link Object since
3.0. A response can declare a link naming the next operation by `operationId`,
with a `parameters` map that pulls its arguments straight out of the current
response body via runtime expressions:

```yaml
responses:
  '200':
    links:
      Cancel:
        operationId: cancelOrder
        parameters:
          orderId: $response.body#/id
```

That is HATEOAS's shape, expressed in a schema. The relationship, the target
operation, and where its arguments come from — all machine-readable. And note
where it lives, which Swagger's own documentation states plainly: the concept
"is somewhat similar to hypermedia, but OpenAPI links do not require the link
information present in the actual responses."

Read that as a design decision and it is the whole article in one line. The
graph was kept. It was moved into the description, where a client can compile
it, and taken out of the payload, where a client would have to traverse it.

Now ask what your generator emits for that `links` block. In practice: a
comment, or nothing. No mainstream generator produces a `follow("Cancel")`
method, and the reason is not neglect — there is nothing useful to produce. To
act on a link the code must already know what cancelling means, which means a
hand-written dispatch over relation names. That is hard-coding, moved later in
the program and typed worse.

So the ledger on a link, honestly kept. It saves you interpolating an id into a
path template. It costs you a nullable field to check, a round trip to learn
about an operation you had already compiled in, and the loss of any build-time
knowledge of what you can call. Teams are not ignoring hypermedia out of
laziness or ignorance of Fielding. They are declining a bad trade, correctly,
without ever writing down why.

GitHub is the case that settles it, because GitHub actually did the work. Its
REST responses are full of affordances — `url`, `html_url`, `commits_url` and
dozens of sibling URI templates, in every payload, one of the most genuinely
hypermedia-flavoured public APIs anyone shipped. Octokit is generated from
GitHub's OpenAPI description and calls routes by name. The publisher of the
links does not consume its own links, and neither does anybody else.

What everyone converges on instead is the useful tenth of the idea. Hard-code
the operations; read their *validity* off the response. An order carrying
`"status": "pending"`, or better, an explicit `"allowedActions": ["cancel",
"refund"]`, tells the client exactly what HATEOAS wanted it to know — which of
the operations it already understands are legal right now — and it does so as a
field. A schema can express it. A generator can turn it into an enum. Nothing
has to be fetched, parsed, or followed to find out.

That is the line the whole industry drew, twice, independently. Know the
operations at build time; ask the server only about availability. Agents did
not discover this. They inherited it, and then MCP wrote it into a spec.

## The one link everybody follows

There is an exception, and it is not a small one. It deserves better than a
footnote, because it is the case where hypermedia is unambiguously right and
where following a link is what every sane client already does.

Pagination.

```json
{
  "items": [ … ],
  "_links": { "next": { "href": "/orders?cursor=eyJvIjoxMjAwfQ" } }
}
```

Nobody writes a comment above that loop explaining why they didn't hard-code
page two. They fetch the href, append the items, repeat until `next` is absent,
and never think about it again. The team that would refuse to follow a `cancel`
link on principle will follow this one without noticing they made a decision.

They are right on both counts, and the reason they are right is worth spelling
out, because it turns out to be the whole boundary condition.

**The relation is domain-generic.** `next`, `prev`, `first`, `last` — the top
row of that registry figure, registered against RFC 8288, meaning the same
thing on every server that has ever existed. The processing rule lives in the
spec, not in your business. This is Fielding's condition genuinely satisfied:
standardized relation names, semantics in the media type, nothing out of band.
It is one of the very few places where it is satisfied.

**There is no decision to make.** HATEOAS's expensive claim is that the
affordance *set* should be derived per response. Here the set has size one:
continue, or stop. Presence or absence of the link is a boolean, and a boolean
requires no understanding. Compare `cancel`, where the client must weigh the
operation against its goal before acting — nothing to choose among means
nothing to comprehend.

**The token must be opaque, and that is the correct place for it.** MCP is
blunt: clients **MUST** treat cursors as opaque tokens — "Don't attempt to
parse or modify cursors" — and must draw no conclusion from a cursor's value
beyond whether one was provided. Google's AIP-158 says the same thing and then
explains why, in a sentence that could have been lifted from
[the article I wrote about who pays for pressure](/articles/who-pays-for-the-pressure/):
page tokens must be opaque and not user-parseable, because "if users are able
to deconstruct these, _they will do so_. This effectively makes the
implementation details of your API's pagination become part of the API
surface."

That is the real dividend, and it is the only routine case I can name where
hypermedia's promised freedom actually gets spent. Because the continuation is
opaque, a server can migrate from `OFFSET` to a keyset predicate to a
point-in-time snapshot scroll — three completely different mechanisms, with
different consistency guarantees — and not one client changes a line. The
pressure stayed inside the implementation, which is exactly where it belonged.

And now the part that makes your instinct exactly right: every RPC-shaped
system rebuilt this, independently, and they all landed on the same control.

<figure class="diagram">
<svg viewBox="0 0 620 200" role="img" aria-label="Four rows on the left, each naming a protocol and its continuation field: HAL with underscore links dot next dot href, MCP with nextCursor, Google AIP-158 with next_page_token, and Relay with pageInfo dot endCursor. Arrows from all four converge into a single filled block on the right reading: follow while present, stop when absent, never parse. A footer notes four protocols, one control, and zero interpretation at the client.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">ONE CONTROL, FOUR SPELLINGS</text>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="0" y="28" width="300" height="22" fill="none" stroke="var(--line)"/>
    <text x="10" y="43" fill="var(--muted)">HAL</text>
    <text x="100" y="43" fill="var(--ink)">_links.next.href</text>
    <rect x="0" y="58" width="300" height="22" fill="none" stroke="var(--line)"/>
    <text x="10" y="73" fill="var(--muted)">MCP</text>
    <text x="100" y="73" fill="var(--ink)">nextCursor</text>
    <rect x="0" y="88" width="300" height="22" fill="none" stroke="var(--line)"/>
    <text x="10" y="103" fill="var(--muted)">AIP-158</text>
    <text x="100" y="103" fill="var(--ink)">next_page_token</text>
    <rect x="0" y="118" width="300" height="22" fill="none" stroke="var(--line)"/>
    <text x="10" y="133" fill="var(--muted)">RELAY</text>
    <text x="100" y="133" fill="var(--ink)">pageInfo.endCursor</text>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M300 39 L350 39 L350 90"/>
    <path d="M300 69 L350 69 L350 90"/>
    <path d="M300 99 L350 99 L350 90"/>
    <path d="M300 129 L350 129 L350 90"/>
    <path d="M350 90 L374 90 M368 86 L374 90 L368 94"/>
  </g>
  <rect x="380" y="52" width="240" height="76" fill="var(--signal)"/>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--paper)" text-anchor="middle">
    <text x="500" y="78">follow while present</text>
    <text x="500" y="96">stop when absent</text>
    <text x="500" y="114">never parse it</text>
  </g>
  <text x="0" y="170" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">four protocols &middot; one control</text>
  <text x="620" y="170" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">zero interpretation at the client</text>
  <text x="0" y="188" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the only relation everybody actually follows</text>
</svg>
<figcaption>Stripe's <code>starting_after</code> and <code>has_more</code>, Kubernetes' <code>continue</code> token, and Elasticsearch's <code>search_after</code> are the same row again. Nobody copied anybody; the shape is forced by the problem.</figcaption>
</figure>

Look at where MCP sits in that list. The protocol whose normative text forbids
the capability surface from varying with client state paginates `tools/list`,
`resources/list`, `resources/templates/list`, and `prompts/list` — with an
opaque server-minted token the client follows blindly until it stops coming.
The one hypermedia control it kept is precisely the one that requires no
semantics to use.

The two camps even spell the continuation differently, and it turns out not to
matter. HAL puts it in `_links.next.href`, a URL you GET. MCP and AIP-158 put
it in a parameter you hand back. That is the same substitution as the basket
handle in the figure further up — control state in the response versus control
state in the request — and here, uniquely, nobody argues about it, because the
client interprets nothing either way. When there is no decision attached, the
link-versus-argument question collapses into a matter of spelling.

The two specs even converged on the same footgun. MCP: a handle "is a name,
not a capability." AIP-158: page tokens "**must not** provide any form of
authorization to the underlying resources." Two teams, different decades of
scar tissue, identical rule — because the moment an opaque token grants access
rather than naming a position, somebody will pass one around.

So the generalization is not "hypermedia failed." It is narrower and more
useful:

> Following a link is free exactly when following it requires no understanding
> of what it means.

`next` clears that bar completely. `cancel` cannot clear it at all. Everything
in this article is downstream of which side of that line an affordance falls
on.

One caveat, because the link is not magic. It names the continuation; it does
not make the continuation correct. Cursor stability under concurrent writes,
token expiry, and what page two means when page one's rows have shifted are
consistency problems the hypermedia control does not touch — which is why MCP
can only say servers **SHOULD** provide stable cursors and handle invalid ones
gracefully. That is a specification conceding that the hard part lives
somewhere the link cannot reach.

## The browser analogy was missing a component

HATEOAS is much easier to believe when you remember it was extracted from a
working system. A browser really can operate a site it has never seen. Send it

```html
<a href="/checkout">Checkout</a>
```

and it renders a link, and following it works. No coupling, no schema, no
client release.

But count the components. The browser knows anchors, forms, methods, and URLs
— the generic mechanics of the media type. It does not know what *checkout*
means. Somebody else supplies that, and that somebody is a human being reading
the page.

That division is one of the best architectural decisions ever made, and it is
still paying. The mistake was carrying the diagram across to machine-to-machine
APIs while quietly deleting the component that made it work, and then
expecting the remaining pieces to cover the gap. They could not, so the
semantics went back into custom client code, which is where they had been all
along.

The strongest evidence for this reading is the hypermedia revival — and it is
worth being precise about where it landed. htmx and the *Hypermedia Systems*
argument are a genuine, thoughtful return to the constraint, and they work
well. They work on the surface where a person is looking at the response. The
same idea, in the same decade, applied to the surface with no human in the
loop, produced `tools/list`. Both communities are behaving rationally. They
differ in whether there is a reader.

## Explicit coupling is not bad coupling

Somewhere along the way the industry started treating all shared knowledge
between client and server as a defect. It is worth separating the two kinds.

Accidental coupling is the client knowing which service owns the routing
table, which region a device was provisioned into, which internal table a
field came from. I have written about
[what that costs and who ends up paying it](/articles/who-pays-for-the-pressure/):
pressure that is real inside the implementation, relocated onto the contract,
where every caller carries it forever.

Semantic coupling is the client knowing what a refund is. An invoicing client
coupled to the semantics of invoices is not a design failure. It is the reason
the client exists. You cannot decouple from it, and a design that appears to
have done so has usually just stopped writing it down.

Ousterhout's test settles which one hypermedia is charging for. A module earns
its keep by the ratio of what it hides to what it charges. Hypermedia charges
every client author a vocabulary — relation names, media types, traversal
rules, a discovery round trip before the first useful call — and what it hides
is URL construction. That is a bad ratio, and it gets worse as the client gets
smarter, because a smart client was never going to struggle with URL
construction.

## What the smart clients turned out to need

The convergence is the part that is hard to argue with. MCP tools, Anthropic's
tool-use API, and OpenAI's function calling were designed by different teams
with different constraints, and they landed on the same three fields: a name,
a natural-language description, and a JSON Schema for the input. MCP's current
revision adds an output schema, an error channel split between protocol errors
the model cannot fix and execution errors it can, and a rule that names be
stable and unique. gRPC reached the same destination from the other end,
starting from a `.proto` service definition and generating the client.

None of these is a rejection of discovery. `tools/list` *is* discovery, and it
is better discovery than most REST APIs offer, because what comes back is
executable rather than merely addressable. What they reject is the specific
claim that discovery should happen by traversing links inside resource
representations.

That separation is the whole finding. Capability discovery and hypermedia were
always independent ideas, and HATEOAS bundled them. Once you can ask a server
*what can I do right now* and get typed answers, the bundle has no remaining
job.

MCP does have one thing that leans the other way, and it is worth naming
rather than hiding. A tool result can include a `resource_link` — a URI
pointing at something the client may then read. Servers can also answer a call
with `input_required`, asking for more input before completing. Both are
server-driven flow. But notice what neither one is: a `resource_link` points
at bytes, not at an operation, and `input_required` asks for another
*argument*, not for a link to be followed. The protocol will happily tell you
where more data lives. It will not tell you what to do next by handing you a
control.

## Where hypermedia still earns it

Pagination is the constraint working in the small. It also works in the large,
and Fielding scoped where, himself, in a passage that gets quoted far less than
the four constraints do. A uniform interface "degrades efficiency, since
information is transferred in a standardized form rather than one which is
specific to an application's needs," and REST is optimized "for the common case
of the Web," which leaves "an interface that is not optimal for other forms of
architectural interaction."

That is the author of REST saying the design is tuned for a particular case.
The cases where it still clearly wins share a family resemblance: many
independent clients that cannot be upgraded together, a genuinely
document-shaped domain, and either a human or a domain-generic consumer at the
end. The web. Atom and RSS. Crawlers and archives. `sitemap.xml`, which this
site generates, is hypermedia doing exactly its job. Long-lived public
document systems where the media type really is the contract.

An internal payments service is not that. Neither is a build service, nor
`create_calendar_event`. Forcing those into resources-and-transitions does not
make them more general; it makes the operations harder to see, and the
operations were the point.

## What the test actually showed

Twenty years of the hypermedia argument had one live hypothesis and no way to
test it: clients are not smart enough yet.

Then we built them, and give an agent a HAL document today and it will happily
follow the links — that part genuinely works now. The excuse expired. The
answer did not change. That is what makes this evidence rather than another
round of the argument.

The diagnosis I would write down is narrow, because the broad version is
wrong. HATEOAS is not a bad idea, and REST is not a mistake. It mistook one
kind of coupling for the important kind, attacking *where the operation lives*
and leaving *what the operation means* exactly where it was: in prose, outside
the media type, unreadable by any tool. For a client that could not read a
schema, moving the URL was the only available win.

We spent two decades waiting for a client good enough to navigate the graph.
It arrived. It read the schema instead.

## References

The MCP quotations are from revision `2026-07-28`, current at the time of
writing. The protocol moves quickly; the clauses cited here — the tool set
non-variance rule and the stateful-tools guidance — are the load-bearing ones
for this argument, and both should be re-read against whatever revision is
current when you do.

**The protocols**

1. Model Context Protocol, specification `2026-07-28`, *Tools* — the
   model-controlled interaction model, the `tools/list` non-variance rule, tool
   `name`/`description`/`inputSchema`/`outputSchema`, the *Stateful Tools*
   handle guidance, and the protocol-versus-execution error split.
   https://modelcontextprotocol.io/specification/2026-07-28/server/tools

2. Model Context Protocol, specification `2026-07-28`, *Resources* — the
   application-driven interaction model, `resources/list` and `resources/read`,
   the contents shape, and the identical non-variance rule.
   https://modelcontextprotocol.io/specification/2026-07-28/server/resources

3. JSON-RPC 2.0 specification — the method-plus-params call model MCP is built
   on.
   https://www.jsonrpc.org/specification

4. gRPC, *Introduction to gRPC* — service definitions, Protocol Buffers as IDL
   and wire format, generated clients.
   https://grpc.io/docs/what-is-grpc/introduction/

5. RFC 8288, *Web Linking* — link relation types, and the rule that extension
   relation types are URIs.
   https://www.rfc-editor.org/rfc/rfc8288

6. IANA Link Relations registry — the registered vocabulary, including
   `payment`, and the absence of any domain verb resembling `cancel` or
   `refund`.
   https://www.iana.org/assignments/link-relations/link-relations.xhtml

7. JSON Hypertext Application Language (HAL) — the `_links` convention the
   first example follows.
   https://datatracker.ietf.org/doc/html/draft-kelly-json-hal

8. Anthropic, tool use overview — `name`, `description`, `input_schema`; the
   convergence claim.
   https://docs.claude.com/en/docs/agents-and-tools/tool-use/overview

9. OpenAPI Specification — the Link Object: `operationId`, `operationRef`, the
   `parameters` map, and runtime expressions such as `$response.body#/id`.
   https://spec.openapis.org/oas/latest.html

10. Swagger documentation, *Links* — “The concept of links is somewhat similar
    to hypermedia, but OpenAPI links do not require the link information
    present in the actual responses.”
    https://swagger.io/docs/specification/v3_0/links/

11. GitHub REST API, and Octokit generated from GitHub's OpenAPI description —
    an API that emits URI templates in every payload, and the official client
    that calls routes by name instead.
    https://github.com/github/rest-api-description

12. Model Context Protocol, specification `2026-07-28`, *Pagination* — the
    opaque cursor model, `cursor` and `nextCursor`, the four list operations
    that support it, and the MUST-treat-as-opaque client rules.
    https://modelcontextprotocol.io/specification/2026-07-28/server/utilities/pagination

13. Google API Improvement Proposal 158, *Pagination* — `page_size`,
    `page_token`, `next_page_token`; tokens must be opaque and not
    user-parseable; and must not confer authorization.
    https://google.aip.dev/158

14. GraphQL Cursor Connections Specification — `edges`, `cursor`, and
    `pageInfo` with `hasNextPage` and `endCursor`.
    https://relay.dev/graphql/connections.htm

15. Stripe API reference, *Pagination* — `starting_after`, `ending_before`,
    `has_more`.
    https://docs.stripe.com/api/pagination

**The ideas**

16. Roy T. Fielding, *Architectural Styles and the Design of Network-based
    Software Architectures*, University of California, Irvine, 2000 — chapter
    5, the four interface constraints, the uniform-interface trade-off in
    5.1.5, and control state concentrated in representations in 5.3.3.
    https://ics.uci.edu/~fielding/pubs/dissertation/rest_arch_style.htm

17. Roy T. Fielding, “REST APIs must be hypertext-driven,” 2008 — fixed
    resource names, descriptive effort in the media type, no prior knowledge
    beyond the initial URI, no typed resources, and out-of-band information as
    the failure mode.
    https://roy.gbiv.com/untangled/2008/rest-apis-must-be-hypertext-driven

18. Martin Fowler, “Richardson Maturity Model,” 2010 — the levels, and why
    level 3 stayed rare.
    https://martinfowler.com/articles/richardsonMaturityModel.html

19. Carson Gross, Adam Stepinski, Deniz Akşimşek, *Hypermedia Systems* — the
    modern case for the constraint, on the surface where a human is reading.
    https://hypermedia.systems/

20. John Ousterhout, *A Philosophy of Software Design*, Second Edition.
    Yaknyam Press, 2021.
    https://web.stanford.edu/~ouster/cgi-bin/book.php

21. Fabio Ellena, “Who Pays for the Pressure,” 2026.
    https://fblln.github.io/articles/who-pays-for-the-pressure/

22. Fabio Ellena, “Architecture Must Follow Pressure,” 2026.
    https://fblln.github.io/articles/architecture-must-follow-pressure/
