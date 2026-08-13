+++
title = "HATEOAS Failed the Agent Test"
date = "2026-08-15"
description = "We finally built clients smart enough to navigate hypermedia. Then we gave them MCP. Every property hypermedia's defenders spent twenty years asking for — a client that reads arbitrary JSON, interprets an unfamiliar relation, holds a goal across many steps, chooses a transition at runtime — arrived at once, in a client nobody built for that purpose. The protocol written for those clients puts the operations in a flat described list, states that the list must not vary as a side effect of other requests on the connection, and carries application state as an ordinary argument. Third of four on hypermedia and agents."
tags = ["API Design", "Protocols", "Architecture", "Complexity"]
+++

*Third of four. [Part I](/articles/the-browser-was-never-the-smart-client/)
argued that the Web's hypermedia client was really two components, and that
machine-to-machine APIs kept the generic half and compiled the other half into
their source. [Part II](/articles/hateoas-priced-the-wrong-change/) itemized
what traversal costs a team that has no comprehension problem at all. This part
is about what happened when the missing half finally showed up.*

For twenty years, hypermedia had one very good answer to one very good
objection.

The objection: nobody programs this way. The answer: because the clients were
not good enough. Real clients are compiled, brittle and literal; they cannot
look at a relation named `cancel` and work out what to do with it, so their
authors hard-code routes instead and the constraint never gets a fair trial.

It was a fair answer, and it was also unfalsifiable, because for two decades
nobody had the client it asked for. It settles nothing on its own; Part II's
bill is still on the table. But it is the one objection a better client could
genuinely answer, and the objection the whole argument was built on.

We have it now. We have clients that read arbitrary JSON, interpret relation
names they have never seen, hold a goal across a dozen steps, and decide at
runtime which affordance serves it. Every property the defence stipulated,
delivered, mostly by people who had never read Fielding and were not trying to
settle an argument about REST.

This should have been hypermedia's moment.

And the protocol those clients actually got, the one that spread across the
industry in about eighteen months, puts the operations in a flat described
list, states that the list **MUST NOT** vary as a side effect of other requests
on the connection, and carries application state as an ordinary argument.

Two caveats first. The Model Context Protocol is not the only way to give an
agent tools, and adoption is not proof of optimality; plenty of widely deployed
protocols are widely deployed for reasons that have nothing to do with being
right.

What MCP is, is evidence, and the best available natural experiment. It was
designed recently, specifically for intelligent machine clients, by people with
no REST installed base to protect and no stake in the hypermedia argument. If
state-driven affordance discovery were the obvious architecture for a smart
client, this is where it would have shown up.

It did not show up. Something related did, and the difference between them is
the whole article.

## What MCP actually exposes

The protocol has nouns. `resources/list` returns URI-identified resources;
`resources/read` returns their contents, as text or a base64 blob, with a URI, a
MIME type, an optional size and optional display annotations. There are URI
templates, custom schemes, change notifications. The raw material for something
hypermedia-shaped is right there.

No links, though. No affordances, no transitions. Reading a resource tells you
what it contains and nothing about what you may do next. The noun half of the
protocol is deliberately inert.

The verbs live somewhere else entirely. `tools/list` returns an array, each
entry carrying a name, an optional title, a natural-language description, a JSON
Schema for its input and optionally one for its output. `tools/call` invokes one
by name. Tools are "model-controlled" by design, the specification's own phrase,
meaning the model discovers and selects them from that metadata.

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
<figcaption>Both columns are runtime discovery, and both are dynamic: the tool list can change, and the protocol has a notification for saying so. The difference is what each is allowed to change <em>in response to</em>. On the left that is the point of the design; on the right it is prohibited.</figcaption>
</figure>

So far this is a shape difference, and shape differences are cheap to argue
about. The normative text is not.

## The sentence that settles it

The clause, from the tools page of revision `2026-07-28`:

> Servers that declare the `tools` capability **MUST** respond to `tools/list`
> requests with the set of tools currently available to the requesting client.
> This set **MAY** be empty and **MAY** change over time … but **MUST NOT**
> vary per-connection or as a side effect of other requests on the connection.

