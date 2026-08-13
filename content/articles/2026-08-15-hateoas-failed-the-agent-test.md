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
nobody had the client it asked for.

It does not, on its own, settle anything — Part II's bill is still on the table,
and no amount of intelligence pays down a round trip or gives a pipeline
something to fail on. But it is the one objection a better client could
genuinely answer, and it is the objection the whole argument was built on.

We have it now. We have clients that read arbitrary JSON, interpret relation
names they have never seen, hold a goal across a dozen steps, and decide at
runtime which affordance serves it. Every property the defence stipulated,
delivered — mostly by people who had never read Fielding and were not trying to
settle an argument about REST.

This should have been hypermedia's moment.

And the protocol those clients actually got — the one that spread across the
industry in about eighteen months — puts the operations in a flat described
list, states that the list **MUST NOT** vary as a side effect of other requests
on the connection, and carries application state as an ordinary argument.

Two caveats before the argument, because the strong version of this claim is
not the defensible one. The Model Context Protocol is not the only way to give
an agent tools, and adoption is not a proof of optimality; plenty of widely
deployed protocols are widely deployed for reasons that have nothing to do with
being right. What MCP is, is evidence — the best available natural experiment.
It was designed recently, specifically for intelligent machine clients, by
people with no REST installed base to protect and no particular stake in the
hypermedia argument. If state-driven affordance discovery were the obvious
architecture for a smart client, this is where it would have shown up.

It did not show up. Something related did, and the difference between them is
the whole article.

## What MCP actually exposes

The protocol has nouns. `resources/list` returns URI-identified resources;
`resources/read` returns their contents. There are URI templates, custom
schemes, change notifications. If you wanted to build something
hypermedia-shaped, the raw material is right there.

A resource read returns this:

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
annotations. No links. No affordances. No transitions. Reading a resource tells
you what it contains and tells you nothing about what you may do next. The noun
half of the protocol is deliberately inert.

