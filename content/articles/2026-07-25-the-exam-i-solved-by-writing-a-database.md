+++
title = "The Exam I Solved by Writing a Database"
date = "2026-07-25"
description = "Handed an assembly exam and not enough time, I did the thing that looks least rational under pressure: I stopped solving the problem and built an access layer first. It was faster. Years later, Rust on an 8-bit microcontroller made the same argument by force."
tags = ["Systems", "Rust", "Embedded"]
+++

The worst place to introduce an abstraction is an exam. There is a clock, there
is no second attempt, and every minute spent on structure is a minute not spent
on the thing being marked. The rational move is to write the shortest code that
produces the right answer and get out.

I did the opposite once, in an assembly exam, and it is still one of the more
useful things I have learned about software.

## The problem

The task was the usual shape: a block of memory holding records, and a set of
operations over them — insert, search, delete, report. Nothing conceptually
hard. In a language with structs it is twenty minutes of work.

In assembly there are no records. There is a base address, and there are
offsets, and the meaning of any given byte lives entirely in your head. Reading
a field is a computation: take the base, multiply the index by the record
stride, add the offset of the field you want, load. Writing it is the same
computation again, in reverse.

Do that inline, at every use site, under time pressure.

## What actually goes wrong

It is not that the arithmetic is difficult. It is that it is *repetitive and
silent*.

Every operation I wrote recomputed the same offsets by hand. Each one was an
opportunity to use the wrong stride, or the right stride with the wrong field
offset, or to clobber a register holding the base address three instructions
before I needed it again. None of those mistakes announce themselves. The
program runs. It produces a number. The number is wrong in a way that looks like
a logic error somewhere else entirely.

I lost time to exactly that, twice, before the pattern registered: I was not
running out of time because the problem was large. I was running out of time
because I was debugging the same class of mistake repeatedly, and each instance
looked new.

<figure class="diagram">
<svg viewBox="0 0 620 168" role="img" aria-label="Diagram contrasting inline offset arithmetic repeated at every use site with a single accessor layer that all operations call through">
  <text x="0" y="14" font-family="var(--font-mono)" font-size="10" fill="var(--muted)">INLINE</text>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="92" y="4" width="104" height="22" fill="none" stroke="var(--line)"/><text x="144" y="19" fill="var(--ink)" text-anchor="middle">insert</text>
    <rect x="204" y="4" width="104" height="22" fill="none" stroke="var(--line)"/><text x="256" y="19" fill="var(--ink)" text-anchor="middle">search</text>
    <rect x="316" y="4" width="104" height="22" fill="none" stroke="var(--line)"/><text x="368" y="19" fill="var(--ink)" text-anchor="middle">delete</text>
    <rect x="428" y="4" width="104" height="22" fill="none" stroke="var(--line)"/><text x="480" y="19" fill="var(--ink)" text-anchor="middle">report</text>
  </g>
  <g stroke="#ff3b00" stroke-width="1">
    <line x1="144" y1="26" x2="144" y2="52"/><line x1="256" y1="26" x2="256" y2="52"/>
    <line x1="368" y1="26" x2="368" y2="52"/><line x1="480" y1="26" x2="480" y2="52"/>
  </g>
  <g font-family="var(--font-mono)" font-size="8" fill="#ff3b00" text-anchor="middle">
    <text x="144" y="64">base+i*S+off</text><text x="256" y="64">base+i*S+off</text>
    <text x="368" y="64">base+i*S+off</text><text x="480" y="64">base+i*S+off</text>
  </g>
  <text x="92" y="82" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">four places to get it wrong, silently</text>
  <line x1="0" y1="100" x2="620" y2="100" stroke="var(--line)"/>
  <text x="0" y="122" font-family="var(--font-mono)" font-size="10" fill="var(--muted)">LAYERED</text>
  <g font-family="var(--font-mono)" font-size="9">
    <rect x="92" y="112" width="104" height="22" fill="none" stroke="var(--line)"/><text x="144" y="127" fill="var(--ink)" text-anchor="middle">insert</text>
    <rect x="204" y="112" width="104" height="22" fill="none" stroke="var(--line)"/><text x="256" y="127" fill="var(--ink)" text-anchor="middle">search</text>
    <rect x="316" y="112" width="104" height="22" fill="none" stroke="var(--line)"/><text x="368" y="127" fill="var(--ink)" text-anchor="middle">delete</text>
    <rect x="428" y="112" width="104" height="22" fill="none" stroke="var(--line)"/><text x="480" y="127" fill="var(--ink)" text-anchor="middle">report</text>
  </g>
  <path d="M144 134 L144 146 L480 146 L480 134" fill="none" stroke="var(--muted)" stroke-width="1"/>
  <path d="M256 134 L256 146 M368 134 L368 146" stroke="var(--muted)" stroke-width="1"/>
  <rect x="248" y="146" width="128" height="20" fill="#ff3b00"/>
  <text x="312" y="160" font-family="var(--font-mono)" font-size="9" fill="#f2f0e9" text-anchor="middle">get_field / put_field</text>
