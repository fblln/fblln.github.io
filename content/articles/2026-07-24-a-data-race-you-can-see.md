+++
title = "A Data Race You Can See"
date = "2026-07-24"
description = "A u32 on an 8-bit CPU is four load instructions, and an interrupt landing between two of them returns a number that was never in memory. Lab 4 of a bare-metal Rust curriculum: the CTC off-by-one hiding in my own solution code, why the compiler is entitled to break this even when the race never fires."
tags = ["Rust", "Embedded", "Concurrency"]
+++

I have spent years telling people that shared mutable state needs
synchronisation, and years being believed on authority rather than on evidence.
Distributed systems hide their races behind latency; a torn read on a 64-bit
core is a paragraph in a memory-model specification, not something you watch.

So I went looking for a machine narrow enough to show me one. An Arduino Uno R3
— ATmega328P, 8-bit, 16 MHz, 2 KB of SRAM — turns out to be an excellent
instrument, precisely because nothing on it is wide enough to hide behind.

This is Lab 4 of a thirteen-lab bare-metal Rust curriculum I am building by
working through it first: `millis()`, and the justification of its critical
section from Rust's abstract machine rather than from convention. Writing it up
found two bugs in my own solution code, which is the best argument I have for
writing things up.

## Provenance, before anything else

The curriculum's organising rule is *no claim without a measurement*, so this
article has to declare where its numbers come from before it uses any. As of
this writing:

| Claim | Status |
|---|---|
| Instruction counts, cycle costs, register semantics | ATmega328P datasheet — primary source |
| The CTC period arithmetic | Derivation, shown in full below |
| Tear probability, carry frequencies | Model, from the above |
| Anything about timing on real silicon | **Not yet run.** Zero of thirteen labs are validated on hardware |

That fourth row is the uncomfortable one, and I have left it uncomfortable
rather than rounding it up to "measured". Where a number could be contradicted
by a board, it is stated as a rate or a percentage, so that a stopwatch is
enough to contradict it.

## The machine is narrower than the number

Everything follows from one physical fact, so it goes before any code. The
ATmega328P's register file is thirty-two 8-bit registers. There is no register
that can hold a `u32`. The value is four bytes at four consecutive SRAM
addresses, and every operation on it is four operations.

<figure class="diagram">
<svg viewBox="0 0 620 168" role="img" aria-label="Diagram showing a 32-bit counter occupying four consecutive one-byte memory cells, and an 8-bit register file that can only move one cell at a time">
  <text x="0" y="21" font-family="var(--font-mono)" font-size="10" fill="var(--muted)">VALUE</text>
  <rect x="92" y="6" width="496" height="24" fill="none" stroke="var(--line)"/>
  <text x="340" y="22" font-family="var(--font-mono)" font-size="10" fill="var(--ink)" text-anchor="middle">COUNTER : u32 = 0x0000_0100</text>
  <path d="M340 30 L340 44 M142 44 L538 44 M142 44 L142 62 M274 44 L274 62 M406 44 L406 62 M538 44 L538 62" fill="none" stroke="var(--line)" stroke-width="1"/>
  <text x="0" y="79" font-family="var(--font-mono)" font-size="10" fill="var(--muted)">SRAM</text>
  <g font-family="var(--font-mono)" font-size="10">
    <rect x="92" y="62" width="100" height="26" fill="none" stroke="var(--line)"/><text x="142" y="79" fill="var(--ink)" text-anchor="middle">0x00</text>
    <rect x="224" y="62" width="100" height="26" fill="none" stroke="var(--line)"/><text x="274" y="79" fill="var(--ink)" text-anchor="middle">0x01</text>
    <rect x="356" y="62" width="100" height="26" fill="none" stroke="var(--line)"/><text x="406" y="79" fill="var(--ink)" text-anchor="middle">0x00</text>
    <rect x="488" y="62" width="100" height="26" fill="none" stroke="var(--line)"/><text x="538" y="79" fill="var(--ink)" text-anchor="middle">0x00</text>
  </g>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">
    <text x="142" y="102">b0 &middot; n+0</text><text x="274" y="102">b1 &middot; n+1</text>
    <text x="406" y="102">b2 &middot; n+2</text><text x="538" y="102">b3 &middot; n+3</text>
  </g>
  <g stroke="#ff3b00" stroke-width="1" fill="none">
    <path d="M142 108 L142 118 L340 118 L340 126 M274 108 L274 118 M406 108 L406 118 M538 108 L538 118 L340 118"/>
  </g>
  <rect x="248" y="126" width="184" height="24" fill="#ff3b00"/>
  <text x="340" y="142" font-family="var(--font-mono)" font-size="10" fill="#f2f0e9" text-anchor="middle">8-bit register file</text>
  <text x="0" y="164" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">lds is 2 words / 2 cycles &mdash; four of them, four separate instants</text>
</svg>
<figcaption>Little-endian, so the low byte sits at the low address. No instruction moves more than one of these cells, which means there is no instant at which the CPU holds the whole value.</figcaption>
</figure>

Hold on to the last line. Not "the read is slow": the read *has no single
instant*. It has four, and anything that runs between them observes a different
machine than the one the read started in.

## The timer, and the off-by-one in my own solution

Blocking is the naive way to measure time and it fails the moment two things
must happen at once — `delay_ms` doesn't pause an LED, it pauses the core. The
fix is a hardware timer that runs whether or not instructions are executing, and
an interrupt that maintains a counter the main loop can sample.

