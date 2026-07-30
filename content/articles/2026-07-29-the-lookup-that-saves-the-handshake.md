+++
title = "The Lookup That Saves the Handshake"
date = "2026-07-29"
description = "RFC 9146 adds a connection identifier to DTLS 1.2, usually summarised as NAT rebinding support. That summary hides the design: what the CID really does is move where the protocol anchors its state, from an address it never chose to an identifier it owns. Nine bytes, and a case study in deep modules and interface ownership."
tags = ["Protocols", "Architecture", "Complexity", "Security"]
+++

A sensor completes a DTLS 1.2 handshake with a server. Certificates are
verified, an elliptic-curve exchange runs, keys are derived. On the server, the
result is a block of state that looks roughly like this:

```text
Peer address:     198.51.100.23:62000
Local address:    203.0.113.7:5684
Protocol:         UDP

Cipher suite:     TLS_ECDHE_ECDSA_WITH_AES_128_CCM_8
Read key:         ...
Write key:        ...
Epoch:            1
Receive sequence: 42
```

Then the device goes to sleep, because it runs on a battery and sleeping is most
of what it does. While it sleeps, the NAT mapping in front of it expires. When it
wakes up and sends its next reading, the router hands it a different external
port:

```text
Before sleep: 198.51.100.23:62000
After sleep:  198.51.100.23:49172
```

Nothing cryptographic has changed. The device still holds the keys. The server
still holds the keys. The record now in flight is perfectly authentic, and the
server will discard it.

That sounds like a contradiction until you look at the order in which a receiver
has to do things.

## The identifier nobody chose

Here is the whole problem in one picture: the fixed part of a DTLS 1.2 record
header, drawn to scale.

<figure class="diagram">
<svg viewBox="0 0 620 132" role="img" aria-label="The thirteen-byte DTLS 1.2 record header drawn to scale at twenty-four pixels per byte: a one-byte content type, two-byte version, two-byte epoch, six-byte sequence number and two-byte length, followed by the encrypted fragment filling the rest of the record. A bracket beneath marks the thirteen bytes the receiver can read before it holds a key, annotated to say that nothing in that span identifies the security association.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">DTLS 1.2 RECORD HEADER &middot; TO SCALE &middot; 1 BYTE = 24 PX</text>
  <rect x="0" y="26" width="24" height="34" fill="var(--ink)" opacity="0.14"/>
  <rect x="312" y="26" width="308" height="34" fill="var(--ink)" opacity="0.06"/>
  <g stroke="var(--line)" fill="none">
    <path d="M24 26 L24 60 M72 26 L72 60 M120 26 L120 60 M264 26 L264 60 M312 26 L312 60"/>
  </g>
  <rect x="0" y="26" width="620" height="34" fill="none" stroke="var(--ink)"/>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">
    <text x="48" y="78">version</text>
    <text x="96" y="78">epoch</text>
    <text x="192" y="78">sequence number</text>
    <text x="288" y="78">length</text>
  </g>
  <text x="0" y="78" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">type</text>
  <text x="0" y="90" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">1</text>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">
    <text x="48" y="90">2</text>
    <text x="96" y="90">2</text>
    <text x="192" y="90">6</text>
    <text x="288" y="90">2</text>
    <text x="466" y="90">encrypted fragment &middot; unreadable until a key is chosen</text>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M0 100 L0 108 L312 108 L312 100"/>
  </g>
  <text x="156" y="126" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="middle">13 readable bytes &middot; none of them name a key</text>
</svg>
<figcaption>Thirteen bytes of framing, and not one field says which session this record belongs to. Every design decision in RFC 9146 follows from that omission.</figcaption>
</figure>

Before it can authenticate or decrypt anything, the receiver has to answer a
question the record does not contain: *which security association is this?* It
needs the read key, the epoch, the replay window. Classic DTLS answers the
question with the only identifier available, which arrived from the layer below:

```text
(source IP, source port, destination IP, destination port, UDP)
```

The five-tuple was never designed to be a session identity. It is a routing
artefact. DTLS adopted it because it was free, and free identifiers are usually
the expensive kind — they come with an owner, and the owner is not you. Here the
owner is every NAT, every carrier-grade address translator, and every idle timer
between the device and the server. Any of them can invalidate your primary key
without notifying anyone, and they do it precisely when the device has been
quiet, which for a battery-powered sensor is the normal state.

So the receive path looks like this, and it fails at step two.

