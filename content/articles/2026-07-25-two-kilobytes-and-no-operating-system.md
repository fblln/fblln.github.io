+++
title = "Two Kilobytes and No Operating System"
date = "2026-07-25"
description = "Part one of a series writing Rust on an Arduino Uno R3. Before any code: what the machine actually is, why compiling for it needs two separate toolchains, and which of its failure modes the type system can catch — because on a chip with no MMU and no guard page, every safety property is either in the types or in your head."
tags = ["Rust", "Embedded", "Systems", "Safety"]
+++

*Rust on Bare Metal, part one. The series builds up a set of labs on an Arduino
Uno R3 — blink, a `millis()` counter, interrupts, serial — and uses each one to
make a safety property concrete rather than asserted. This first part builds no
lab at all. It establishes the machine, because every claim the later parts make
about safety is a claim about* this *machine, and the arguments do not survive
being separated from it.*

I have spent most of my career on systems with an operating system underneath —
which means most of my safety intuitions are really intuitions about what the OS
was quietly doing on my behalf. A process that overruns its stack gets a
segfault. A pointer into freed memory usually faults before it corrupts
something interesting. Memory I allocate is memory somebody else does not have.

None of that is true on an ATmega328P.

That is the entire reason this series exists. The chip is small enough, and
bare enough, that the mechanisms are visible instead of inferred. When something
goes wrong there is nothing between you and the wreckage — no scheduler, no
virtual memory, no fault handler, no log. You get the wrong answer, silently,
and the only way to have prevented it was to have known.

## What an Arduino Uno actually is

Your laptop and an Arduino Uno are both computers, and they differ by roughly
six orders of magnitude. Every design decision in this series follows from that
gap.

| | Laptop (typical) | Arduino Uno R3 |
|---|---|---|
| Working memory (RAM) | 16 GB | **2 KB** — about 8 million times less |
| Program storage | 512 GB SSD | **32 KB** flash |
| Clock speed | ~3 GHz, multiple cores | 16 MHz, one core |
| Data width | 64 bits at a time | 8 bits at a time |
| Operating system | macOS / Linux / Windows | **none** |

The last row is the one that matters. There is no operating system on the Uno.
Nothing manages memory, nothing schedules tasks, nothing catches your mistakes,
and there is no screen, keyboard, or filesystem. When you power the board it
begins executing *your* code directly, and your code is the only thing running.
Forever — there is nothing to exit *to*.

The chip doing this is an **ATmega328P**, made by Microchip. "AVR" is the name
of its instruction set, in the same way "x86-64" names your laptop's — a
different and much simpler vocabulary of machine operations, which is why you
cannot simply run a normal program on it. The board around it is the chip plus
supporting parts: a 16 MHz crystal for timing, a voltage regulator, pin headers,
an LED wired to pin 13, and a second small chip whose only job is translating
USB into the serial protocol the main chip speaks.

### Why two kilobytes is the recurring villain

2048 bytes holds about 2000 characters of text — this section, roughly.
Everything your program needs at runtime lives there: variables, the call stack,
and any text you print. There is no warning when you run out. The program simply
begins corrupting its own data and behaves bizarrely.

<figure class="diagram">
<svg viewBox="0 0 620 128" role="img" aria-label="The 2048 bytes of SRAM drawn as a single bar to scale. Initialised variables and zeroed variables occupy the left end, a 200-byte string literal is highlighted next to them, free space fills the middle, and the stack occupies the right end growing leftwards into the free space with no boundary between them.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">SRAM &middot; 2048 BYTES &middot; TO SCALE</text>
  <rect x="0" y="24" width="70" height="28" fill="var(--ink)" opacity="0.14"/>
  <rect x="70" y="24" width="80" height="28" fill="var(--ink)" opacity="0.08"/>
  <rect x="150" y="24" width="61" height="28" fill="var(--signal)"/>
  <rect x="540" y="24" width="80" height="28" fill="var(--ink)" opacity="0.14"/>
  <rect x="0" y="24" width="620" height="28" fill="none" stroke="var(--line)"/>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="35" y="68" text-anchor="middle">.data</text>
    <text x="110" y="68" text-anchor="middle">.bss</text>
    <text x="375" y="68" text-anchor="middle">free &middot; and nothing guarding it</text>
    <text x="580" y="68" text-anchor="middle">stack</text>
  </g>
  <text x="180" y="68" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="middle">"..."</text>
  <g stroke="var(--ink)" stroke-width="1" fill="none">
    <path d="M540 84 L470 84 M476 80 L470 84 L476 88"/>
  </g>
  <text x="460" y="88" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="end">grows this way, into your variables</text>
  <text x="180" y="106" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="middle">200 chars = 10%</text>
  <text x="0" y="124" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">no MMU &middot; no guard page &middot; no fault &middot; the collision is silent</text>