<figure class="diagram">
<svg viewBox="0 0 620 150" role="img" aria-label="Timing diagram comparing a blocking delay, which occupies the CPU for its whole duration, with a timer that fires a compare match every millisecond and leaves the main loop free">
  <text x="0" y="16" font-family="var(--font-mono)" font-size="10" fill="var(--muted)">BLOCKING</text>
  <rect x="92" y="6" width="214" height="16" fill="#ff3b00" opacity="0.22"/>
  <rect x="306" y="6" width="10" height="16" fill="var(--muted)"/>
  <rect x="316" y="6" width="214" height="16" fill="#ff3b00" opacity="0.22"/>
  <text x="199" y="18" font-family="var(--font-mono)" font-size="9" fill="#ff3b00" text-anchor="middle">delay_ms &middot; core unavailable</text>
  <text x="423" y="18" font-family="var(--font-mono)" font-size="9" fill="#ff3b00" text-anchor="middle">delay_ms &middot; core unavailable</text>
  <text x="311" y="38" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">&#8593; the only work you get to do</text>
  <text x="0" y="74" font-family="var(--font-mono)" font-size="10" fill="var(--muted)">TC0</text>
  <path d="M92 80 L116 80 L116 64 L120 64 L120 80 L144 80 L144 64 L148 64 L148 80 L172 80 L172 64 L176 64 L176 80 L200 80 L200 64 L204 64 L204 80 L228 80 L228 64 L232 64 L232 80 L256 80 L256 64 L260 64 L260 80 L284 80 L284 64 L288 64 L288 80 L312 80 L312 64 L316 64 L316 80 L340 80 L340 64 L344 64 L344 80 L368 80 L368 64 L372 64 L372 80 L396 80 L396 64 L400 64 L400 80 L424 80 L424 64 L428 64 L428 80 L452 80 L452 64 L456 64 L456 80 L480 80 L480 64 L484 64 L484 80 L508 80 L508 64 L512 64 L512 80 L530 80" fill="none" stroke="var(--ink)" stroke-width="1.5"/>
  <text x="92" y="96" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">OCF0A compare match every 1 ms &middot; peripheral, not code</text>
  <text x="0" y="130" font-family="var(--font-mono)" font-size="10" fill="var(--muted)">MAIN</text>
  <rect x="92" y="120" width="438" height="16" fill="var(--ink)" opacity="0.12"/>
  <text x="311" y="132" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">free the whole time</text>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="92" y="148">0</text><text x="530" y="148" text-anchor="end">15 ms</text>
  </g>
</svg>
<figcaption>Blocking spends the core to measure time. A timer spends a peripheral instead. The compare match is generated in hardware, so ISR duration does not move the tick.</figcaption>
</figure>

Timer/Counter 0 in CTC mode — Clear Timer on Compare Match, waveform generation
mode 2 — counts up to `OCR0A`, raises `OCF0A`, and resets to zero. Overflow mode
would give a fixed 256 counts and no choice of period; CTC is the mode that lets
you pick.

Here is the arithmetic as I first wrote it, and it is wrong:

```
16 MHz ÷ 64 (prescaler) ÷ 250 (counts) = 1000 Hz = 1 ms      ✓ correct
const TIMER_COUNTS: u32 = 250;
tc0.ocr0a().write(|w| w.set(TIMER_COUNTS as u8));            ✗ off by one
```

CTC counts **through zero**. The sequence is `0, 1, … OCR0A`, then reset — which
is `OCR0A + 1` counts, not `OCR0A`. The divisor 250 is right; the register value
for it is 249. Write 250 and the timer counts 251:

```
16 MHz ÷ 64 ÷ 251 = 996.016 Hz   →  1.004016 ms per "millisecond"
error = 1 − 996.016/1000         →  0.398 %  →  239 ms slow per minute
```

The constant stays the divisor; the register gets one less than it:

```rust
tc0.ocr0a().write(|w| w.set((TIMER_COUNTS - 1) as u8));   // 249, for 250 counts
```

I like this bug more than the data race. It is one character. It lives in a line
that reads correctly out loud, under a comment that is genuinely true. It
survives every test that asserts the counter increments, because the counter
does increment. And 0.4 % is close enough to plausible clock error that my own
lab manual's expected-results table waves at it and calls it *crystal-limited* —
which cannot be right, because an Uno R3's 16 MHz crystal is a ±50 ppm part.
That is 0.005 %. For the manual's explanation to hold, the crystal would have to
be off by eighty times its own tolerance.

The only instrument that catches this is one outside the chip: a stopwatch
against sixty seconds of serial output.

## Reading the four registers

Configuring a peripheral on this chip means writing bytes to memory-mapped I/O
registers, where the datasheet assigns a meaning to each of the eight bits.
Timer 0 needs four of them:

- **`TCCR0A`** and **`TCCR0B`** — Timer/Counter Control Register A and B. Split
  across two bytes for historical reasons, not logical ones. Between them they
  hold the waveform-generation mode (`WGM02:WGM01:WGM00`) and the clock source
  and prescaler (`CS02:CS01:CS00`).
- **`TIMSK0`** — Timer Interrupt Mask Register. One bit per timer event,
  deciding which of them are allowed to raise an interrupt at all.
- **`OCR0A`** — Output Compare Register A. Not a bit field: an 8-bit *value*, the
  number the counter is compared against. This is the one holding the off-by-one.

The convention in the strip below is the datasheet's own. Bit 7 is on the left,
bit 0 on the right. A filled cell is a bit I set to 1; a named unfilled cell is
a bit that exists and stays 0; a dash is reserved and must stay 0. The byte on
the right is what actually lands in the register.

