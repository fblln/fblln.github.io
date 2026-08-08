//! `sram` — instruments "Two Kilobytes and No Operating System".
//!
//! The article's claim is that on a chip with no MMU and no guard page, every
//! safety property is either in the types or in your head. A static memory map
//! can show the layout; what it cannot show is the *asymmetry* that makes the
//! layout dangerous, because one half of it is the absence of an event:
//!
//! - `malloc` **is** checked. avr-libc keeps `__malloc_margin` (128 bytes by
//!   default) between the top of the heap and the stack pointer, and returns
//!   NULL rather than crossing it. The failure is visible and handleable.
//! - The stack is **not** checked. Nothing on the AVR compares the stack pointer
//!   to the top of the heap. A deep enough call chain simply writes through
//!   whatever is there, and execution continues with corrupted data and no
//!   signal of any kind — no fault, no trap, no return code.
//!
//! So the instrument lets a reader do both and compare. Allocating until the
//! heap is full produces a polite NULL. Recursing until the stack overruns the
//! heap produces silence, which is the entire point of the article, and is
//! precisely the thing a diagram renders as "nothing".
//!
//! Every constant below is a real property of the ATmega328P on an Arduino Uno
//! R3 or of avr-libc, not a tuned number, because a fabricated figure would
//! break the one rule the writing runs on.

pub(crate) mod logic {
    use crate::instrument::{Level, Log};

    /// ATmega328P internal SRAM: 2048 bytes at 0x0100..=0x08FF.
    pub(crate) const RAM_START: u16 = 0x0100;
    pub(crate) const RAM_END: u16 = 0x08FF;
    pub(crate) const SRAM: u16 = RAM_END - RAM_START + 1;

    /// `.data` + `.bss` for a sketch that pulls in `Serial`. The Arduino core's
    /// serial buffers dominate this; an empty sketch is far smaller.
    pub(crate) const GLOBALS: u16 = 200;

    /// One call frame carrying a 32-byte local buffer: the buffer, the two-byte
    /// AVR return address, and saved registers.
    pub(crate) const FRAME: u16 = 40;

    /// One `malloc(64)` request, plus avr-libc's two-byte block header.
    pub(crate) const CHUNK: u16 = 64;
    pub(crate) const CHUNK_OVERHEAD: u16 = 2;

    /// avr-libc `__malloc_margin`, the gap malloc refuses to allocate into so a
    /// modest amount of stack growth stays survivable.
    pub(crate) const MALLOC_MARGIN: u16 = 128;

    #[derive(Clone, PartialEq, Eq, Debug, Default)]
    pub(crate) struct Sram {
        /// Successful `malloc(64)` blocks currently held.
        pub(crate) blocks: u16,
        /// Current call depth.
        pub(crate) depth: u16,
        /// Deepest incursion of the stack into the heap, in bytes. Never
        /// decreases: returning from a function does not un-corrupt the data it
        /// overwrote, which is the property that makes this class of bug so hard
        /// to localise.
        pub(crate) corrupted: u16,
        /// Times `malloc` refused rather than crossing the margin.
        pub(crate) refusals: u16,
        pub(crate) log: Log,
    }

    impl Sram {
        pub(crate) fn new() -> Self {
            let mut sram = Self::default();
            sram.log.note(
                Level::Ok,
                format!(
                    "reset · {SRAM} bytes sram · globals occupy {GLOBALS} · heap base 0x{:04X} · sp 0x{RAM_END:04X}",
                    sram.heap_end()
                ),
            );
            sram
        }

        /// Bytes of heap currently committed.
        pub(crate) fn heap_bytes(&self) -> u16 {
            self.blocks * (CHUNK + CHUNK_OVERHEAD)
        }

        /// First address above the heap.
        pub(crate) fn heap_end(&self) -> u16 {
            RAM_START + GLOBALS + self.heap_bytes()
        }

        /// The stack pointer, descending from RAMEND. Saturates at RAM_START so
        /// an absurd depth reports a full overrun rather than wrapping.
        pub(crate) fn sp(&self) -> u16 {
            RAM_END
                .saturating_sub(self.depth.saturating_mul(FRAME))
                .max(RAM_START)
        }

        /// Unused bytes between the heap and the stack. Negative once they
        /// overlap, which is exactly the state the hardware will not tell you
        /// about.
        pub(crate) fn free(&self) -> i32 {
            self.sp() as i32 - self.heap_end() as i32
        }

        pub(crate) fn collided(&self) -> bool {
            self.sp() < self.heap_end()
        }

