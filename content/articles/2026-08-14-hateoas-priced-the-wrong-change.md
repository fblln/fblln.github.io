+++
title = "HATEOAS Priced the Wrong Change"
date = "2026-08-14"
description = "GitHub emits URI templates in every payload and generates its own SDK from an OpenAPI document instead of following them. That is not ignorance of hypermedia — it is a team declining a trade nobody ever wrote down. Traversal costs a round trip before the first useful call, a build-time contract your pipeline can fail on, an operation name your dashboard can group by, and a method your retry logic can trust. And the change it insures you against — the server moving its URLs — is the one change mature APIs almost never make. Second of four on hypermedia and agents."
tags = ["API Design", "Protocols", "Architecture", "Complexity"]
+++

*Second of four. [Part I](/articles/the-browser-was-never-the-smart-client/)
argued that a program cannot learn a business concept from the fact that the
server gave that concept a URL. This part grants the opposite, a team that
understands the domain perfectly, and asks why they still do not follow the
links.*

GitHub's REST API is one of the most genuinely hypermedia-flavoured public APIs
anybody has shipped. Open any payload and it is full of affordances: `url`,
`html_url`, `commits_url`, `comments_url`, dozens of sibling URI templates,
emitted on every response, for years, at enormous scale.

GitHub's own documentation states that it "uses the OpenAPI description to
generate the Octokit SDKs."

The publisher of the links does not consume its own links. Neither does anybody
else. And GitHub cannot be accused of not understanding hypermedia — they
implemented it more thoroughly than the people arguing for it usually do.

Part I explained why a program cannot *understand* an unfamiliar affordance.
This article is about a different and more awkward observation: even when the
client author understands the domain completely, has read every page of the
documentation, and knows exactly what every relation in the response means,
they still compile the routes in. That decision has nothing to do with
comprehension, and in fifteen years of watching teams make it I have never seen
the reasons written down. They are transmitted as a shrug.

So I am going to write them down.

## The graph was kept. It moved.

First, dispose of the comfortable explanation, which is that hypermedia is fine
and the tooling never caught up.

OpenAPI has carried a Link Object since 3.0. A response can declare a link
naming the next operation by `operationId`, with a `parameters` map pulling its
arguments straight out of the current response body via runtime expressions:

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
operation, and where its arguments come from — all machine-readable, all
available since 2017.

And note where it lives, which Swagger's own documentation states outright: the
concept "is somewhat similar to hypermedia, but OpenAPI links do not require the
link information present in the actual responses."

As a design decision, that is this whole article in one sentence. The
graph was kept. It was moved into the description, where a client can compile
it, and taken out of the payload, where a client would have to traverse it.

Nobody was missing the mechanism. They had it, and they put it somewhere else on
purpose.

## What a link actually saves

Be fair to the link before charging it for anything. A link in a response
genuinely does two things.

It saves you interpolating an identifier into a path template. And it means the
server can restructure its URL space without breaking you — the decoupling
claim, and a real one.

There is a third thing it does that is usually counted as a saving and is
actually a cost in disguise: it makes every affordance conditional. `_links` is
an open map, `cancel` may or may not be in it, and a client that reads it must
branch. Compared against a client that knows `cancelOrder` exists and asks the
server whether it applies, that is the same check with worse ergonomics and no
type behind it — a nullable lookup on every response instead of a field a
generator turned into an enum.

Now the other column.

<figure class="diagram">
<svg viewBox="0 0 620 250" role="img" aria-label="A two-column ledger. The left column, headed what a link saves, lists two items: interpolating an id into a path template, and the server restructuring its URL space. The right column, headed what following one costs, lists five items: a round trip before the first useful call, no build-time surface so no pipeline gate, a URL instead of a route so no low-cardinality span name, no method so retry logic cannot tell what is safe, and a nullable field to check on every response. A footer notes that teams read this ledger correctly and never wrote it down.">
  <g font-family="var(--font-mono)" font-size="9">
    <text x="0" y="12" fill="var(--muted)">WHAT A LINK SAVES</text>
    <rect x="0" y="20" width="270" height="58" fill="none" stroke="var(--line)"/>
    <text x="12" y="40" fill="var(--ink)">interpolating an id into a path</text>
    <text x="12" y="62" fill="var(--ink)">the server moving its URL space</text>
    <text x="330" y="12" fill="var(--signal)">WHAT FOLLOWING ONE COSTS</text>
    <rect x="330" y="20" width="290" height="146" fill="var(--signal)"/>
    <text x="342" y="40" fill="var(--paper)">a round trip before the first call</text>
    <text x="342" y="66" fill="var(--paper)">no build-time surface, so no CI gate</text>
    <text x="342" y="92" fill="var(--paper)">a URL, where telemetry needs a route</text>
    <text x="342" y="118" fill="var(--paper)">no method, so retries cannot be safe</text>
    <text x="342" y="144" fill="var(--paper)">a nullable field to check every time</text>
  </g>
  <path d="M0 196 L620 196" stroke="var(--line)" fill="none"/>
  <text x="0" y="216" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">every team reads this ledger correctly</text>
  <text x="620" y="216" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">almost none of them write it down</text>
  <text x="0" y="236" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the left column is a convenience</text>
  <text x="620" y="236" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="end">the right column is an on-call rotation</text>