<figure class="diagram">
<svg viewBox="0 0 620 250" role="img" aria-label="Bit layout of the four Timer 0 registers. TCCR0A bit 1 WGM01 set, giving 0x02. TCCR0B bits 1 and 0, CS01 and CS00, set, giving 0x03. TIMSK0 bit 1 OCIE0A set, giving 0x02. OCR0A holds the value 249, shown as a number rather than a bit field.">
  <g font-family="var(--font-mono)" font-size="8" fill="var(--muted)" text-anchor="middle">
    <text x="98" y="10">7</text><text x="156" y="10">6</text><text x="214" y="10">5</text><text x="272" y="10">4</text>
    <text x="330" y="10">3</text><text x="388" y="10">2</text><text x="446" y="10">1</text><text x="504" y="10">0</text>
  </g>
  <g font-family="var(--font-mono)" font-size="8">
    <text x="0" y="30" font-size="10" fill="var(--ink)">TCCR0A</text>
    <rect x="69" y="16" width="58" height="20" fill="none" stroke="var(--line)"/><text x="98" y="30" fill="var(--muted)" text-anchor="middle">COM0A1</text>
    <rect x="127" y="16" width="58" height="20" fill="none" stroke="var(--line)"/><text x="156" y="30" fill="var(--muted)" text-anchor="middle">COM0A0</text>
    <rect x="185" y="16" width="58" height="20" fill="none" stroke="var(--line)"/><text x="214" y="30" fill="var(--muted)" text-anchor="middle">COM0B1</text>
    <rect x="243" y="16" width="58" height="20" fill="none" stroke="var(--line)"/><text x="272" y="30" fill="var(--muted)" text-anchor="middle">COM0B0</text>
    <rect x="301" y="16" width="58" height="20" fill="none" stroke="var(--line)"/><text x="330" y="30" fill="var(--line)" text-anchor="middle">&mdash;</text>
    <rect x="359" y="16" width="58" height="20" fill="none" stroke="var(--line)"/><text x="388" y="30" fill="var(--line)" text-anchor="middle">&mdash;</text>
    <rect x="417" y="16" width="58" height="20" fill="#ff3b00"/><text x="446" y="30" fill="#f2f0e9" text-anchor="middle">WGM01</text>
    <rect x="475" y="16" width="58" height="20" fill="none" stroke="var(--line)"/><text x="504" y="30" fill="var(--muted)" text-anchor="middle">WGM00</text>
    <text x="543" y="30" fill="var(--ink)" font-size="9">= 0x02</text>
    <text x="69" y="48" fill="var(--muted)">waveform mode &rarr; CTC (mode 2): count to OCR0A, then clear</text>
    <text x="0" y="78" font-size="10" fill="var(--ink)">TCCR0B</text>
    <rect x="69" y="64" width="58" height="20" fill="none" stroke="var(--line)"/><text x="98" y="78" fill="var(--muted)" text-anchor="middle">FOC0A</text>
    <rect x="127" y="64" width="58" height="20" fill="none" stroke="var(--line)"/><text x="156" y="78" fill="var(--muted)" text-anchor="middle">FOC0B</text>
    <rect x="185" y="64" width="58" height="20" fill="none" stroke="var(--line)"/><text x="214" y="78" fill="var(--line)" text-anchor="middle">&mdash;</text>
    <rect x="243" y="64" width="58" height="20" fill="none" stroke="var(--line)"/><text x="272" y="78" fill="var(--line)" text-anchor="middle">&mdash;</text>
    <rect x="301" y="64" width="58" height="20" fill="none" stroke="var(--line)"/><text x="330" y="78" fill="var(--muted)" text-anchor="middle">WGM02</text>
    <rect x="359" y="64" width="58" height="20" fill="none" stroke="var(--line)"/><text x="388" y="78" fill="var(--muted)" text-anchor="middle">CS02</text>
    <rect x="417" y="64" width="58" height="20" fill="#ff3b00"/><text x="446" y="78" fill="#f2f0e9" text-anchor="middle">CS01</text>
    <rect x="475" y="64" width="58" height="20" fill="#ff3b00"/><text x="504" y="78" fill="#f2f0e9" text-anchor="middle">CS00</text>
    <text x="543" y="78" fill="var(--ink)" font-size="9">= 0x03</text>
    <text x="69" y="96" fill="var(--muted)">clock select &rarr; CS02:CS01:CS00 = 011 = clk/64 &middot; also starts the timer</text>
    <text x="0" y="126" font-size="10" fill="var(--ink)">TIMSK0</text>
    <rect x="69" y="112" width="58" height="20" fill="none" stroke="var(--line)"/><text x="98" y="126" fill="var(--line)" text-anchor="middle">&mdash;</text>
    <rect x="127" y="112" width="58" height="20" fill="none" stroke="var(--line)"/><text x="156" y="126" fill="var(--line)" text-anchor="middle">&mdash;</text>
    <rect x="185" y="112" width="58" height="20" fill="none" stroke="var(--line)"/><text x="214" y="126" fill="var(--line)" text-anchor="middle">&mdash;</text>
    <rect x="243" y="112" width="58" height="20" fill="none" stroke="var(--line)"/><text x="272" y="126" fill="var(--line)" text-anchor="middle">&mdash;</text>
    <rect x="301" y="112" width="58" height="20" fill="none" stroke="var(--line)"/><text x="330" y="126" fill="var(--line)" text-anchor="middle">&mdash;</text>
    <rect x="359" y="112" width="58" height="20" fill="none" stroke="var(--line)"/><text x="388" y="126" fill="var(--muted)" text-anchor="middle">OCIE0B</text>
    <rect x="417" y="112" width="58" height="20" fill="#ff3b00"/><text x="446" y="126" fill="#f2f0e9" text-anchor="middle">OCIE0A</text>
    <rect x="475" y="112" width="58" height="20" fill="none" stroke="var(--line)"/><text x="504" y="126" fill="var(--muted)" text-anchor="middle">TOIE0</text>
    <text x="543" y="126" fill="var(--ink)" font-size="9">= 0x02</text>
    <text x="69" y="144" fill="var(--muted)">interrupt mask &rarr; let compare-match A reach the TIMER0_COMPA vector</text>
  </g>
  <line x1="0" y1="164" x2="620" y2="164" stroke="var(--line)"/>
  <text x="0" y="192" font-family="var(--font-mono)" font-size="10" fill="var(--ink)">OCR0A</text>
  <rect x="69" y="178" width="464" height="22" fill="none" stroke="#ff3b00"/>
  <text x="301" y="193" font-family="var(--font-mono)" font-size="11" fill="#ff3b00" text-anchor="middle">249</text>
  <text x="543" y="193" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">= 0xF9</text>
  <g font-family="var(--font-mono)" font-size="8" fill="var(--muted)">
    <text x="69" y="214">not a bit field &mdash; an 8-bit number the counter is compared against</text>
    <text x="69" y="230" fill="#ff3b00">TOP is inclusive: the counter visits 0..=249, which is 250 counts</text>
    <text x="69" y="246">write 250 here and you get 251 counts, and a clock 0.4 % slow</text>
  </g>
</svg>
<figcaption>Four bits and one byte, across four registers. The bits are the easy part — each one is a named flag the datasheet spells out, and getting one wrong usually means nothing works at all. The byte is the dangerous part: every value in it is legal, so a wrong one produces a timer that runs perfectly at the wrong speed.</figcaption>
</figure>