The first half is a distraction, and it is the half most arguments stop at. The
set may change over time, and there is a `notifications/tools/list_changed`
message specifically so servers can announce it. MCP tools are not static, and
anybody who tells you they are has not read the page.

The specification spells out one dimension along which the set is explicitly
allowed to differ, and the reasoning is the useful part:

> The set **MAY** vary by the authorization presented on the request — for
> example, returning only the tools the caller's granted scopes permit — since
> credentials are per-request input, not connection state.

*Per-request input, not connection state.* That sentence is the design in
miniature. The tool surface may be a function of what you presented; it may not
be a function of where you have got to. Vary it because of what the client did
three calls ago and you have made the enumerated capability surface into a state
machine, which is exactly the thing being ruled out.

So the precise claim, the one people routinely overshoot, is not that MCP
forbids dynamism. It is this:

> The enumerated tool surface is not permitted to act as per-connection
> application state.

Set against Fielding's fourth constraint, that is close to an inversion. Under
HATEOAS, `cancel` appearing in an order representation and vanishing after
shipment is not a quirk of the encoding; it *is* the mechanism. MCP declines to
make the tool list that function. It says nothing about whether a call may fail,
or whether a result may report that shipment has already happened, since the
protocol has a whole error channel for that. What it removes is the idea that
the *list of what exists* should narrow and widen as you move.

The same rule reaches the nouns: this revision's changelog states flatly that
`tools/list`, `resources/list` and `prompts/list` no longer vary per-connection.
Neither half of the protocol may be an engine of application state.

## Why that rule is there

The rule reads as arbitrary until you notice what the rest of the revision is
optimizing for, at which point it reads as forced.

A tool list is not just a directory. For a language model it is *context*: the
descriptions and schemas are serialized into the prompt, they cost tokens on
every single turn, and they are the same tokens every turn. That makes the tool
list the single best caching target in the entire system, and this revision goes
after it hard.

Servers **SHOULD** return tools in a deterministic order, and the specification
says why in one sentence: deterministic ordering "enables clients to reliably
cache the tool list and improves LLM prompt cache hit rates when tools are
included in model context." `ttlMs` and `cacheScope` are no longer optional
garnish. They are *required* fields on the results of `tools/list`,
`prompts/list`, `resources/list`, `resources/read` and
`resources/templates/list`, one a freshness hint and the other a statement about
whether a shared intermediary may hold the response.

Now imagine the hypermedia mechanism dropped into that design. A capability
surface that mutated as a consequence of prior calls could not be given a
meaningful TTL, could not be cached by an intermediary, could not be returned
in a stable order, and would invalidate the model's prompt prefix on every
turn. For this class of client, that mechanism is a cache invalidation event on
every single call.

That is not a protocol designer failing to appreciate hypermedia. That is a
protocol designer looking at a client whose context window is the scarcest
resource in the system and declining to put a moving target in it.

## Application state became ordinary data

If the representations do not carry control state, something else must, and the
answer is the precise inverse of the constraint. This revision removed
protocol-level sessions outright: no `Mcp-Session-Id`, no `initialize` handshake,
every request carrying its own version and capabilities. There is nowhere for
implicit per-connection state to live any more.

So state is carried explicitly:

> Servers that need to maintain state across calls … should do so by returning
> an explicit handle from a creation tool and accepting that handle as an
> argument on subsequent calls.

And then, flatly: "The model is responsible for carrying `basket_id` forward."

That section is design guidance rather than a MUST, and the note attached to it
is the most telling sentence on the page: the protocol "has no concept of a
state handle; from the wire's perspective a handle is an ordinary string in a
tool result and an ordinary argument to subsequent tool calls."

That is a larger claim than a normative one, not a smaller one. The protocol
does not model application state at all. It declines to have an opinion, and
what falls out of that refusal is state as data: a string the model reads out of
one result and types into the next.

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
<figcaption>A handle is application state, and it travels in the argument list rather than in a link. The handle names a cart; it does not say what may be done to the cart. That stays in the tool list, which is not allowed to change because a cart now exists.</figcaption>
</figure>