<figure class="diagram">
<svg viewBox="0 0 620 152" role="img" aria-label="A five-stage receive pipeline reading left to right: datagram, five-tuple, association, keys, verify. A large cross is drawn over the association stage. Below, two lookup rows: the known port sixty-two thousand resolving to context four one seven, and the new port forty-nine one seven two resolving to nothing.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">RECEIVE PATH &middot; CLASSIC DTLS</text>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="0" y="26" width="112" height="30" fill="var(--ink)" opacity="0.08"/>
    <text x="56" y="45" fill="var(--ink)" text-anchor="middle">DATAGRAM</text>
    <rect x="126" y="26" width="112" height="30" fill="none" stroke="var(--line)"/>
    <text x="182" y="45" fill="var(--ink)" text-anchor="middle">FIVE-TUPLE</text>
    <rect x="252" y="26" width="112" height="30" fill="none" stroke="var(--signal)"/>
    <text x="308" y="45" fill="var(--ink)" text-anchor="middle">ASSOCIATION</text>
    <rect x="378" y="26" width="112" height="30" fill="none" stroke="var(--line)"/>
    <text x="434" y="45" fill="var(--ink)" text-anchor="middle">KEYS</text>
    <rect x="504" y="26" width="116" height="30" fill="none" stroke="var(--line)"/>
    <text x="562" y="45" fill="var(--ink)" text-anchor="middle">VERIFY</text>
  </g>
  <g stroke="var(--ink)" fill="none">
    <path d="M112 41 L124 41 M119 37 L124 41 L119 45"/>
    <path d="M238 41 L250 41 M245 37 L250 41 L245 45"/>
    <path d="M364 41 L376 41 M371 37 L376 41 L371 45"/>
    <path d="M490 41 L502 41 M497 37 L502 41 L497 45"/>
  </g>
  <g stroke="var(--signal)" stroke-width="1.5" fill="none">
    <path d="M288 28 L328 54 M328 28 L288 54"/>
  </g>
  <g font-family="var(--font-mono)" font-size="9">
    <text x="0" y="86" fill="var(--muted)">198.51.100.23:62000</text>
    <text x="150" y="86" fill="var(--muted)">&rarr;</text>
    <text x="172" y="86" fill="var(--ink)">DTLS CONTEXT #417</text>
    <text x="0" y="106" fill="var(--signal)">198.51.100.23:49172</text>
    <text x="150" y="106" fill="var(--signal)">&rarr;</text>
    <text x="172" y="106" fill="var(--signal)">NO MATCHING CONTEXT</text>
    <text x="356" y="106" fill="var(--muted)">&middot; same device, same keys</text>
  </g>
  <text x="0" y="132" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the record is authentic &middot; the receiver has no way to discover that</text>
  <text x="620" y="132" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="end">verification cannot start</text>
</svg>
<figcaption>The failure is not cryptographic. It is a missing row in a hash table, three stages before any cryptography would have run.</figcaption>
</figure>

The obvious escape is to try every key until one verifies. It is also the escape
that must not be taken. With a million associations, every unmatched datagram
becomes a million AEAD verifications — which is to say, an attacker with a UDP
socket and a random payload generator has found your CPU budget. The trial
decryption is not merely slow. It is a denial-of-service primitive dressed as
robustness.

So the problem is not that the packet cannot be authenticated. The problem is
that authentication cannot *begin* until the receiver knows which key to use, and
the only field it had for that purpose belongs to someone else.

RFC 9146 supplies the field.

## Negotiating a connection ID

The extension is `connection_id`, number 54, and the detail that trips everyone
up on first reading is the direction: **each endpoint declares the CID it wants
to receive**, not the one it intends to send.

<figure class="diagram">
<svg viewBox="0 0 620 158" role="img" aria-label="Client and server exchange connection identifiers during the handshake. The ClientHello carries connection_id equals C, the ServerHello carries connection_id equals S. A legend below shows that records travelling client to server carry S, and records travelling server to client carry C.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">NEGOTIATION &middot; EACH END NAMES ITS OWN INBOX</text>
  <rect x="0" y="30" width="120" height="72" fill="none" stroke="var(--line)"/>
  <text x="60" y="70" font-family="var(--font-mono)" font-size="10" fill="var(--ink)" text-anchor="middle">CLIENT</text>
  <rect x="500" y="30" width="120" height="72" fill="none" stroke="var(--line)"/>
  <text x="560" y="70" font-family="var(--font-mono)" font-size="10" fill="var(--ink)" text-anchor="middle">SERVER</text>
  <g stroke="var(--ink)" fill="none">
    <path d="M120 50 L494 50 M488 46 L494 50 L488 54"/>
  </g>
  <text x="307" y="42" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">ClientHello &middot; connection_id = C</text>
  <text x="130" y="70" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">&ldquo;write C on what you send me&rdquo;</text>
  <g stroke="var(--ink)" fill="none">
    <path d="M500 86 L126 86 M132 82 L126 86 L132 90"/>
  </g>
  <text x="313" y="102" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">ServerHello &middot; connection_id = S</text>
  <text x="490" y="70" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="end">&ldquo;write S on what you send me&rdquo;</text>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="0" y="118" width="300" height="20" fill="var(--signal)"/>
    <text x="150" y="132" fill="var(--paper)" text-anchor="middle">client &rarr; server records carry S</text>
    <rect x="320" y="118" width="300" height="20" fill="none" stroke="var(--line)"/>
    <text x="470" y="132" fill="var(--ink)" text-anchor="middle">server &rarr; client records carry C</text>
  </g>
  <text x="310" y="154" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">two independent values &middot; different lengths &middot; either may be absent</text>
</svg>
<figcaption>Nothing here is symmetric by requirement. Two directions, two identifiers, each chosen by the side that will have to look it up.</figcaption>
</figure>

If the server answers with `7A 91 03 52 10 B4 68 CC`, then the client stamps that
value on every protected record it sends, and the value is meaningful only inside
the server's own receive table. It is not an identity, not a name, not a
negotiated shared symbol. It is a key into one specific data structure on one
specific machine.

A zero-length CID is legal and says something useful: *I support the extension
and will happily label records for you, but do not label the ones you send me.*
That asymmetry is the common case in practice. A constrained client talks to
exactly one server, keeps exactly one association, and can resolve any inbound
record without help. The server is holding a million associations and needs every
byte of help it can get. The protocol lets the cost land where the need is,
instead of insisting both ends pay for symmetry neither asked for.

## The record on the wire

Once records are protected, a sender with a non-empty CID switches to a new outer
content type, `tls12_cid` (25), and inserts the CID after the sequence number.
Same scale as before, so the two headers can be read as a before and after.