</svg>
<figcaption>The decoupling argument compares the left column against hard-coded URLs and stops there. The right column is what a team actually weighs, mostly without articulating it, which is why the decision looks like laziness from the outside and like obviousness from the inside.</figcaption>
</figure>

## Four line items

**You have to arrive before you can act.** A typed client cancels an order in
one request:

```ts
await api.cancelOrder({ orderId, reason });
```

A conforming hypermedia client cannot, and this is not an implementation detail
— it is the constraint working exactly as specified. Fielding is explicit that a
REST API should be entered "with no prior knowledge beyond the initial URI
(bookmark) and set of standardized media types," which means the path to any
operation is discovered by walking there:

```text
GET  /                     → find the orders link
GET  /orders               → find the templated item link
GET  /orders/123           → is there a "cancel" in _links?
POST /orders/123/cancel    → finally, the thing you came to do
```

Three requests to learn what one method signature already encoded. Caching
flattens the walk on repeat passes and does nothing for the first, and cold
paths are exactly the ones that turn up in a p99. Fielding scoped this honestly
himself, in a passage quoted far less often than the four constraints: a uniform
interface "degrades efficiency, since information is transferred in a
standardized form rather than one which is specific to an application's needs."

**And machine-to-machine work is disproportionately batch.** This is the item I
would put first if I were arguing this in a design review. This is the part
that separates M2M from the interactive Web most cleanly. A person cancels one
order; a reconciliation job touches forty thousand. The interactive case hides
the walk behind think time (nobody notices three extra requests while a human
reads the page) and the batch case multiplies it by the row count. Worse, the
affordance set is per-resource *by design*, so there is no bulk form of the
question: you cannot ask "which of these forty thousand orders are currently
cancellable" in one call, because the answer lives distributed across forty
thousand representations. The obvious fix is a bulk endpoint returning states,
at which point you have stopped traversing and gone back to calling a named
operation with a schema.

**Nothing in your pipeline can fail.** This is the big one, and it is almost
never mentioned in the argument. The reason teams love OpenAPI and Protobuf is
not the generated client. It is that a breaking change fails a build. `oasdiff`
diffs two OpenAPI documents, classifies what broke, and runs as a GitHub Action
on the pull request; Buf does the equivalent for Protobuf schemas. Pact goes
further, in the other direction: a consumer's tests generate a contract,
and "contract by example" is the project's own phrase for it. The provider's pipeline
verifies against the real expectations of its real consumers, so only the parts
actually used get tested. Now ask what the equivalent is for a hypermedia API.
The contract is the media type. There is no artifact to diff, no document to
version, no schema for a CI job to fail on. A relation quietly disappearing from
a response, the precise thing the design says should happen freely, is
indistinguishable from a bug, at runtime, in production, to a client that has no
way to have asserted otherwise. The evolution story is real; the safety net
under it is not.

**The dashboard needs a name.** This one is settled by a standard rather than by
taste. The OpenTelemetry HTTP semantic conventions say a span name should be
`{method} {target}` where a *low-cardinality* target is available, and then
prohibit the obvious shortcut: "Instrumentation MUST NOT default to using URI
path as a `{target}`." The `http.route` attribute is defined as the matched
route *template*, which "MUST be low-cardinality and include all static path
segments, with dynamic path segments represented with placeholders" — and if the
framework cannot supply it, the convention says leave it empty rather than
substitute the path, because "the URI path can NOT substitute it."