There is a second constant worth a compile-time assertion, because its failure
mode is also silent:

```rust
const MILLIS_INCREMENT: u32 = PRESCALER * TIMER_COUNTS / 16_000;
const _: () = assert!(MILLIS_INCREMENT > 0);
```

With the 64/250 pair this is exactly 1. With other perfectly reasonable prescaler
choices the integer division truncates to zero and time simply never advances,
while every register in the diagram above is correct.

## What claiming TC0 costs the rest of the system

Worth stating early because it constrains twelve later labs: on an ATmega328P
there are three timers, and `millis()` has just taken one.

<figure class="diagram">
<svg viewBox="0 0 620 152" role="img" aria-label="Timer allocation across the curriculum: TC0 claimed by millis, TC1 reserved for servo output, TC2 available for PWM experiments">
  <g font-family="var(--font-mono)" font-size="9">
    <text x="0" y="24" font-size="10" fill="var(--muted)">TC0</text>
    <rect x="92" y="10" width="330" height="22" fill="#ff3b00"/>
    <text x="257" y="25" fill="#f2f0e9" text-anchor="middle">millis() &middot; CTC &middot; 1 kHz &middot; claimed lab 4</text>
    <text x="432" y="25" fill="var(--muted)">collateral: D5 / D6 PWM gone</text>
    <text x="0" y="66" font-size="10" fill="var(--muted)">TC1</text>
    <rect x="92" y="52" width="330" height="22" fill="none" stroke="var(--line)"/>
    <text x="257" y="67" fill="var(--ink)" text-anchor="middle">reserved &middot; servo, Fast PWM mode 14</text>
    <text x="432" y="67" fill="var(--muted)">ICR1 = 39999, clk/8 &rarr; 50 Hz</text>
    <text x="0" y="108" font-size="10" fill="var(--muted)">TC2</text>
    <rect x="92" y="94" width="330" height="22" fill="none" stroke="var(--line)"/>
    <text x="257" y="109" fill="var(--ink)" text-anchor="middle">free &middot; PWM experiments on D3</text>
    <text x="432" y="109" fill="var(--muted)">use this, not D9</text>
  </g>
  <line x1="0" y1="130" x2="620" y2="130" stroke="var(--line)"/>
  <text x="0" y="146" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">three timers, thirteen labs &mdash; allocation is a design decision made once, in week 4</text>
</svg>
<figcaption>The reason TC1 is ring-fenced: the HAL's <code>simple_pwm</code> exposes fixed prescalers landing near 30, 122 and 490 Hz. None of those is the 50 Hz a hobby servo needs, so the servo lab has to drive TC1 raw — and it cannot do that if an earlier lab spent it.</figcaption>
</figure>

## The version everyone writes first

```rust
static mut COUNTER: u32 = 0;

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() { unsafe { COUNTER += 1; } }

fn millis() -> u32 { unsafe { COUNTER } }
```

The objection writes itself, and students make it every time: single core, no
threads, the handler is a handful of instructions. What is there to interleave?

The four loads. On AVR a `u32` return value comes back in `r22:r25`, and filling
that quad is four `lds` — two words and two cycles each:

```
lds  r22, COUNTER
lds  r23, COUNTER+1     ; an interrupt taken here sees a half-read value
lds  r24, COUNTER+2
lds  r25, COUNTER+3
```

That listing is what the calling convention and the instruction set require, not
a disassembly of a build I have in front of me. What a given build emits is
LLVM's decision: the four loads may survive, or the whole read may be lifted out
of a loop and never repeated. The source does not constrain the choice, and that
is the more important half of the argument.

## What 511 looks like

"The value is torn" is a phrase, not a number. Take the counter crossing its
first byte boundary — 255 to 256, `0x0000_00FF` to `0x0000_0100`, where both
low bytes change — and let the compare match land mid-read.

<figure class="diagram">
<svg viewBox="0 0 620 216" role="img" aria-label="Two timelines of a four-byte read interrupted mid-sequence: reading low byte first yields 511, reading high byte first yields 0, while the true value is 255 or 256">
  <text x="0" y="14" font-family="var(--font-mono)" font-size="10" fill="var(--ink)">COUNTER: 0x0000_00FF &#8594; 0x0000_0100 &nbsp;&nbsp; (255 &#8594; 256)</text>
  <g font-family="var(--font-mono)" font-size="9">
    <text x="0" y="46" font-size="10" fill="var(--muted)">LOW FIRST</text>
    <rect x="92" y="32" width="96" height="22" fill="none" stroke="var(--line)"/><text x="140" y="47" fill="var(--ink)" text-anchor="middle">b0 = FF</text>
    <rect x="192" y="32" width="96" height="22" fill="#ff3b00"/><text x="240" y="47" fill="#f2f0e9" text-anchor="middle">ISR ++</text>
    <rect x="292" y="32" width="96" height="22" fill="none" stroke="var(--line)"/><text x="340" y="47" fill="var(--ink)" text-anchor="middle">b1 = 01</text>
    <rect x="392" y="32" width="96" height="22" fill="none" stroke="var(--line)"/><text x="440" y="47" fill="var(--ink)" text-anchor="middle">b2 = 00</text>
    <rect x="492" y="32" width="96" height="22" fill="none" stroke="var(--line)"/><text x="540" y="47" fill="var(--ink)" text-anchor="middle">b3 = 00</text>
    <text x="92" y="70" fill="var(--muted)">assembled &#8594; 0x0000_01FF</text>
    <text x="588" y="70" fill="#ff3b00" text-anchor="end" font-size="11">511</text>
    <text x="0" y="122" font-size="10" fill="var(--muted)">HIGH FIRST</text>
    <rect x="92" y="108" width="96" height="22" fill="none" stroke="var(--line)"/><text x="140" y="123" fill="var(--ink)" text-anchor="middle">b3 = 00</text>
    <rect x="192" y="108" width="96" height="22" fill="none" stroke="var(--line)"/><text x="240" y="123" fill="var(--ink)" text-anchor="middle">b2 = 00</text>
    <rect x="292" y="108" width="96" height="22" fill="none" stroke="var(--line)"/><text x="340" y="123" fill="var(--ink)" text-anchor="middle">b1 = 00</text>
    <rect x="392" y="108" width="96" height="22" fill="#ff3b00"/><text x="440" y="123" fill="#f2f0e9" text-anchor="middle">ISR ++</text>
    <rect x="492" y="108" width="96" height="22" fill="none" stroke="var(--line)"/><text x="540" y="123" fill="var(--ink)" text-anchor="middle">b0 = 00</text>
    <text x="92" y="146" fill="var(--muted)">assembled &#8594; 0x0000_0000</text>
    <text x="588" y="146" fill="#ff3b00" text-anchor="end" font-size="11">0</text>
  </g>
  <line x1="0" y1="168" x2="620" y2="168" stroke="var(--line)"/>
  <text x="0" y="190" font-family="var(--font-mono)" font-size="10" fill="var(--muted)">TRUE VALUE</text>
  <text x="140" y="190" font-family="var(--font-mono)" font-size="10" fill="var(--ink)">255 before the tick &nbsp;&middot;&nbsp; 256 after it</text>
  <text x="0" y="210" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">load order is the compiler's choice, and you did not specify it</text>