The design notes make the intent unambiguous. Handles should be opaque, because
ones "that encode internal structure invite parsing or guessing," and
authorization-checked on every call, because "a handle is a name, not a
capability." That is a designer looking directly at the hypermedia idea (a URL
you were handed, which is simultaneously the address and the permission) and
separating the two halves on purpose.

## MCP did not reject discovery

MCP performs runtime discovery. `tools/list` *is* discovery, and it is better
discovery than most REST APIs offer, because what comes back is executable
rather than merely addressable: a name you can call, a schema you can validate
against, a description you can reason about, and an output shape you can check.
An agent that has never seen a server before learns its entire vocabulary in
one round trip.

What was declined is a specific *form* of discovery. Hypermedia derives the
operation, its invocation and its availability from one document, in one
channel. MCP splits them into two: a vocabulary stable enough to cache, and
state that arrives separately, as data. Neither client knew the server in
advance.

So the finding is narrower than "hypermedia lost," and considerably more
useful:

> Capability discovery and hypermedia traversal are not the same thing.

They were always independent ideas, and HATEOAS bundled them. The same mechanism
that told you an operation *exists* also told you it was *available now* and
*how to invoke it*, so twenty years of argument treated accepting one as
accepting all three. MCP took the bundle apart and kept the pieces it wanted.

MCP's unit of discovery is a described callable operation. HATEOAS's is a state
transition embedded in a representation. That sentence is the difference between
the two architectures, and everything else is consequence.

## The irony had already been noticed

People saw it coming and said so in public, before the result was in. In
developer discussion from around the time agent tooling started spreading, the
observation turns up unprompted: the irony behind HATEOAS is that LLMs are the
mythical "evolvable agents" necessary to make it work in the first place. These
are comment threads rather than evidence about protocol behaviour, and I offer
them as a record of what practitioners expected.

Which makes the outcome interesting rather than merely disappointing. The
resurrection was predicted, by the right people, for the right reasons, at the
right moment. The dominant protocol still chose described tools.

## What a schema does not tell you

Six months ago I would have concluded that once you can ask a server *what can I
do* and get typed answers back, the hypermedia bundle has no remaining job. That
is too strong, and the distinction in the last section is why.

MCP answers **operation identity** extremely well. What is this operation
called, what does it do, what arguments does it take, what comes back, what
kinds of error can it raise, is it read-only or destructive. Those questions
get a name, a description, two JSON Schemas and a set of annotations, all
machine-readable, all validatable, all stable enough to cache.

MCP does not, on its own, answer **operation availability**. A tool list says:

```text
these are the tools this server exposes
```

It does not say:

```text
for this order, in this state, right now, these are the valid actions
```

An agent holding `cancel_order` in its vocabulary and an order representation in
its context still has to work out whether cancelling *this* order is currently
legal. It can guess from a `status` field, if the server thought to include one.
It can try the call and read the error, and the protocol supports that well,
splitting execution errors a model can act on from protocol errors it cannot.
But guessing and probing are what the fourth constraint was designed to make
unnecessary, and on this question hypermedia's answer was better.

The protocol does lean this way in places. A tool result can carry a
`resource_link`, a URI pointing at something the client may then read. A call can
come back `input_required`, and the client retries with the answers. Both are
server-driven flow, but a `resource_link` points at bytes rather than an
operation, and `input_required` asks for another *argument* rather than a control
to follow. The protocol will tell you where more data lives. It will not hand
you the next move.

## What the test actually showed

Twenty years of the hypermedia argument had one live hypothesis and no way to
test it: the clients are not smart enough yet.

Then we built them. Give an agent a HAL document today and it will follow the
links; that part genuinely works now, and Part IV has the measurement. The
excuse expired. The answer did not change. That is what makes this evidence
rather than another round of the same argument.

The diagnosis I would write down is narrow, because the broad version is wrong.
HATEOAS is not a bad idea and REST is not a mistake. The constraint attacked
*where the operation lives* and left *what the operation means* exactly where it
was: in prose, outside the media type, unreadable by any tool. For a client that
could not read a schema, moving the URL was the only available win. For a client
that can, it is the smaller half of the problem, and the protocol built for
those clients spent its effort on the larger half.

We spent two decades waiting for a client good enough to navigate the graph. It
arrived. It read the schema instead.

