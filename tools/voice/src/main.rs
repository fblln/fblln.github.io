//! `voice` — check an article draft against the voice of the published corpus.
//!
//! Usage:
//!   voice <draft.md>                    check a draft against the stored bands
//!   voice <draft.md> --recalibrate      check against the corpus as it is now
//!   voice --calibrate [--write]         show the bands, or regenerate bands.txt
//!
//! The output is written to be read by an agent mid-draft: strongest signal
//! first, every failure carries the direction to move in, and tic hits are
//! quoted so the fix is a deletion rather than a search. Exit status is the
//! gate — 0 passes, 1 drifts, 2 could not run.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use voice::{
    Band, CHECKED, Metrics, TICS, band, corpus_metrics, measure, render_bands, stored_bands, strip,
};

const DEFAULT_CORPUS: &str = "content/articles";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut corpus = PathBuf::from(DEFAULT_CORPUS);
    let mut draft: Option<PathBuf> = None;
    let mut calibrate = false;
    let mut write = false;
    let mut recalibrate = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--calibrate" => calibrate = true,
            "--write" => write = true,
            "--recalibrate" => recalibrate = true,
            "--corpus" => match args.get(i + 1) {
                Some(dir) => {
                    corpus = PathBuf::from(dir);
                    i += 1;
                }
                None => return fail("--corpus needs a directory"),
            },
            "-h" | "--help" => {
                println!("{}", include_str!("usage.txt"));
                return ExitCode::SUCCESS;
            }
            other if other.starts_with('-') => return fail(&format!("unknown flag {other}")),
            path => draft = Some(PathBuf::from(path)),
        }
        i += 1;
    }

    // Checking against the stored bands needs no corpus on disk; everything
    // else reads it.
    if (calibrate || recalibrate) && !corpus.is_dir() {
        return fail(&format!(
            "no corpus at {}; run from the repository root or pass --corpus",
            corpus.display()
        ));
    }
    match (calibrate, draft) {
        (true, _) => run_calibrate(&corpus, write),
        (false, Some(path)) => run_check(&corpus, &path, recalibrate),
        (false, None) => fail("give a draft to check, or --calibrate"),
    }
}

fn fail(message: &str) -> ExitCode {
    eprintln!("voice: {message}");
    ExitCode::from(2)
}

/// Bands for the five checked metrics, measured over the corpus.
fn bands(reference: &[(PathBuf, Metrics)]) -> Vec<Band> {
    CHECKED
        .iter()
        .map(|(_, get, _)| band(&reference.iter().map(|(_, m)| get(m)).collect::<Vec<_>>()))
        .collect()
}

fn run_check(corpus: &Path, draft: &Path, recalibrate: bool) -> ExitCode {
    let source = match std::fs::read_to_string(draft) {
        Ok(s) => s,
        Err(e) => return fail(&format!("cannot read {}: {e}", draft.display())),
    };

    // Stored bands by default. A recomputed band cannot fail — publish one
    // article that drifts and the edge moves to admit the next one — so moving
    // it is a committed edit, and `--recalibrate` is the escape hatch for
    // asking what the corpus says today.
    let (bands, provenance) = if recalibrate {
        // The draft is excluded from its own reference set when it already
        // lives in the corpus directory, so an edit cannot widen its own band.
        let reference = match corpus_metrics(corpus, Some(draft)) {
            Ok(r) if !r.is_empty() => r,
            Ok(_) => return fail("corpus has no other articles to calibrate against"),
            Err(e) => return fail(&format!("cannot read corpus: {e}")),
        };
        let n = reference.len();
        (
            bands(&reference),
            format!(
                "recalibrated against {n} articles in {} (not the stored bands)",
                corpus.display()
            ),
        )
    } else {
        match stored_bands() {
            Ok(b) => (b, "stored bands (tools/voice/bands.txt)".to_string()),
            Err(e) => return fail(&e),
        }
    };

    let m = measure(&source);
    println!(
        "{} — {} words, {} paragraphs, {} sentences",
        draft.display(),
        m.words,
        m.paragraphs,
        m.sentences
    );
    println!("{provenance}\n");

    let mut drifted = 0usize;
    for ((label, get, signal), b) in CHECKED.iter().zip(&bands) {
        let value = get(&m);
        let inside = value >= b.low && value <= b.high;
        if !inside {
            drifted += 1;
        }
        let verdict = if inside {
            "ok  ".to_string()
        } else if value > b.high {
            // Two decimals: a miss of 0.03 printed as "HIGH by 0.0" reads like
            // a bug in the tool rather than a marginal draft.
            format!("HIGH by {:.2}", value - b.high)
        } else {
            format!("LOW by {:.2}", b.low - value)
        };
        println!("  {verdict:<14} {label:<30} {value:7.1}   band {b}   [{signal}]");
    }

    let prose = strip(&source);
    let mut tic_hits = 0usize;
    let mut tic_lines = Vec::new();
    for (name, find) in TICS.iter() {
        for hit in find(&prose) {
            tic_hits += 1;
            tic_lines.push(format!("  {name:<24} {hit}"));
        }
    }
    if tic_hits > 0 {
        println!("\nbanned constructions ({tic_hits}):");
        for line in &tic_lines {
            println!("{line}");
        }
    }

    if m.words < 400 {
        println!(
            "\nnote: {} words. These rates are noisy under about 400 words; treat a single \
             failure on a short draft as directional.",
            m.words
        );
    }

    println!();
    if drifted == 0 && tic_hits == 0 {
        println!(
            "pass: inside the corpus band on all {} measurements",
            CHECKED.len()
        );
        return ExitCode::SUCCESS;
    }
    println!(
        "drift: {drifted} of {} measurements outside the band, {tic_hits} banned construction(s)",
        CHECKED.len()
    );
    if drifted > 0 {
        println!(
            "A band is his own published range, not a target to hit exactly. Move toward the \
             median; do not overshoot past it."
        );
    }
    ExitCode::FAILURE
}