</svg>
<figcaption>One error message costs a tenth of the machine. The stack grows downward into whatever is below it, and nothing in hardware notices the moment they meet.</figcaption>
</figure>

That bar is the shape of most of the unusual choices in this series. Keeping RAM
usage small is ordinary embedded discipline; keeping it *visible* is the part
that turns out to matter, because the failure mode is not a crash. It is a
program that worked until you added one more function call.

## Compiling is not one step — it is four

People say "compiling" for the whole process, but four distinct programs run,
and knowing which is which cuts debugging time enormously, because each produces
a different *style* of error.

First, **cross-compiling** — the reason this is not the usual arrangement. A
compiler turns source you can read into machine code a specific chip can
execute, and normally it produces code for the machine it is running on. Here
you compile on your Mac to produce AVR machine code. The Mac cannot run the
result at all. That single fact explains most of the friction: you cannot just
run your program to see whether it works, and you cannot use ordinary debugging
tools. The compiler's name for "what kind of machine am I producing code for" is
a **target**, and ours is `avr-none` — AVR architecture, no operating system.

<figure class="diagram">
<svg viewBox="0 0 620 158" role="img" aria-label="Four stages of the build shown left to right: rustc compiling to object files, avr-gcc linking, avr-libc startup code being linked in, and avrdude flashing the chip. A bracket beneath marks the first stage as the LLVM half and the middle two as the GNU half.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">ONE COMMAND, FOUR PROGRAMS</text>
  <g font-family="var(--font-mono)" font-size="10">
    <rect x="0" y="26" width="140" height="34" fill="var(--signal)"/>
    <text x="70" y="47" fill="var(--paper)" text-anchor="middle">rustc</text>
    <rect x="160" y="26" width="140" height="34" fill="none" stroke="var(--ink)"/>
    <text x="230" y="47" fill="var(--ink)" text-anchor="middle">avr-gcc</text>
    <rect x="320" y="26" width="140" height="34" fill="none" stroke="var(--ink)"/>
    <text x="390" y="47" fill="var(--ink)" text-anchor="middle">crt1.o</text>
    <rect x="480" y="26" width="140" height="34" fill="none" stroke="var(--line)"/>
    <text x="550" y="47" fill="var(--ink)" text-anchor="middle">avrdude</text>
  </g>
  <g stroke="var(--ink)" stroke-width="1" fill="none">
    <path d="M140 43 L155 43 M149 39 L155 43 L149 47"/>
    <path d="M300 43 L315 43 M309 39 L315 43 L309 47"/>
    <path d="M460 43 L475 43 M469 39 L475 43 L469 47"/>
  </g>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">
    <text x="70" y="76">object files,</text><text x="70" y="88">with holes</text>
    <text x="230" y="76">fills the holes,</text><text x="230" y="88">assigns addresses</text>
    <text x="390" y="76">runs before main,</text><text x="390" y="88">zeroes .bss</text>
    <text x="550" y="76">copies it over USB</text><text x="550" y="88">into flash</text>
  </g>
  <g stroke="var(--line)" fill="none">
    <path d="M0 104 L0 112 L140 112 L140 104"/>
    <path d="M160 104 L160 112 L460 112 L460 104"/>
  </g>
  <text x="70" y="126" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="middle">LLVM half</text>
  <text x="310" y="126" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">GNU half &middot; installed by brew install avr-gcc</text>
  <text x="0" y="150" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">ICEs and wrong codegen come from the left &middot; "undefined reference" and "relocation truncated" from the middle</text>
</svg>
<figcaption>The toolchain is a hybrid, and that is not a detail. Every error message you will ever see here comes from one half or the other, and they are distinguishable on sight.</figcaption>
</figure>