        /// `malloc(64)`. Checked: refuses rather than crossing the margin, and
        /// the caller gets a NULL it can act on.
        pub(crate) fn malloc(&mut self) -> bool {
            let need = CHUNK + CHUNK_OVERHEAD;
            let ceiling = self.sp().saturating_sub(MALLOC_MARGIN);
            if self.heap_end() + need > ceiling {
                self.refusals += 1;
                self.log.note(
                    Level::Held,
                    format!(
                        "malloc({CHUNK}) returned NULL · heap end 0x{:04X} + {need} would cross the {MALLOC_MARGIN}-byte margin below sp 0x{:04X} — refused, nothing corrupted",
                        self.heap_end(),
                        self.sp()
                    ),
                );
                return false;
            }
            self.blocks += 1;
            self.log.note(
                Level::Ok,
                format!(
                    "malloc({CHUNK}) → 0x{:04X} · heap now {} bytes · {} free to sp",
                    self.heap_end() - need,
                    self.heap_bytes(),
                    self.free()
                ),
            );
            true
        }

        pub(crate) fn free_block(&mut self) {
            if self.blocks == 0 {
                self.log
                    .note(Level::Held, "free() · no blocks held".to_string());
                return;
            }
            self.blocks -= 1;
            self.log.note(
                Level::Ok,
                format!(
                    "free() · heap now {} bytes · {} free to sp",
                    self.heap_bytes(),
                    self.free()
                ),
            );
        }

        /// Push one call frame. Deliberately unconditional: there is no check to
        /// model, because the hardware has none. Any mutation that makes this
        /// refuse is a mutation that describes a different machine.
        pub(crate) fn call(&mut self) {
            // Saturating, not `+= 1`: an overflow panic here would abort the
            // whole wasm module and take the article's page down with it.
            self.depth = self.depth.saturating_add(1);
            if self.collided() {
                let overlap = self.heap_end() - self.sp();
                self.corrupted = self.corrupted.max(overlap);
                self.log.note(
                    Level::Fail,
                    format!(
                        "frame pushed · depth {} · sp 0x{:04X} is {overlap} bytes below heap end 0x{:04X} — heap data overwritten, no fault raised",
                        self.depth,
                        self.sp(),
                        self.heap_end()
                    ),
                );
            } else {
                self.log.note(
                    Level::Ok,
                    format!(
                        "frame pushed · depth {} · sp 0x{:04X} · {} free to heap",
                        self.depth,
                        self.sp(),
                        self.free()
                    ),
                );
            }
        }

        /// Pop one frame. Note what this does *not* do: reduce `corrupted`.
        pub(crate) fn ret(&mut self) {
            if self.depth == 0 {
                self.log
                    .note(Level::Held, "return · already at top frame".to_string());
                return;
            }
            self.depth -= 1;
            let note = if self.corrupted > 0 {
                format!(
                    "return · depth {} · sp 0x{:04X} — the {} corrupted bytes stay corrupted",
                    self.depth,
                    self.sp(),
                    self.corrupted
                )
            } else {
                format!("return · depth {} · sp 0x{:04X}", self.depth, self.sp())
            };
            let level = if self.corrupted > 0 {
                Level::Fail
            } else {
                Level::Ok
            };
            self.log.note(level, note);
        }

        pub(crate) fn reset(&mut self) {
            *self = Self::new();
        }

        /// Segment widths as percentages of total SRAM, for the memory map:
        /// globals, heap, free, stack. When the stack and heap overlap the
        /// overlap is reported separately and the free span is zero.
        pub(crate) fn segments(&self) -> Segments {
            let pct = |bytes: u16| (bytes as f32 / SRAM as f32) * 100.0;
            let stack_bytes = RAM_END - self.sp() + 1;
            let overlap = if self.collided() {
                self.heap_end() - self.sp()
            } else {
                0
            };
            Segments {
                globals: pct(GLOBALS),
                heap: pct(self.heap_bytes().saturating_sub(overlap)),
                overlap: pct(overlap),
                free: pct(self.free().max(0) as u16),
                stack: pct(stack_bytes.saturating_sub(overlap)),
            }
        }
    }

    #[derive(Clone, Copy, PartialEq, Debug)]
    pub(crate) struct Segments {
        pub(crate) globals: f32,
        pub(crate) heap: f32,
        pub(crate) overlap: f32,
        pub(crate) free: f32,
        pub(crate) stack: f32,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn the_machine_matches_the_atmega328p() {
            assert_eq!(SRAM, 2048);
            let sram = Sram::new();
            assert_eq!(sram.heap_end(), RAM_START + GLOBALS);
            assert_eq!(sram.sp(), RAM_END);
            assert_eq!(sram.free(), (RAM_END - RAM_START - GLOBALS) as i32);
            assert_eq!(sram.corrupted, 0);
            assert_eq!(sram.log.len(), 1);
        }

