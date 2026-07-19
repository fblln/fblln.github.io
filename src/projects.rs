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
        slug: "race-telemetry",
        name: "Race Telemetry Workbench",
        category: "Telemetry",
        stack: ".NET · TIMESCALEDB · MCP · AI",
        statement: "Turn raw Formula 1 telemetry into engineering and race-strategy insight.",
        evidence: "A local-first analysis platform with replay, strategy, race control, head-to-head comparison, and an MCP-backed AI surface.",
        detail: "FastF1 data is imported into TimescaleDB and exposed through typed .NET query primitives. The same bounded contracts power an engineering desktop and autonomous analysis through MCP—keeping natural-language answers grounded in real session data.",
        metric: "42.7×",
        metric_label: "raw telemetry compression",
        image: "/assets/race-telemetry.webp",
        image_alt: "Race Telemetry Workbench replay interface",
        repo: "https://github.com/fblln/race-telemetry-workbench",
        tags: &["F1", "PostgreSQL", "OpenTelemetry", "Agents"],
    },
    Project {
        slug: "lithograph",
        name: "Lithograph",
        category: "Code Intelligence",
        stack: "RUST · TREE-SITTER · LADYBUGDB · MCP",
        statement: "Turn a source tree into a queryable, evidence-backed architecture graph.",
        evidence: "29 typed relations, 25 MCP tools, hybrid language resolution, drift detection, ADRs, and offline-first architecture documentation.",
        detail: "Lithograph treats repository understanding as a deterministic systems problem rather than a prompt. It combines syntax extraction, typed graph construction, search, architecture analysis, evidence tracking, incremental regeneration, and optional model augmentation.",
        metric: "29",
        metric_label: "typed relation kinds",
        image: "",
        image_alt: "",
        repo: "https://github.com/fblln/Lithograph",
        tags: &["Rust", "Graphs", "Local-first", "Code Intelligence"],
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
        metric_label: "warm-cache compute speedup",
        image: "/assets/ridgeline.webp",
        image_alt: "Ridgeline 3D terrain visualization",
        repo: "https://github.com/fblln/ridgeline",
        tags: &["Rust", "WASM", "Terrain", "GPX"],
    },
    Project {
        slug: "apexline",
        name: "Apexline",
        category: "Research",
        stack: "PYTHON · FASTF1 · GEOMETRY · POLYLINES",
        statement: "Prove whether a telemetry lap has the same shape as an oracle circuit.",
        evidence: "26,689 race laps inspected across 24 circuits with auditable recovery, rejection, fitting, and compact polyline output.",
        detail: "Apexline normalizes lap-boundary overlap, rejects invalid evidence, fits closed paths without arbitrary warping, and reports residuals that explain whether each lap is useful, recoverable, suspicious, or invalid.",
        metric: "93.7%",
        metric_label: "2025 laps classified good",
        image: "/assets/apexline.svg",
        image_alt: "Apexline Canadian Grand Prix geometry diagnostics",
        repo: "https://github.com/fblln/apexline",
        tags: &["Geometry", "F1", "Validation", "Compression"],
    },
];

pub(crate) const CATEGORIES: [&str; 5] = [
    "All",
    "Telemetry",
    "Code Intelligence",
    "Geospatial",
    "Research",
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
    fn every_project_has_evidence_and_a_repository() {
        for project in PROJECTS {
            assert!(!project.evidence.is_empty());
            assert!(project.repo.starts_with("https://github.com/fblln/"));
        }
    }

    #[test]
    fn filtering_covers_category_stack_and_tags() {
        assert!(matches(PROJECTS[1], "Code Intelligence", "tree-sitter"));
        assert!(matches(PROJECTS[2], "All", "wasm"));
        assert!(!matches(PROJECTS[0], "Research", "telemetry"));
    }
}