But a schema answers what an operation *is*, not when it is valid for the
particular thing the agent is looking at, and that question did not stop
mattering just because the tool list stopped answering it. That missing piece is
where the older hypermedia work becomes interesting again, and it is not the
part this article has been arguing with. HAL-FORMS, Hydra and the Web of Things
do not make basic HAL's trade. Part IV is about what happens when you take them
seriously.

## References

MCP citations are from revision `2026-07-28`, current at the time of writing.
The protocol moves quickly; the clauses this argument rests on — the
`tools/list` non-variance rule with its per-request authorization carve-out,
the required cache metadata, and the stateful-tools guidance — should be
re-read against whatever revision is current when you do.

**The protocol**

1. Model Context Protocol, specification `2026-07-28`, *Tools* — the
   model-controlled interaction model, the `tools/list` non-variance rule and
   its authorization carve-out, deterministic ordering for prompt-cache hit
   rates, tool `name`/`description`/`inputSchema`/`outputSchema`, the
   non-normative *Stateful Tools* handle guidance, `resource_link` and
   `input_required` results, and the protocol-versus-execution error split.
   https://modelcontextprotocol.io/specification/2026-07-28/server/tools

2. Model Context Protocol, specification `2026-07-28`, *Key Changes* — removal
   of protocol-level sessions and the `initialize` handshake, list endpoints no
   longer varying per connection, server-minted handles as ordinary tool
   arguments, and `ttlMs`/`cacheScope` as required fields on list results.
   https://modelcontextprotocol.io/specification/2026-07-28/changelog

3. Model Context Protocol, specification `2026-07-28`, *Resources* — the
   application-driven interaction model, `resources/list` and `resources/read`,
   and the contents shape.
   https://modelcontextprotocol.io/specification/2026-07-28/server/resources

4. Model Context Protocol, specification `2026-07-28`, *Caching* — `ttlMs` and
   `cacheScope`, and what a shared intermediary may hold.
   https://modelcontextprotocol.io/specification/2026-07-28/server/utilities/caching

5. JSON-RPC 2.0 specification — the method-plus-params call model MCP is built
   on.
   https://www.jsonrpc.org/specification

6. Anthropic, tool use overview — `name`, `description`, `input_schema`; the
   convergence with MCP's tool shape.
   https://docs.claude.com/en/docs/agents-and-tools/tool-use/overview

**The constraint**

7. Roy T. Fielding, *Architectural Styles and the Design of Network-based
   Software Architectures*, 2000 — chapter 5, and control state concentrated in
   representations in 5.3.3.
   https://ics.uci.edu/~fielding/pubs/dissertation/rest_arch_style.htm

8. Roy T. Fielding, “REST APIs must be hypertext-driven,” 2008.
   https://roy.gbiv.com/untangled/2008/rest-apis-must-be-hypertext-driven

9. JSON Hypertext Application Language (HAL) — the `_links` convention.
   https://www.ietf.org/archive/id/draft-kelly-json-hal-11.html

**Community observation**

10. Hacker News, “OpenAI adds MCP support to Agents SDK,” March 2026 — the
    observation that LLMs are the “evolvable agents” HATEOAS required, and that
    browser-like clients conformed by delegating the hard parts to a human.
    Cited as developer argument, not as authority on protocol behaviour.
    https://news.ycombinator.com/item?id=43485566

11. Hacker News, “MCP vs API Explained,” 2026 — the argument that AI could
    produce a new hypermedia-style Web.
    https://news.ycombinator.com/item?id=43302297

12. Hacker News, “MCP: An (Accidentally) Universal Plugin System,” 2026 —
    whether machine hypermedia clients can work, and whether HATEOAS assumes
    human agency.
    https://news.ycombinator.com/item?id=44405245

**In this series**

13. Fabio Ellena, “The Browser Was Never the Smart Client,” 2026 — Part I.
    https://fblln.github.io/articles/the-browser-was-never-the-smart-client/

14. Fabio Ellena, “HATEOAS Priced the Wrong Change,” 2026 — Part II.
    https://fblln.github.io/articles/hateoas-priced-the-wrong-change/
