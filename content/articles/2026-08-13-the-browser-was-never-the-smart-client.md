+++
title = "The Browser Was Never the Smart Client"
date = "2026-08-13"
description = "A browser can operate a site it has never seen, which is the strongest argument hypermedia has ever had, and it is usually told with one component missing. The browser understands anchors, forms, methods and URLs. It does not understand what cancelling an order costs, whether it is reversible, or how it differs from a refund — a human standing in front of the screen understands that. Delete the human and the same architecture stops working, not because links are hard to parse, but because a program cannot learn a business concept from the fact that the server gave it a URL. First of four on hypermedia, agents, and the part of the constraint worth keeping."
tags = ["API Design", "Protocols", "Architecture", "Complexity"]
+++

*First of four. This part is about what a machine client cannot understand.
Part II is about what traversal costs even when it does. Part III is about what
happened when the clients stopped being ordinary. Part IV is about the piece of
the idea that survives all three.*

Here is a hypermedia control. It is one line, it has barely changed since 1993,
and it underwrites the most successful distributed system anybody has ever
shipped.

```html
<a href="/orders/123/cancel">Cancel order</a>
```

The browser understands a great deal about that line, all of it generic. That
this is an anchor, that the thing inside the quotes is a
URL, that the URL can be resolved against the current document, that activating
it means an HTTP GET, that the response will carry a status and a content type,
that a `3xx` means somewhere else and a `4xx` means it went wrong. That is a
genuinely large amount of knowledge, all of it generic, none of it specific to
this server, and all of it shipped years before this particular order existed.

About *cancelling* it understands nothing. It does not know whether cancelling
this order is free or carries a restocking fee, whether it can be undone,
whether it is the same as a refund or a precondition for one, whether it will
annoy the warehouse, or whether it is the right thing to do at all given that
the customer only wanted to change the delivery address. Every one of those
questions has an answer, the answer matters, and the answer is not in the
markup.

Somebody supplies it. That somebody is a person, sitting in front of the
screen, who read the words "Cancel order" and already knew what those words
mean because they have lived in a world with orders in it. The browser rendered
an affordance. The human decided whether the affordance was worth taking.

<figure class="diagram">
<svg viewBox="0 0 620 258" role="img" aria-label="Two four-stage chains side by side. On the left, labelled the Web with a reader: server, then browser which knows anchors forms methods and URLs, then human who knows what cancelling costs and whether to do it, then action. On the right, labelled an API without one: server, then program which knows JSON relation names methods and URLs, then a dashed empty box where the reader would be, then a box reading meaning compiled into the client. A footer contrasts the client learning at read time with the meaning being compiled in before the first call.">
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="0" y="12">THE WEB &middot; WITH A READER</text>
    <text x="330" y="12">AN API &middot; WITHOUT ONE</text>
  </g>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="0" y="26" width="270" height="24" fill="none" stroke="var(--line)"/>
    <text x="135" y="42" fill="var(--ink)" text-anchor="middle">SERVER &middot; text/html</text>
    <rect x="0" y="72" width="270" height="40" fill="none" stroke="var(--line)"/>
    <text x="12" y="88" fill="var(--ink)">BROWSER</text>
    <text x="12" y="104" fill="var(--muted)">anchors, forms, methods, URLs</text>
    <rect x="0" y="134" width="270" height="40" fill="var(--signal)"/>
    <text x="12" y="150" fill="var(--paper)">HUMAN</text>
    <text x="12" y="166" fill="var(--paper)">what cancel costs &middot; whether to</text>
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
    <text x="475" y="42" fill="var(--ink)" text-anchor="middle">SERVER &middot; application/hal+json</text>
    <rect x="330" y="72" width="290" height="40" fill="none" stroke="var(--line)"/>
    <text x="342" y="88" fill="var(--ink)">PROGRAM</text>
    <text x="342" y="104" fill="var(--muted)">JSON, relation names, methods, URLs</text>
    <rect x="330" y="134" width="290" height="40" fill="none" stroke="var(--line)" stroke-dasharray="3 3"/>
    <text x="342" y="158" fill="var(--muted)">&mdash; nobody &mdash;</text>
    <rect x="330" y="196" width="290" height="24" fill="none" stroke="var(--ink)" stroke-width="1.5"/>
    <text x="475" y="212" fill="var(--ink)" text-anchor="middle">MEANING, COMPILED IN EARLIER</text>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M475 50 L475 66 M471 60 L475 66 L479 60"/>
    <path d="M475 112 L475 128 M471 122 L475 128 L479 122"/>
  </g>
  <path d="M475 174 L475 190" stroke="var(--line)" stroke-dasharray="3 3" fill="none"/>
  <text x="0" y="240" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the semantics arrive at read time</text>
  <text x="620" y="240" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">the semantics arrived at build time</text>
  <text x="0" y="254" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">and the third row is why the left column works</text>