The four, in order. **The compiler** (`rustc`) translates each source file into
machine code, producing **object files** — machine code with holes in it,
because a file that calls `blink()` does not yet know what address `blink()`
will live at. **The linker** (here `avr-gcc`, acting as one) takes the object
files, fills the holes, and decides the final address of every function and
variable; its errors sound like "undefined reference" or "relocation truncated"
and are always about *connecting things*, never about your syntax. **Startup
code** gets linked in alongside — your `main` is not the first thing that runs.
And **the flasher** (`avrdude`, driven by `ravedude`) copies the finished
program over USB into the chip's flash, then lets it run.

The collective name for that set of programs is a **toolchain**. When the setup
step says `brew install avr-gcc`, it is installing an entire AVR toolchain: a
linker, the standard startup code, and a library of helper routines. Rust cannot
do without it, and the reason is worth stating plainly rather than treating as a
packaging accident — `rustc` is a compiler. It is not a linker, and it does not
ship a C runtime. On a hosted target that is invisible because the system C
compiler is already installed and rustc quietly uses it. Here nothing is
pre-installed, and one of the missing pieces is the program's beginning.

## Flash, RAM, and registers

Three kinds of memory, easy to conflate, and the distinction matters constantly.

* **Flash (32 KB)** — permanent storage for your *program*. Survives power-off,
  written once when you flash the board, read-only in practice. Code and
  constant data live here.
* **RAM (2 KB)** — scratch space while running. Erased on power-off. Variables
  and the call stack live here.
* **Registers** — 32 tiny 8-bit slots *inside* the CPU, the only place it can
  actually do arithmetic. Also, confusingly, the word for special memory
  addresses that control hardware.

That second meaning is how you control anything. There is no `turnOnLED()`
instruction. Specific memory addresses are wired to physical behaviour: writing
the value 32 to address `0x25` — named `PORTB` — sets pin 13 high and lights the
LED. Reading address `0x23` tells you the voltage on some pins. Controlling
hardware *is* writing numbers to particular addresses. That is the whole game,
and every abstraction layer in the next section exists to stop you playing it by
hand with magic numbers.

### The Harvard split

On your laptop, code and data share one address space; a memory address is a
memory address, and code is just bytes you could in principle read as data. On
AVR they are separately numbered and reached by *different machine
instructions*.

<figure class="diagram">
<svg viewBox="0 0 620 160" role="img" aria-label="Two separate address spaces drawn as parallel bars. Flash spans 32 kilobytes and is reached by the LPM instruction; SRAM spans 2 kilobytes and is reached by LD and ST. Address 0x100 is marked on both bars to show the same number names two unrelated locations.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">TWO ADDRESS SPACES, ONE NUMBERING</text>
  <text x="0" y="42" font-family="var(--font-mono)" font-size="10" fill="var(--ink)">FLASH</text>
  <rect x="92" y="28" width="440" height="24" fill="none" stroke="var(--line)"/>
  <text x="312" y="44" font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="middle">32 KB &middot; word-addressed &middot; your program</text>
  <text x="540" y="44" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">LPM</text>
  <rect x="95" y="28" width="2" height="24" fill="var(--signal)"/>
  <text x="0" y="98" font-family="var(--font-mono)" font-size="10" fill="var(--ink)">SRAM</text>
  <rect x="92" y="84" width="28" height="24" fill="none" stroke="var(--line)"/>
  <text x="130" y="100" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">2 KB &middot; byte-addressed &middot; everything else</text>
  <text x="540" y="100" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">LD / ST</text>
  <rect x="92" y="84" width="2" height="24" fill="var(--signal)"/>
  <g stroke="var(--signal)" fill="none">
    <path d="M96 52 L96 66 M93 84 L93 70 M93 70 L96 66" stroke-dasharray="3 3"/>
    <path d="M96 68 L150 68"/>
  </g>
  <text x="156" y="71" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">0x100 in both &middot; unrelated places &middot; no pointer type can span them</text>
  <text x="0" y="134" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">consequence: Rust has no stable syntax for "this constant lives in flash",</text>
  <text x="0" y="150" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">so every static &mdash; including every string literal &mdash; is copied into the 2 KB bar above.</text>
</svg>
<figcaption>LLVM models the distinction as address spaces. Rust has no stable way to request the flash one, which is why "my string literals ate my RAM" is the classic failure of this stack.</figcaption>
</figure>

