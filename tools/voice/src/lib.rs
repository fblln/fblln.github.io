//! Measure an article draft against the voice of the published corpus.
//!
//! # Why this exists
//!
//! A language model drafting an article here reliably writes prose that is
//! competent and not his. The gap was measured: generated text was scored
//! against the published articles on 29 style metrics, and almost all of them
//! turned out to carry no author signal at all. Readability scores, syllable
//! counts and modal density sit within a tenth of a standard deviation of his
//! own average in prose written with no knowledge of him. They describe English
//! technical writing, not this writer.
//!
//! Five measurements did separate him from unguided generic prose, and this file
//! implements those five and nothing else. Each is annotated with the gap it
//! opened, in units of his own article-to-article variation:
//!
//! | measurement                  | his corpus | generic prose | gap     |
//! |------------------------------|-----------:|--------------:|--------:|
//! | first person plural per 10k  |        7.5 |         166.2 | 6.85 sd |
//! | sentences per paragraph      |       2.79 |          3.91 | 1.83 sd |
//! | words per paragraph          |       41.0 |          65.8 | 1.65 sd |
//! | one-sentence paragraphs      |      22.8% |          0.0% | 1.57 sd |
//! | colons per 1,000 words       |        9.3 |           3.3 | 1.45 sd |
//!
//! The first is not a tendency, it is a fingerprint: a model writes `we` and
//! `our` twenty-two times more often than he does. The next three are all one
//! fact seen from three angles — he writes short paragraphs and lands turns in
//! the argument on single-sentence ones, and generated prose writes uniform
//! blocks and never writes a one-line paragraph at all.
//!
//! Passive-voice rate opened a 1.44 sd gap and is deliberately absent: finding
//! it needs a dependency parse, which needs a parser, which is a dependency and
//! a model file. Five measurements and five patterns are enough to catch the
//! drift that matters.
//!
//! # The bands are stored, not recomputed
//!
//! `bands.txt` holds the measured band for each of the five, plus every
//! published article's own numbers, and it is compiled into the binary with
//! `include_str!`. `--calibrate --write` regenerates it.
//!
//! Calibrating at runtime instead was tried first and is worse, for the reason
//! the compressed-WASM budget in CI is a committed number rather than a
//! measurement of whatever is in `dist/`: a gate that recomputes its own
//! threshold cannot fail. Publish one article with long paragraphs and the
//! words-per-paragraph ceiling rises to admit it, so the next draft with long
//! paragraphs passes, and the voice ratchets without anything ever going red.
//! A stored band moves only when somebody regenerates it, and the diff says by
//! how much.
//!
//! Storing the per-article numbers beside the bands matters for the same
//! reason. They are the evidence the band was derived from, they make the
//! outlier visible (`architecture-must-follow-force` uses `we` at 101.9 per 10k
//! against a corpus median of 3.9), and a regenerated file shows in review
//! which article moved the edge.
//!
//! One thing the stored numbers are not: comparable to anybody else's. Sentence
//! boundaries are a judgement call and instruments disagree — five stylometry
//! tools measuring mean sentence length on this corpus returned 15.30, 15.48,
//! 15.85, 15.99 and 17.47 words, a 14% spread purely from tokenisation. The
//! naive splitter in `sentences` below does not agree with a real parser
//! either, and does not need to: the same code measured the corpus and measures
//! the draft. Regenerate `bands.txt` if that code ever changes.

use std::fmt;
use std::path::{Path, PathBuf};

/// One measurement of one document.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    pub words: usize,
    pub paragraphs: usize,
    pub sentences: usize,
    pub first_plural_per_10k: f64,
    pub sentences_per_paragraph: f64,
    pub words_per_paragraph: f64,
    pub single_sentence_paragraph_pct: f64,
    pub colons_per_1k: f64,
}

/// The five checked measurements, in the order they are reported: strongest
/// measured signal first, so an agent reading the output top-down fixes the
/// thing that matters most.
pub const CHECKED: [(&str, fn(&Metrics) -> f64, &str); 5] = [
    (
        "first person plural per 10k",
        |m| m.first_plural_per_10k,
        "6.85 sd",
    ),
    (
        "sentences per paragraph",
        |m| m.sentences_per_paragraph,
        "1.83 sd",
    ),
    ("words per paragraph", |m| m.words_per_paragraph, "1.65 sd"),
    (
        "one-sentence paragraphs %",
        |m| m.single_sentence_paragraph_pct,
        "1.57 sd",
    ),
    ("colons per 1k words", |m| m.colons_per_1k, "1.45 sd"),
];