</svg>
<figcaption>The browser is not the smart client. It is the generic half of a two-part client whose other half is a person. Carry the diagram over to a machine-to-machine API and the third row does not become smaller; it moves, into the client's source code, where it was always going to be written by hand.</figcaption>
</figure>

That missing row is the subject of this article. Not because it makes
hypermedia wrong (the left column of that figure is one of the best
architectural decisions anybody has made, and it is still paying) but because
almost every argument about REST in machine-to-machine APIs is conducted as
though the two columns were the same diagram, and they are not.

## The constraint, stated properly

State the idea in its strong form before arguing with it. The weak form is a
straw man and there is no sport in it.

Fielding lists four interface constraints: identification of resources,
manipulation of resources through representations, self-descriptive messages,
and hypermedia as the engine of application state. The fourth has a mechanism
attached. REST "concentrates all of the control state into the representations
received in response to interactions" — the client's model of what it may do
next comes from the last thing the server sent, and from nowhere else.

In practice that produces the familiar shape:

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

The order is pending, so `cancel` is there. Ship it, and `cancel` is gone. The
client never encodes the state machine; it re-reads the available transitions
out of each representation, and because they are derived from state the server
actually holds, they are never stale.

This is genuinely attractive, and I want to be honest about how attractive.
Every team that has ever shipped a bug where the client's idea of "cancellable"
drifted from the server's (a disabled button that should have been enabled, a
request rejected with a 409 that the UI swore was legal) has shipped the bug
this constraint exists to prevent. The server is the authority on its own
state, and having it say so in the response, in a form the client reads rather
than re-derives, is not a purist's affectation but the correct division of
responsibility.

Fielding's 2008 restatement is stricter than most people who quote it remember.
A REST API "must not define fixed resource names or hierarchies," should "spend
almost all of its descriptive effort in defining the media type(s)," should be
entered "with no prior knowledge beyond the initial URI (bookmark) and set of
standardized media types," and should "never have 'typed' resources that are
significant to the client." Any processing rule living outside the media type
is out-of-band information driving the interaction, which is the failure mode
the constraint exists to prevent.

That last clause decides everything that follows, because it determines whether
the constraint is about *coupling* or about *where the coupling is written
down*.

## The browser had a component nobody counted

The reason HATEOAS is easy to believe is that it was not invented; it was
extracted. There was a working system first. A browser really can operate a
site it has never seen, and no client release is needed when the site adds a
page, and this is not a thought experiment — it is Tuesday.

Carson Gross made the argument that explains why, and it deserves more credit
than it usually gets. His framing is about agency: a hypermedia client has to
decide, at runtime, which of the offered controls serves its purpose, and
"code doesn't (yet) have agency." A person does. The browser supplies uniform
mechanics — it knows what an anchor is, what a form is, how a method works —
and delegates every question of *meaning* upward, to a consumer that already
understands orders and refunds and shipping and consequences. Gross's
conclusion is blunt and, on the evidence, correct: "HATEOAS is largely wasted
on machines."

The precise formulation matters here, because the sloppy version invites an
easy rebuttal. It is not that HTML is smart. HTML is a standardized hypermedia
vocabulary, interpreted by a generic browser, and presented to an intelligent
reader. That *combination* is what makes evolution cheap. Change the form, add
a field, rename the button, introduce a whole new step in the flow, and no
client anywhere needs recompiling — because the component that has to
understand the change is a person, and people are extremely good at
understanding a changed page.

The observation has been rediscovered independently many times, usually in
comment threads rather than papers. One version, from a discussion of agent
tooling, puts it as well as any specification does: browser-like clients could
conform to the constraint because they "delegate all the hard parts … to a
human."

So the honest description of the Web's success is a two-part client. Generic
mechanics below, semantic agency above. When people carried the architecture
across to machine-to-machine APIs, they carried the bottom half and quietly
dropped the top, and then expressed surprise that the remaining piece did not
cover the gap.

## Semantic coupling is not accidental coupling

I have built this integration more times than I can count, and it goes the same
way every time.

