+++
title = "HATEOAS Failed the Agent Test"
date = "2026-08-08"
description = "For twenty years hypermedia's defenders had one answer to every objection: clients are too dumb to follow the links. Then we built clients that aren't. LLM agents can read arbitrary JSON, interpret unfamiliar relations, and plan several moves ahead — and the protocol we handed them puts capabilities in a flat list, forbids that list from changing in response to what the client just did, and carries application state in an opaque argument. Read what the Model Context Protocol actually specifies, and it is a point-by-point inversion of Fielding's fourth constraint."
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
affordance set is a function of application state, and traversing it is how
the client's state advances. MCP forbids exactly that function. The verb list
may vary with who is asking — the spec allows it to vary by presented
authorization, on the reasoning that credentials are per-request input rather
than connection state — and it may vary because the server's own world
changed. It may not vary because of where the client has got to.

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

Set the two designs beside each other and the substitution is exact.
Hypermedia says: application state advances by following controls the server
embedded in the last representation. MCP says: application state is an opaque
token the *client* carries forward, and the set of things it can do with that
token is fixed independently of it.

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

HATEOAS aims at the transition structure — the client should not encode the
state machine. Worthwhile, and there is a real version of it: an order object
carrying `"status": "pending"` and a documented list of which operations are
valid in which status is the same information, in a form a schema can express
and a client can check before calling. What the constraint asks for on top of
that is that the *set of callable operations* be derived per-response, which is
the expensive half and the one MCP declined.

Ousterhout's test applies cleanly here. A module earns its keep by the ratio
of what it hides to what it charges. Hypermedia charges every client author a
vocabulary — relation names, media types, traversal rules, a discovery
sequence before the first useful call — and what it hides is URL construction.
That is a bad ratio, and it gets worse as the client gets smarter, because a
smart client was never going to struggle with URL construction.

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

The constraint is not wrong. It is scoped, and Fielding scoped it himself, in
a passage that gets quoted far less than the four constraints do. A uniform
interface "degrades efficiency, since information is transferred in a
standardized form rather than one which is specific to an application's
needs," and REST is optimized "for the common case of the Web," which leaves
"an interface that is not optimal for other forms of architectural
interaction."

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

Then we built them. Natural-language understanding, arbitrary JSON parsing,
planning, runtime selection among unfamiliar affordances — the whole list.
Give an agent a HAL document and it will follow the links; that part genuinely
works now. And when the same industry sat down to specify how those clients
should reach real systems, it wrote a protocol whose nouns return inert bytes,
whose verbs live in a separate list, and whose normative text forbids that
list from varying as a side effect of what the client just did.

The excuse expired and the answer did not change. That is what makes it
evidence rather than another round of the argument.

The diagnosis I would write down is narrow, because the broad version is
wrong. HATEOAS is not a bad idea, and REST is not a mistake. HATEOAS mistook
one kind of coupling for the important kind. It attacked *where the operation
lives* and left *what the operation means* exactly where it was — in prose,
outside the media type, unreadable by any tool. For a client that could not
read a schema, moving the URL was the only available win. For a client that
can, the schema was always the prize.

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

**The ideas**

9. Roy T. Fielding, *Architectural Styles and the Design of Network-based
   Software Architectures*, University of California, Irvine, 2000 — chapter 5,
   the four interface constraints, the uniform-interface trade-off in 5.1.5,
   and control state concentrated in representations in 5.3.3.
   https://ics.uci.edu/~fielding/pubs/dissertation/rest_arch_style.htm

10. Roy T. Fielding, “REST APIs must be hypertext-driven,” 2008 — fixed
    resource names, descriptive effort in the media type, no prior knowledge
    beyond the initial URI, no typed resources, and out-of-band information as
    the failure mode.
    https://roy.gbiv.com/untangled/2008/rest-apis-must-be-hypertext-driven

11. Martin Fowler, “Richardson Maturity Model,” 2010 — the levels, and why
    level 3 stayed rare.
    https://martinfowler.com/articles/richardsonMaturityModel.html

12. Carson Gross, Adam Stepinski, Deniz Akşimşek, *Hypermedia Systems* — the
    modern case for the constraint, on the surface where a human is reading.
    https://hypermedia.systems/

13. John Ousterhout, *A Philosophy of Software Design*, Second Edition.
    Yaknyam Press, 2021.
    https://web.stanford.edu/~ouster/cgi-bin/book.php

14. Fabio Ellena, “Who Pays for the Pressure,” 2026.
    https://fblln.github.io/articles/who-pays-for-the-pressure/

15. Fabio Ellena, “Architecture Must Follow Pressure,” 2026.
    https://fblln.github.io/articles/architecture-must-follow-pressure/