</svg>
<figcaption>Neither result is a rounding error. 511 is a quarter of a second in the future; 0 is the machine claiming it just booted. Both are assembled entirely from bytes that were individually correct at the moment they were read.</figcaption>
</figure>

A `millis()` returning 511 does not present as a memory bug. It presents as a
scheduling bug in whatever code compared it against a deadline, three modules
away, and that is where the engineer will spend the afternoon.

## Why your tests will never see it

Two independent conditions have to coincide, and multiplying them out explains
the failure distribution exactly.

<figure class="diagram">
<svg viewBox="0 0 620 132" role="img" aria-label="Two conditions in series: the interrupt must land inside the 500 nanosecond read window, one chance in 2000, and that tick must carry into a higher byte, one chance in 256, giving one wrong read in 512,000">
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="0" y="18" width="128" height="44" fill="none" stroke="var(--line)"/>
    <text x="64" y="38" fill="var(--ink)" text-anchor="middle">every</text>
    <text x="64" y="52" fill="var(--ink)" text-anchor="middle">millis() read</text>
    <path d="M136 40 L164 40" stroke="var(--muted)" stroke-width="1"/>
    <path d="M158 36 L164 40 L158 44" fill="none" stroke="var(--muted)"/>
    <rect x="172" y="10" width="164" height="60" fill="none" stroke="var(--line)"/>
    <text x="254" y="28" fill="var(--muted)" text-anchor="middle">the tick lands inside</text>
    <text x="254" y="42" fill="var(--ink)" text-anchor="middle">the ~500 ns window</text>
    <text x="254" y="60" fill="#ff3b00" text-anchor="middle">1 in 2000</text>
    <path d="M344 40 L372 40" stroke="var(--muted)" stroke-width="1"/>
    <path d="M366 36 L372 40 L366 44" fill="none" stroke="var(--muted)"/>
    <rect x="380" y="10" width="164" height="60" fill="none" stroke="var(--line)"/>
    <text x="462" y="28" fill="var(--muted)" text-anchor="middle">and that tick carries</text>
    <text x="462" y="42" fill="var(--ink)" text-anchor="middle">into a higher byte</text>
    <text x="462" y="60" fill="#ff3b00" text-anchor="middle">1 in 256</text>
  </g>
  <line x1="0" y1="92" x2="620" y2="92" stroke="var(--line)"/>
  <text x="0" y="114" font-family="var(--font-mono)" font-size="11" fill="#ff3b00">1 wrong read in 512,000</text>
  <text x="0" y="128" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">model &mdash; 8 cycles &divide; 1 ms tick, times 1 &divide; 256. Miss the second gate and the read is merely stale by one.</text>
</svg>
<figcaption>The severity gate is the interesting one. A tick landing mid-read that only touches the low byte yields a value one millisecond out of date — invisible, harmless, and overwhelmingly the common case. The carry is what turns a stale read into a false one.</figcaption>
</figure>

Three caveats a reviewer should press on, because the model is not the board:

**The window is not exactly 8 cycles.** Tearing requires the tick to fall
*between* the first and last load, which is three inter-instruction boundaries,
not four. Call it 6–8 cycles and treat the 1-in-2000 as an order of magnitude.

**Uniform phase is an assumption, and often a false one.** The derivation
assumes reads are uniformly distributed against the tick. A polling loop whose
period is correlated with the tick — which is exactly what a `millis()`-driven
scheduler is — can push the true rate to zero or to one. Phase-locking is why
this bug reproduces on one build and vanishes on the next.

**The rate is not a property of this module.** One read in 512,000 becomes once
a fortnight or once an hour depending on how often the caller asks. The failure
frequency is set by code that has never read this file.

And then the honest part: when I build
the `static mut` version to watch it fail, I expect it not to. At `opt-level =
"s"` on a current toolchain the loads may well come out intact and the counter
may read correctly for as long as anyone is willing to watch. The incorrect
program passes.

## The race is not the bug

That expected non-failure is the important part of the lab, and it is where a
hardware-only explanation stops being sufficient.

Two contexts accessing the same non-atomic location without synchronisation,
where at least one writes, is a data race, and a data race is undefined
behaviour in Rust. Not "risky", not "may tear" — undefined. It does not become
undefined at the moment two accesses happen to overlap. It is undefined as
written, and the optimiser has already made decisions on the assumption that it
never happens.

Two precisions that are easy to get wrong, and that I had wrong in my own
instructor notes:

