Finalized - DO NOT EDIT

# Sprint 8 Build Plan

## Context

Sprint 8 is a scope-defining sprint, not a feature sprint. Research found the
decisive structural fact: `sdr.rs` is **9 crates and ~8,680 LOC of well-tested
internals with nothing composing them into an application**, and its 8 intent
chapters named no application, no GUI, and none of the project's larger
ambitions.

Research created five category intents (INT-0009..INT-0013) and settled the
front-end question — `egui` shell + `wgpu` visualization, Bevy deferred against
documented triggers. Three decisions then came from the user:

1. This sprint produces **roadmap documentation only** — no feature code.
2. **All four** candidate categories are adopted as intents.
3. Sprint 9 resumes the mesh work: T-113 (ARQ) then INT-0011 messaging.

This plan executes those decisions.

### Intent states are deliberately not advanced

INT-0009..INT-0017 stay `proposed` through this sprint. The sprint **authors**
those chapters; it does not advance work toward their acceptance criteria.
Moving them to `planned` would assert that implementation is scheduled, which is
false and would contradict the distinction the roadmap itself rests on: a
category being named is intent, and only intent state schedules work. The
chapters are therefore created `proposed`, and Sprint 9 moves the ones it takes
up.

## Tasks

### T-039: Adopt the four candidate categories as intent chapters
- **Intents:** INT-0014, INT-0015, INT-0016, INT-0017 (created `proposed`)
- **Touches:** `docs/intents/INT-001{4,5,6,7}-*.md` (new), `docs/SUMMARY.md`,
  `docs/README.md`
- **Depends on:** nothing
- Four chapters to the schema used by INT-0009..INT-0013 — Intent, Acceptance
  criteria, Rationale, Alternatives, Consequences, Transition history:
  - **INT-0014 Satellite and space operations** — pass prediction, Doppler
    correction, APT/LRPT weather-satellite imagery, telemetry. Shares tracking,
    pointing and Doppler machinery with INT-0012.
  - **INT-0015 Distributed sensing and direction finding** — TDOA geolocation
    across multiple receivers. Rests on INT-0008's mesh as the sensor fabric;
    **time synchronization across nodes is the hard part** and the chapter must
    say so rather than treating it as a detail.
  - **INT-0016 Propagation and beacon reporting** — WSPR/RBN-style monitoring
    and reporting; extends INT-0007.
  - **INT-0017 Test and measurement** — signal generator, scalar network
    analyzer, noise-figure meter. Gated on transmit, so it inherits INT-0005's
    compliance gate and the on-air constraint (T-108).
- The Book README's "Candidate categories, not yet adopted" section is removed;
  all four join the taxonomy proper.
- **Also corrects a pre-existing gap (plan critique C-001):** `SUMMARY.md` links
  only 7 of the 13 existing chapters — INT-0001..INT-0006 predate the navigation
  convention and were never added. All six links are added here, because this is
  the sprint that establishes the reachability invariant and the invariant must
  hold over the whole Book, not just the new part.
- **EARS:**
  - WHEN a category is adopted, THEN it **SHALL** have an intent chapter that
    `check-book.sh` validates, carrying a unique `INT-NNNN` id.
  - WHEN an intent chapter exists, THEN it **SHALL** carry a non-empty
    `## Acceptance criteria` section. (Narrowed per critique C-003:
    `check-book.sh` does **not** verify this, so a test provides it rather than
    the clause claiming a guarantee nothing supplies.)
  - WHEN an intent chapter exists, THEN it **SHALL** be reachable from
    `docs/SUMMARY.md` — for **every** chapter, including those predating the
    convention.
  - WHEN a category is adopted, THEN it **SHALL NOT** remain listed as a
    candidate in the Book README.
- **Tests:** `test_every_intent_is_reachable_from_summary`,
  `test_every_intent_has_acceptance_criteria`,
  `test_adopted_categories_are_not_listed_as_candidates`, and `check-book.sh`
  reporting 17 valid chapters.