<figure class="diagram">
<svg viewBox="0 0 620 214" role="img" aria-label="Two telemetry views. On the left, a single low-cardinality span name POST slash orders slash braces id slash cancel with an aggregate count of 41,209 and a p99, labelled one row you can alert on. On the right, four distinct span names for individual order URLs each with a count of one, trailing off, labelled forty-one thousand rows you cannot group.">
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="0" y="12">COMPILED ROUTE</text>
    <text x="330" y="12">FOLLOWED HREF</text>
  </g>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="0" y="22" width="290" height="46" fill="var(--signal)"/>
    <text x="12" y="40" fill="var(--paper)">POST /orders/{id}/cancel</text>
    <text x="12" y="58" fill="var(--paper)">n=41209   p99=180ms   err=0.4%</text>
    <rect x="330" y="22" width="290" height="22" fill="none" stroke="var(--line)"/>
    <text x="342" y="37" fill="var(--ink)">POST /orders/8f21/cancel   n=1</text>
    <rect x="330" y="48" width="290" height="22" fill="none" stroke="var(--line)"/>
    <text x="342" y="63" fill="var(--ink)">POST /orders/a034/cancel   n=1</text>
    <rect x="330" y="74" width="290" height="22" fill="none" stroke="var(--line)"/>
    <text x="342" y="89" fill="var(--ink)">POST /orders/c7de/cancel   n=1</text>
    <rect x="330" y="100" width="290" height="22" fill="none" stroke="var(--line)"/>
    <text x="342" y="115" fill="var(--ink)">POST /orders/1b90/cancel   n=1</text>
    <text x="342" y="139" fill="var(--muted)">… 41,205 more</text>
  </g>
  <path d="M0 164 L620 164" stroke="var(--line)" fill="none"/>
  <text x="0" y="184" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">one row you can alert on</text>
  <text x="620" y="184" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="end">no row at all</text>
  <text x="0" y="204" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the route template is the operation's name</text>
  <text x="620" y="204" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="end">the href threw it away</text>
</svg>
<figcaption>A client that compiled the route knows the operation's identity and can label the span with it. A client that followed an href it was handed has a URL and nothing else. The SLO, the rate limit, the error budget and the capacity plan are all defined per operation — and the operation is the thing traversal declines to name.</figcaption>
</figure>

That is a specification, written by people with no stake in the REST argument,
stating that the URL is not an adequate identity for an operation. A client that
compiled `cancelOrder` has the route template and can label the span with it. A
client that followed an href has a string with an order id in it. One of those
aggregates into a row you can alert on; the other is forty thousand singleton
rows. And the operation is the unit of everything ops cares about — the SLO, the
rate limit, the error budget, the capacity plan, the thing the incident review
names.

**Retry is a property of the method, and the link does not carry one.** RFC 9110
defines an idempotent method as one where "the intended effect on the server of
multiple identical requests with that method is the same as the effect for a
single such request," and this is what licenses a client to automatically retry
after a connection failure. Which method is behind `_links.cancel.href`? Basic
HAL does not say — the omission Carson Gross names directly when he points out
that links in JSON lack method information. So the client either guesses, or
hard-codes a mapping from relation name to verb, which is the hard-coding it was
supposed to avoid, relocated and typed worse. Retry logic needs to know what is
safe to repeat. A bare link is exactly the wrong shape for telling it.

## It priced the wrong change

Now the part that makes the whole trade look strange rather than merely
expensive.

Hypermedia's insurance policy covers one specific event: the server restructures
its URL space. Ask how often mature APIs actually do that.

Stripe is the useful case because it evolves aggressively and documents its
rules. It versions by date, `2026-07-29.dahlia` at the time of writing,
selected with a `Stripe-Version` header, with major releases carrying breaking
changes and monthly releases carrying only compatible ones. And its published
list of what counts as backwards-compatible is entirely about *payloads and
parameters*: adding new resources, adding optional request parameters, adding
properties to responses, reordering properties, changing the format of opaque
strings, adding event types.

Nothing in that list is about URLs. The URLs have been stable for over a decade,
across an enormous amount of change, because moving them would break every
integration on earth for no product benefit whatsoever.