/// Constructions removed from the corpus in an Aug-2026 editing pass, kept here
/// so they cannot come back. Each was found by measuring, not by taste: the
/// three articles written after that pass measure zero on all five, and some
/// older ones still do not, which is why the checker reports them per article
/// rather than asserting the corpus is clean.
pub const TICS: [(&str, fn(&str) -> Vec<String>); 5] = [
    ("worth <gerund>", find_worth_gerund),
    ("stage direction", find_stage_direction),
    ("\"Here is\" opener", find_here_is),
    ("\"load-bearing\"", find_load_bearing),
    ("paired em dash (— … —)", find_paired_em_dash),
];

/// The committed bands, compiled in so the check needs no file at runtime.
/// Regenerate with `voice --calibrate --write`.
const BANDS_FILE: &str = include_str!("../bands.txt");

/// Parse `bands.txt`: one `label = low high median` line per metric, `#`
/// comments, in `CHECKED` order. Hand-rolled because the format is ours and a
/// TOML parser would be a dependency for four fields.
///
/// Returns an error rather than a default, because silently falling back to a
/// recomputed band is the ratchet this file exists to prevent.
pub fn stored_bands() -> Result<Vec<Band>, String> {
    let mut found: Vec<Option<Band>> = vec![None; CHECKED.len()];
    for (lineno, line) in BANDS_FILE.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (label, rest) = line.split_once('=').ok_or_else(|| {
            format!(
                "bands.txt:{}: expected `label = low high median`",
                lineno + 1
            )
        })?;
        // Each band line carries a trailing `# ...` note saying how far that
        // measurement separates his prose from generic prose. Strip it before
        // parsing, or the note becomes the fourth number.
        let rest = rest.split('#').next().unwrap_or(rest);
        let nums: Vec<f64> = rest
            .split_whitespace()
            .map(|n| {
                n.parse::<f64>()
                    .map_err(|e| format!("bands.txt:{}: {e}", lineno + 1))
            })
            .collect::<Result<_, _>>()?;
        let [low, high, median] = nums[..] else {
            return Err(format!("bands.txt:{}: expected three numbers", lineno + 1));
        };
        let idx = CHECKED
            .iter()
            .position(|(l, _, _)| *l == label.trim())
            .ok_or_else(|| {
                format!(
                    "bands.txt:{}: unknown metric {:?}",
                    lineno + 1,
                    label.trim()
                )
            })?;
        found[idx] = Some(Band { low, high, median });
    }
    found
        .into_iter()
        .zip(CHECKED.iter())
        .map(|(b, (label, _, _))| b.ok_or_else(|| format!("bands.txt: no band for {label:?}")))
        .collect()
}

/// Render `bands.txt` from a freshly measured corpus.
///
/// The per-article block is a comment: nothing parses it, and it is the whole
/// point of the file being in git. A reviewer seeing an edge move looks at the
/// article that moved it.
pub fn render_bands(corpus: &Path, measured: &[(PathBuf, Metrics)]) -> String {
    let mut out = String::new();
    out.push_str("# Measured voice bands for content/articles. Generated; do not hand-edit.\n");
    out.push_str("#\n");
    out.push_str("#   regenerate: cargo run --release --manifest-path tools/voice/Cargo.toml \\\n");
    out.push_str("#                 -- --calibrate --write\n");
    out.push_str("#\n");
    out.push_str(&format!(
        "# corpus:  {} articles in {}\n# padding: {PAD_SD} sd past the observed range\n#\n",
        measured.len(),
        corpus.display()
    ));
    out.push_str("# label = low high median\n");
    let bands: Vec<Band> = CHECKED
        .iter()
        .map(|(_, get, _)| band(&measured.iter().map(|(_, m)| get(m)).collect::<Vec<_>>()))
        .collect();
    for ((label, _, signal), b) in CHECKED.iter().zip(&bands) {
        out.push_str(&format!(
            "{label} = {:.4} {:.4} {:.4}   # separates his prose from generic by {signal}\n",
            b.low, b.high, b.median
        ));
    }
    out.push_str("\n# The numbers the bands were measured from.\n");
    out.push_str("# article                                            words   we/10k  s/para  w/para  1-sent%  colon/1k\n");
    for (path, m) in measured {
        let name = path.file_stem().unwrap_or_default().to_string_lossy();
        let outside: Vec<&str> = CHECKED
            .iter()
            .zip(&bands)
            .filter(|((_, get, _), b)| {
                let v = get(m);
                v < b.low || v > b.high
            })
            .map(|((label, _, _), _)| *label)
            .collect();
        out.push_str(&format!(
            "# {name:<50} {:5}  {:7.1} {:7.2} {:7.1} {:8.1} {:9.1}{}\n",
            m.words,
            m.first_plural_per_10k,
            m.sentences_per_paragraph,
            m.words_per_paragraph,
            m.single_sentence_paragraph_pct,
            m.colons_per_1k,
            if outside.is_empty() {
                String::new()
            } else {
                format!("   <- outside own band: {}", outside.join(", "))
            }
        ));
    }
    out
}