<figure class="diagram">
<svg viewBox="0 0 620 156" role="img" aria-label="The DTLS CID record header drawn at the same twenty-four pixels per byte scale: outer type twenty-five, version, epoch, sequence number, then an eight-byte connection identifier highlighted, then length, then the encrypted content. An arrow runs from the connection identifier field down to a lookup table entry resolving the identifier to DTLS context four one seven.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">tls12_cid RECORD HEADER &middot; SAME SCALE &middot; 13 &rarr; 21 BYTES</text>
  <rect x="0" y="26" width="24" height="34" fill="var(--ink)" opacity="0.14"/>
  <rect x="264" y="26" width="192" height="34" fill="var(--signal)"/>
  <rect x="504" y="26" width="116" height="34" fill="var(--ink)" opacity="0.06"/>
  <g stroke="var(--line)" fill="none">
    <path d="M24 26 L24 60 M72 26 L72 60 M120 26 L120 60 M456 26 L456 60 M504 26 L504 60"/>
  </g>
  <rect x="0" y="26" width="620" height="34" fill="none" stroke="var(--ink)"/>
  <text x="360" y="47" font-family="var(--font-mono)" font-size="10" fill="var(--paper)" text-anchor="middle">CONNECTION ID</text>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">
    <text x="12" y="78">25</text>
    <text x="48" y="78">version</text>
    <text x="96" y="78">epoch</text>
    <text x="192" y="78">sequence</text>
    <text x="480" y="78">length</text>
  </g>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">
    <text x="360" y="78">8 bytes</text>
    <text x="562" y="78">content</text>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M360 88 L360 104 M356 98 L360 104 L364 98"/>
  </g>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="196" y="108" width="328" height="22" fill="none" stroke="var(--signal)"/>
    <text x="360" y="123" fill="var(--ink)" text-anchor="middle">7A91035210B468CC &rarr; DTLS CONTEXT #417</text>
  </g>
  <text x="0" y="150" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">one hash lookup &middot; constant time &middot; independent of the source address</text>
  <text x="620" y="150" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">+8 bytes</text>
</svg>
<figcaption>The same thirteen bytes, plus a field that finally belongs to the receiver. Everything else in the record is unchanged, which is the point — this is an insertion, not a redesign.</figcaption>
</figure>

In structure form:

```text
DTLSCiphertext {
    outer_type      = tls12_cid;     // value 25
    version;
    epoch;
    sequence_number;
    cid[cid_length];                 // new
    length;
    encrypted_content[length];
}
```

Now the part I find genuinely elegant: **there is no CID length field in the
record.** The record carries the identifier but not its shape. That looks like an
oversight and is the opposite of one. The receiver chose the CID, so the receiver
already knows how long it is. Putting a length on the wire would be paying, on
every single packet, for information one end already possesses.

A deployment can simply declare that all its CIDs are eight bytes, and the parser
reads eight bytes. Variable lengths are allowed too, provided the encoding is
self-delineating — an implementation might spend two bits of the first byte:

```text
00xxxxxx  ->  4-byte CID
01xxxxxx  ->  8-byte CID
10xxxxxx  -> 12-byte CID
11xxxxxx  -> reserved
```

That scheme is nowhere in RFC 9146. It cannot be, and should not be, because it
is not the protocol's business. Which gives the underlying rule:

> The endpoint that performs the lookup owns the format of the identifier.

This is interface design, not packet formatting. The party that bears the cost of
a decision gets to make it, and the specification declines to standardise
something it would only be standardising for the sake of symmetry. Compare it to
the failure mode we all know from application code: a shared identifier format
negotiated across a boundary, so that any change to one side's indexing strategy
becomes a protocol change, a version bump, and a migration for everybody. RFC
9146 refused to create that coupling. The CID is opaque *by construction* — it
has to be, because only one end is ever allowed to interpret it.

## What moved inside the ciphertext

The visible content type of a CID record is always 25. The real type — alert,
handshake, application data — moves inside the protected payload.

```text
DTLSInnerPlaintext {
    content;
    real_type;
    zero_padding;
}
```

<figure class="diagram">
<svg viewBox="0 0 620 186" role="img" aria-label="Two rows compare the plaintext and the wire format. The top row shows the inner plaintext: a CoAP message, then a one-byte real type of twenty-three, then optional zero padding. The bottom row shows what an observer sees: outer type twenty-five, epoch, sequence, connection identifier and length readable, then an opaque ciphertext block. A dashed boundary separates the readable fields from the opaque region.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">INNER PLAINTEXT &middot; BEFORE PROTECTION</text>
  <rect x="0" y="24" width="300" height="30" fill="var(--ink)" opacity="0.08"/>
  <text x="150" y="43" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">CoAP message</text>
  <rect x="300" y="24" width="72" height="30" fill="var(--signal)"/>
  <text x="336" y="43" font-family="var(--font-mono)" font-size="9" fill="var(--paper)" text-anchor="middle">type = 23</text>
  <rect x="372" y="24" width="248" height="30" fill="none" stroke="var(--line)" stroke-dasharray="3 3"/>
  <text x="496" y="43" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">00 00 00 00 &middot; optional padding</text>
  <text x="372" y="68" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="end">the real type, now cargo</text>
  <text x="382" y="68" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">a 7-byte reading can look like a 60-byte one</text>
  <g stroke="var(--ink)" fill="none">
    <path d="M310 78 L310 96 M306 90 L310 96 L314 90"/>
  </g>
  <text x="0" y="112" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">ON THE WIRE &middot; WHAT AN OBSERVER READS</text>
  <rect x="0" y="124" width="240" height="30" fill="none" stroke="var(--ink)"/>
  <g stroke="var(--line)" fill="none">
    <path d="M40 124 L40 154 M88 124 L88 154 M192 124 L192 154"/>
  </g>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">
    <text x="20" y="143">25</text>
    <text x="64" y="143">epoch</text>
    <text x="140" y="143">seq</text>
    <text x="216" y="143">CID</text>
  </g>
  <rect x="248" y="124" width="372" height="30" fill="var(--ink)" opacity="0.14"/>
  <text x="434" y="143" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">A7 3B 1F 9D &hellip; opaque</text>
  <g stroke="var(--signal)" fill="none">
    <path d="M244 118 L244 160" stroke-dasharray="3 3"/>
  </g>
  <text x="0" y="178" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">visible: association, epoch, sequence, length</text>
  <text x="620" y="178" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">hidden: what kind of record this is</text>
