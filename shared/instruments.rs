//! The instrument registry, shared by the wasm bundle that mounts instruments
//! and the generator that builds the lab page.
//!
//! Two surfaces have to agree about which instruments exist: `mount_all` needs
//! a slug to dispatch on, and the lab page needs something to advertise. When
//! those lists live apart they drift, and the failure mode is a page promising
//! a figure that never appears. Keeping the metadata here and asserting the
//! dispatcher covers it (see `instrument::tests`) makes that drift a test
//! failure rather than a blank rectangle.
//!
//! Adding an instrument is two edits: an entry here, and a match arm in
//! `mount_all` that names its component. Rust cannot make the second one data,
//! so the test is what holds them together.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Instrument {
    /// The `data-instrument` attribute value. Also the lab page anchor.
    pub slug: &'static str,
    pub title: &'static str,
    /// What this instrument shows that a static diagram structurally cannot.
    /// This is the bar for existing at all, so every entry has to answer it.
    pub shows: &'static str,
    /// The article that embeds it, or `None` while it is still only a toy on
    /// the bench. Unattached instruments are the point of the lab.
    pub article: Option<&'static str>,
}

pub const INSTRUMENTS: [Instrument; 4] = [
    Instrument {
        slug: "dtls-cid",
        title: "DTLS connection identifier",
        shows: "That anchoring session state on an address costs a full handshake per NAT rebinding — and that a connection identifier alone is not enough, because the binding must not move until the path answers back.",
        article: Some("/articles/the-lookup-that-saves-the-handshake/"),
    },
    Instrument {
        slug: "sram",
        title: "2 KB of SRAM, no MMU",
        shows: "A failure that raises nothing at all: malloc refuses politely at the margin while the stack writes straight through the heap in silence, and unwinding does not repair it.",
        article: Some("/articles/two-kilobytes-and-no-operating-system/"),
    },
    Instrument {
        slug: "isolated",
        title: "Isolated project configuration",
        shows: "That the speedup is a restriction rather than an optimisation — a cross-project reference does not get slower under isolation, it stops being legal.",
        article: Some("/articles/performance-is-making-gradle-less-gradle/"),
    },
    Instrument {
        slug: "partitioning",
        title: "Key partitioning and the hash ring",
        shows: "What a partition count change costs. Kafka's default partitioner relocates almost every key and voids per-key ordering; a hash ring relocates close to the 1/n floor. The two are the same mechanism with different assignment functions, which is only visible side by side.",
        article: None,
    },
];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::INSTRUMENTS;

    /// The slug is a DOM attribute value and a URL fragment at once, so a
    /// duplicate would mount two components into one host and break the lab
    /// page's anchors in the same stroke.
    #[test]
    fn instrument_slugs_are_unique_and_url_safe() {
        let slugs: HashSet<_> = INSTRUMENTS.iter().map(|i| i.slug).collect();
        assert_eq!(slugs.len(), INSTRUMENTS.len());
        for instrument in INSTRUMENTS {
            assert!(!instrument.slug.is_empty());
            assert!(
                instrument
                    .slug
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c == '-'),
                "{} is not url-safe",
                instrument.slug
            );
        }
    }

    /// Every entry has to justify itself against the bar, and any article it
    /// claims has to be a real absolute route rather than a relative guess.
    #[test]
    fn every_instrument_states_what_it_shows_and_links_absolutely() {
        for instrument in INSTRUMENTS {
            assert!(
                instrument.shows.len() > 40,
                "{} does not say what it shows",
                instrument.slug
            );
            assert!(!instrument.title.is_empty());
            if let Some(article) = instrument.article {
                assert!(article.starts_with("/articles/"), "{article}");
                assert!(article.ends_with('/'), "{article}");
            }
        }
    }
}