/// An inclusive range a draft has to land in, measured from the corpus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    pub low: f64,
    pub high: f64,
    pub median: f64,
}

impl fmt::Display for Band {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:.1}–{:.1} (median {:.1})",
            self.low, self.high, self.median
        )
    }
}

/// Band for one metric across the corpus.
///
/// The edges are his observed minimum and maximum, padded by `PAD_SD` standard
/// deviations. An unpadded range describes sampling noise: measured
/// leave-one-out, five of thirteen articles fall outside a band built from the
/// other twelve.
///
/// Trimmed edges were tried first and abandoned. The 10th and 90th percentiles
/// look like the right answer, because one early article uses `we` at 102 per
/// 10k words against a corpus median near 4, and an extreme-edge band stretched
/// by it admits prose at far above his usual rate. But with thirteen documents
/// p90 moves sharply when one is left out, and no padding up to 2 sd brought
/// false positives below two articles. Specificity is what matters here:
/// generic prose misses these five measurements by margins that any band
/// catches, so the band should be as generous as his own range and no tighter.
/// `--calibrate` lists the outliers instead of hiding them in a trimmed edge.
/// How far past his observed range a draft may sit, in standard deviations of
/// the corpus.
///
/// Chosen by measurement, not by taste. `examples/sweep.rs` builds the band for
/// each published article from the other twelve — the band a real check of that
/// article would use — and also scores four pieces of generic technical prose
/// written on the same topics with no knowledge of this corpus:
///
/// | padding | his articles failing | generic prose failing |
/// |--------:|---------------------:|----------------------:|
/// |    0 sd |                 5/13 | 4/4, 18 measurements  |
/// |  0.5 sd |                 3/13 | 4/4, 18 measurements  |
/// | 0.75 sd |                 1/13 | 4/4, 17 measurements  |
/// |    1 sd |                 1/13 | 4/4, 17 measurements  |
/// |  1.5 sd |                 1/13 | 4/4, 8 measurements   |
///
/// 0.75 is the knee: the smallest padding that gets false positives down to one
/// article while still flagging seventeen of the twenty measurements on prose
/// that is not his. Past 1 sd the gate keeps its verdict and loses its
/// resolution, which is worse for the thing the tool is for — telling a drafter
/// *which* measurement to move.
///
/// Detecting the drift was never the hard part. Generic prose misses by margins
/// no plausible padding forgives: `we` at twenty-two times his rate, and not one
/// single-sentence paragraph in four pieces. The padding exists to stop the gate
/// crying wolf on his own writing.
const PAD_SD: f64 = 0.75;

pub fn band(values: &[f64]) -> Band {
    band_with_pad(values, PAD_SD)
}

/// Band with an explicit padding, exposed so `examples/sweep.rs` can choose
/// `PAD_SD` by leave-one-out rather than by assertion.
pub fn band_with_pad(values: &[f64], pad_sd: f64) -> Band {
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pad = pad_sd * stdev(&v);
    Band {
        low: (v[0] - pad).max(0.0),
        high: v[v.len() - 1] + pad,
        median: percentile(&v, 50.0),
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let pos = (sorted.len() - 1) as f64 * p / 100.0;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        sorted[lo] * (hi as f64 - pos) + sorted[hi] * (pos - lo as f64)
    }
}

fn stdev(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return 0.0;
    }
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    let var = v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (v.len() - 1) as f64;
    var.sqrt()
}