</svg>
<figcaption>The type field did not disappear; it changed audience. An observer keeps the routing metadata and loses the semantics — which is a fair description of what a record layer should be leaking in the first place.</figcaption>
</figure>

Two things come along for the ride. An observer can no longer tell an alert from
application data from a rekeying handshake message, and padding becomes possible,
so a seven-byte temperature reading need not be identifiable *as* a seven-byte
temperature reading.

The structural bill for all of this, with an eight-byte CID and no padding:

```text
CID in the record header          8 bytes
Encrypted real content type       1 byte
                                 --------
                                  9 bytes
```

Nine bytes. Hold that number; I want to come back to what it buys.

## A visible identifier that is not a credential

The CID is not secret. Anyone on the path reads it, and anyone can copy it into a
datagram with a forged source address and random ciphertext. That forged packet
*will* find context #417. The lookup succeeds — and then nothing else does. For
AEAD suites, RFC 9146 folds the CID and the record metadata into the
authenticated additional data:

```text
additional_data =
      8 octets of 0xFF        // stands in for the classic seq_num field
    + tls12_cid               // 25
    + cid_length
    + tls12_cid               // 25, again
    + version
    + epoch
    + sequence_number
    + cid
    + length_of_inner_plaintext
```

The content type appearing twice is not elegance. It is compatibility — the
construction preserves the shape the classic computation had, so existing code
paths and their length assumptions survive. Specifications that live in deployed
firmware make this trade constantly, and it is usually the right one; an ugly
constant costs a line of code, whereas a re-shaped computation costs an
interoperability matrix.

The important property is the guarantee that falls out. Modify the CID, the
epoch, the sequence number, or the payload, and the tag fails. So:

> The CID is a lookup hint, protected by the record it labels. It is not a bearer
> token, and by itself it proves nothing.

I would put that sentence on a wall somewhere near most session-handling code I
have reviewed, because the common mistake is the exact inverse: an identifier used
simultaneously for *finding* state and for *authorising* access to it. A session
cookie that is both the row key and the proof of ownership. A tenant ID pulled
from a header and trusted because it was specific enough to work. Once one string
does both jobs, disclosure of the identifier becomes escalation of privilege, and
you cannot log it, cache it, put it in a URL, or hand it to a load balancer
without thinking about who is watching.

DTLS keeps the two jobs apart, and the separation is what makes the CID safe to
print in clear text on every packet. Addressing is public. Authority stays with
the keys.

## Finding the association is not finding the peer

With a CID in place, the sleeping sensor wakes up, sends its record from a new
port, and the server resolves it in one lookup:

```python
if outer_type == tls12_cid:
    association = associations_by_cid[parse_cid(record)]
else:
    association = associations_by_five_tuple[packet.five_tuple]
```

The record authenticates. The server now knows, with cryptographic certainty,
that this datagram was produced by the holder of the connection keys.

It still does not know where to send the reply.

Consider an attacker who captured a valid record earlier — say sequence 40,
carrying "give me current state" — and replays it verbatim with a forged source
address. Everything checks out, because everything *is* authentic; it was
authentic when it was recorded. If the server treats a successful decryption as
permission to move the peer address, it will now start sending state reports to
an address of the attacker's choosing, and it will amplify small forged datagrams
into large real ones. The protocol has been turned into a reflector.

RFC 9146 therefore sets three conditions before a peer address may be updated:

```text
1. The record passes cryptographic verification.

2. Its epoch and sequence number make it newer than the newest
   record previously received.

3. The implementation has a strategy for proving that the peer can
   receive and process records at the new address.
```

The second condition does quiet, necessary work: it stops a late-arriving old
packet from dragging the binding backwards.

```text
sequence 43 from address B   ->  binding moves to B
sequence 42 from address A   ->  ignored; A is the past
```

The third condition is the interesting one, because in RFC 9146 it is not a
mechanism at all. It is a requirement to have one. The specification names the
obligation and leaves the discharge to the application protocol or the
implementation.

I have mixed feelings about holes like this, and I recognise them from
architecture reviews. Leaving the mechanism unspecified was defensible: DTLS
sits underneath application protocols that may already have a heartbeat, and
mandating a redundant one would have been the protocol legislating a concern it
does not own. But an obligation with no mechanism is an obligation that gets
implemented differently everywhere, or quietly not at all — and the failure mode
of "not at all" is an open reflector, which is silent, remote, and not visible in
any test the implementer was likely to write. Unspecified is not free. It is
design debt with the interest payable by whoever ships.