You are handed an OpenAPI document. You do not read it. You run a generator
(`openapi-generator`, `oapi-codegen`, `openapi-typescript`, NSwag, whichever one
your language uses) and out comes a client with one method per `operationId`,
typed parameters, typed responses, typed errors. Then you write this:

```ts
const order = await api.getOrder({ orderId });
if (order.status === "pending") await api.cancelOrder({ orderId });
```

And inside the generated method, compiled into your build artifact, is the
string `/orders/{orderId}/cancel`.

<figure class="diagram">
<svg viewBox="0 0 620 214" role="img" aria-label="On the left a vertical pipeline: openapi.yaml feeds a code generator which produces a typed client exposing getOrder, cancelOrder and refundOrder. On the right a filled block titled what the developer supplied, listing that cancel ends an order before fulfilment, that refund moves money and is a different operation, and that cancelling after shipping is the wrong move. A footer notes the generator produced the path and the developer produced the meaning.">
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="0" y="12">WHAT THE TOOLING PRODUCED</text>
    <text x="330" y="12">WHAT IT COULD NOT</text>
  </g>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="0" y="26" width="270" height="24" fill="none" stroke="var(--line)"/>
    <text x="135" y="42" fill="var(--ink)" text-anchor="middle">openapi.yaml</text>
    <rect x="0" y="72" width="270" height="24" fill="none" stroke="var(--line)"/>
    <text x="135" y="88" fill="var(--ink)" text-anchor="middle">code generator</text>
    <rect x="0" y="118" width="270" height="76" fill="none" stroke="var(--ink)" stroke-width="1.5"/>
    <text x="12" y="136" fill="var(--muted)">TYPED CLIENT</text>
    <text x="12" y="154" fill="var(--ink)">getOrder(orderId)</text>
    <text x="12" y="170" fill="var(--ink)">cancelOrder(orderId)</text>
    <text x="12" y="186" fill="var(--ink)">refundOrder(paymentId, amount)</text>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M135 50 L135 66 M131 60 L135 66 L139 60"/>
    <path d="M135 96 L135 112 M131 106 L135 112 L139 106"/>
  </g>
  <rect x="330" y="26" width="290" height="168" fill="var(--signal)"/>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--paper)">
    <text x="342" y="52">cancel ends an order</text>
    <text x="342" y="70">&mdash; only before fulfilment</text>
    <text x="342" y="96">refund moves money</text>
    <text x="342" y="114">&mdash; a different operation entirely</text>
    <text x="342" y="140">after shipping, neither is right</text>
    <text x="342" y="158">&mdash; you want a return</text>
    <text x="342" y="184">none of this is in the document</text>
  </g>
  <text x="0" y="208" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the generator produced the path</text>
  <text x="620" y="208" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">the developer produced the meaning</text>
</svg>
<figcaption>A link would have saved the left column's last mile — interpolating an id into a path template. It would not have touched the right column, and the right column is where the work is.</figcaption>
</figure>

Fielding is unambiguous about what that generated client is: a REST API "must
not define fixed resource names or hierarchies (an obvious coupling of client
and server)," and defining fixed resource names is the generated client's
entire purpose. Nobody on the team experiences this as a violation. They
experience it as having finished by lunchtime.

The interesting question is not whether they broke a rule. It is which coupling
they actually took on, because there are two and they are usually billed as
one.

**Accidental coupling** is the client knowing which service owns the routing
table, which region a device was provisioned into, which internal table a field
came from. I have written before about
[what that costs and who ends up paying it](/articles/who-pays-for-the-pressure/):
pressure that was real inside the implementation, relocated onto the contract,
where every caller carries it forever.

**Semantic coupling** is the client knowing what a refund is. An invoicing
client coupled to the semantics of invoices is not a design failure — it is the
reason the client exists. You cannot decouple from it. A design that appears to
have done so has usually just stopped writing it down.

Ousterhout's test is the one I keep coming back to for deciding which one
hypermedia is charging for. A module earns its
keep by the ratio of what it hides to what it charges. Hypermedia charges every
client author a vocabulary: relation names, media types, traversal rules, a
discovery round trip before the first useful call. What it hides is URL
construction. That is a poor ratio, and it gets worse as the client gets
better, because a capable client was never going to struggle with URL
construction in the first place.

## Discoverability was never the expensive coupling

The strongest case for the constraint is decoupling: put URLs in responses and
the client stops hard-coding them, so the server can restructure freely. A real
benefit, just not the one the bill is for.