**It is not about `&mut`.** The usual formulation — "taking `&mut` to a `static
mut` violates uniqueness" — does not describe this code, which never forms a
reference. `COUNTER += 1` and a by-value read are place accesses. Rust 2024's
`static_mut_refs` error will not save you here, because there is no reference
for it to reject. The UB is the unsynchronised access itself.

**The mechanism is ordinary optimisation, not malice.** The load is neither
`volatile` nor atomic, so loop-invariant code motion is entitled to hoist it:

<figure class="diagram">
<svg viewBox="0 0 620 168" role="img" aria-label="Side by side comparison: source code polling millis in a loop, versus emitted code that loads the counter once before the loop and compares the same stale register forever">
  <text x="0" y="14" font-family="var(--font-mono)" font-size="10" fill="var(--muted)">WHAT YOU WROTE</text>
  <text x="316" y="14" font-family="var(--font-mono)" font-size="10" fill="var(--muted)">WHAT LICM MAY EMIT</text>
  <line x1="304" y1="24" x2="304" y2="152" stroke="var(--line)"/>
  <g font-family="var(--font-mono)" font-size="10" fill="var(--ink)">
    <text x="0" y="46">let end = millis() + 500;</text>
    <text x="0" y="66">while millis() &lt; end {</text>
    <text x="0" y="86">&nbsp;&nbsp;&nbsp;&nbsp;poll_sensors();</text>
    <text x="0" y="106">}</text>
  </g>
  <g font-family="var(--font-mono)" font-size="10">
    <text x="316" y="46" fill="#ff3b00">lds r22, COUNTER &nbsp;; once</text>
    <text x="316" y="66" fill="var(--ink)">.loop:</text>
    <text x="316" y="86" fill="var(--ink)">&nbsp;&nbsp;call poll_sensors</text>
    <text x="316" y="106" fill="var(--ink)">&nbsp;&nbsp;cp &nbsp; r22, r20</text>
    <text x="316" y="126" fill="var(--ink)">&nbsp;&nbsp;brlo .loop</text>
  </g>
  <line x1="0" y1="152" x2="620" y2="152" stroke="var(--line)"/>
  <text x="0" y="166" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the load is hoisted out of the loop, the comparison never changes, the loop never exits</text>
</svg>
<figcaption>Legal, because the language told LLVM nothing else writes there. Note that this failure mode contains no torn value at all — the loop simply hangs, on a board with no debugger attached, in a build that worked yesterday.</figcaption>
</figure>

So the argument for the critical section is not that the race is probable. It is
that the miscompilation is licensed regardless of probability, and licensed by
the type system rather than by the hardware. That distinction is the difference
between a student who avoids `static mut` because they were told to and one who
can say what the compiler is promising in exchange.

## The fix, and what each token costs

```rust
use core::cell::Cell;
use avr_device::interrupt::{self, Mutex};

static MILLIS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    interrupt::free(|cs| {
        let c = MILLIS.borrow(cs);
        c.set(c.get().wrapping_add(MILLIS_INCREMENT));
    })
}