## The missing mechanism arrived four years later

RFC 9853, in March 2026, closes the hole with a standardised Return Routability
Check. When a valid record turns up from a new address, the receiver challenges
the path with an unpredictable 64-bit cookie before it will trust it.

<figure class="diagram">
<svg viewBox="0 0 620 234" role="img" aria-label="A sequence diagram between a client at a new address B and the server, drawn as two vertical lifelines. The client sends a CID record with sequence forty-three, the server replies with a path challenge carrying a cookie, the client echoes the cookie in a path response, and the binding is then updated. A shaded span marks the period while the address is unvalidated and the anti-amplification limit is in force. Below, a byte budget bar shows the bytes received from the unvalidated address and a block three times its width marking the maximum the server may send in return.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">RETURN ROUTABILITY CHECK &middot; RFC 9853</text>
  <rect x="110" y="44" width="360" height="96" fill="var(--signal)" opacity="0.07"/>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">
    <text x="110" y="36">ADDRESS B</text>
    <text x="470" y="36">SERVER</text>
  </g>
  <g stroke="var(--line)" fill="none">
    <path d="M110 44 L110 140 M470 44 L470 140"/>
  </g>
  <g stroke="var(--ink)" fill="none">
    <path d="M110 64 L470 64 M464 60 L470 64 L464 68"/>
    <path d="M110 132 L470 132 M464 128 L470 132 L464 136"/>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M470 98 L110 98 M116 94 L110 98 L116 102"/>
  </g>
  <text x="290" y="58" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">CID record, seq 43</text>
  <text x="290" y="92" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="middle">path_challenge(cookie=8F21&hellip;)</text>
  <text x="290" y="126" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">path_response(cookie=8F21&hellip;)</text>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--signal)">
    <text x="486" y="88">address unvalidated</text>
    <text x="486" y="100">anti-amplification</text>
    <text x="486" y="112">limit in force</text>
  </g>
  <g stroke="var(--ink)" fill="none">
    <path d="M0 150 L620 150" stroke-dasharray="3 3"/>
  </g>
  <text x="290" y="166" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">address B validated &middot; CID-to-address binding updated</text>
  <text x="0" y="190" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">BYTE BUDGET WHILE UNVALIDATED</text>
  <rect x="0" y="198" width="52" height="18" fill="var(--signal)"/>
  <text x="26" y="211" font-family="var(--font-mono)" font-size="9" fill="var(--paper)" text-anchor="middle">recv</text>
  <rect x="52" y="198" width="156" height="18" fill="none" stroke="var(--signal)"/>
  <text x="130" y="211" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">may send &le; 3&times;</text>
  <text x="220" y="211" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">buffered data waits &middot; no gain left for a reflector</text>
  <text x="0" y="230" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the cookie must come back through the path being tested &middot; that is the entire proof</text>
</svg>
<figcaption>Two round trips of caution, bounded by a three-times budget so that even the caution cannot be turned into amplification. The check proves reachability and nothing more — which is exactly the claim being made.</figcaption>
</figure>

So the complete receive path now has two distinct gates, one cryptographic and
one about reachability.

<figure class="diagram">
<svg viewBox="0 0 620 216" role="img" aria-label="A decision flow in three rows. Top row: parse the connection identifier, locate the association, authenticate, then ask whether the source address is unchanged. A branch drops from authenticate labelled tag invalid, discarded with no reply. If the address is unchanged the record is delivered. If it changed, a second decision asks whether the record is newer: if not, deliver without moving the binding; if it is, validate the path and only then update the address binding.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">RECEIVE PATH &middot; WITH CID AND RETURN ROUTABILITY</text>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="0" y="30" width="104" height="28" fill="var(--ink)" opacity="0.08"/>
    <text x="52" y="48" fill="var(--ink)" text-anchor="middle">PARSE CID</text>
    <rect x="118" y="30" width="120" height="28" fill="none" stroke="var(--line)"/>
    <text x="178" y="48" fill="var(--ink)" text-anchor="middle">LOCATE ASSOC</text>
    <rect x="252" y="30" width="120" height="28" fill="none" stroke="var(--signal)"/>
    <text x="312" y="48" fill="var(--ink)" text-anchor="middle">AUTHENTICATE</text>
    <rect x="386" y="30" width="134" height="28" fill="none" stroke="var(--line)"/>
    <text x="453" y="48" fill="var(--ink)" text-anchor="middle">ADDRESS SAME?</text>
  </g>
  <g stroke="var(--ink)" fill="none">
    <path d="M104 44 L116 44 M111 40 L116 44 L111 48"/>
    <path d="M238 44 L250 44 M245 40 L250 44 L245 48"/>
    <path d="M372 44 L384 44 M379 40 L384 44 L379 48"/>
  </g>
  <g stroke="var(--line)" fill="none">
    <path d="M312 58 L312 78 M308 72 L312 78 L316 72"/>
  </g>
  <text x="300" y="78" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="end">tag invalid &rarr; discarded, no reply</text>
  <g stroke="var(--ink)" fill="none">
    <path d="M520 44 L560 44 L560 96 M556 90 L560 96 L564 90"/>
    <path d="M420 58 L420 96 M416 90 L420 96 L424 90"/>
  </g>
  <text x="566" y="40" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">yes</text>
  <text x="426" y="76" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">no</text>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="300" y="100" width="160" height="28" fill="none" stroke="var(--line)"/>
    <text x="380" y="118" fill="var(--ink)" text-anchor="middle">NEWER RECORD?</text>
    <rect x="490" y="100" width="130" height="28" fill="none" stroke="var(--line)"/>
    <text x="555" y="118" fill="var(--ink)" text-anchor="middle">DELIVER</text>
  </g>
  <g stroke="var(--ink)" fill="none">
    <path d="M380 128 L380 156 M376 150 L380 156 L384 150"/>
    <path d="M300 114 L200 114 L200 156 M196 150 L200 156 L204 150"/>
  </g>
  <text x="386" y="146" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">yes</text>
  <text x="296" y="108" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="end">no &middot; keep the old binding</text>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="130" y="160" width="140" height="28" fill="none" stroke="var(--line)"/>
    <text x="200" y="178" fill="var(--ink)" text-anchor="middle">DELIVER</text>
    <rect x="300" y="160" width="160" height="28" fill="var(--signal)"/>
    <text x="380" y="178" fill="var(--paper)" text-anchor="middle">VALIDATE PATH</text>
    <rect x="474" y="160" width="146" height="28" fill="none" stroke="var(--signal)"/>
    <text x="547" y="178" fill="var(--ink)" text-anchor="middle">UPDATE BINDING</text>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M460 174 L472 174 M467 170 L472 174 L467 178"/>
  </g>
  <text x="0" y="212" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">gate one: do you hold the keys &middot; gate two: are you actually there</text>
