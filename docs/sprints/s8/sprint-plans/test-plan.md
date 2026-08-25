Finalized - DO NOT EDIT

# Sprint 8 Test Plan

## What is being verified, and what cannot be

This sprint's deliverable is Book structure: intent chapters, a roadmap, and
navigation. Documentation correctness in the *semantic* sense — whether the
roadmap's ordering is wise, whether a category is worth pursuing — is a human
judgment and is **not** testable. Nothing here claims otherwise.

What *is* testable is Book **integrity**: that every intent is reachable, that
the roadmap covers every intent, and that adopted categories are no longer
listed as candidates. Those are mechanical invariants that will silently rot as
the Book grows, which is exactly why they get executable guards rather than a
docs-only exemption.

## Intent Traceability

| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0014](../../../intents/INT-0014-satellite-space-operations.md) | chapter exists, valid, reachable | T-039 / WHEN adopted THEN valid chapter; WHEN chapter exists THEN reachable from SUMMARY | `check-book.sh`, `test_every_intent_is_reachable_from_summary` |
| [INT-0015](../../../intents/INT-0015-distributed-sensing-df.md) | chapter exists, valid, reachable | T-039 / same | `check-book.sh`, `test_every_intent_is_reachable_from_summary` |
| [INT-0016](../../../intents/INT-0016-propagation-beacon-reporting.md) | chapter exists, valid, reachable | T-039 / same | `check-book.sh`, `test_every_intent_is_reachable_from_summary` |
| [INT-0017](../../../intents/INT-0017-test-and-measurement.md) | chapter exists, valid, reachable | T-039 / same | `check-book.sh`, `test_every_intent_is_reachable_from_summary` |
| all 17 chapters | every chapter has non-empty acceptance criteria | T-039 / WHEN a chapter exists THEN non-empty `## Acceptance criteria` | `test_every_intent_has_acceptance_criteria` |
| all 17 chapters | roadmap names blocking dependencies | T-040 / WHEN an intent has a blocking dependency THEN the roadmap SHALL name it | `test_roadmap_names_blocking_dependencies` |
| all 17 chapters | no adopted category still listed as a candidate | T-039 / WHEN adopted THEN SHALL NOT remain a candidate | `test_adopted_categories_are_not_listed_as_candidates` |
| all 17 chapters | roadmap covers every intent exactly once | T-040 / WHEN published THEN every chapter appears exactly once | `test_roadmap_covers_every_intent_exactly_once` |
| all 17 chapters | roadmap is navigable | T-040 / WHEN published THEN reachable from SUMMARY | `test_roadmap_is_reachable_from_summary` |
| all 17 chapters | drift fails loudly | T-041 / WHEN a chapter lacks a SUMMARY link, roadmap entry, or acceptance criteria THEN the suite SHALL fail | all six tests above |

Note that the four new chapters have no *implementation* acceptance criteria
verified here — they are `proposed`, and nothing in this sprint advances them.
The criteria verified above are the criteria this sprint actually creates: that
the Book is structurally sound.

## Unit Tests

None. This sprint adds no library code, so there is no unit under test. Stating
that plainly rather than inventing a unit suite to fill the section.

## Integration Tests

### Book integrity — `crates/sdr-cli/tests/book_it.rs` (new, T-041)

Each test parses `docs/` at run time, so it reflects the Book's real state
rather than a snapshot baked in at authoring time.

- **`test_every_intent_is_reachable_from_summary`** — enumerates
  `docs/intents/INT-*.md` and asserts each is linked from `docs/SUMMARY.md`.
  Failure message names the missing chapters so the fix is obvious. **Applies to
  every chapter, not just new ones:** six pre-existing chapters (INT-0001..
  INT-0006) are unlinked today and are corrected by T-039 rather than exempted
  (critique C-001).
- **`test_adopted_categories_are_not_listed_as_candidates`** — asserts
  `docs/README.md` no longer carries a "Candidate categories" section listing
  INT-0014..INT-0017, i.e. that adopting them actually moved them.
- **`test_roadmap_covers_every_intent_exactly_once`** — asserts every
  `INT-NNNN` id appears in `docs/roadmap.md`, and appears **exactly once** in
  the phase listing. The exactly-once half matters: an intent listed under two
  phases is an ordering contradiction, not a harmless duplicate.
- **`test_roadmap_is_reachable_from_summary`** — asserts `docs/roadmap.md` is
  linked from `docs/SUMMARY.md`.
- **`test_every_intent_has_acceptance_criteria`** (critique C-003) — asserts
  every chapter carries a non-empty `## Acceptance criteria` section.
  `check-book.sh` does not check this, so without the test the corresponding
  EARS clause would assert a guarantee nothing provides.
- **`test_roadmap_names_blocking_dependencies`** (critique C-002) — asserts the
  load-bearing dependency edges appear in the roadmap's table: INT-0011 ← T-113,
  INT-0010 ← INT-0009, INT-0015 ← INT-0008, INT-0012 ← INT-0002, INT-0017 ←
  T-108. The dependency map is the roadmap's most useful content and the part
  most likely to rot as intents are added.

These are negative-capable by construction: each fails if the corresponding
Book invariant is broken, which is the property T-041 exists to provide.

## End-to-End Tests

- **Status:** not applicable, and not a cop-out. There is no runtime behaviour
  in this sprint — no code path executes differently before and after it. The
  Book-integrity tests above *are* the end-to-end check for a documentation
  deliverable: they exercise the real artifacts on disk.
- **Regression contract — carried forward unchanged:** every test from Sprints
  0-7 must pass untouched (32 suites green at Sprint 7 close). A documentation
  sprint that changes a test result has done something wrong. This contract
  caught two real defects in Sprint 6 and validated T-035 in Sprint 7, which is
  why it is maintained rather than rebaselined.
- **Not-yet-possible (named unlockers):**
  - Verifying INT-0010's **≥30 fps** waterfall claim → requires an egui/wgpu
    prototype and a GPU-capable environment; deferred with INT-0010 and labelled
    unverified in that chapter.
  - Verifying INT-0012's accumulator numerics and Pluto clock stability →
    requires the astronomy spike (Phase 4).
  - Any claim that the roadmap's ordering is *correct* → human judgment; the
    user reorders at any sprint boundary.

## Lint and Format

`cargo fmt` and `cargo clippy --workspace --all-targets` must be clean of
errors. Pre-existing warnings remain tracked as backlog T-105; this sprint must
add none.
