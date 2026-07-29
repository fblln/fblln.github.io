//! Portfolio project data and the pure filtering rule used by the work list.
//!
//! Keeping content and its query predicate together means the view only decides
//! how to present projects; the search contract remains independently testable.

#[derive(Clone, Copy)]
pub(crate) struct Project {
    pub(crate) slug: &'static str,
    pub(crate) name: &'static str,
    pub(crate) category: &'static str,
    pub(crate) stack: &'static str,
    pub(crate) statement: &'static str,
    pub(crate) evidence: &'static str,
    pub(crate) detail: &'static str,
    pub(crate) metric: &'static str,
    pub(crate) metric_label: &'static str,
    pub(crate) image: &'static str,
    pub(crate) image_alt: &'static str,
    pub(crate) repo: &'static str,
    pub(crate) tags: &'static [&'static str],
}

pub(crate) const PROJECTS: [Project; 4] = [
    Project {
        slug: "slim2m",
        name: "slim2m",
        category: "Embedded",
        stack: "RUST · LWM2M · COAP · NO_STD",
        statement: "Run an LwM2M client from Linux-class devices down to bare-metal microcontrollers.",
        evidence: "",
        detail: "slim2m separates the LwM2M and CoAP state machines from sockets, clocks, entropy, and storage. The core performs no I/O and owns no payload buffers; the hosted runtime connects it to UDP and DTLS today, with no-OS runtime support planned.",
        metric: "0",
        metric_label: "heap allocations in the core",
        image: "/assets/slim2m-protocol-map.webp",
        image_alt: "Detailed protocol map showing a sensor, microcontroller, gateway, and server connected by packet flows above a four-state device lifecycle",
        repo: "https://github.com/fblln/slim2m",
        tags: &["Rust", "Embedded", "LwM2M", "no_std"],
    },
    Project {
        slug: "ridgeline",
        name: "Ridgeline",
        category: "Geospatial",
        stack: "RUST · WASM · GDAL · THREE.JS",
        statement: "Transform a GPX route and elevation data into an interactive terrain artifact.",
        evidence: "A real asset pipeline for DEM sampling, projected geometry, terrain textures, route replay, and 7200×5400 export.",
        detail: "Ridgeline combines a high-performance geospatial pipeline with a browser-native viewer. Instead of hiding the work, it exposes each stage: GPX parsing, DEM acquisition, sampling, relief, slope, forest layers, and final asset handoff.",
        metric: "6.7×",
        metric_label: "warm-cache CPU vs the Python baker, same GDAL underneath",
        image: "/assets/ridgeline.webp",
        image_alt: "Ridgeline 3D terrain visualization",
        repo: "https://github.com/fblln/ridgeline",
        tags: &["Rust", "WASM", "Terrain", "GPX"],
    },
    Project {
        slug: "lithograph",
        name: "Lithograph",
        category: "Code Intelligence",
        stack: "RUST · TREE-SITTER · CYPHER · MCP",
        statement: "Turn a source tree into a queryable, evidence-backed architecture graph.",
        evidence: "29 typed relations and 25 MCP tools over a versioned graph snapshot, with hybrid semantic retrieval and incremental reindexing.",
        detail: "No model key required: scans, graph construction, and every query run offline and reproducible. `update` reindexes only what changed, and each generated page stays tied to source evidence rather than a prompt.",
        metric: "29",
        metric_label: "typed relation kinds",
        image: "",
        image_alt: "",
        repo: "https://github.com/fblln/Lithograph",
        tags: &["Rust", "Graphs", "Local-first", "Code Intelligence"],
    },
    Project {
        slug: "race-telemetry",
        name: "Race Telemetry Workbench",
        category: "Telemetry",
        stack: ".NET · MCP · AG-UI · TIMESCALEDB",
        statement: "Serve a race session as a time-series store that both charts and agents can query.",
        evidence: "TimescaleDB behind typed REST for the views and MCP for the agent, with answers streamed back over SSE.",
        detail: "Seven analysis primitives are compiled to hypertable queries and exposed twice—REST to the desktop, MCP to the agent—so a question never pulls raw telemetry across the wire.",
        metric: "7",
        metric_label: "primitives, one query layer",
        image: "/assets/race-telemetry-ai.webp",
        image_alt: "A streaming assistant comparing the pit strategies of the top three finishers, beside a generated race summary with winner, fastest lap, tyre stints, and race control timeline",
        repo: "https://github.com/fblln/race-telemetry-workbench",
        tags: &["F1", "MCP", "Agents", "OpenTelemetry"],
    },
];

pub(crate) const CATEGORIES: [&str; 5] = [
    "All",
    "Telemetry",
    "Code Intelligence",
    "Geospatial",
    "Embedded",
];

/// A project matches when its category is selected and the query is present in
/// user-facing discovery fields. Case-folding once preserves predictable search.
pub(crate) fn matches(project: Project, category: &str, query: &str) -> bool {
    let category_match = category == "All" || project.category == category;
    let query = query.trim().to_ascii_lowercase();
    let query_match = query.is_empty()
        || project.name.to_ascii_lowercase().contains(&query)
        || project.statement.to_ascii_lowercase().contains(&query)
        || project.stack.to_ascii_lowercase().contains(&query)
        || project
            .tags
            .iter()
            .any(|tag| tag.to_ascii_lowercase().contains(&query));
    category_match && query_match
}

#[cfg(test)]
mod tests {
    use super::{PROJECTS, matches};

    #[test]
    fn project_slugs_are_unique() {
        for (index, project) in PROJECTS.iter().enumerate() {
            assert!(
                PROJECTS
                    .iter()
                    .skip(index + 1)
                    .all(|other| other.slug != project.slug)
            );
        }
    }

    #[test]
    fn every_project_has_a_repository() {
        for project in PROJECTS {
            assert!(project.repo.starts_with("https://github.com/fblln/"));
        }
    }

    #[test]
    fn filtering_covers_category_stack_and_tags() {
        assert!(matches(PROJECTS[2], "Code Intelligence", "tree-sitter"));
        assert!(matches(PROJECTS[1], "All", "wasm"));
        assert!(matches(PROJECTS[0], "Embedded", "lwm2m"));
        assert!(!matches(PROJECTS[3], "Embedded", "telemetry"));
    }
}
