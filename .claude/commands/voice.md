---
description: Measure an article draft against the voice of the published corpus
argument-hint: [path to draft .md, or omit for the most recently edited article]
allowed-tools: Bash(cargo run --quiet --release --manifest-path tools/voice/Cargo.toml:*), Bash(git status:*), Bash(git diff:*), Read, Edit
---

Measure a draft against the published corpus, then fix what drifted.

Target: `$1` — if that is empty, use the article in `content/articles/` with
uncommitted changes, and say which one you picked.

## Run the check

```
cargo run --quiet --release --manifest-path tools/voice/Cargo.toml -- <target>
```

It exits 0 when the draft is inside the band on all five measurements with no
banned constructions, and 1 when it is not.

The bands live in `tools/voice/bands.txt`, measured from the published articles
and compiled in, with every article's own numbers beside them. They are stored
rather than recomputed per run because a gate that recalibrates itself cannot
fail: publish one article with long paragraphs and the ceiling rises to admit
the next one, and the voice ratchets with nothing ever going red.

So do not regenerate the bands to make a draft pass. If a band genuinely needs
to move, say so and leave it; `--calibrate` reports whether the stored bands
still match the corpus and lists every article's numbers, and `--calibrate
--write` regenerates the file as a reviewable diff. `--recalibrate` on a check
answers "what would the corpus say today" without touching anything.

## What the five measurements mean

These five are not a general style checklist. They are the measurements that
separated this corpus from generic technical prose written with no knowledge of
it, out of 29 that were tested; readability scores, syllable counts and modal
density separated nothing and are deliberately not measured. The gap each one
opens is in `tools/voice/src/lib.rs`.

**first person plural per 10k** — the strongest signal in the corpus by a wide
margin, a 6.85 sd gap. A model writes `we` and `our` about twenty-two times more
often than he does. Almost always the fix is first person singular where a claim
is his (*"the diagnosis I would write down"*), or the concrete actor where it is
not (*"the client builds a retry loop"*, not *"we build"*). `we` is legitimate
when it means the industry.

**sentences per paragraph**, **words per paragraph**, **one-sentence
paragraphs %** — one fact from three angles. His paragraphs are short and a
quarter of them are a single sentence, carrying a turn in the argument. Generated
prose writes uniform blocks and almost never writes a one-line paragraph. If all
three are high, the fix is not trimming words: it is breaking paragraphs where
the argument turns, and letting the turn stand alone.

**colons per 1k words** — he leads into evidence, a consequence or a list with a
colon far more than generic prose does.

A band is his own published range, not a target. Move toward the median and stop
there; overshooting past it is a different drift, not a fix.

## Banned constructions

Any hit fails the check. These were found by measuring the corpus in Aug 2026 and
removing them, so they are prohibitions rather than preferences, and the tool
quotes each hit so the fix is a deletion rather than a search.

`worth <gerund>` · stage directions (`Notice how`, `Look again at`, `Ask what`,
`Watch`, `Sit with`, `Hold onto`) · `Here is X` openers · `is load-bearing` as
abstract praise · paired em dashes (`— … —`; use parentheses or recast).

Two of them need judgement rather than deletion. `load-bearing` is fine as a
literal structural metaphor and banned as a synonym for "important". A stage
direction is a pointer at the prose; `Consider what a fleet operator actually
has` introduces evidence and is his own voice.

## Relation to tools/writing/check_prose.py

That script already reports phrase-level editorial cues — stock emphasis,
editorial preamble, sweeping claims, stock transitions, rhetorical contrast —
and deliberately reports rather than gates. This tool measures distributions
against a stored band and does gate. Run both; they look at different things.

They overlap in one place: `check_prose.py`'s "editorial preamble" rule matches
`it is worth noting|stating|being`, which is a subset of the `worth <gerund>`
ban here. A hit from either is the same problem.

## Then

Report each failure with the measured value, the band, and the edit you made.
Re-run until it exits 0.

If a measurement will not come inside the band without damaging the argument,
stop and say so instead of padding the prose to hit a number. The bands come
from thirteen articles, and some of his own published pieces sit outside their
own band on one measurement — `--calibrate` names them. A single failure on a
draft under about 400 words is noise; the tool says so when it applies.