This is the most consequential fact about the chip, and it has an awkward
consequence specifically for Rust. Text like `"hello"` in your source could
perfectly well live in the 32 KB of flash. It ends up copied into your precious
2 KB of RAM instead, because Rust has no standard way to say "this constant
stays in flash". C solved this decades ago with a non-standard GCC extension —
which is exactly what Arduino's `F("...")` macro wraps — and Rust's stricter
type system makes the equivalent harder to bolt on. It is a real, current
disadvantage of this stack, and I would rather write that down here than
discover it in part four.

## The first milliseconds after reset

Worth knowing before writing anything, because several later parts read the
evidence of it out of the compiled file.

<figure class="diagram">
<svg viewBox="0 0 620 136" role="img" aria-label="A boot timeline in five stages: reset, the interrupt vector table at flash address zero, avr-libc startup code setting the stack pointer and copying and zeroing variables, then main, which ends in an infinite loop with no exit.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">POWER-ON TO STEADY STATE</text>
  <path d="M0 46 L560 46" stroke="var(--line)" fill="none"/>
  <g font-family="var(--font-mono)" font-size="9">
    <circle cx="8" cy="46" r="4" fill="var(--ink)"/>
    <circle cx="150" cy="46" r="4" fill="var(--ink)"/>
    <circle cx="300" cy="46" r="4" fill="var(--ink)"/>
    <circle cx="450" cy="46" r="4" fill="var(--signal)"/>
    <text x="0" y="34" fill="var(--ink)">reset</text>
    <text x="142" y="34" fill="var(--ink)">vector table</text>
    <text x="292" y="34" fill="var(--ink)">__init</text>
    <text x="442" y="34" fill="var(--signal)">main</text>
  </g>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)">
    <text x="0" y="66">PC = 0</text>
    <text x="142" y="66">entry 0 is reset,</text><text x="142" y="78">so jump to startup</text>
    <text x="292" y="66">set stack pointer,</text><text x="292" y="78">copy .data, zero .bss</text>
    <text x="442" y="66">-&gt; ! &middot; never returns</text>
  </g>
  <path d="M560 46 L586 46 L586 100 L560 100 L572 100" fill="none" stroke="var(--signal)" stroke-width="1.5"/>
  <path d="M566 96 L560 100 L566 104" fill="none" stroke="var(--signal)" stroke-width="1.5"/>
  <text x="450" y="104" font-family="var(--font-mono)" font-size="9" fill="var(--signal)" text-anchor="end">loop {}</text>
  <text x="0" y="130" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">the vector table costs 104 bytes of flash whether or not you use interrupts</text>
</svg>
<figcaption>Your <code>main</code> is the fourth thing to run, not the first. The three before it are supplied by avr-libc, which is why removing avr-gcc does not merely break linking — it removes the program's beginning.</figcaption>
</figure>

At flash address 0 sits the **interrupt vector table** — a list of jump
addresses, one per kind of hardware event. Entry 0 is "reset", so the CPU's
first act is a jump into the startup code, which sets the stack pointer to the
top of RAM, copies initial values of variables from flash into RAM, zeroes the
rest, and calls `main`. Your `main` then runs and must never return: on a laptop
returning from `main` hands control back to the OS, and here there is no OS, so
it is declared `-> !` and ends in an infinite `loop {}`.

**Interrupts**, mentioned above, are the mechanism where hardware pauses your
program, runs a small designated function, and resumes exactly where it left
off. It is how you react to events without constantly checking for them, it is
the subject of a later part of this series, and it is also where the first
genuinely interesting safety argument lives — a function that can begin between
any two instructions of another function is a concurrency problem on a
single-core chip with no threads.

## The layers between your code and the hardware

You could write directly to address `0x25`. Nobody does, because it is
unreadable and unsafe. So there is a stack of libraries, each translating the
layer below into something more human. In Rust a library is called a **crate**.