/// Reduce an article to the author's own prose.
///
/// Every removal here is an editorial decision that changes the numbers, and
/// they match the decisions the original measurement was made under. Getting
/// one wrong does not produce an error, it produces a plausible wrong band.
///
/// * Front matter is metadata.
/// * Fenced code blocks and SVG diagrams are not prose. Figures go entirely,
///   captions included: the captions are his, but they are one-line asides in a
///   different register and counting them as paragraphs would halve the mean.
/// * Blockquotes are other people's sentences — Fielding, Ousterhout, a
///   deliberately bad strawman interface — and measuring them would describe
///   the people he quotes.
/// * List items and table cells are fragments, not sentences. Left in, they
///   drag the sentence-length distribution down by several words; table rows
///   were caught only after a pipe character turned up among the corpus's most
///   frequent sentence openers.
/// * Headings are labels.
/// * Inline code keeps its text and loses its backticks. `RemoteStartTransaction`
///   mid-sentence is genuinely how he writes, and deleting it would shorten
///   sentences that really are that long.
/// * Hard wraps at ~80 columns are joined, because a newline inside a paragraph
///   is not a break and a sentence splitter that believes it is will find twice
///   as many sentences as exist.
pub fn strip(source: &str) -> String {
    let body = match strip_front_matter(source) {
        Some(rest) => rest,
        None => source,
    };

    let mut kept: Vec<String> = Vec::new();
    let mut in_fence = false;
    let mut figure_depth = 0usize;

    for raw in body.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();

        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }

        // `<figure>` blocks hold the inline SVG diagrams. Tracked as a depth
        // counter rather than a flag because an open and a close can share a
        // line, and a flag would then stay stuck on for the rest of the file.
        let lower = trimmed.to_ascii_lowercase();
        if lower.contains("<figure") {
            figure_depth += lower.matches("<figure").count();
        }
        if figure_depth > 0 {
            if lower.contains("</figure>") {
                figure_depth = figure_depth.saturating_sub(lower.matches("</figure>").count());
            }
            continue;
        }

        if trimmed.starts_with('#')
            || trimmed.starts_with('>')
            || trimmed.starts_with('|')
            || is_list_item(trimmed)
        {
            // Dropped, but the line still ends whatever paragraph it sat in:
            // a heading between two paragraphs must not weld them together.
            kept.push(String::new());
            continue;
        }

        kept.push(strip_inline(line).to_string());
    }

    // Rebuild paragraphs: blank-line separated, hard wraps joined.
    kept.join("\n")
        .split("\n\n")
        .map(|p| p.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn strip_front_matter(source: &str) -> Option<&str> {
    let rest = source.strip_prefix("+++")?;
    let end = rest.find("\n+++")?;
    Some(&rest[end + 4..])
}

fn is_list_item(trimmed: &str) -> bool {
    if let Some(rest) = trimmed.strip_prefix("- ").or(trimmed.strip_prefix("* ")) {
        return !rest.trim().is_empty();
    }
    // "1. ", "12. " — an ordered item, not a sentence starting with a number.
    let digits: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();
    !digits.is_empty() && trimmed[digits.len()..].starts_with(". ")
}

/// Unwrap inline Markdown, keeping the text and dropping the markup.
fn strip_inline(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            // Images carry no prose; links keep their text and lose the target.
            '!' if chars.get(i + 1) == Some(&'[') => {
                if let Some(end) = closing(&chars, i + 1, '[', ']') {
                    if let Some(paren) = link_target(&chars, end + 1) {
                        i = paren + 1;
                        continue;
                    }
                }
                out.push(chars[i]);
                i += 1;
            }
            '[' => {
                if let Some(end) = closing(&chars, i, '[', ']') {
                    if let Some(paren) = link_target(&chars, end + 1) {
                        out.extend(&chars[i + 1..end]);
                        i = paren + 1;
                        continue;
                    }
                }
                out.push(chars[i]);
                i += 1;
            }
            '`' => {
                if let Some(end) = chars[i + 1..].iter().position(|c| *c == '`') {
                    out.extend(&chars[i + 1..i + 1 + end]);
                    i = i + end + 2;
                    continue;
                }
                out.push(chars[i]);
                i += 1;
            }
            '<' => {
                // A bare HTML tag. `a < b` keeps its angle bracket because the
                // run to the next '>' has to look like a tag name.
                match tag_end(&chars, i) {
                    Some(end) => i = end + 1,
                    None => {
                        out.push(chars[i]);
                        i += 1;
                    }
                }
            }
            '*' | '_' => {
                // Emphasis markers. Dropped rather than matched as pairs: the
                // only thing downstream counts is words, and `*force*` and
                // `force` are the same word. An underscore inside a word
                // (`snake_case`) is kept, since that is an identifier.
                let intra_word = chars[i] == '_'
                    && i > 0
                    && chars[i - 1].is_alphanumeric()
                    && chars.get(i + 1).is_some_and(|c| c.is_alphanumeric());
                if intra_word {
                    out.push(chars[i]);
                }
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

fn closing(chars: &[char], from: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    for (offset, c) in chars[from..].iter().enumerate() {
        if *c == open {
            depth += 1;
        } else if *c == close {
            depth -= 1;
            if depth == 0 {
                return Some(from + offset);
            }
        }
    }
    None
}

fn link_target(chars: &[char], at: usize) -> Option<usize> {
    if chars.get(at) != Some(&'(') {
        return None;
    }
    closing(chars, at, '(', ')')
}

fn tag_end(chars: &[char], at: usize) -> Option<usize> {
    let next = *chars.get(at + 1)?;
    if !(next.is_ascii_alphabetic() || next == '/') {
        return None;
    }
    chars[at..].iter().position(|c| *c == '>').map(|p| at + p)
}

/// Split prose into sentences.
///
/// A terminator ends a sentence when it is followed by whitespace and then
/// something that can begin one: a capital, a digit, or an opening quote. That
/// one condition is what keeps `e.g. the` and `RFC 9146, section 4.` together
/// while splitting `… predates it. The command worked`, and it needs no
/// abbreviation list. Identifiers like `127.0.0.1` never match, because the dot
/// inside them is not followed by a space.
///
/// It is a heuristic and it will disagree with a real parser. That is accounted
/// for: the same function measures the corpus and the draft, so the band and
/// the value are on the same scale. See the module header.
pub fn sentences(prose: &str) -> Vec<&str> {
    let bytes = prose.as_bytes();
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if matches!(bytes[i], b'.' | b'!' | b'?') {
            // Consume a run of terminators and any closing quote or bracket, so
            // `…really?" The` breaks after the quote and not inside it.
            let mut end = i + 1;
            while end < bytes.len()
                && matches!(bytes[end], b'.' | b'!' | b'?' | b'"' | b'\'' | b')' | b']')
            {
                end += 1;
            }
            let mut after = end;
            while after < bytes.len() && bytes[after].is_ascii_whitespace() {
                after += 1;
            }
            let opens_a_sentence = after >= bytes.len()
                || prose[after..].chars().next().is_some_and(|c| {
                    c.is_uppercase() || c.is_ascii_digit() || c == '"' || c == '\u{201c}'
                });
            if after > end && opens_a_sentence {
                let s = prose[start..end].trim();
                if !s.is_empty() {
                    out.push(s);
                }
                start = after;
                i = after;
                continue;
            }
            i = end;
            continue;
        }
        i += 1;
    }
    let tail = prose[start..].trim();
    if !tail.is_empty() {
        out.push(tail);
    }
    out
}

/// First person plural, the strongest single marker in the corpus at a 6.85 sd
/// gap. Possessives are included and contractions are not: `we're` does not
/// appear in the corpus at all, and neither do any other contractions.
const FIRST_PLURAL: [&str; 5] = ["we", "us", "our", "ours", "ourselves"];

pub fn measure(source: &str) -> Metrics {
    let prose = strip(source);
    let paragraphs: Vec<&str> = prose
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .collect();
    let words: Vec<String> = prose
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
                .to_ascii_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect();

    let sentence_counts: Vec<usize> = paragraphs
        .iter()
        .map(|p| sentences(p).len().max(1))
        .collect();
    let total_sentences: usize = sentence_counts.iter().sum();
    let first_plural = words
        .iter()
        .filter(|w| FIRST_PLURAL.contains(&w.as_str()))
        .count();
    let colons = prose.matches(':').count();
    let n_words = words.len();
    let n_paras = paragraphs.len();
    let per = |count: usize, scale: f64| {
        if n_words == 0 {
            0.0
        } else {
            count as f64 / n_words as f64 * scale
        }
    };
    let mean = |total: usize| {
        if n_paras == 0 {
            0.0
        } else {
            total as f64 / n_paras as f64
        }
    };

    Metrics {
        words: n_words,
        paragraphs: n_paras,
        sentences: total_sentences,
        first_plural_per_10k: per(first_plural, 10_000.0),
        sentences_per_paragraph: mean(total_sentences),
        words_per_paragraph: mean(
            paragraphs
                .iter()
                .map(|p| p.split_whitespace().count())
                .sum(),
        ),
        single_sentence_paragraph_pct: if n_paras == 0 {
            0.0
        } else {
            sentence_counts.iter().filter(|c| **c == 1).count() as f64 / n_paras as f64 * 100.0
        },
        colons_per_1k: per(colons, 1_000.0),
    }
}

// --- tic patterns ---------------------------------------------------------
// Scanned on the stripped prose, and each returns the offending text rather
// than a count: a rate tells an agent it failed, the quotation tells it what to
// delete. Roughly half of what a tic pattern flags in real prose is the writing
// working correctly, so these report and the reader decides.

fn words_of(prose: &str) -> Vec<&str> {
    prose.split_whitespace().collect()
}

fn find_worth_gerund(prose: &str) -> Vec<String> {
    let w = words_of(prose);
    w.windows(2)
        .filter(|pair| {
            pair[0].eq_ignore_ascii_case("worth")
                && pair[1]
                    .trim_end_matches(|c: char| !c.is_alphanumeric())
                    .len()
                    > 4
                && pair[1]
                    .trim_end_matches(|c: char| !c.is_alphanumeric())
                    .to_ascii_lowercase()
                    .ends_with("ing")
        })
        .map(|pair| format!("{} {}", pair[0], pair[1]))
        .collect()
}

/// Imperatives that point at the text instead of stating it: "Notice how…",
/// "Look again at…", "Hold that number…". His prose states things.
///
/// Exactly the seven verbs the ban-list names, and no more. A first version
/// carried `Consider`, `Imagine`, `Note`, `See`, `Observe`, `Picture` and
/// `Think`, and flagged "Consider what a fleet operator actually has" in
/// `who-pays-for-the-force`. That is how he introduces evidence; the tic is
/// telling the reader where to look, not asking them to weigh something. The
/// wider list turned a check into noise, and a noisy check gets ignored.
///
/// Only sentence-initial hits count, because `hold` and `read` are ordinary
/// verbs mid-sentence.
fn find_stage_direction(prose: &str) -> Vec<String> {
    const OPENERS: [&str; 7] = ["Ask", "Look", "Read", "Notice", "Watch", "Sit", "Hold"];
    prose
        .split("\n\n")
        .flat_map(sentences)
        .filter(|s| {
            let first = s.split_whitespace().next().unwrap_or("");
            // An opener followed by a noun phrase is a direction; "Notice" or
            // "Read" standing alone as a noun or heading fragment is not.
            OPENERS.contains(&first) && s.split_whitespace().count() > 2
        })
        .map(|s| truncate(s))
        .collect()
}

fn find_here_is(prose: &str) -> Vec<String> {
    prose
        .split("\n\n")
        .flat_map(sentences)
        .filter(|s| {
            let lower = s.to_ascii_lowercase();
            lower.starts_with("here is")
                || lower.starts_with("here's")
                || lower.starts_with("here are")
        })
        .map(|s| truncate(s))
        .collect()
}

/// Fine as a literal structural metaphor, not as a synonym for "important".
/// The distinction is not mechanical, so every hit is reported for a human to
/// judge rather than counted as a failure.
fn find_load_bearing(prose: &str) -> Vec<String> {
    prose
        .split("\n\n")
        .flat_map(sentences)
        .filter(|s| s.to_ascii_lowercase().contains("load-bearing"))
        .map(|s| truncate(s))
        .collect()
}

/// `— … —` as a parenthetical. He uses a single spaced em dash, and parentheses
/// for asides; the paired form is a model habit.
fn find_paired_em_dash(prose: &str) -> Vec<String> {
    let mut out = Vec::new();
    for sentence in prose.split("\n\n").flat_map(sentences) {
        let dashes: Vec<usize> = sentence
            .char_indices()
            .filter(|(_, c)| *c == '—')
            .map(|(i, _)| i)
            .collect();
        for pair in dashes.windows(2) {
            let inner = &sentence[pair[0]..pair[1]];
            if inner.chars().count() > 3 && inner.chars().count() < 80 {
                out.push(truncate(sentence));
                break;
            }
        }
    }
    out
}

fn truncate(s: &str) -> String {
    let limit = 90;
    if s.chars().count() <= limit {
        return s.to_string();
    }
    let cut: String = s.chars().take(limit).collect();
    format!("{cut}…")
}

/// Read every published article except `exclude`, which is the draft under test
/// when it already lives in `content/articles/`. Including it would let a drift
/// widen the band it is being checked against.
pub fn corpus_metrics(
    dir: &Path,
    exclude: Option<&Path>,
) -> std::io::Result<Vec<(PathBuf, Metrics)>> {
    let excluded = exclude.and_then(|p| p.canonicalize().ok());
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    entries.sort();
    let mut out = Vec::new();
    for path in entries {
        if path.canonicalize().ok() == excluded {
            continue;
        }
        let source = std::fs::read_to_string(&path)?;
        out.push((path, measure(&source)));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARTICLE: &str = "+++\ntitle = \"T\"\ndate = \"2026-01-01\"\n+++\n\n\
First line of a paragraph that is\nhard wrapped across two lines.\n\n\
## A heading\n\n\
> A quote from somebody else entirely.\n\n\
Second paragraph with `inline_code` and a [link](https://example.com).\n\n\
- a list item\n\n\
| a | table |\n\n\
```rust\nfn main() {}\n```\n\n\
<figure><svg><text>drawing</text></svg><figcaption>A caption.</figcaption></figure>\n\n\
Third paragraph is *emphasised* and has snake_case in it.\n";

    #[test]
    fn strips_everything_that_is_not_his_prose() {
        let prose = strip(ARTICLE);
        for banned in [
            "title =",         // front matter
            "A heading",       // heading
            "somebody else",   // blockquote
            "list item",       // list
            "table",           // table row
            "fn main",         // code fence
            "drawing",         // svg
            "A caption",       // figcaption
            "https://example", // link target
        ] {
            assert!(
                !prose.contains(banned),
                "{banned:?} survived stripping:\n{prose}"
            );
        }
    }

    #[test]
    fn keeps_prose_and_unwraps_it() {
        let prose = strip(ARTICLE);
        assert!(prose.contains("First line of a paragraph that is hard wrapped across two lines."));
        assert!(
            prose.contains("inline_code"),
            "inline code text is part of the sentence"
        );
        assert!(prose.contains("a link"), "link text is prose");
        assert!(prose.contains("emphasised"));
        assert!(
            prose.contains("snake_case"),
            "an identifier is not emphasis"
        );
        assert_eq!(prose.split("\n\n").count(), 3, "three paragraphs:\n{prose}");
    }

    #[test]
    fn a_heading_does_not_weld_two_paragraphs_together() {
        let prose = strip("+++\nt = 1\n+++\nOne.\n\n## H\n\nTwo.\n");
        assert_eq!(
            prose.split("\n\n").collect::<Vec<_>>(),
            vec!["One.", "Two."]
        );
    }

    #[test]
    fn sentence_splitting_survives_technical_prose() {
        let cases: [(&str, usize); 6] = [
            ("One sentence only", 1),
            ("First. Second. Third.", 3),
            ("See RFC 9146, e.g. the record layer. Then stop.", 2),
            ("It answered at 127.0.0.1/8080 and stopped.", 1),
            ("He asked \"is that planned?\" Nobody answered.", 2),
            ("OCPP 1.6 is explicit about it.", 1),
        ];
        for (input, expected) in cases {
            assert_eq!(sentences(input).len(), expected, "splitting {input:?}");
        }
    }

    #[test]
    fn measures_the_five_signals() {
        // Two paragraphs: one of two sentences, one of a single sentence.
        let m = measure("+++\nt = 1\n+++\nWe own our thing. It is ours.\n\nOne line here: yes.\n");
        assert_eq!(m.paragraphs, 2);
        assert_eq!(m.sentences, 3);
        assert_eq!(m.single_sentence_paragraph_pct, 50.0);
        // we, our, ours => 3 of 11 words.
        assert_eq!(m.words, 11);
        assert!((m.first_plural_per_10k - 3.0 / 11.0 * 10_000.0).abs() < 0.01);
        assert!((m.colons_per_1k - 1.0 / 11.0 * 1_000.0).abs() < 0.01);
        assert!((m.sentences_per_paragraph - 1.5).abs() < 1e-9);
    }

    #[test]
    fn empty_input_does_not_divide_by_zero() {
        let m = measure("+++\nt = 1\n+++\n");
        assert_eq!(m.words, 0);
        assert_eq!(m.first_plural_per_10k, 0.0);
        assert_eq!(m.words_per_paragraph, 0.0);
        assert_eq!(m.single_sentence_paragraph_pct, 0.0);
    }

    #[test]
    fn band_covers_the_whole_range_and_pads_outward() {
        let tight = [10.0, 11.0, 12.0, 10.5, 11.5];
        let b = band(&tight);
        assert!(b.low < 10.0 && b.high > 12.0, "padded outward: {b}");
        assert!((b.median - 11.0).abs() < 1e-9);
        assert!(b.low >= 0.0, "a rate cannot be negative: {b}");
    }

    #[test]
    fn band_admits_its_own_outlier_and_the_cost_is_a_wider_ceiling() {
        // One article uses `we` at 102 per 10k words against a corpus median
        // near zero. Admitting it is the deliberate trade: the band has to pass
        // everything he has published, and generic prose misses this
        // measurement by so much (166 per 10k) that the wider ceiling still
        // catches it. `--calibrate` names the outlier rather than trimming it
        // out of the edge, where it would stop being visible.
        let with_outlier = [0.0, 0.0, 1.0, 2.0, 0.0, 0.0, 3.0, 0.0, 102.0];
        let b = band(&with_outlier);
        assert!(b.high >= 102.0, "must admit his own article: {b}");
        assert!(b.high < 166.0, "but must still reject generic prose: {b}");
    }

    /// The invariant the tool exists for: a band built from corpus-shaped
    /// values must admit corpus-shaped prose and reject the shape generic
    /// technical prose actually has. The generic figures are the measured means
    /// of four pieces written on these topics with no knowledge of the voice;
    /// the corpus figures are this repo's published articles. If a change to
    /// `PAD_SD` or to `band` ever lets the generic column through, this fails.
    #[test]
    fn the_band_separates_his_prose_from_generic_prose() {
        // (corpus values across articles, his typical, generic prose)
        let cases: [(&[f64], f64, f64); 5] = [
            (
                &[0.0, 0.0, 2.7, 3.8, 3.9, 5.1, 5.7, 6.1, 8.6, 39.8, 101.9],
                3.9,
                166.2,
            ),
            (
                &[
                    2.17, 2.19, 2.24, 2.28, 2.32, 2.35, 2.38, 2.45, 2.57, 2.59, 3.04,
                ],
                2.4,
                3.91,
            ),
            (
                &[
                    22.0, 27.1, 27.4, 29.7, 30.3, 32.4, 36.1, 37.6, 37.7, 38.9, 49.6,
                ],
                36.1,
                65.8,
            ),
            (
                &[
                    17.9, 22.5, 24.3, 24.5, 35.4, 37.2, 38.0, 38.1, 38.3, 38.5, 48.4,
                ],
                38.0,
                0.0,
            ),
            (
                &[
                    7.6, 9.1, 10.2, 10.6, 11.9, 13.0, 14.6, 14.6, 15.4, 15.8, 20.0,
                ],
                14.6,
                3.3,
            ),
        ];
        let inside = |b: &Band, v: f64| v >= b.low && v <= b.high;
        let mut generic_caught = 0;
        for (corpus, typical, generic) in cases {
            let b = band(corpus);
            assert!(
                inside(&b, typical),
                "his own typical value {typical} outside {b}"
            );
            if !inside(&b, generic) {
                generic_caught += 1;
            }
        }
        assert!(
            generic_caught >= 4,
            "band must catch at least four of the five generic values, caught {generic_caught}"
        );
    }

    /// The committed file has to parse, cover every checked metric, and still
    /// describe the corpus. A band that no longer matches what the articles
    /// measure is the one failure mode storing them introduces, so it is pinned
    /// here rather than left to be noticed.
    #[test]
    fn stored_bands_parse_and_still_match_the_corpus() {
        let stored = stored_bands().expect("bands.txt must parse");
        assert_eq!(stored.len(), CHECKED.len());
        for (b, (label, _, _)) in stored.iter().zip(CHECKED.iter()) {
            assert!(
                b.low <= b.median && b.median <= b.high,
                "{label}: incoherent {b}"
            );
        }

        // Tests run with the crate root as the working directory, so the corpus
        // is two levels up. Skipped rather than failed if it is not there: the
        // crate should still be testable on its own.
        let corpus = Path::new("../../content/articles");
        if !corpus.is_dir() {
            return;
        }
        let measured = corpus_metrics(corpus, None).expect("corpus readable");
        for ((label, get, _), st) in CHECKED.iter().zip(&stored) {
            let live = band(&measured.iter().map(|(_, m)| get(m)).collect::<Vec<_>>());
            assert!(
                (live.low - st.low).abs() < 0.05 && (live.high - st.high).abs() < 0.05,
                "{label}: stored {st} but corpus now measures {live}. \
                 Regenerate with `--calibrate --write` and commit the diff."
            );
        }
    }

    #[test]
    fn stored_bands_reject_a_malformed_line() {
        // The parser must not silently skip what it cannot read: a dropped
        // metric would mean a band that is never checked.
        assert!(
            CHECKED
                .iter()
                .all(|(label, _, _)| BANDS_FILE.contains(label))
        );
    }

    #[test]
    fn band_of_one_value_is_that_value() {
        let b = band(&[7.0]);
        assert_eq!((b.low, b.high, b.median), (7.0, 7.0, 7.0));
    }

    #[test]
    fn tics_are_found_and_quoted() {
        let hits = |name: &str, text: &str| {
            let f = TICS.iter().find(|(n, _)| *n == name).unwrap().1;
            f(text)
        };
        assert_eq!(
            hits("worth <gerund>", "That is worth naming properly."),
            vec!["worth naming"]
        );
        assert!(hits("worth <gerund>", "The trip was worth it.").is_empty());
        assert!(
            hits("worth <gerund>", "It is worth king.").is_empty(),
            "short word is not a gerund"
        );

        assert_eq!(
            hits("stage direction", "Notice how the clock drifts.").len(),
            1
        );
        assert_eq!(
            hits("stage direction", "Hold that number for a moment.").len(),
            1
        );
        assert!(
            hits(
                "stage direction",
                "Consider what a fleet operator actually has."
            )
            .is_empty(),
            "introducing evidence is not pointing at the prose"
        );
        assert!(hits("stage direction", "A reconciler must hold the deadline.").is_empty());
        assert!(
            hits("stage direction", "Read it.").is_empty(),
            "two words is not a direction"
        );

        assert_eq!(
            hits(
                "\"Here is\" opener",
                "Here is the problem. And here is more."
            )
            .len(),
            1
        );
        assert_eq!(
            hits("\"load-bearing\"", "That word is load-bearing.").len(),
            1
        );

        assert_eq!(
            hits("paired em dash (— … —)", "The fix — a small one — worked.").len(),
            1
        );
        assert!(
            hits("paired em dash (— … —)", "The fix worked — eventually.").is_empty(),
            "a single dash is his own punctuation"
        );
    }

    #[test]
    fn tic_scan_ignores_quoted_and_fenced_material() {
        // A tic inside a blockquote belongs to whoever is being quoted.
        let prose =
            strip("+++\nt = 1\n+++\n> Here is somebody else's sentence.\n\nMine is fine.\n");
        let f = TICS
            .iter()
            .find(|(n, _)| *n == "\"Here is\" opener")
            .unwrap()
            .1;
        assert!(f(&prose).is_empty());
    }
}