        #[test]
        fn malloc_commits_a_block_with_its_header() {
            let mut sram = Sram::new();
            assert!(sram.malloc());
            assert_eq!(sram.blocks, 1);
            assert_eq!(sram.heap_bytes(), CHUNK + CHUNK_OVERHEAD);
            assert!(!sram.collided());
        }

        #[test]
        fn freeing_returns_the_block_and_freeing_nothing_is_a_no_op() {
            let mut sram = Sram::new();
            sram.malloc();
            sram.free_block();
            assert_eq!(sram.blocks, 0);
            sram.free_block();
            assert_eq!(sram.blocks, 0, "free with no blocks must not underflow");
        }

        #[test]
        fn returning_from_the_top_frame_is_a_no_op() {
            let mut sram = Sram::new();
            sram.ret();
            assert_eq!(sram.depth, 0, "depth must not underflow");
        }

        #[test]
        fn calls_and_returns_move_the_stack_pointer_by_one_frame() {
            let mut sram = Sram::new();
            sram.call();
            assert_eq!(sram.sp(), RAM_END - FRAME);
            sram.call();
            assert_eq!(sram.sp(), RAM_END - 2 * FRAME);
            sram.ret();
            assert_eq!(sram.sp(), RAM_END - FRAME);
        }

        /// The article's central asymmetry, half one: the checked path refuses.
        #[test]
        fn invariant_malloc_never_crosses_the_margin() {
            let mut sram = Sram::new();
            // Fill the heap as far as malloc will allow.
            while sram.malloc() {}
            assert!(sram.refusals > 0, "malloc must eventually refuse");
            assert!(
                !sram.collided(),
                "a refusing malloc must never leave heap and stack overlapping"
            );
            assert_eq!(
                sram.corrupted, 0,
                "no amount of allocation may corrupt anything"
            );
            assert!(
                sram.free() >= MALLOC_MARGIN as i32,
                "the margin below sp must survive every allocation"
            );
        }

        /// The article's central asymmetry, half two: the unchecked path is
        /// silent. Any mutation that makes `call` refuse, or that reports the
        /// overrun through a return value, fails here.
        #[test]
        fn invariant_the_stack_is_never_prevented_from_overrunning_the_heap() {
            let mut sram = Sram::new();
            while sram.malloc() {}
            let blocks = sram.blocks;
            for expected_depth in 1..=60 {
                sram.call();
                assert_eq!(
                    sram.depth, expected_depth,
                    "nothing on this machine may refuse a call"
                );
            }
            assert!(sram.collided(), "60 frames must overrun a full heap");
            assert!(sram.corrupted > 0, "the overrun must be recorded as damage");
            assert_eq!(
                sram.blocks, blocks,
                "the heap is unaware it was overwritten — block count is unchanged"
            );
        }

        /// Corruption is not undone by unwinding. This is why the bug surfaces
        /// far from its cause.
        #[test]
        fn invariant_returning_never_reduces_corruption() {
            let mut sram = Sram::new();
            while sram.malloc() {}
            for _ in 0..40 {
                sram.call();
            }
            let peak = sram.corrupted;
            assert!(peak > 0);
            for _ in 0..40 {
                sram.ret();
                assert_eq!(sram.corrupted, peak, "unwinding must not heal memory");
            }
            assert_eq!(sram.depth, 0);
            assert!(!sram.collided(), "the pointers separate again");
            assert_eq!(sram.corrupted, peak, "but the damage is permanent");
        }

        #[test]
        fn an_absurd_depth_saturates_instead_of_wrapping() {
            let mut sram = Sram::new();
            sram.depth = u16::MAX;
            assert_eq!(sram.sp(), RAM_START);
            assert!(sram.collided());
            // Must not panic on overflow when recording the damage.
            sram.call();
            assert!(sram.corrupted > 0);
        }

        #[test]
        fn segments_span_the_whole_of_sram() {
            let mut sram = Sram::new();
            sram.malloc();
            sram.call();
            let s = sram.segments();
            let total = s.globals + s.heap + s.overlap + s.free + s.stack;
            assert!(
                (total - 100.0).abs() < 0.01,
                "segments must tile all of sram, got {total}"
            );
        }

        #[test]
        fn segments_report_the_overlap_once_the_regions_collide() {
            let mut sram = Sram::new();
            while sram.malloc() {}
            for _ in 0..40 {
                sram.call();
            }
            let s = sram.segments();
            assert!(s.overlap > 0.0, "a collision must be visible in the map");
            assert_eq!(s.free, 0.0, "there is no free span once they overlap");
            let total = s.globals + s.heap + s.overlap + s.free + s.stack;
            assert!((total - 100.0).abs() < 0.01, "still tiles sram: {total}");
        }

