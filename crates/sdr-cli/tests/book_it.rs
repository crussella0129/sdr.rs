//! Book-integrity tests.
//!
//! The Project Book under `docs/` is the project's semantic authority, and its
//! structural invariants rot silently as it grows: a chapter added without a
//! navigation link, or missing from the roadmap, is invisible until someone
//! notices it is missing. These tests make that failure loud.
//!
//! Each test parses `docs/` at run time rather than asserting against a
//! snapshot, so it reflects the Book's real state.
//!
//! Precedent for documentation tests in this workspace:
//! `crates/sdr-mesh/tests/readme_it.rs` and
//! `crates/sdr-station/tests/cloudlog_it.rs`. These live in `sdr-cli` because
//! they assert Book-wide rather than crate-local facts.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Repository root, derived from this crate's manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("resolve repository root")
}

fn read(rel: &str) -> String {
    let path = repo_root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// Every `INT-NNNN` id that has a chapter on disk.
fn intent_ids() -> BTreeSet<String> {
    let dir = repo_root().join("docs").join("intents");
    let mut ids = BTreeSet::new();
    for entry in std::fs::read_dir(&dir).expect("read docs/intents") {
        let name = entry
            .expect("dir entry")
            .file_name()
            .to_string_lossy()
            .to_string();
        // Chapters are `INT-NNNN-<slug>.md`; README.md and anything else is not.
        if name.starts_with("INT-") && name.ends_with(".md") && name.len() > 8 {
            ids.insert(name[..8].to_string());
        }
    }
    assert!(!ids.is_empty(), "no intent chapters found — wrong root?");
    ids
}

#[test]
fn test_every_intent_is_reachable_from_summary() {
    let summary = read("docs/SUMMARY.md");
    let missing: Vec<String> = intent_ids()
        .into_iter()
        .filter(|id| !summary.contains(id.as_str()))
        .collect();

    assert!(
        missing.is_empty(),
        "these intent chapters exist but are not linked from docs/SUMMARY.md, \
         so they are unreachable when the Book is read as a document: {missing:?}"
    );
}

#[test]
fn test_every_intent_has_acceptance_criteria() {
    // `check-book.sh` validates chapter structure but does NOT check that
    // acceptance criteria are present, so without this a chapter could ship
    // with an empty section and still be reported valid.
    let mut empty = Vec::new();
    for id in intent_ids() {
        let dir = repo_root().join("docs").join("intents");
        let path = std::fs::read_dir(&dir)
            .expect("read docs/intents")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| {
                p.file_name()
                    .map(|n| n.to_string_lossy().starts_with(&id))
                    .unwrap_or(false)
            })
            .unwrap_or_else(|| panic!("chapter file for {id}"));

        let body = std::fs::read_to_string(&path).expect("read chapter");
        let after = body
            .split_once("## Acceptance criteria")
            .map(|(_, rest)| rest)
            .unwrap_or("");
        // Content up to the next section heading.
        let section = after.split("\n## ").next().unwrap_or("");
        if section.trim().is_empty() {
            empty.push(id);
        }
    }

    assert!(
        empty.is_empty(),
        "these chapters have a missing or empty `## Acceptance criteria` section, \
         so nothing observable proves the intent: {empty:?}"
    );
}

#[test]
fn test_adopted_categories_are_not_listed_as_candidates() {
    let readme = read("docs/README.md");

    assert!(
        !readme.contains("Candidate categories"),
        "all four candidate categories were adopted as intents in Sprint 8, so the \
         Book README must no longer carry a 'Candidate categories' section"
    );

    // Adopting them means they appear in the taxonomy proper.
    for id in ["INT-0014", "INT-0015", "INT-0016", "INT-0017"] {
        assert!(
            readme.contains(id),
            "{id} was adopted but does not appear in the Book README taxonomy"
        );
    }
}

/// The roadmap's dependency map, which carries exactly one row per intent.
///
/// The locked plan phrased this as "exactly once in the phase listing". The
/// dependency map is the structure that actually provides a canonical one-entry
/// -per-intent index: the phase listing is forward-looking and deliberately
/// omits already-realized chapters, so it cannot hold every intent exactly once
/// without becoming misleading. Asserting against the table verifies what the
/// clause is for — total coverage, no duplicates, no omissions.
fn dependency_map(roadmap: &str) -> &str {
    let start = roadmap
        .find("## Dependency map")
        .expect("roadmap must have a `## Dependency map` section");
    let rest = &roadmap[start..];
    let end = rest.find("## Standing constraints").unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn test_roadmap_covers_every_intent_exactly_once() {
    let roadmap = read("docs/roadmap.md");
    let map = dependency_map(&roadmap);

    let mut problems = Vec::new();
    for id in intent_ids() {
        // One table row per intent, each beginning `| [INT-NNNN](`.
        let row_marker = format!("| [{id}](");
        let rows = map.matches(&row_marker).count();
        if rows != 1 {
            problems.push(format!("{id}: {rows} rows (expected exactly 1)"));
        }
        assert!(
            roadmap.contains(id.as_str()),
            "{id} does not appear in the roadmap at all"
        );
    }

    assert!(
        problems.is_empty(),
        "every intent needs exactly one row in the roadmap dependency map — \
         a missing row hides an intent, a duplicate row is an ordering \
         contradiction: {problems:?}"
    );
}

#[test]
fn test_roadmap_names_blocking_dependencies() {
    let roadmap = read("docs/roadmap.md");
    let map = dependency_map(&roadmap);

    // The load-bearing edges the roadmap itself calls out. These determine most
    // of the ordering, and are the part most likely to rot as intents are added.
    let edges: [(&str, &str, &str); 5] = [
        (
            "INT-0011",
            "T-113",
            "messaging cannot be reliable before ARQ is wired in",
        ),
        (
            "INT-0010",
            "INT-0009",
            "the GUI needs an application layer to present",
        ),
        ("INT-0015", "INT-0008", "distributed sensing rides the mesh"),
        (
            "INT-0012",
            "INT-0002",
            "long integration may outrun the radio's clock",
        ),
        ("INT-0017", "T-108", "measurement requires transmitting"),
    ];

    for (intent, blocker, why) in edges {
        let row = map
            .lines()
            .find(|l| l.starts_with(&format!("| [{intent}](")))
            .unwrap_or_else(|| panic!("no dependency-map row for {intent}"));
        assert!(
            row.contains(blocker),
            "roadmap must name {blocker} as blocking {intent} ({why}); row was: {row}"
        );
    }
}

#[test]
fn test_roadmap_is_reachable_from_summary() {
    let summary = read("docs/SUMMARY.md");
    assert!(
        summary.contains("roadmap.md"),
        "docs/roadmap.md must be linked from docs/SUMMARY.md"
    );
}