fn millis() -> u32 {
    interrupt::free(|cs| MILLIS.borrow(cs).get())
}
```

`Mutex` is an actively misleading name here. It never blocks and there is no
lock. It is a token system: `interrupt::free` disables interrupts and hands the
closure a `CriticalSection`, and `Mutex::borrow` demands one. You cannot reach
the value without evidence, checked at compile time, that interrupts are off.
Runtime cost `cli` + `sei`; compile-time cost a proof.

Five details that matter more than the shape of the code.

**`Cell`, not `RefCell`.** `Cell` is get/set on a `Copy` type with no runtime
borrow flag — zero bytes of state and no panic path. `RefCell` buys dynamic
borrow checking you do not need here and costs a byte plus a branch. Reach for
it only when the payload is not `Copy`.

**`interrupt::free` restores, it does not enable.** The implementation saves
`SREG`, clears the I-bit, runs the closure, and writes `SREG` back. It does not
blindly `sei` on exit, which is what makes it compose: nesting two of them
leaves interrupts disabled at the inner exit, as it must.

**Which means the one in the handler is redundant.** AVR clears the I-bit in
hardware on interrupt entry, so interrupts are already off inside
`TIMER0_COMPA`. The `interrupt::free` there is correct but is buying a guarantee
the silicon already gave. I keep it for call-site symmetry, and that is only a
defensible trade if the handler has margin against its deadline — which is a
claim requiring a measurement, not an opinion.

**There is no `AtomicU32`.** AVR has no atomic read-modify-write instruction and
the target exposes no wide atomics. `portable-atomic` will give you the type,
and implements it by taking a critical section — the same `cli`/`sei`, one
abstraction further away. On an 8-bit machine the critical section is not the
fallback; it is the mechanism.

**Narrowing the type does not remove the requirement.** A `u16` is still two
`lds` and still tears. Only `u8` is a single-instruction access — and even then
the abstract-machine argument stands unchanged, because unsynchronised access is
UB whether or not the hardware could have torn it.

## The other bug, which is not a concurrency bug at all

Everything above concerns *writing* the counter safely. This section is about
reading it, and it is where I found the second defect in my own lab solution.
Here is the main loop as it ships:

```rust
let mut next = 1000u32;
loop {
    if millis() >= next {
        ufmt::uwriteln!(&mut serial, "{}", next).ok();
        next += 1000;
    }
}
```

Correct for 49 days, 17 hours, 2 minutes and 47.296 seconds. Then it isn't.

### Ordering is not defined on a ring

`millis()` returns a `u32`, so its value space is the integers modulo 2³². The
counter does not run out at 4,294,967,295; it returns to zero and keeps going.
A monotonic quantity has been mapped onto a circle, and `>=` is an operator
that assumes a line.

<figure class="diagram">
<svg viewBox="0 0 620 246" role="img" aria-label="The u32 counter drawn as a ring. Two values, next at 4294967200 and now at 100, sit 196 milliseconds apart across the wrap point, yet the comparison now is greater than or equal to next evaluates to false.">
  <circle cx="150" cy="126" r="82" fill="none" stroke="var(--line)" stroke-width="1"/>
  <path d="M223.5 64.3 A96 96 0 0 1 233.1 174" fill="none" stroke="var(--muted)" stroke-width="1.5"/>
  <path d="M230.1 179.2 L230.7 169.2 L238.5 173.7 Z" fill="var(--muted)"/>
  <text x="150" y="152" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">counts</text>
  <text x="150" y="164" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">clockwise</text>
  <line x1="150" y1="38" x2="150" y2="60" stroke="#ff3b00" stroke-width="1.5"/>
  <text x="150" y="76" font-family="var(--font-mono)" font-size="9" fill="#ff3b00" text-anchor="middle">wrap &middot; 0</text>
  <circle cx="130" cy="46" r="3.5" fill="var(--ink)"/>
  <circle cx="170" cy="46" r="3.5" fill="var(--ink)"/>
  <path d="M130 46 L96 20" fill="none" stroke="var(--line)"/>
  <path d="M170 46 L204 20" fill="none" stroke="var(--line)"/>
  <text x="93" y="17" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="end">next = 4294967200</text>
  <text x="207" y="17" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">now = 100</text>
  <text x="150" y="228" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">196 ms apart on the ring</text>
  <line x1="272" y1="14" x2="272" y2="232" stroke="var(--line)"/>
  <text x="300" y="34" font-family="var(--font-mono)" font-size="10" fill="var(--muted)">THE NAIVE COMPARISON</text>
  <g font-family="var(--font-mono)" font-size="10" fill="var(--ink)">
    <text x="300" y="58">now &gt;= next</text>
    <text x="300" y="76">100 &gt;= 4294967200</text>
  </g>
  <text x="300" y="96" font-family="var(--font-mono)" font-size="11" fill="#ff3b00">false &mdash; and false for 49.7 days</text>
  <line x1="300" y1="116" x2="608" y2="116" stroke="var(--line)"/>
  <text x="300" y="140" font-family="var(--font-mono)" font-size="10" fill="var(--muted)">THE MODULAR ONE</text>
  <g font-family="var(--font-mono)" font-size="10" fill="var(--ink)">
    <text x="300" y="164">now.wrapping_sub(next)</text>
    <text x="300" y="182">100 &minus; 4294967200 mod 2&#179;&#178;</text>
  </g>
  <text x="300" y="202" font-family="var(--font-mono)" font-size="11" fill="var(--ink)">= 196 &mdash; the true elapsed time</text>
  <text x="300" y="224" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the wrap in the operand and the wrap in the</text>
  <text x="300" y="236" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">subtraction cancel exactly</text>
</svg>
<figcaption>Two instants 196 ms apart. As integers they are as far apart as two <code>u32</code>s can be, and every absolute comparison between them is meaningless. The subtraction is not a trick — modular arithmetic is what the hardware was doing all along, and <code>wrapping_sub</code> is the operator that admits it.</figcaption>
</figure>

That cancellation is the whole idea and it is worth doing once by hand. The
mathematical difference is −4,294,967,100, which is not representable; adding
2³² brings it back to 196, and that is exactly what wrapping subtraction
computes. As long as the *real* elapsed time is less than 2³² ms, the modular
difference **is** the true difference. No approximation.

### Which way it fails depends on who wraps first

The naive form has two distinct failure modes, and which one you get depends on
whether the deadline or the clock crosses zero first.

<figure class="diagram">
<svg viewBox="0 0 620 200" role="img" aria-label="Two failure timelines. When the deadline wraps first the scheduler fires continuously in a burst; when the clock wraps first the scheduler goes silent for 49.7 days.">
  <line x1="300" y1="14" x2="300" y2="180" stroke="#ff3b00" stroke-width="1" stroke-dasharray="3 3"/>
  <text x="306" y="12" font-family="var(--font-mono)" font-size="9" fill="#ff3b00">wrap</text>
  <text x="0" y="34" font-family="var(--font-mono)" font-size="10" fill="var(--muted)">DEADLINE FIRST</text>
  <line x1="92" y1="52" x2="600" y2="52" stroke="var(--ink)" stroke-width="1"/>
  <g stroke="var(--ink)" stroke-width="1.5">
    <path d="M120 52 L120 40 M160 52 L160 40 M200 52 L200 40 M240 52 L240 40 M280 52 L280 40"/>
  </g>
  <g stroke="#ff3b00" stroke-width="1.5">
    <path d="M304 52 L304 36 M310 52 L310 36 M316 52 L316 36 M322 52 L322 36 M328 52 L328 36 M334 52 L334 36 M340 52 L340 36 M346 52 L346 36 M352 52 L352 36 M358 52 L358 36 M364 52 L364 36 M370 52 L370 36 M376 52 L376 36 M382 52 L382 36 M388 52 L388 36 M394 52 L394 36 M400 52 L400 36 M406 52 L406 36 M412 52 L412 36 M418 52 L418 36"/>
  </g>
  <text x="92" y="70" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">next += 1000 overflows &rarr; next is now tiny, now is still huge</text>
  <text x="92" y="84" font-family="var(--font-mono)" font-size="9" fill="#ff3b00">fires every single loop iteration until next climbs back &mdash; ~4.3 M spurious writes</text>
  <line x1="0" y1="104" x2="620" y2="104" stroke="var(--line)"/>
  <text x="0" y="130" font-family="var(--font-mono)" font-size="10" fill="var(--muted)">CLOCK FIRST</text>
  <line x1="92" y1="148" x2="600" y2="148" stroke="var(--ink)" stroke-width="1"/>
  <g stroke="var(--ink)" stroke-width="1.5">
    <path d="M120 148 L120 136 M160 148 L160 136 M200 148 L200 136 M240 148 L240 136 M280 148 L280 136"/>
  </g>
  <rect x="300" y="140" width="300" height="8" fill="#ff3b00" opacity="0.22"/>
  <text x="92" y="166" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">now wraps to 0 while next is still near the top of the range</text>
  <text x="92" y="180" font-family="var(--font-mono)" font-size="9" fill="#ff3b00">nothing fires at all for 49 d 17 h &mdash; no error, no log, no symptom but silence</text>
</svg>
<figcaption>A periodic scheduler always holds a deadline ahead of the clock, so it hits the top case: a storm, not a stall. A one-shot timeout armed just before the boundary can hit either. Neither is a crash, which is what makes both expensive to diagnose.</figcaption>
</figure>

There is a third outcome that depends on your profile. `next += 1000` on an
overflow is a panic in debug builds, and with `panic_halt` linked that is a board
which simply stops. In release, with `overflow-checks = false`, it wraps
silently. The same source line is a freeze or a storm depending on a flag in
`Cargo.toml`.

### The two correct forms

They are not interchangeable — one is for periodic work, one for deadlines.

```rust
// Periodic: has a full interval elapsed since the last run?
if now.wrapping_sub(last) >= INTERVAL_MS {
    last = last.wrapping_add(INTERVAL_MS);   // not `last = now`
    do_work();
}