<figure class="diagram">
<svg viewBox="0 0 620 226" role="img" aria-label="Two horizontal frequency bars. The top bar, labelled what actually changes, is filled and dense, listing new fields, new optional parameters, new resources, changed behaviour, new error cases, deprecations, and marked every release. The bottom bar, labelled what hypermedia insures against, is a single narrow mark labelled the URL space moves, marked almost never. A footer reads: you paid a per-call premium on the wrong policy.">
  <g font-family="var(--font-mono)" font-size="9">
    <text x="0" y="12" fill="var(--signal)">WHAT ACTUALLY CHANGES &middot; EVERY RELEASE</text>
    <rect x="0" y="20" width="620" height="62" fill="var(--signal)"/>
    <text x="12" y="40" fill="var(--paper)">new fields  &middot;  new optional parameters  &middot;  new resources</text>
    <text x="12" y="58" fill="var(--paper)">changed behaviour  &middot;  new error cases  &middot;  deprecations</text>
    <text x="12" y="76" fill="var(--paper)">none of which a link protects you from</text>
    <text x="0" y="118" fill="var(--muted)">WHAT HYPERMEDIA INSURES AGAINST &middot; ALMOST NEVER</text>
    <rect x="0" y="126" width="96" height="26" fill="none" stroke="var(--line)"/>
    <text x="12" y="143" fill="var(--ink)">URLs move</text>
  </g>
  <path d="M0 176 L620 176" stroke="var(--line)" fill="none"/>
  <text x="0" y="196" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the premium is paid on every call</text>
  <text x="620" y="196" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">the claim is filed once a decade</text>
  <text x="0" y="216" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">Stripe versions by date and has never moved its URLs</text>
</svg>
<figcaption>An API under real evolutionary pressure changes its fields, its parameters, its behaviour and its error surface constantly, and its URL space essentially never. Hypermedia decouples the client from the second category and leaves it fully exposed to the first.</figcaption>
</figure>

So the trade, as I read it: pay a round trip, a missing CI gate, an unnameable
span and an unknown method on every single call, in order to be insulated from a change that
happens roughly never — while remaining completely exposed to the changes that
happen every sprint, because a link says nothing about a renamed field, a
narrowed enum, a new required parameter or an altered side effect.

This is the same failure I described in
[who pays for the pressure](/articles/who-pays-for-the-pressure/), inverted.
There, real pressure inside an implementation got relocated onto the contract,
where every caller carried it forever. Here, a contract cost is being paid
continuously to hedge a pressure that was never going to arrive. Both are
mispricings. Teams feel the second one immediately and cannot articulate it,
which is why the argument keeps being lost on paper and won in practice.

None of this makes hypermedia's design incoherent. It makes it *tuned for a
different deployment*: many independent clients that cannot be upgraded
together, a genuinely document-shaped domain, and a consumer — human or
domain-generic — that needs no build-time contract because it has no build. The
Web. Feeds. Crawlers. `sitemap.xml`. Those clients pay none of the four line
items above, which is precisely why they follow links happily.

An internal payments service is not that deployment. Neither is a build system,
nor a warehouse integration, nor anything with an on-call rotation attached.

## The tenth that everybody kept

What teams kept is small, and I want to name it precisely, because the rest of
this series is built on it.

Hard-code the operations; read their *validity* off the response. An order
carrying `"status": "pending"`, or better, an explicit
`"allowedActions": ["cancel", "refund"]`, tells the client exactly what the
fourth constraint wanted it to know — which of the operations it already
understands are legal right now — and it does so as a field. A schema can
express it. A generator can turn it into an enum. A pipeline can fail when it
changes. A span can still be named. Nothing has to be fetched, parsed or
followed to find out.

That recovers most of the original promise and drops one specific part of it. It
keeps the server as the authority on current availability. It
drops the claim that the client should learn the operation's identity and
invocation by traversal.

Those two ideas arrived bundled, and almost every argument about HATEOAS is
really an argument about the second one conducted in the vocabulary of the
first. Contemporary API guidance mostly does not even frame it as a rejection —
it describes production APIs as sitting around Richardson level 2 and treats
that as the pragmatic default. Which it is. Nobody is defying Fielding. They are
declining one clause of him and keeping the rest.

## What a smarter client does not fix

This is where the two bills separate.

Part I's charge, that a program cannot learn a business concept from a URL, is
answerable in principle. Build a client smart enough to read an unfamiliar
affordance and reason about it, and that charge is dropped.

Not one of the four line items in this article is dropped with it.