<figure class="diagram">
<svg viewBox="0 0 620 230" role="img" aria-label="Five stacked layers from your code down to the chip: led.toggle, then arduino-hal the board crate, then avr-hal the hardware abstraction layer, then avr-device the peripheral access crate, then the chip at address 0x25. A bracket on the right notes that all layers compile down to a single sbi instruction, but only when optimisation is enabled.">
  <text x="0" y="12" font-family="var(--font-mono)" font-size="9" fill="var(--muted)">FIVE LAYERS, ZERO RUNTIME COST</text>
  <g font-family="var(--font-mono)" font-size="10">
    <rect x="0" y="24" width="440" height="30" fill="var(--signal)"/>
    <text x="12" y="44" fill="var(--paper)">led.toggle()</text>
    <rect x="0" y="62" width="440" height="30" fill="none" stroke="var(--line)"/>
    <text x="12" y="82" fill="var(--ink)">arduino-hal</text>
    <rect x="0" y="100" width="440" height="30" fill="none" stroke="var(--line)"/>
    <text x="12" y="120" fill="var(--ink)">avr-hal</text>
    <rect x="0" y="138" width="440" height="30" fill="none" stroke="var(--line)"/>
    <text x="12" y="158" fill="var(--ink)">avr-device</text>
    <rect x="0" y="176" width="440" height="30" fill="var(--ink)" opacity="0.1"/>
    <rect x="0" y="176" width="440" height="30" fill="none" stroke="var(--line)"/>
    <text x="12" y="196" fill="var(--ink)">the chip</text>
  </g>
  <g font-family="var(--font-mono)" font-size="9" fill="var(--muted)" text-anchor="end">
    <text x="428" y="82">board crate &middot; "pin 13 on an Uno", 16 MHz</text>
    <text x="428" y="120">HAL &middot; generic "set an output pin"</text>
    <text x="428" y="158">PAC &middot; names every register: PORTB, DDRB</text>
    <text x="428" y="196">address 0x25</text>
  </g>
  <path d="M452 24 L462 24 L462 206 L452 206" fill="none" stroke="var(--line)"/>
  <g font-family="var(--font-mono)" font-size="9">
    <text x="474" y="102" fill="var(--ink)">inlined away entirely</text>
    <text x="474" y="120" fill="var(--signal)">1 instruction: sbi</text>
    <text x="474" y="138" fill="var(--muted)">0 bytes of RAM</text>
  </g>
  <text x="0" y="226" font-family="var(--font-mono)" font-size="9" fill="var(--signal)">&mdash; but only with optimisation on, which makes an optimisation setting a correctness setting</text>
</svg>
<figcaption>The whole blink program is 96 bytes of machine code. The abstraction is free at runtime, and that guarantee is the reason this stack is worth the setup cost at all.</figcaption>
</figure>

The jargon you will meet for these: a **PAC** (Peripheral Access Crate) is the
bottom layer, `avr-device` — nothing but precise names and types for every
hardware register, generated automatically from Microchip's own machine-readable
description of the chip. A **HAL** (Hardware Abstraction Layer), `avr-hal`,
turns registers into concepts: pins, delays, serial ports. A **board crate**,
`arduino-hal`, fills in the specifics of *this* board.

The remarkable property, and the reason this is worth doing, is the bracket on
the right of that diagram: **these layers cost nothing at runtime.** The compiler
inlines them all away, so `led.toggle()` becomes the same one or two machine
instructions you would have written by hand. But that guarantee depends on
optimisation being switched on — which is a claim I am going to have to measure
rather than repeat, and a later part does exactly that, because a debug build
where the abstraction is *not* free is a debug build that does not fit in 2 KB.

## Why this is harder than the Arduino IDE

The IDE hides all of the above. You write `digitalWrite(13, HIGH)`, press
Upload, and it works. It can do that because it ships a runtime that
pre-configures the chip for you and a HAL that looks up pin numbers at runtime.

The trade is that mistakes are silent. Writing to a pin you configured as an
input compiles fine and misbehaves quietly. Exhausting RAM is easy and
unannounced. And this is the point where the series' actual subject appears,
because the Rust stack refuses to hide those things — and in exchange the
compiler can *prove* whole categories of mistake impossible before the code
reaches the board.

Not all of them. That is the interesting part. On this chip the safety story
splits cleanly in three, and being honest about which third you are in is most
of the skill:

* **Eliminated by the type system.** Pin modes are encoded as **typestate** —
  an output pin and an input pin are genuinely different types, so `set_high()`
  on an input is a compile error, and a pin the chip does not physically have
  does not exist as a type to name. Nothing is checked at runtime because there
  is nothing left to check.