// One-shot deadline: is `deadline` now in the past?
if (now.wrapping_sub(deadline) as i32) >= 0 {
    fire();
}
```

The `last = last.wrapping_add(INTERVAL_MS)` detail matters independently of
rollover. Writing `last = now` folds each iteration's latency into the next
period, so the schedule drifts by however late you were, permanently and
cumulatively. Advancing by the interval keeps the average period exact even when
individual firings are late. Both lines look correct; only one is a clock.

The signed cast in the second form deserves a sentence, because it looks like a
hack and is not. Reinterpreting the modular difference as `i32` splits the ring
into "recently past" (small positive) and "soon future" (large unsigned, which
reads as negative). It is a *half-range* test:

<figure class="diagram">
<svg viewBox="0 0 620 150" role="img" aria-label="The 32-bit difference space split at 2 to the 31. Differences below the midpoint mean the deadline has passed; above it means the deadline is in the future. The usable window is 24.85 days in each direction.">
  <text x="0" y="16" font-family="var(--font-mono)" font-size="10" fill="var(--muted)">now.wrapping_sub(deadline)</text>
  <rect x="92" y="30" width="254" height="26" fill="#ff3b00" opacity="0.18"/>
  <rect x="346" y="30" width="254" height="26" fill="none" stroke="var(--line)"/>
  <line x1="346" y1="24" x2="346" y2="62" stroke="var(--ink)" stroke-width="1.5"/>
  <text x="219" y="47" font-family="var(--font-mono)" font-size="9" fill="var(--ink)" text-anchor="middle">reads as i32 &ge; 0 &mdash; deadline has passed</text>
  <text x="473" y="47" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">reads as i32 &lt; 0 &mdash; still in the future</text>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="92" y="74">0</text>
    <text x="346" y="74" text-anchor="middle">2&#179;&#185;</text>
    <text x="600" y="74" text-anchor="end">2&#179;&#178;</text>
    <text x="219" y="90" text-anchor="middle">up to 24.85 days late</text>
    <text x="473" y="90" text-anchor="middle">up to 24.85 days early</text>
  </g>
  <line x1="0" y1="108" x2="620" y2="108" stroke="var(--line)"/>
  <text x="0" y="128" font-family="var(--font-mono)" font-size="9" fill="var(--ink)">the contract: you must sample often enough to never be &gt; 24.85 days late</text>
  <text x="0" y="142" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">break it and the test silently inverts &mdash; a passed deadline reads as a future one</text>
</svg>
<figcaption>The trick buys rollover-correctness at the price of a bounded lateness assumption. For a 1 ms scheduler polled in a tight loop that bound is absurdly generous; for a task that sleeps for weeks it is a real constraint, and it should be written down rather than assumed.</figcaption>
</figure>

My own instructor manual writes this as `diff < u32::MAX / 2`, which is the same
test off by a single millisecond every 24.85 days — `u32::MAX / 2` is
`i32::MAX`, and the `<` excludes the one value the signed form includes. It has
never mattered and never will. I mention it because the alternative is a
curriculum that teaches an idiom nobody has checked.

### Why it never shows up in testing

Because the trigger is a *duration*, and durations are the one input a test suite
cannot fake. There is no argument to pass, no boundary value to fuzz, no branch
to reach — you reach it by leaving the board powered for seven weeks.

The mitigation is to make the counter's origin a parameter. If `millis()` is
wrapped in a type you can seed, a unit test can start the clock at
`u32::MAX - 5000`, step it across the boundary, and assert the scheduler still
fires. That test runs in microseconds and pins the exact behaviour that fifty
days of uptime would otherwise be needed to observe.

Two documented instances of exactly this arithmetic, at opposite ends of the
consequence scale:

**Windows 95 hung after 49.7 days** of uptime — the same constant as this
article, because it is the same `u32` of milliseconds. Microsoft's advisory,
[KB 216641, "Computer Hangs After 49.7
Days"](https://www.betaarchive.com/wiki/index.php?title=Microsoft_KB_Archive/216641),
puts the cause in a timing algorithm in `Vtdapi.vxd`. Almost nobody hit it,
because almost nobody left a Windows 95 machine up for seven weeks.

**The Boeing 787 lost all AC power after 248 days.** [FAA Airworthiness
Directive
2015-09-07](https://www.federalregister.gov/documents/2015/05/01/2015-10066/airworthiness-directives-the-boeing-company-airplanes)
(docket FAA-2015-0936, effective 1 May 2015) states it plainly: an airplane
powered continuously for 248 days can lose all AC electrical power, because a
software counter internal to the generator control units overflows and all of
them drop into failsafe at once. The arithmetic that fits is a signed 32-bit
counter incremented at 100 Hz — 2³¹ centiseconds is 248.55 days. Until Boeing
shipped new GCU software, the mandated remedy was a repetitive maintenance task:
power the aircraft down before it got there.

Both are the defect at the top of this section. Neither was found by testing;
both were found in service, by the passage of time.

I have left the naive version in my lab-4 solution rather than quietly patching
it, because lab 6 introduces the wrapping form and its manual notes that *most
students write the naive comparison*. Mine did too. The honest fix is to teach
the diff at that point rather than to pretend the earlier code was always right.

## What the small machine is for

Three defects, and the one in the title is the one least likely to ever show
itself.

The off-by-one is deterministic. It reproduces on every run of every build, and
it still needs an instrument outside the chip to notice, because inside the chip
everything agrees with itself: a timer 0.4 % slow tells the same wrong time to
every line of code that asks.

The unsynchronised counter is not deterministic, and not really about tearing.
The torn read is its most visible symptom and I expect not to see it; the
hoisted load is the same defect with no wrong value in it anywhere. Neither can
be reproduced on demand, which is what undefined means.

The rollover comparison has no input a test can supply, because its trigger is a
duration. Windows 95 shipped it, a 787 shipped it, and so did my own solution
code.

None of the three is caught by testing the thing it breaks. Two need you to know
what the machine does between the instructions you wrote; the third, what its
arithmetic does when the numbers run out. The only way I know to acquire either
is to work somewhere small enough that "between the instructions" is a place you
can point at.

Two kilobytes of SRAM is not a limitation for this. It is the instrument.