</svg>
<figcaption>The same arithmetic either lives at four call sites or at one. Only one of those versions can be fixed once.</figcaption>
</figure>

## Stopping to build the boring thing

So I stopped, with the clock running, and wrote a handful of subroutines that
did nothing interesting: given a record index and a field, compute the address.
Load it. Store it. A convention for which register held the base, which held the
index, and which came back with the value.

It felt like the wrong call while I was doing it. I was writing code that would
not be marked, in an exam where I was already behind.

Then the rest of the program collapsed. Every operation became a loop with two
or three calls in it, and it read almost like pseudocode. More importantly: when
something was wrong, there was exactly one place where addresses were computed,
so there was exactly one place to look. The bugs stopped being novel.

I finished. But the grade is not the interesting part.

## What I actually learned

Not "abstraction is good." I have spent years arguing the opposite when it is
applied before a problem exists — that
[architecture must follow pressure](/articles/architecture-must-follow-pressure/),
and that structure introduced on principle rather than in response to a real
failure mode is just cost.

This was consistent with that, and it took me a while to see why.

The abstraction I built was not architectural. I did not invent a record type
system or a query language. I wrapped **load and store** — the two operations
that were producing all of my errors — and nothing else. The pressure was a
measured failure mode: I had lost time twice to silently miscomputed offsets,
and I was going to lose it again. The boundary went exactly there, and nowhere
else.

That is the distinction I have used since. Abstracting control flow speculatively
is how codebases become mazes. Abstracting *data access* is how you stop making
the same arithmetic mistake in eleven places. The first is a guess about the
future; the second is a response to something already going wrong.

And the constraint argument runs backwards from what people assume. The tighter
the environment, the *more* an access layer pays — because in a tight
environment you have no type system, no debugger, and no margin for a class of
bug that reproduces silently. Fewer tools, more need for the one you can build
yourself.

## The same argument, made by a compiler

Years later I started writing Rust for an ATmega328P: 8 bit, 16 MHz, 2 KB of
RAM. As constrained as anything I have touched since that exam, and the same
temptation — it is small, just poke the registers.

Rust will not let you, and the way it refuses is precisely the lesson.

A pin is not an address you remember the meaning of. It is a value with a type,
and `into_output()` *consumes* the input-configured pin and returns a
differently-typed one. Reading it as an input afterwards is not a bug you find
at 2 a.m. with an LED; the method does not exist. The peripheral singleton
enforces that one piece of code owns the hardware. The HAL is, in the end, an
access layer over load and store — the same subroutines I wrote by hand under
exam conditions, except someone wrote them properly and the borrow checker
enforces that you go through them.

The part that closes the loop: I checked what it costs. A HAL call to set a pin
high and a raw register write compile to the **same single instruction**. The
abstraction is not paid for at runtime at all. It is paid for once, at compile
time, by someone else.

Which is what I had discovered badly and by hand, with fifteen minutes left on a
clock: the layer is not overhead you accept in exchange for tidiness. Done at the
right seam — around the operations that are silently going wrong — it is the
cheapest way to go faster.

The exam version cost me maybe ten minutes and saved me the rest of the hour.
The Rust version costs zero instructions and saves the whole category.