* **Made visible, but still yours.** RAM exhaustion, stack collision, flash
  budget. The toolchain will tell you the numbers if you ask. It will not stop
  you.
* **Genuinely harder here than in C.** Constants in flash, as the Harvard
  section covered. Pretending otherwise would make the rest of the series less
  useful.

The reason to start a safety series on a machine this small is that the first
category is only impressive if you can see the second and third clearly. A type
system that eliminates a bug you could not have observed is a marketing claim.
On an Uno, you can observe all of them.

## Vocabulary for the rest of the series

Every term the later parts assume. Skim now, return as needed.

| Term | Meaning |
|---|---|
| **ABI** | The conventions for how compiled code passes arguments and returns values. Two pieces of machine code must agree on it to interoperate. |
| **avrdude** | The program that copies your compiled firmware into the chip over USB. |
| **avr-gcc** | The GNU C compiler for AVR. Used here *as the linker*, and for its startup code and helper library — not to compile any C. |
| **avr-libc** | The AVR standard C library. Supplies the startup code and the linker scripts describing the chip's memory layout. |
| **bootloader** | A tiny program permanently in the top 512 bytes of flash that receives new firmware over serial and writes it. On the Uno it is **Optiboot**, and it is why you need no special programming hardware. |
| **`.bss` / `.data` / `.text`** | Regions of a compiled program: `.text` is machine code, `.data` is variables with initial values, `.bss` is variables starting at zero. Their sizes tell you flash and RAM usage. |
| **crate** | A Rust library or program. The unit of compilation and of dependency. |
| **`core`** | Rust's standard library minus everything needing an OS. `std` is the full one, and is unavailable here. |
| **ELF** | The container format for compiled programs on Unix-like systems. Your `.elf` is firmware plus symbol and debug tables. |
| **firmware** | Software that runs on a device with no OS. What you are writing. |
| **ICE** | Internal Compiler Error — the compiler itself crashed. Always a compiler bug, never yours. |
| **inlining** | Pasting a called function's body into the caller, removing call overhead. The mechanism by which the library layers become free. |
| **interrupt / ISR** | Hardware pauses your program to run a short Interrupt Service Routine, then resumes. |
| **libgcc** | Routines for operations the chip lacks — 32-bit multiply, division, floating point. The AVR has no divide instruction, so `a / b` becomes a call into libgcc. |
| **`no_std`** | A Rust crate declaring it does not use the OS-dependent standard library. Mandatory here. |
| **PAC / HAL / board crate** | The three abstraction layers above. |
| **panic** | Rust's response to an unrecoverable error. On a laptop it prints and exits; here you must supply a handler, and there is nothing to exit to. |
| **relocation** | A "hole" in an object file that the linker fills with a final address. "Relocation truncated to fit" means an address did not fit the space the instruction allows. |
| **Tier 3** | Rust's designation for a target that is built but *not tested*, and for which no precompiled `core` is shipped. AVR is Tier 3, and part two spells out what that costs. |
| **typestate** | Encoding a thing's current state in its *type*, so misuse is a compile error. Used for pin modes. |
| **zero-cost abstraction** | A layer of convenience the optimiser removes entirely, leaving no runtime penalty. |

## Next

Part two takes the toolchain apart properly: why the AVR target is Tier 3, why
that forces a nightly compiler and a rebuild of `core` from source, and why the
nightly is *pinned* to a specific date rather than tracking latest. The short
version is that a miscompile on this target is silent, which makes the pin a
safety measure rather than conservatism — but the long version is the one worth
reading, because it is the first time in this series that the answer is
"something you rely on is genuinely, currently broken, and here is how to tell".

No code yet. There is a reason for that: every safety claim the labs make is a
claim about the machine described above, and I have watched too many embedded
tutorials teach a mechanism without the constraint that justifies it. The
constraint comes first.

## References

1. Microchip, *ATmega328P Datasheet* (DS40002061), §7 "AVR Memories",
   §36 "Instruction Set Summary".

2. `avr-libc` user manual, "Memory Sections".
   https://www.nongnu.org/avr-libc/user-manual/mem_sections.html

3. Rust platform support and target tier policy.
   https://doc.rust-lang.org/rustc/target-tier-policy.html

4. `avr-hal` — the HAL and board crates used throughout this series.
   https://github.com/Rahix/avr-hal