The verbs live somewhere else entirely. `tools/list` returns an array, each
entry carrying a name, an optional title, a natural-language description, a
JSON Schema for its input and optionally one for its output. `tools/call`
invokes one by name. Tools are "model-controlled" by design (the
specification's own phrase) meaning the model discovers and selects them from
that metadata.

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
<figcaption>Both columns are dynamic — the tool list can change, and the protocol has a notification for saying so. The difference is what it is allowed to change <em>in response to</em>. On the left that is the point of the design; on the right it is prohibited.</figcaption>
</figure>

So far this is a shape difference, and shape differences are cheap to argue
about. The normative text is not.

## The sentence that settles it

The clause, from the tools page of revision `2026-07-28`:

> Servers that declare the `tools` capability **MUST** respond to `tools/list`
> requests with the set of tools currently available to the requesting client.
> This set **MAY** be empty and **MAY** change over time … but **MUST NOT**
> vary per-connection or as a side effect of other requests on the connection.

The first half is a distraction, and it is the half most arguments stop at.
Yes, the set may change over time. There is a
`notifications/tools/list_changed` message specifically so servers can announce
it, and a client is not entitled to assume the surface it saw last week. MCP
tools are not static, and anybody who tells you they are has not read the page.

The specification even spells out one dimension along which the set is
explicitly allowed to differ, and the reasoning is the useful part:

> The set **MAY** vary by the authorization presented on the request — for
> example, returning only the tools the caller's granted scopes permit — since
> credentials are per-request input, not connection state.

*Per-request input, not connection state.* That sentence is the design in
miniature. The tool surface may be a function of what you presented; it may not
be a function of where you have got to. Vary the answer by who is asking, fine.
Vary it because of what the client did three calls ago, and you have made the
enumerated capability surface into a state machine, which is exactly the thing
being ruled out.

So the precise claim, the defensible one that people routinely overshoot, is
not that MCP forbids dynamism. It is this:

> The enumerated tool surface is not permitted to act as per-connection
> application state.

Set against Fielding's fourth constraint, that is close to an inversion. Under
HATEOAS, `cancel` appearing in an order representation and vanishing after
shipment is not a quirk of the encoding; it *is* the mechanism, the affordance
set as a function of application state. MCP declines to make the tool list that
function. It says nothing about whether a call may fail, or whether a result
may report that shipment has already happened — servers do that constantly, and
the protocol has a whole error channel for it. What it removes is the idea that
the *list of what exists* should narrow and widen as you move.

The same rule reaches the nouns: this revision's changelog states flatly that
`tools/list`, `resources/list` and `prompts/list` no longer vary
per-connection. Neither half of the protocol is permitted to be an engine of
application state.

## Why that rule is there

The rule reads as arbitrary until you notice what the rest of the revision is
optimizing for, at which point it reads as forced.

A tool list is not just a directory. For a language model it is *context* — the
descriptions and schemas are serialized into the prompt, they cost tokens on
every single turn, and they are the same tokens every turn. That makes the tool
list the single best caching target in the entire system, and this revision
goes after it hard. Servers **SHOULD** return tools in a deterministic order,
and the specification says why in one sentence: deterministic ordering "enables
clients to reliably cache the tool list and improves LLM prompt cache hit rates
when tools are included in model context." `ttlMs` and `cacheScope` are no
longer optional garnish; they are *required* fields on the results of
`tools/list`, `prompts/list`, `resources/list`, `resources/read` and
`resources/templates/list`, one a freshness hint and the other a statement
about whether a shared intermediary may hold the response.

Now imagine the hypermedia mechanism dropped into that design. A capability
surface that mutated as a consequence of prior calls could not be given a
meaningful TTL, could not be cached by an intermediary, could not be returned
in a stable order, and would invalidate the model's prompt prefix on every
turn. Hypermedia's central mechanism, the affordance set as a function of
where you are, is for this class of client a cache invalidation event on
every single call.

That is not a protocol designer failing to appreciate hypermedia. That is a
protocol designer looking at a client whose context window is the scarcest
resource in the system and declining to put a moving target in it.

## Application state became ordinary data

If the representations do not carry control state, something else must, and the
answer is the precise inverse of the constraint. This revision removed
protocol-level sessions outright — no `Mcp-Session-Id`, no `initialize`
handshake, every request carrying its own version and capabilities. There is
nowhere for implicit per-connection state to live any more.

So state is carried explicitly, and the guidance repays a careful reading,
including its own framing:

> MCP has no protocol-level session, so a server cannot rely on implicit
> per-connection state to relate one tool call to the next. Servers that need
> to maintain state across calls … should do so by returning an explicit handle
> from a creation tool and accepting that handle as an argument on subsequent
> calls.

And then, flatly: "The model is responsible for carrying `basket_id` forward."

That section is labelled non-normative, design guidance rather than a MUST, and
the note attached to it is the most telling sentence on the page: the
protocol "has no concept of a state handle; from the wire's perspective a
handle is an ordinary string in a tool result and an ordinary argument to
subsequent tool calls."

That is a larger claim than a normative one, not a smaller one. The protocol
does not model application state at all. It declines to have an
opinion, and what falls out of that refusal is state as data — a string the
model reads out of one result and types into the next.

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

The design notes around handles make the intent unambiguous. They should be
opaque, because ones "that encode internal structure invite parsing or
guessing." They should be authorization-checked on every call, because "a
handle is a name, not a capability." That is a designer looking directly at the
hypermedia idea (a URL you were handed, which is simultaneously the address
and the permission) and separating the two halves on purpose.

## MCP did not reject discovery

This is the distinction that most readings of the situation get wrong, and it
matters more than the rest of the argument.

MCP performs runtime discovery. `tools/list` *is* discovery, and it is better
discovery than most REST APIs offer, because what comes back is executable
rather than merely addressable: a name you can call, a schema you can validate
against, a description you can reason about, and an output shape you can check.
An agent that has never seen a server before learns its entire vocabulary in
one round trip.

What was declined is a specific *form* of discovery.

<figure class="diagram">
<svg viewBox="0 0 620 226" role="img" aria-label="Two runtime discovery shapes. On the left, hypermedia: fetch a resource, read the state-specific affordance out of the representation, learn how to invoke it from the same representation, act. On the right, MCP: two separate inputs, a discovered operation vocabulary and the current state obtained separately, converge on the model, which selects and invokes. A footer notes both discover at runtime and that they differ in what is discovered.">
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="0" y="12">HYPERMEDIA &middot; ONE CHANNEL</text>
    <text x="330" y="12">MCP &middot; TWO CHANNELS</text>
  </g>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="0" y="26" width="270" height="24" fill="none" stroke="var(--line)"/>
    <text x="12" y="42" fill="var(--ink)">fetch the resource</text>
    <rect x="0" y="70" width="270" height="24" fill="none" stroke="var(--line)"/>
    <text x="12" y="86" fill="var(--ink)">read the affordance for this state</text>
    <rect x="0" y="114" width="270" height="24" fill="none" stroke="var(--line)"/>
    <text x="12" y="130" fill="var(--ink)">learn invocation from the same doc</text>
    <rect x="0" y="158" width="270" height="24" fill="var(--signal)"/>
    <text x="12" y="174" fill="var(--paper)">act</text>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M135 50 L135 64 M131 58 L135 64 L139 58"/>
    <path d="M135 94 L135 108 M131 102 L135 108 L139 102"/>
    <path d="M135 138 L135 152 M131 146 L135 152 L139 146"/>
  </g>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="330" y="26" width="138" height="40" fill="none" stroke="var(--line)"/>
    <text x="342" y="42" fill="var(--muted)">VOCABULARY</text>
    <text x="342" y="58" fill="var(--ink)">tools/list</text>
    <rect x="482" y="26" width="138" height="40" fill="none" stroke="var(--line)"/>
    <text x="494" y="42" fill="var(--muted)">STATE</text>
    <text x="494" y="58" fill="var(--ink)">results, resources</text>
    <rect x="330" y="106" width="290" height="34" fill="var(--signal)"/>
    <text x="475" y="127" fill="var(--paper)" text-anchor="middle">MODEL SELECTS</text>
    <rect x="330" y="158" width="290" height="24" fill="none" stroke="var(--ink)" stroke-width="1.5"/>
    <text x="342" y="174" fill="var(--ink)">tools/call</text>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M399 66 L399 86 L475 86 L475 100 M471 94 L475 100 L479 94"/>
    <path d="M551 66 L551 86 L475 86"/>
    <path d="M475 140 L475 152 M471 146 L475 152 L479 146"/>
  </g>
  <text x="0" y="206" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">both of these are runtime discovery</text>
  <text x="620" y="206" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">they differ in what gets discovered</text>
  <text x="0" y="220" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">one channel carries meaning and availability together</text>
</svg>
<figcaption>The left column derives the operation, its invocation and its availability from one document, in one channel. The right column splits them: a vocabulary that is stable enough to cache, and state that arrives separately as data. Neither client knew the server in advance.</figcaption>
</figure>

So the finding is narrower than "hypermedia lost," and considerably more
useful:

> Capability discovery and hypermedia traversal are not the same thing.

They were always independent ideas. HATEOAS bundled them. The same mechanism
that told you an operation *exists* also told you it was *available now* and
*how to invoke it*, and because they arrived bundled, twenty years of argument
treated accepting one as accepting all three. MCP took the bundle apart and
kept the pieces it wanted.

MCP's unit of discovery is a described callable operation. HATEOAS's unit of
discovery is a state transition embedded in a representation. That single
sentence is the difference between the two architectures, and everything else
is consequence.

## The irony had already been noticed

The most striking thing about all this is that people saw it coming and said so
out loud, in public, before the result was in.

In developer discussion from around the time agent tooling started spreading,
the observation turns up unprompted: the irony behind
HATEOAS is that LLMs are the mythical "evolvable agents" necessary to make
HATEOAS work in the first place. The same commenter supplies the diagnosis Part
I spent three thousand words on — that only browser-like clients ever
conformed, because they "delegate all the hard parts … to a human."

Elsewhere in the same period people argued explicitly that AI would produce a
new hypermedia Web, that agents would finally be the generic runtime client the
constraint had been waiting for. These are comment threads, not evidence about
protocol behaviour, and I offer them as nothing more than a record of what
practitioners expected.

Which is what makes the outcome interesting rather than merely disappointing.
The resurrection was predicted, by the right people, for the right reasons, at
the right moment. The dominant protocol still chose described tools.

## What a schema does not tell you

I have to argue here with the version of this article I would have written six
months ago, which concluded that once you can ask a server *what
can I do* and get typed answers back, the hypermedia bundle has no remaining
job.

That is too strong, and the reason it is too strong is visible in the
distinction the last section drew.

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

An agent holding `cancel_order` in its vocabulary and an order representation
in its context still has to work out whether cancelling *this* order is
currently legal. It can guess from a `status` field, if the server thought to
include one and the description explains what the values imply. It can try the
call and read the error — and the protocol supports that well, splitting
execution errors, which a model can act on, from protocol errors, which it
cannot. But guessing and probing are what the fourth constraint was designed to
make unnecessary, and on this specific question hypermedia's answer was better.

The protocol does lean this way in places, and I would rather name that than
hide it. A tool result can carry a `resource_link` — a URI pointing at something
the client may then read. A call can come back `input_required`, asking for
more input before completing, and the client retries the original request with
the answers. Both are server-driven flow. But notice what neither one is: a
`resource_link` points at bytes, not at an operation, and `input_required` asks
for another *argument*, not for a control to be followed. The protocol will
happily tell you where more data lives. It will not hand you the next move.

## What the test actually showed

Twenty years of the hypermedia argument had one live hypothesis and no way to
test it: the clients are not smart enough yet.

Then we built them. Give an agent a HAL document today and it will follow the
links; that part genuinely works now, and anyone who has tried it knows it
works. The excuse expired. The answer did not change. That is what makes this
evidence rather than another round of the same argument.

The diagnosis I would write down is narrow, because the broad version is wrong.
HATEOAS is not a bad idea and REST is not a mistake. The constraint attacked
*where the operation lives* and left *what the operation means* exactly where
it was — in prose, outside the media type, unreadable by any tool. For a client
that could not read a schema, moving the URL was the only available win. For a
client that can, moving the URL is the smaller half of the problem, and the
protocol built for those clients spent its effort on the larger half.

We spent two decades waiting for a client good enough to navigate the graph. It
arrived. It read the schema instead.

But a schema answers what an operation *is*. It does not answer when that
operation is valid for the particular thing the agent is looking at — and
that question did not stop mattering just because the tool list stopped
answering it.

That missing piece is where the older hypermedia work becomes interesting
again, and it is not the part of hypermedia this article has been arguing with.
Basic HAL trades a schema for a relation name, which is the trade Part I called
a straight loss for a capable client. HAL-FORMS, Hydra and the Web of Things do
not make that trade. Part IV is about what happens when you take them
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
