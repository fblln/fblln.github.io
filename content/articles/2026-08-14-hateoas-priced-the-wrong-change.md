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
else. And GitHub cannot be accused of not understanding hypermedia; they
implemented it more thoroughly than the people arguing for it usually do.

Even when the client author understands the domain completely, has read every
page of the documentation, and knows exactly what every relation in the response
means, they still compile the routes in. That decision has nothing to do with
comprehension, and in fifteen years of watching teams make it I have never seen
the reasons written down. They are transmitted as a shrug.

So I am going to write them down.

## The graph was kept. It moved.

First, dispose of the comfortable explanation: hypermedia is fine and the
tooling never caught up.

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

That is HATEOAS's shape, expressed in a schema: the relationship, the target
operation, and where its arguments come from, all machine-readable and available
since 2017.

Where it lives is the whole article in one sentence, and Swagger's own
documentation states it outright. The concept "is somewhat similar to
hypermedia, but OpenAPI links do not require the link information present in the
actual responses." The graph was kept. It moved into the description, where a
client can compile it, and out of the payload, where a client would have to
traverse it. Nobody was missing the mechanism. They had it and put it somewhere
else on purpose.

## What a link actually saves

Be fair to the link before charging it for anything. It saves you interpolating
an identifier into a path template, and it means the server can restructure its
URL space without breaking you. That decoupling claim is real.

A third thing is usually counted as a saving and is a cost in disguise: it makes
every affordance conditional. `_links` is an open map, `cancel` may or may not be
in it, and a client that reads it must branch. Against a client that knows
`cancelOrder` exists and asks the server whether it applies, that is the same
check with worse ergonomics and no type behind it, a nullable lookup on every
response instead of a field a generator turned into an enum.

Now the other column.

| What a link saves | What following one costs |
|---|---|
| interpolating an id into a path template | a nullable field to check on every response |
| the server moving its URL space | a round trip before the first useful call |
| — | no build-time surface, so no CI gate |
| — | a URL, where telemetry needs a route |
| — | no method, so retries cannot be safe |

The decoupling argument compares the left column against hard-coded URLs and
stops there. The right column is what a team actually weighs, which is why the
decision looks like laziness from the outside and like obviousness from the
inside. The first cost is the one just described. The other four are the rest of
this section.

## Four line items

**You have to arrive before you can act.** A typed client cancels an order in
one request:

```ts
await api.cancelOrder({ orderId, reason });
```

A conforming hypermedia client cannot, and this is not an implementation detail.
It is the constraint working exactly as specified. Entry with no prior knowledge
beyond the initial URI, Fielding's own criterion, means the path to any
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
himself: a uniform interface "degrades efficiency, since information is
transferred in a standardized form rather than one which is specific to an
application's needs."

And machine-to-machine work is disproportionately batch. A person cancels one
order; a reconciliation job touches forty thousand. Think time hides the walk in
the interactive case, and the row count multiplies it in the batch one.

There is no bulk form of the question either, because the affordance set is
per-resource *by design*. You cannot ask which of forty thousand orders are
currently cancellable in one call; the answer lives distributed across forty
thousand representations. The obvious fix is a bulk endpoint returning states,
at which point you have stopped traversing and gone back to calling a named
operation with a schema.

**Nothing in your pipeline can fail.** This is the big one, and it is almost
never mentioned in the argument. The reason teams love OpenAPI and Protobuf is
not the generated client. It is that a breaking change fails a build.

`oasdiff` diffs two OpenAPI documents, classifies what broke, and runs as a
GitHub Action on the pull request. The artifact is the thing that makes that
possible: two versions of a file, a machine that can tell you what changed
between them, and a red build when the answer is "something a caller depended
on."

Now ask what the equivalent is for a hypermedia API. The contract is the media
type. There is no artifact to diff, no document to version, no schema for a CI
job to fail on. A relation quietly disappearing from a response, the precise
thing the design says should happen freely, is indistinguishable from a bug, at
runtime, in production, to a client that has no way to have asserted otherwise.
The evolution story is real; the safety net under it is not.

**The dashboard needs a name.** This one is settled by a standard rather than by
taste. The OpenTelemetry HTTP semantic conventions say a span name should be
`{method} {target}` where a *low-cardinality* target is available, and then
prohibit the obvious shortcut: "Instrumentation MUST NOT default to using URI
path as a `{target}`." If the framework cannot supply the matched route
template, the convention says leave the attribute empty rather than substitute
the path, because "the URI path can NOT substitute it."