An intelligent client still has to arrive before it can act; comprehension does
not remove a round trip. It still gives your pipeline nothing to fail on. It
still hands your tracing backend a URL where a route template belonged. It still
cannot tell whether the thing behind that href is safe to retry. Intelligence
buys understanding, and every bill here is an operational bill.

So when somebody finally sat down to design a protocol for the smartest clients
we have ever deployed, the interesting question is not whether they had heard of
hypermedia. It is which of these two bills they were looking at.

They were looking at both. And the choices they made — a stable named
vocabulary, schemas a client can validate against, a surface deliberately
prevented from shifting under the caller — read very differently once you know
what the second bill costs.

That is Part III.

## References

**The publisher of the links**

1. GitHub, *About the OpenAPI description for the REST API* — “GitHub uses the
   OpenAPI description to generate the Octokit SDKs.”
   https://docs.github.com/en/rest/about-the-rest-api/about-the-openapi-description-for-the-rest-api

2. GitHub REST API description repository.
   https://github.com/github/rest-api-description

3. OpenAPI Specification — the Link Object: `operationId`, `operationRef`, the
   `parameters` map, and runtime expressions such as `$response.body#/id`.
   https://spec.openapis.org/oas/latest.html

4. Swagger documentation, *Links* — “The concept of links is somewhat similar to
   hypermedia, but OpenAPI links do not require the link information present in
   the actual responses.”
   https://swagger.io/docs/specification/v3_0/links/

**The round trip**

5. Roy T. Fielding, “REST APIs must be hypertext-driven,” 2008 — entry with no
   prior knowledge beyond the initial URI and the standardized media types.
   https://roy.gbiv.com/untangled/2008/rest-apis-must-be-hypertext-driven

6. Roy T. Fielding, *Architectural Styles and the Design of Network-based
   Software Architectures*, 2000 — section 5.1.5 on the uniform interface
   degrading efficiency, and REST being optimized for the common case of the
   Web.
   https://ics.uci.edu/~fielding/pubs/dissertation/rest_arch_style.htm

**The pipeline gate**

7. Pact — consumer-driven contract testing; the contract generated during the
   consumer's tests, “contract by example,” and verification of only what
   consumers actually use.
   https://docs.pact.io/

8. oasdiff — command-line OpenAPI diff and breaking-change detection, run
   locally or as a CI action on the pull request.
   https://github.com/oasdiff/oasdiff

9. Buf — breaking-change detection for Protocol Buffers schemas against a
   previous version, wired into CI.
   https://buf.build/docs/breaking/

**The dashboard**

10. OpenTelemetry, *Semantic Conventions for HTTP Spans* — span names as
    `{method} {target}` with a low-cardinality target; “Instrumentation MUST NOT
    default to using URI path as a `{target}`”; `http.route` as the matched
    low-cardinality route template, not to be substituted by the URI path.
    https://opentelemetry.io/docs/specs/semconv/http/http-spans/

**The retry**

11. RFC 9110, *HTTP Semantics* — idempotent methods, and automatic retry after
    a connection failure.
    https://www.rfc-editor.org/rfc/rfc9110#name-idempotent-methods

12. Carson Gross, “Hypermedia Clients” — links in JSON carrying no method
    information.
    https://four.htmx.org/essays/hypermedia-clients

**The mispricing**

13. Stripe, *Versioning* — dated releases, the `Stripe-Version` header, major
    versus monthly releases.
    https://docs.stripe.com/api/versioning

14. Stripe, *Upgrades* — the published list of backwards-compatible changes:
    new resources, new optional request parameters, new response properties,
    property reordering, opaque string format, new event types. No entry
    concerns URLs.
    https://docs.stripe.com/upgrades

15. MLflow, “REST API for AI Models Explained: 2026 Guide” — contemporary
    guidance describing most production APIs as level-2 shaped, and treating
    that as a deliberate tradeoff.
    https://mlflow.org/articles/rest-api-for-ai-models-explained-2026-guide/

16. Martin Fowler, “Richardson Maturity Model,” 2010 — the levels, and why level
    3 stayed rare.
    https://martinfowler.com/articles/richardsonMaturityModel.html

17. Fabio Ellena, “Who Pays for the Pressure,” 2026 — pressure relocated onto a
    contract, and who carries it.
    https://fblln.github.io/articles/who-pays-for-the-pressure/

**In this series**

18. Fabio Ellena, “The Browser Was Never the Smart Client,” 2026 — Part I.
    https://fblln.github.io/articles/the-browser-was-never-the-smart-client/