### T-040: Publish the roadmap: phases and dependency map
- **Intents:** all 17 (sequencing only; no semantic change to any chapter)
- **Touches:** `docs/roadmap.md` (new), `docs/SUMMARY.md`
- **Depends on:** T-039 (all 17 chapters must exist to be covered)
- One page answering "what order, and what blocks what":
  - **Phase 1 — Mesh reliability and messaging** (in flight): T-113 ARQ into the
    receive path, then INT-0011. T-107 (`tun`) completes INT-0008 criterion 1.
  - **Phase 2 — Application layer:** INT-0009.
  - **Phase 3 — Desktop GUI:** INT-0010.
  - **Phase 4 — Astronomy:** INT-0012, spiked early because its two unknowns
    (accumulator numerics, Pluto clock stability) are cheap to measure and
    expensive to discover late.
  - **Phase 5 — ML:** INT-0013.
  - **Later:** INT-0014..INT-0017.
  - **Continuous:** INT-0002 hardware breadth (T-101), INT-0005 compliance.
- A dependency table naming what blocks each intent. Load-bearing edges:
  INT-0011 ← T-113; INT-0010 ← INT-0009; INT-0015 ← INT-0008 + node time sync;
  INT-0012 ← INT-0002 clock stability; INT-0017 ← transmit + T-108.
- The roadmap states plainly that **phases are ordering, not commitment**, and
  that intent state is what schedules work.
- **EARS:**
  - WHEN the roadmap is published, THEN every `INT-NNNN` chapter in
    `docs/intents/` **SHALL** appear in it exactly once.
  - WHEN an intent has a blocking dependency, THEN the roadmap **SHALL** name
    that dependency.
- **Tests:** `test_roadmap_covers_every_intent_exactly_once`,
  `test_roadmap_names_blocking_dependencies`,
  `test_roadmap_is_reachable_from_summary`.

### T-041: Book-integrity tests
- **Intents:** all (executable guard on Book structure)
- **Touches:** `crates/sdr-cli/tests/book_it.rs` (new)
- **Depends on:** T-039, T-040
- Implements the tests named above by parsing `docs/` at test time. Follows this
  repo's existing precedent for documentation tests
  (`crates/sdr-mesh/tests/readme_it.rs`,
  `crates/sdr-station/tests/cloudlog_it.rs`), placed in `sdr-cli` because these
  assert Book-wide rather than crate-local facts.
- The purpose is durability: an intent added later without a `SUMMARY.md` link
  or a roadmap entry fails the suite instead of drifting silently. This is the
  sprint's one executable artifact, and the reason a documentation sprint still
  carries verification rather than claiming a docs-only exemption.
- **EARS:**
  - WHEN an intent chapter exists without a `SUMMARY.md` link, a roadmap entry,
    or non-empty acceptance criteria, THEN the suite **SHALL** fail.
- **Tests:** the **six** named above, all run by `cargo test --workspace`.

## Files created / modified
- `docs/intents/INT-0014..INT-0017-*.md` (new, 4 chapters)
- `docs/roadmap.md` (new)
- `docs/README.md`, `docs/SUMMARY.md` (taxonomy + navigation)
- `crates/sdr-cli/tests/book_it.rs` (new)

No new dependencies and no implementation source touched. `sdr-cli` already has
a `tests/` directory, so no manifest change is required.

## Verification
- `cargo test --workspace` — the six Book-integrity tests pass alongside the
  existing suite (32 suites green at Sprint 7 close; no regression permitted).
- `check-book.sh` reports a valid v2 Book with **17** intent chapters.
- `cargo fmt` and `cargo clippy --workspace --all-targets` clean.
- **Regression contract:** every prior sprint's test carries forward unchanged.

## Honest limits to state, not imply away
- **A roadmap is a statement of intent, not a schedule or an estimate.** No
  dates and no effort figures appear in it, because none would be
  evidence-backed.
- **Adopting a category costs nothing to build but adds Book surface.** Four
  more chapters must be maintained and reconciled every sprint — the accepted
  cost of recording them properly rather than losing them.
- **Phase ordering is a recommendation derived from the dependency structure**,
  reorderable at any sprint boundary.
- **INT-0010's ≥30 fps figure remains unverified** until a prototype measures
  it. It is labelled as such in the chapter and is *not* validated here.

## Out of scope (→ backlog / later sprints)
- Any feature code — the user's explicit scope decision for this sprint.
- T-113 ARQ and INT-0011 messaging → Sprint 9.
- Any egui/wgpu prototype → deferred with INT-0010.