<figure class="diagram">
<svg viewBox="0 0 620 194" role="img" aria-label="Two telemetry views. On the left, a single low-cardinality span name POST slash orders slash braces id slash cancel with an aggregate count of 41,209 and a p99, labelled one row you can alert on. On the right, four distinct span names for individual order URLs each with a count of one, trailing off, labelled no row at all.">
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
</svg>
<figcaption>A client that compiled the route knows the operation's identity and can label the span with it. A client that followed an href it was handed has a URL and nothing else. The operation is the thing traversal declines to name.</figcaption>
</figure>

That is a specification, written by people with no stake in the REST argument,
stating that the URL is not an adequate identity for an operation. And the
operation is the unit of everything ops cares about: the SLO, the rate limit,
the error budget, the capacity plan, the thing the incident review names.

**Retry is a property of the method, and the link does not carry one.** RFC 9110
defines an idempotent method as one where "the intended effect on the server of
multiple identical requests with that method is the same as the effect for a
single such request," which is what licenses a client to retry automatically
after a connection failure. Which method is behind `_links.cancel.href`? Basic
HAL does not say, an omission Carson Gross names directly. So the client either
guesses or hard-codes a mapping from relation name to verb, which is the
hard-coding it was supposed to avoid, relocated and typed worse.

## It priced the wrong change

Hypermedia's insurance policy covers one specific event: the server restructures
its URL space. Ask how often mature APIs actually do that.

Stripe is the useful case because it evolves aggressively and documents its
rules. It versions by date, `2026-07-29.dahlia` at the time of writing, selected
with a `Stripe-Version` header. And its published list of what counts as
backwards-compatible is entirely about *payloads and parameters*: new resources,
new optional request parameters, new response properties, reordered properties,
changed opaque-string formats, new event types.

Nothing in that list is about URLs. They have been stable for over a decade,
across an enormous amount of change, because moving them would break every
integration on earth for no product benefit whatsoever.

<figure class="diagram">
<svg viewBox="0 0 620 214" role="img" aria-label="A decade-long timeline from 2016 to 2026 with two lanes drawn against the same axis. The upper lane, what actually changes, is ruled with a tick mark for every dated release across the whole decade: fields, optional parameters, resources, behaviour, error cases, deprecations. The lower lane, what hypermedia insures against, is the same width and completely empty, annotated that the URL space has not moved once. A footer notes the premium is paid on every call and the claim has never been filed.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">STRIPE &middot; ONE DECADE, TO SCALE</text>
  <text x="0" y="38" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">WHAT ACTUALLY CHANGES</text>
  <rect x="0" y="44" width="620" height="28" fill="none" stroke="var(--line)"/>
  <path d="M6 44 L6 72 M18 44 L18 72 M30 44 L30 72 M42 44 L42 72 M54 44 L54 72 M66 44 L66 72 M78 44 L78 72 M90 44 L90 72 M102 44 L102 72 M114 44 L114 72 M126 44 L126 72 M138 44 L138 72 M150 44 L150 72 M162 44 L162 72 M174 44 L174 72 M186 44 L186 72 M198 44 L198 72 M210 44 L210 72 M222 44 L222 72 M234 44 L234 72 M246 44 L246 72 M258 44 L258 72 M270 44 L270 72 M282 44 L282 72 M294 44 L294 72 M306 44 L306 72 M318 44 L318 72 M330 44 L330 72 M342 44 L342 72 M354 44 L354 72 M366 44 L366 72 M378 44 L378 72 M390 44 L390 72 M402 44 L402 72 M414 44 L414 72 M426 44 L426 72 M438 44 L438 72 M450 44 L450 72 M462 44 L462 72 M474 44 L474 72 M486 44 L486 72 M498 44 L498 72 M510 44 L510 72 M522 44 L522 72 M534 44 L534 72 M546 44 L546 72 M558 44 L558 72 M570 44 L570 72 M582 44 L582 72 M594 44 L594 72 M606 44 L606 72 M618 44 L618 72" stroke="var(--signal)" fill="none"/>
  <text x="0" y="88" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">fields &middot; optional parameters &middot; resources &middot; behaviour &middot; error cases &middot; deprecations</text>
  <text x="0" y="118" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">WHAT HYPERMEDIA INSURES AGAINST</text>
  <rect x="0" y="124" width="620" height="28" fill="none" stroke="var(--line)"/>
  <text x="310" y="142" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">&mdash; the URL space has not moved &mdash;</text>
  <path d="M0 168 L620 168" stroke="var(--line)" fill="none"/>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="0" y="182">2016</text>
    <text x="130" y="182" text-anchor="middle">2018</text>
    <text x="254" y="182" text-anchor="middle">2020</text>
    <text x="378" y="182" text-anchor="middle">2022</text>
    <text x="502" y="182" text-anchor="middle">2024</text>
    <text x="620" y="182" text-anchor="end">2026</text>
  </g>
  <text x="0" y="206" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the premium is paid on every call; the claim has never been filed</text>