</svg>
<figcaption>Delivery and rebinding are decided separately, and a record can pass one gate while failing the other. Collapsing the two decisions into one is precisely the bug the flow exists to prevent.</figcaption>
</figure>

Two mechanisms, two proofs, two questions:

> **Who holds the keys?** Answered by the record. Answered by the CID's presence
> in the authenticated data.
>
> **Who is at this address?** Not answered by the record at all. Answered only by
> something arriving back through the path.

The CID solves security-context discovery. Return routability solves path
ownership. They look like one problem — "the client moved" — and they are not,
and the four years between the two documents is the evidence.

## The eight bytes are an interface

There is a way of reading the CID that I like better than "a NAT workaround".

The security association is a large, stateful, complicated thing. Keys, epochs,
a replay window, cipher parameters, retransmission timers, path state, sometimes
a handshake transcript. It is exactly the sort of object whose complexity you
want to be able to *not think about* from the outside.

<figure class="diagram">
<svg viewBox="0 0 620 188" role="img" aria-label="An eight-byte connection identifier drawn as a narrow bar above a much larger block representing the security association, which contains read and write keys, cipher parameters, epoch, sequence number and replay window, peer and local address state, retransmission timers and handshake transcript. Labels give the ratio of eight bytes of interface to the whole implementation beneath.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">DEPTH &middot; INTERFACE OVER IMPLEMENTATION</text>
  <text x="310" y="34" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">8 bytes, on every packet</text>
  <rect x="250" y="40" width="120" height="18" fill="var(--signal)"/>
  <text x="310" y="53" font-family="var(--font-mono)" font-size="9" fill="var(--paper)" text-anchor="middle">7A91&hellip;68CC</text>
  <rect x="40" y="58" width="540" height="108" fill="none" stroke="var(--line)"/>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">
    <text x="310" y="80">read key &middot; write key &middot; cipher suite &middot; AEAD parameters</text>
    <text x="310" y="98">epoch &middot; sequence number &middot; replay window</text>
    <text x="310" y="116">peer address &middot; local address &middot; path validation state</text>
    <text x="310" y="134">retransmission timers &middot; handshake transcript</text>
    <text x="310" y="152">certificates &middot; session resumption material</text>
  </g>
  <text x="0" y="182" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the caller names it &middot; the caller understands none of it</text>
  <text x="620" y="182" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">this ratio is the design</text>
</svg>
<figcaption>Ousterhout's deep module, drawn in bytes rather than method signatures. The interface is eight opaque bytes chosen by the implementation, which is about as narrow as an interface to something this large can get.</figcaption>
</figure>

That is a deep module in the sense
[I have written about before](/articles/risk-complexity-and-pressure/): the
interface is dramatically simpler than what it hides, and the ratio is the whole
point. The CID names the association without describing it. The sender knows the
eight bytes and nothing else — not the format, not the table, not the sharding
scheme, not whether the server keeps associations in memory or reconstructs them
from an encrypted blob. All of it is free to change on the receiving side without
a single packet on the wire changing shape.

And note what happens without the CID. The interface to the association was the
five-tuple, which is not opaque, not chosen by the owner, and not stable — an
interface made of someone else's mutable implementation detail. Every property you
want from a module boundary was violated by the identifier, and the visible
symptom was a sensor that could not talk to its server after a nap.

Bad identifiers are bad interfaces. It is the same failure, at a different scale,
as a public method that takes a struct the caller had to reverse-engineer from
your storage layout.

## What nine bytes buy

Now the accounting, because a design that adds bytes to every packet on a
constrained network needs to answer for them.