/// Print the bands and every article's value against them.
///
/// This is the mode that keeps the tool honest. An article of his own falling
/// outside its own band means the band is too tight, not that the article is
/// wrong, and it is the signal to revisit the padding rule rather than the
/// prose. Tic hits in the older articles are expected: they predate the
/// Aug-2026 editing pass that produced the ban-list.
fn run_calibrate(corpus: &Path, write: bool) -> ExitCode {
    let reference = match corpus_metrics(corpus, None) {
        Ok(r) if !r.is_empty() => r,
        Ok(_) => return fail(&format!("no articles in {}", corpus.display())),
        Err(e) => return fail(&format!("cannot read corpus: {e}")),
    };
    let bands = bands(&reference);

    if write {
        let path = Path::new("tools/voice/bands.txt");
        let rendered = render_bands(corpus, &reference);
        if let Err(e) = std::fs::write(path, &rendered) {
            return fail(&format!("cannot write {}: {e}", path.display()));
        }
        println!("wrote {} ({} articles)", path.display(), reference.len());
        println!("Rebuild to compile it in; commit the diff so the move is reviewable.");
        return ExitCode::SUCCESS;
    }

    // Say whether the committed bands still describe the corpus. This is the
    // whole value of storing them: the gate only moves when somebody moves it,
    // and this line is how you find out that it is due.
    match stored_bands() {
        Ok(stored) => {
            let moved: Vec<&str> = CHECKED
                .iter()
                .zip(&bands)
                .zip(&stored)
                .filter(|((_, live), st)| {
                    (live.low - st.low).abs() > 0.05 || (live.high - st.high).abs() > 0.05
                })
                .map(|(((label, _, _), _), _)| *label)
                .collect();
            if moved.is_empty() {
                println!("stored bands match the corpus.\n");
            } else {
                println!(
                    "stored bands are out of date on: {}\n  regenerate with --calibrate --write\n",
                    moved.join(", ")
                );
            }
        }
        Err(e) => println!("stored bands unreadable ({e}); showing live values only.\n"),
    }

    println!("{} articles in {}\n", reference.len(), corpus.display());
    for ((label, _, signal), b) in CHECKED.iter().zip(&bands) {
        println!("  {label:<30} band {b}   [{signal}]");
    }

    println!("\nper article:");
    let name = |p: &Path| {
        p.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    };
    for (path, m) in &reference {
        let outside: Vec<&str> = CHECKED
            .iter()
            .zip(&bands)
            .filter(|((_, get, _), b)| {
                let v = get(m);
                v < b.low || v > b.high
            })
            .map(|((label, _, _), _)| *label)
            .collect();
        let prose = strip(&std::fs::read_to_string(path).unwrap_or_default());
        let tics: usize = TICS.iter().map(|(_, find)| find(&prose).len()).sum();
        let mut notes = Vec::new();
        if !outside.is_empty() {
            notes.push(format!("outside own band: {}", outside.join(", ")));
        }
        if tics > 0 {
            notes.push(format!("{tics} banned construction(s)"));
        }
        println!(
            "  {:<44} {:5}w  we/10k {:6.1}  s/para {:4.2}  w/para {:5.1}  1-sent {:5.1}%  colon/1k {:5.1}{}",
            name(path),
            m.words,
            m.first_plural_per_10k,
            m.sentences_per_paragraph,
            m.words_per_paragraph,
            m.single_sentence_paragraph_pct,
            m.colons_per_1k,
            if notes.is_empty() {
                String::new()
            } else {
                format!("\n      ^ {}", notes.join("; "))
            }
        );
    }
    ExitCode::SUCCESS
}