</svg>
<figcaption>Both lanes span the same decade at the same scale. An API under real evolutionary pressure changes its fields, its behaviour and its error surface on every dated release, and its URL space on none of them. Hypermedia decouples the client from the empty lane.</figcaption>
</figure>

So the trade, as I read it: pay a round trip, a missing CI gate, an unnameable
span and an unknown method on every single call, to be insulated from a change
that happens roughly never.

And stay fully exposed to the changes that happen every sprint, because a link
says nothing about a renamed field, a narrowed enum, a new required parameter or
an altered side effect.

This is the same failure I described in
[who pays for the pressure](/articles/who-pays-for-the-pressure/), inverted.
There, real pressure inside an implementation got relocated onto the contract.
Here, a contract cost is paid continuously to hedge a pressure that was never
going to arrive. Teams feel the second one immediately and cannot articulate it,
which is why the argument keeps being lost on paper and won in practice.

None of this makes hypermedia's design incoherent. It makes it *tuned for a
different deployment*: many independent clients that cannot be upgraded
together, a document-shaped domain, and a consumer that needs no build-time
contract because it has no build. The Web. Feeds. Crawlers. `sitemap.xml`. Those
clients pay none of the four line items above, which is precisely why they
follow links happily.

An internal payments service is not that deployment. Neither is a build system,
nor a warehouse integration, nor anything with an on-call rotation attached.

## The tenth that everybody kept

What teams kept is small, and the rest of this series is built on it.

Hard-code the operations; read their *validity* off the response. An order
carrying `"status": "pending"`, or better, an explicit
`"allowedActions": ["cancel", "refund"]`, tells the client which of the
operations it already understands are legal right now, and it does so as a
field. A schema can express it. A generator can turn it into an enum. A pipeline
can fail when it changes. A span can still be named.

That keeps the server as the authority on current availability and drops the
claim that the client should learn the operation's identity and invocation by
traversal. Those two ideas arrived bundled, and almost every argument about
HATEOAS is really an argument about the second conducted in the vocabulary of
the first. Nobody is defying Fielding. They are declining one clause of him and
keeping the rest.

## What a smarter client does not fix

This is where the two bills separate.

Part I's charge is answerable in principle. Build a client smart enough to read
an unfamiliar affordance and reason about it, and that charge is dropped.

Not one of the four line items above is dropped with it. Intelligence buys
understanding, and every bill in this article is an operational bill.

So when somebody finally sat down to design a protocol for the smartest clients
we have ever deployed, the interesting question is not whether they had heard of
hypermedia. It is which of these two bills they were looking at.

They were looking at both, and the choices they made (a stable named vocabulary,
schemas a client can validate against, a surface deliberately prevented from
shifting under the caller) read very differently once you know what the second
bill costs.

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

7. oasdiff — command-line OpenAPI diff and breaking-change detection, run
   locally or as a CI action on the pull request.
   https://github.com/oasdiff/oasdiff

8. OpenTelemetry, *Semantic Conventions for HTTP Spans* — span names as
   `{method} {target}` with a low-cardinality target; “Instrumentation MUST NOT
   default to using URI path as a `{target}`”; `http.route` as the matched
   low-cardinality route template, not to be substituted by the URI path.
   https://opentelemetry.io/docs/specs/semconv/http/http-spans/

**The retry**

9. RFC 9110, *HTTP Semantics* — idempotent methods, and automatic retry after
   a connection failure.
   https://www.rfc-editor.org/rfc/rfc9110#name-idempotent-methods

10. Carson Gross, “Hypermedia Clients” — links in JSON carrying no method
    information.
    https://four.htmx.org/essays/hypermedia-clients

**The mispricing**

11. Stripe, *Versioning* — dated releases, the `Stripe-Version` header, major
    versus monthly releases.
    https://docs.stripe.com/api/versioning

12. Stripe, *Upgrades* — the published list of backwards-compatible changes:
    new resources, new optional request parameters, new response properties,
    property reordering, opaque string format, new event types. No entry
    concerns URLs.
    https://docs.stripe.com/upgrades

13. Fabio Ellena, “Who Pays for the Pressure,” 2026 — pressure relocated onto a
    contract, and who carries it.
    https://fblln.github.io/articles/who-pays-for-the-pressure/

**In this series**

14. Fabio Ellena, “The Browser Was Never the Smart Client,” 2026 — Part I.
    https://fblln.github.io/articles/the-browser-was-never-the-smart-client/