<figure class="diagram">
<svg viewBox="0 0 620 176" role="img" aria-label="A to-scale comparison. A long bar represents a full DTLS handshake of roughly sixteen hundred bytes across six flights with a certificate chain and elliptic curve operations. Beneath it, a sliver a few pixels wide represents the nine bytes of CID overhead per record, magnified in a callout to show its true relative size.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">TO SCALE &middot; WHAT WAKING UP COSTS</text>
  <rect x="0" y="26" width="600" height="30" fill="var(--ink)" opacity="0.14"/>
  <rect x="0" y="26" width="600" height="30" fill="none" stroke="var(--ink)"/>
  <g stroke="var(--paper)" fill="none">
    <path d="M100 26 L100 33 M200 26 L200 33 M300 26 L300 33 M400 26 L400 33 M500 26 L500 33"/>
    <path d="M100 49 L100 56 M200 49 L200 56 M300 49 L300 56 M400 49 L400 56 M500 49 L500 56"/>
  </g>
  <text x="300" y="45" font-family="var(--font-mono)" font-size="10" fill="var(--ink)" text-anchor="middle">FULL HANDSHAKE &middot; ~1600 BYTES &middot; SIX FLIGHTS</text>
  <text x="0" y="72" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">certificate chain &middot; two elliptic-curve operations &middot; three round trips of radio, awake, at full power</text>
  <rect x="0" y="92" width="3.4" height="30" fill="var(--signal)"/>
  <g stroke="var(--signal)" fill="none">
    <path d="M4 107 L122 100"/>
  </g>
  <rect x="124" y="88" width="120" height="22" fill="var(--signal)"/>
  <text x="184" y="103" font-family="var(--font-mono)" font-size="9" fill="var(--paper)" text-anchor="middle">9 BYTES</text>
  <text x="256" y="103" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">per record &middot; that sliver on the left is the true width</text>
  <text x="256" y="119" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">no round trips &middot; no asymmetric crypto &middot; no new state</text>
  <g stroke="var(--line)" fill="none">
    <path d="M0 138 L620 138"/>
  </g>
  <text x="0" y="158" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">178 protected records at the CID's overhead &asymp; one handshake you did not have to run</text>
  <text x="620" y="158" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">and the radio stays asleep</text>
</svg>
<figcaption>The comparison is not really about bytes. It is about which of the two costs the device can afford while running on a battery it will not have replaced for a decade.</figcaption>
</figure>

Without a CID, a NAT rebinding means the association is unreachable, and the only
recovery is a new handshake: several flights, a certificate chain, asymmetric
operations at both ends, and — the part that actually kills the device — multiple
round trips with the radio powered up. Nine bytes per record against that, and
the device gets to send one datagram and go back to sleep.

That is the trade in one line: a small, permanent, predictable cost on the common
path, in exchange for removing a large, occasional, unpredictable one. It is the
same trade as an index on a table, or a version field in a serialised struct, or
a request ID threaded through a distributed system. Nobody notices the nine bytes.
Everybody notices the handshake storm when ten thousand sensors wake up at
sunrise and all of them have new ports.

## The price of persistence

And now the part that keeps the design honest, because the property that lets a
connection survive a network change is the same property that lets someone follow
it across one.

<figure class="diagram">
<svg viewBox="0 0 620 172" role="img" aria-label="Two network paths, a mobile network and home Wi-Fi, each with a different source address but carrying the same connection identifier, converge on an on-path observer which links them as the same device. A note states that DTLS 1.2 negotiates the identifier once and has no mechanism to rotate it.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">CORRELATION &middot; THE SAME BYTES, TWO NETWORKS</text>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="0" y="26" width="230" height="28" fill="none" stroke="var(--line)"/>
    <text x="12" y="44" fill="var(--muted)">mobile</text>
    <text x="70" y="44" fill="var(--ink)">203.0.113.18:55001</text>
    <rect x="0" y="94" width="230" height="28" fill="none" stroke="var(--line)"/>
    <text x="12" y="112" fill="var(--muted)">Wi-Fi</text>
    <text x="70" y="112" fill="var(--ink)">198.51.100.23:49172</text>
  </g>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="250" y="26" width="150" height="28" fill="var(--signal)"/>
    <text x="325" y="44" fill="var(--paper)" text-anchor="middle">CID 7A91&hellip;68CC</text>
    <rect x="250" y="94" width="150" height="28" fill="var(--signal)"/>
    <text x="325" y="112" fill="var(--paper)" text-anchor="middle">CID 7A91&hellip;68CC</text>
  </g>
  <g stroke="var(--line)" fill="none">
    <path d="M230 40 L246 40 M230 108 L246 108"/>
  </g>
  <g stroke="var(--signal)" fill="none">
    <path d="M400 40 L460 40 L460 68 L500 68 M494 64 L500 68 L494 72"/>
    <path d="M400 108 L460 108 L460 68"/>
  </g>
  <rect x="504" y="52" width="116" height="32" fill="none" stroke="var(--signal)"/>
  <text x="562" y="72" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">ON-PATH EYE</text>
  <text x="562" y="98" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="middle">same device</text>
  <text x="0" y="146" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">negotiated once, at handshake time &middot; DTLS 1.2 has no rotation mechanism</text>
  <text x="0" y="164" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">structured CID &nbsp;[region][cluster][shard][conn]&nbsp; &rarr; the identifier also describes your deployment</text>
</svg>
<figcaption>A stable identifier is a linkable identifier; there is no version of this feature where that is not true. What varies is only whether the deployment decided that trade knowingly.</figcaption>
</figure>

DTLS 1.2 negotiates CIDs once, at the start of the session, and offers no way to
rotate them mid-session. RFC 9146 says so plainly and advises against CIDs in
mobility or multihoming deployments where cross-path correlation matters. A
sensor bolted to a wall behind one NAT is a different proposition from a device
that roams between a phone hotspot and an office network several times a day.

The receiver's control over the CID format cuts both ways here too. A structured
CID — region, cluster, shard, connection — is convenient for routing, and it also
publishes a sketch of your infrastructure to anyone with a packet capture, and
gives an attacker a way to aim. Random and opaque leaks less. Owning the format
means owning what it discloses.