A client that wants to use `"cancel"` must know whether cancelling serves its
goal, what cancelling costs, what it requires,
what errors it can raise, whether it is reversible, and how it differs from
deleting, voiding, refunding and closing. The `href` was the cheap part. It was
always the cheap part.

That is not rhetoric; it is visible in the registry. Fielding's answer to this
objection is precise — relation names must be standardized, and descriptive
effort belongs in the media type. So go and look at what has actually been
standardized. The IANA link relations registry is a small, stable vocabulary
about documents: `next`, `prev`, `first`, `last`, `up`; `alternate`,
`stylesheet`, `icon`; `author`, `describedby`, `edit`, `search`. And, yes,
`payment`, registered against RFC 8288, whose entire published meaning is that
it indicates a resource where payment is accepted.

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
  <text x="0" y="70" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">registered &middot; a small stable set &middot; the same everywhere</text>
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

There is no `cancel`. There is no `refund`. There will not be, and it is not an
oversight — RFC 8288 handles this case by saying that extension relation types
are URIs. Which is the correction, and also the concession: you mint
`https://example.com/rels/refund`, you publish a page explaining what it means,
and every client author reads that page before writing a line. The processing
rules now live outside the media type, which by Fielding's own 2008 criterion
is out-of-band information driving the interaction.

So the honest comparison was never *contract versus no contract*. It is:

| | Where the meaning is written | What tooling can check |
|---|---|---|
| Hard-coded RPC | prose docs, client source | nothing |
| Hypermedia + custom rels | prose docs, per relation | that a link is present |
| Typed IDL | a schema the client compiles | types, at build time |

Basic link-style hypermedia moves the contract. It does not remove it. What it
removes is the machine-readable part: you trade a schema a validator can
enforce for a relation name a human has to look up. For a client that could not
read a schema anyway, that trade was nearly free.

That "basic" is doing a lot of work in that sentence, and I come back to it in
Part IV, because richer hypermedia formats do not make this trade and the
comparison above is unfair to them. For now the point stands against the shape
most people actually mean when they say hypermedia: a relation name and a
target.

## The one class of links everybody follows

There is an exception, and it is not a small one, because it marks the boundary
precisely.

Pagination.

```json
{
  "items": [ … ],
  "_links": { "next": { "href": "/orders?cursor=eyJvIjoxMjAwfQ" } }
}
```

Nobody writes a comment above that loop explaining why they didn't hard-code
page two. They fetch the href, append the items, repeat until `next` is absent,
and never think about it again. The same team that would refuse on principle to
follow a `cancel` link follows this one without noticing they made a decision.

They are right both times, and the reason is the whole boundary condition.

The relation is domain-generic: `next`, `prev`, `first`, `last` are the top row
of the figure above, registered, meaning the same thing on every server that
has ever existed. The processing rule lives in the specification, not in your
business. This is Fielding's condition genuinely satisfied, and it is one of
the very few places where it is.

And there is no decision to make. HATEOAS's expensive claim is that the
affordance *set* should be derived per response; here the set has size one —
continue, or stop. Presence or absence of the link is a boolean, and a boolean
requires no comprehension. Compare `cancel`, where the client must weigh an
operation against a goal before acting.

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

The dividend is real, and it is the one case where hypermedia's promised
freedom actually gets spent. Because the continuation is opaque — every one of
those four specifications insists on it, and Google's AIP-158 explains why, in
a line that could have come from
[the article I wrote about who pays for pressure](/articles/who-pays-for-the-pressure/):
if users can deconstruct a token, "_they will do so_," and your pagination
implementation becomes part of your API surface — a server can migrate from
`OFFSET` to a keyset predicate to a point-in-time snapshot scroll and not one
client changes a line.

So the generalization is not "links are useless." It is narrower and much more
useful:

> Following a link is cheap exactly when following it requires almost no
> understanding of what it means.

`next` clears that bar completely. `cancel` cannot clear it at all. Everything
in this series is downstream of which side of that line an affordance falls on.

## What actually failed

The diagnosis I would write down is narrow. HATEOAS did not fail because links
are hard to parse; JSON parsers are excellent. It failed because a static
program cannot discover a new business concept merely because the server gave
that concept a URL — and giving concepts URLs was the mechanism on offer.

The Web's client worked because it had two halves: generic mechanics that knew
nothing about the domain, and a reader that knew everything about it. Machine
integrations kept the first half. The second half did not disappear; it moved
into the client's source code, at build time, written by a developer who read
the documentation once and compiled their understanding into
`cancelOrder(orderId)`. That is where the semantics have lived for twenty
years, and it is why a link named `cancel` never saved anybody very much.

