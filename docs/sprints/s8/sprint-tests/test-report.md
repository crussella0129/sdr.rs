# Sprint 8 — Test Report

- **Verdict:** pass, with the caveats below
- **Critique:** [critique.md](critique.md) — `proceed-with-caveats`, all four concerns addressed
- **Date:** 2026-08-23
- **Tested head:** `3cb6098916c71dd46c051570f868595a0dd55f5e`
- **Canonical runner:** `cargo test --workspace` — **33 suites, 0 failed**
  (32 at Sprint 7 close, +1 for `book_it`)
- **Lint:** `cargo clippy --workspace --all-targets` — 0 errors
- **Book:** `check-book.sh` — valid v2 Book, **17 intent chapters** (was 13)
- **CI authority:** no hosted CI on this repository; the local workspace runner
  is the canonical suite.

## What this sprint verified — and what it did not

This was a scope-defining sprint. It produced Book structure: four adopted
intent chapters, a roadmap with a dependency map, corrected navigation, and six
tests that keep those invariants honest.

**No intent acceptance criterion was verified by this sprint** (critique C-001).
The nine `proposed` chapters — INT-0009 through INT-0017 — gain **no test
evidence** and remain `proposed`. What was verified is that the Book is
structurally sound: every chapter reachable, every chapter carrying acceptance
criteria, the roadmap covering every intent with its blocking dependencies
named.

That distinction is the whole point. A chapter's acceptance criteria are targets
to be measured by the sprint that implements it, not claims established by
writing them down.

## EARS clause coverage

| Task | EARS clause | Executed test | Result |
|---|---|---|---|
| T-039 | WHEN a category is adopted THEN it SHALL have a chapter `check-book.sh` validates | `check-book.sh` (17 chapters) | pass |
| T-039 | WHEN a chapter exists THEN it SHALL carry non-empty acceptance criteria | `test_every_intent_has_acceptance_criteria` | pass |
| T-039 | WHEN a chapter exists THEN it SHALL be reachable from `SUMMARY.md` | `test_every_intent_is_reachable_from_summary` | pass |
| T-039 | WHEN a category is adopted THEN it SHALL NOT remain a candidate | `test_adopted_categories_are_not_listed_as_candidates` | pass |
| T-040 | WHEN the roadmap is published THEN every chapter SHALL appear exactly once | `test_roadmap_covers_every_intent_exactly_once` | pass (see deviation) |
| T-040 | WHEN an intent has a blocking dependency THEN the roadmap SHALL name it | `test_roadmap_names_blocking_dependencies` | pass |
| T-041 | WHEN a chapter lacks a link, roadmap entry or criteria THEN the suite SHALL fail | all six, each demonstrated failing | pass |

Every clause maps to an executed test, and every test traces to a clause.

## Two defects the plan and test phases caught

- **`SUMMARY.md` linked only 7 of 13 chapters.** INT-0001 through INT-0006
  predate the navigation convention and were never added, so the reachability
  test would have failed on day one. The tempting response — weakening the test
  to whatever currently passed — would have destroyed the invariant it exists to
  provide. The Book was corrected instead (T-039). All 17 are now reachable.
- **An EARS clause asserted a check nothing performed.** T-039's original clause
  claimed `check-book.sh` validates non-empty acceptance criteria. It does not.
  Rather than let the clause stand, it was narrowed and a test added that
  actually performs the check.

## Deviation from the locked plan — recorded, not hidden

T-040's locked clause required every intent to appear "exactly once in the
**phase listing**". Implementing it showed the clause is not satisfiable by a
roadmap worth reading: the phase listing is forward-looking and deliberately
omits the six already-realized chapters, and prose mentioning an intent a second
time is useful rather than a defect.

The test therefore asserts against the roadmap's **dependency map** — the
structure that genuinely holds one canonical row per intent — plus at-least-one
mention anywhere in the document. This verifies what the clause is *for*: total
coverage, no duplicates, no omissions. Rewording the roadmap to satisfy the
literal clause would have made it worse. The reasoning is recorded in the test's
doc comment and the T-040 completion entry.

## Negative capability verified for all six tests

A green test that cannot go red proves nothing. Each invariant was broken
deliberately, the corresponding test observed to fail, and the Book restored to
6 passed with a clean `git status`: removing a `SUMMARY.md` link, deleting a
dependency-map row, dropping a blocking dependency, re-adding a candidates
section, emptying a chapter's acceptance criteria, and unlinking the roadmap.
The matrix is in [integration-tests.md](integration-tests.md).

## Caveats — stated, not implied away

1. **No intent acceptance criteria verified.** As above. The nine `proposed`
   chapters carry no test evidence and are not claimed to.
2. **The roadmap's ordering is not verified and cannot be.** It is a
   recommendation derived from the dependency structure, and a product judgment
   the user reorders at any sprint boundary. No dates or effort estimates appear
   anywhere in it, because none would be evidence-backed.
3. **Two assertions are shape checks, not substance checks** (critique C-003). A
   renamed section heading, or acceptance criteria reading "TBD", would pass.
   They catch omission and drift — the realistic failure — not deliberate
   evasion, which is not mechanizable.
4. **The Book tests assume the repository layout** (critique C-004). They
   resolve `docs/` relative to the crate manifest and would fail if `sdr-cli`
   were built outside the workspace. Acceptable for workspace-internal tests on
   an unpublished crate.
5. **Unverified figures now live in the Book.** INT-0010's ≥30 fps waterfall,
   INT-0013's classifier accuracy and latency, INT-0015's TDOA accuracy and
   INT-0017's level accuracy are all written as open claims in `proposed`
   chapters, labelled unverified. They become evidence when measured.
6. **Nothing went on air.** Still gated on **T-108** and explicit
   authorization. The `#[ignore]`d hardware tests were not re-run, since no code
   they exercise changed; Sprint 7's live result stands.

## Regression contract

Every suite from Sprints 0-7 ran **unchanged** and passed. A documentation
sprint that alters a test result has done something wrong. This contract caught
two real defects in Sprint 6 and validated T-035 in Sprint 7.

## Test evidence links

- [unit-tests.md](unit-tests.md) — none added; no library code in this sprint
- [integration-tests.md](integration-tests.md)
- [e2e-tests.md](e2e-tests.md) — not applicable, with rationale and unlockers
- [critique.md](critique.md)