This is what an honest mechanism looks like: it solves the problem it claims to
solve, it names the new problem it creates, and it declines to pretend the second
one is small.

## Where the state is anchored

Strip the document back and RFC 9146 introduces: an extension number, a content
type, a field in a header, a modification to the AEAD input, a relocation of one
byte into the ciphertext, optional padding, and a set of rules for what to do
when a peer's address changes. Nine items, none of them individually clever.

The change that matters is the first operation performed on an incoming packet.

<figure class="diagram">
<svg viewBox="0 0 620 164" role="img" aria-label="Two chains compared. Without a CID: network address to security context to authentication, with the first link marked as owned by the network. With a CID: connection identifier to security context to authentication to path validation, with the first link marked as owned by the receiver.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">WHERE THE STATE IS ANCHORED</text>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="0" y="28" width="150" height="26" fill="none" stroke="var(--line)" stroke-dasharray="3 3"/>
    <text x="75" y="45" fill="var(--muted)" text-anchor="middle">NETWORK ADDRESS</text>
    <rect x="176" y="28" width="150" height="26" fill="none" stroke="var(--line)"/>
    <text x="251" y="45" fill="var(--ink)" text-anchor="middle">SECURITY CONTEXT</text>
    <rect x="352" y="28" width="150" height="26" fill="none" stroke="var(--line)"/>
    <text x="427" y="45" fill="var(--ink)" text-anchor="middle">AUTHENTICATION</text>
  </g>
  <g stroke="var(--line)" fill="none">
    <path d="M150 41 L170 41 M164 37 L170 41 L164 45"/>
    <path d="M326 41 L346 41 M340 37 L346 41 L340 45"/>
  </g>
  <text x="75" y="70" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">owned by the network</text>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="0" y="94" width="140" height="26" fill="var(--signal)"/>
    <text x="70" y="111" fill="var(--paper)" text-anchor="middle">CONNECTION ID</text>
    <rect x="156" y="94" width="140" height="26" fill="none" stroke="var(--ink)"/>
    <text x="226" y="111" fill="var(--ink)" text-anchor="middle">SECURITY CONTEXT</text>
    <rect x="312" y="94" width="140" height="26" fill="none" stroke="var(--ink)"/>
    <text x="382" y="111" fill="var(--ink)" text-anchor="middle">AUTHENTICATION</text>
    <rect x="468" y="94" width="152" height="26" fill="none" stroke="var(--signal)"/>
    <text x="544" y="111" fill="var(--ink)" text-anchor="middle">PATH VALIDATION</text>
  </g>
  <g stroke="var(--ink)" fill="none">
    <path d="M140 107 L150 107 M144 103 L150 107 L144 111"/>
    <path d="M296 107 L306 107 M300 103 L306 107 L300 111"/>
    <path d="M452 107 L462 107 M456 103 L462 107 L456 111"/>
  </g>
  <text x="70" y="136" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="middle">owned by the receiver</text>
  <text x="620" y="136" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="end">the address became data, not identity</text>
  <text x="0" y="158" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">nine bytes moved the anchor &middot; everything else in this chain is unchanged</text>
</svg>
<figcaption>Same session, same keys, same records. The only structural difference is which link in the chain the protocol depends on for its identity — and who is allowed to change it.</figcaption>
</figure>

The pressure here was concrete and measurable: NAT timers shorter than device
sleep cycles, and a battery that cannot pay for a public-key handshake every time
the network forgets about it. The risk was handshake storms, dropped readings, and
devices that appear dead for reasons no log explains. The control was a stable
identifier owned by the party doing the lookup. The mechanism was nine bytes and
a return routability check.

Nothing about that chain is protocol-specific. The recurring engineering mistake
it corrects is anchoring durable state to a borrowed, mutable identifier, and it
is everywhere: sessions keyed by IP, caches keyed by URL when the URL contains a
rotating token, entities keyed by an email address the user is about to change,
distributed state keyed by a hostname that the orchestrator will reassign at the
next deploy. The system works, right up until the identifier's real owner
exercises their right to change it, and then the failure looks like corruption or
a mystery rather than a design decision made years earlier by someone who wanted
to avoid inventing a key.

The fix, whenever it is available, is the same as RFC 9146's: mint your own
identifier, keep it opaque, let the party performing the lookup choose its shape,
and never let it double as proof of anything.

For a sensor on a wall, that is the difference between one protected datagram and
an entire handshake.

## References

1. E. Rescorla, H. Tschofenig, T. Fossati, A. Kraus, *Connection Identifier for
   DTLS 1.2*, RFC 9146, March 2022.
   https://www.rfc-editor.org/rfc/rfc9146.html

2. H. Tschofenig, T. Fossati, *Return Routability Check for DTLS 1.2 and DTLS
   1.3*, RFC 9853, March 2026.
   https://www.rfc-editor.org/rfc/rfc9853.html

3. E. Rescorla, N. Modadugu, *Datagram Transport Layer Security Version 1.2*,
   RFC 6347, January 2012.
   https://www.rfc-editor.org/rfc/rfc6347.html

4. John Ousterhout, *A Philosophy of Software Design*, Second Edition. Yaknyam
   Press, 2021. https://web.stanford.edu/~ouster/cgi-bin/book.php

5. Fabio Ellena, “Risk, Complexity, and Pressure,” 2026.
   https://fblln.github.io/articles/risk-complexity-and-pressure/