        #[test]
        fn reset_restores_the_fresh_machine() {
            let mut sram = Sram::new();
            while sram.malloc() {}
            for _ in 0..30 {
                sram.call();
            }
            sram.reset();
            assert_eq!(sram, Sram::new());
            assert_eq!(sram.corrupted, 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) mod view {
    use super::logic::{CHUNK, FRAME, GLOBALS, MALLOC_MARGIN, SRAM, Sram as Machine};
    use leptos::prelude::*;

    #[component]
    pub(crate) fn Sram() -> impl IntoView {
        let m = RwSignal::new(Machine::new());

        view! {
            <div class="inst" role="group" aria-label="ATmega328P SRAM instrument">
                <div class="inst-head">
                    <span>"SRAM/ATMEGA328P"</span>
                    <span class="inst-anchor">{SRAM}" BYTES · NO MMU"</span>
                </div>

                <div class="inst-map" role="img"
                    aria-label=move || m.with(|m| format!(
                        "Memory map. Globals {GLOBALS} bytes, heap {} bytes, {} bytes free, stack {} bytes, {} bytes overwritten.",
                        m.heap_bytes(), m.free().max(0), (0x08FFu16 - m.sp() + 1), m.corrupted))>
                    <span class="seg-globals" style:width=move || m.with(|m| format!("{}%", m.segments().globals))></span>
                    <span class="seg-heap" style:width=move || m.with(|m| format!("{}%", m.segments().heap))></span>
                    <span class="seg-overlap" style:width=move || m.with(|m| format!("{}%", m.segments().overlap))></span>
                    <span class="seg-free" style:width=move || m.with(|m| format!("{}%", m.segments().free))></span>
                    <span class="seg-stack" style:width=move || m.with(|m| format!("{}%", m.segments().stack))></span>
                </div>
                <div class="inst-legend">
                    <span><i class="seg-globals"></i>"GLOBALS"</span>
                    <span><i class="seg-heap"></i>"HEAP →"</span>
                    <span><i class="seg-free"></i>"FREE"</span>
                    <span><i class="seg-stack"></i>"← STACK"</span>
                    <span><i class="seg-overlap"></i>"OVERWRITTEN"</span>
                </div>

                <div class="inst-readout">
                    <div>
                        <span>"HEAP END"</span>
                        <strong>{move || m.with(|m| format!("0x{:04X}", m.heap_end()))}</strong>
                    </div>
                    <div>
                        <span>"STACK POINTER"</span>
                        <strong>{move || m.with(|m| format!("0x{:04X}", m.sp()))}</strong>
                    </div>
                    <div>
                        <span>"FREE BYTES"</span>
                        <strong>{move || m.with(|m| m.free())}</strong>
                    </div>
                    <div>
                        <span>"MALLOC REFUSED"</span>
                        <strong>{move || m.with(|m| m.refusals)}</strong>
                    </div>
                    <div class="inst-cost">
                        <span>"BYTES CORRUPTED, SILENTLY"</span>
                        <strong class:hot=move || m.with(|m| m.corrupted > 0)>
                            {move || m.with(|m| m.corrupted)}
                        </strong>
                    </div>
                </div>

                <Show when=move || m.with(|m| m.corrupted > 0)>
                    <p class="inst-pending">
                        "The stack has written through the top of the heap. No fault was raised, no
                         allocation failed, and execution continues. Nothing on this chip will tell
                         you — which is why the article argues the property has to live in the types."
                    </p>
                </Show>

                <div class="inst-actions">
                    <button type="button" on:click=move |_| m.update(|m| { m.malloc(); })>
                        {format!("MALLOC({CHUNK})")}
                    </button>
                    <button type="button" on:click=move |_| m.update(|m| m.free_block())>"FREE"</button>
                    <button type="button" on:click=move |_| m.update(|m| m.call())>
                        {format!("CALL · +{FRAME} B FRAME")}
                    </button>
                    <button type="button" on:click=move |_| m.update(|m| m.ret())>"RETURN"</button>
                    <button type="button" on:click=move |_| m.update(|m| { while m.malloc() {} })>
                        "FILL HEAP"
                    </button>
                    <button type="button" on:click=move |_| m.update(|m| m.reset())>"RESET"</button>
                </div>

                <p class="inst-note">
                    "avr-libc keeps "{MALLOC_MARGIN}" bytes between the heap and the stack pointer.
                     malloc respects it. The stack does not know it exists."
                </p>

                <ol class="inst-log">
                    {move || m.with(|m| m.log.tail().iter().map(|line| {
                        view! { <li class=line.level.class()>{line.text.clone()}</li> }
                    }).collect_view())}
                </ol>
            </div>
        }
    }
}