That is the semantic case, and I want to be clear that on its own it is only
half the indictment, because it can be answered. Hypermedia's defenders have
answered it the same way for twenty years: the clients are too stupid. Give me
a client that can read an unfamiliar affordance, work out what it means, and
choose the transition that serves its goal, and the constraint works exactly as
advertised.

Fine. Grant it. Now look at a team that has no comprehension problem at all —
a payments team that understands refunds perfectly, has read the documentation,
knows precisely what every relation in the response means — and watch them not
follow the links anyway.

That happens for reasons that have nothing to do with understanding, and it is
the half of the case nobody writes down. It is about round trips, and pipelines
that cannot fail on a change nobody declared, and the fact that a URL you were
handed at runtime is not a thing you can put on a dashboard. The bill comes due
in operations, not in comprehension, and it is itemized in Part II.

## References

**The constraint**

1. Roy T. Fielding, *Architectural Styles and the Design of Network-based
   Software Architectures*, University of California, Irvine, 2000 — chapter 5,
   the four interface constraints, the uniform-interface trade-off in 5.1.5,
   and control state concentrated in representations in 5.3.3.
   https://ics.uci.edu/~fielding/pubs/dissertation/rest_arch_style.htm

2. Roy T. Fielding, “REST APIs must be hypertext-driven,” 2008 — fixed resource
   names, descriptive effort in the media type, no prior knowledge beyond the
   initial URI, no typed resources, and out-of-band information as the failure
   mode.
   https://roy.gbiv.com/untangled/2008/rest-apis-must-be-hypertext-driven

3. Martin Fowler, “Richardson Maturity Model,” 2010 — the levels, and why level
   3 stayed rare.
   https://martinfowler.com/articles/richardsonMaturityModel.html

**The missing component**

4. Carson Gross, “HATEOAS is for Humans,” 2016 — agency as the missing
   ingredient, the browser as a generic client, and the argument that the
   constraint is largely wasted on machines.
   https://intercoolerjs.org/2016/05/08/hatoeas-is-for-humans.html

5. Carson Gross, “Hypermedia Clients” — what a uniform client actually is, and
   why adding links to JSON does not produce one.
   https://four.htmx.org/essays/hypermedia-clients

6. Carson Gross, Adam Stepinski, Deniz Akşimşek, *Hypermedia Systems* — the
   modern case for the constraint, on the surface where a human is reading.
   https://hypermedia.systems/

7. Hacker News, “OpenAI adds MCP support to Agents SDK,” March 2026 — community
   discussion, cited as recurring developer intuition rather than authority;
   the observation that browser-like clients conform by delegating the hard
   parts to a human.
   https://news.ycombinator.com/item?id=43485566

**The vocabulary**

8. RFC 8288, *Web Linking* — link relation types, and the rule that extension
   relation types are URIs.
   https://www.rfc-editor.org/rfc/rfc8288

9. IANA Link Relations registry — the registered vocabulary, including
   `payment`, and the absence of any domain verb resembling `cancel` or
   `refund`.
   https://www.iana.org/assignments/link-relations/link-relations.xhtml

10. JSON Hypertext Application Language (HAL) — the `_links` convention the
    examples follow.
    https://www.ietf.org/archive/id/draft-kelly-json-hal-11.html

**Pagination**

11. Google API Improvement Proposal 158, *Pagination* — `page_size`,
    `page_token`, `next_page_token`; tokens must be opaque and not
    user-parseable; and must not confer authorization.
    https://google.aip.dev/158

12. Model Context Protocol, *Pagination* — the opaque cursor model, `cursor`
    and `nextCursor`, and the MUST-treat-as-opaque client rules.
    https://modelcontextprotocol.io/specification/2026-07-28/server/utilities/pagination

13. GraphQL Cursor Connections Specification — `edges`, `cursor`, and
    `pageInfo` with `hasNextPage` and `endCursor`.
    https://relay.dev/graphql/connections.htm

14. Stripe API reference, *Pagination* — `starting_after`, `ending_before`,
    `has_more`.
    https://docs.stripe.com/api/pagination

**The ideas**

15. John Ousterhout, *A Philosophy of Software Design*, Second Edition. Yaknyam
    Press, 2021.
    https://web.stanford.edu/~ouster/cgi-bin/book.php

16. Fabio Ellena, “Who Pays for the Pressure,” 2026.
    https://fblln.github.io/articles/who-pays-for-the-pressure/
