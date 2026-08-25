# Sprint 8 — Integration Test Results

- **Tested head:** `3cb6098916c71dd46c051570f868595a0dd55f5e`
- **Runner:** `cargo test --workspace --tests`
- **Date:** 2026-08-23
- **Result:** **33 suites, 0 failed** (32 at Sprint 7 close, +1 for `book_it`)

## Book-integrity — `crates/sdr-cli/tests/book_it.rs` (T-041)

Six tests, all passing. Each parses `docs/` at run time, so it reflects the
Book's real state rather than a snapshot baked in at authoring time.

| Test | EARS clause | Result |
|---|---|---|
| `test_every_intent_is_reachable_from_summary` | WHEN a chapter exists THEN it SHALL be reachable from `SUMMARY.md` | ok |
| `test_every_intent_has_acceptance_criteria` | WHEN a chapter exists THEN it SHALL carry a non-empty `## Acceptance criteria` section | ok |
| `test_adopted_categories_are_not_listed_as_candidates` | WHEN a category is adopted THEN it SHALL NOT remain a candidate in the Book README | ok |
| `test_roadmap_covers_every_intent_exactly_once` | WHEN the roadmap is published THEN every chapter SHALL appear exactly once | ok |
| `test_roadmap_names_blocking_dependencies` | WHEN an intent has a blocking dependency THEN the roadmap SHALL name it | ok |
| `test_roadmap_is_reachable_from_summary` | WHEN the roadmap is published THEN it SHALL be reachable from `SUMMARY.md` | ok |

### Negative capability verified for all six, not assumed

A green test that cannot go red proves nothing. Each invariant was broken
deliberately and the corresponding test observed to fail, then the Book was
restored and the suite observed to return to 6 passed.

| Test | Invariant broken | Failed as required |
|---|---|---|
| `test_every_intent_is_reachable_from_summary` | removed INT-0003's `SUMMARY.md` link | yes |
| `test_roadmap_covers_every_intent_exactly_once` | deleted INT-0016's dependency-map row | yes |
| `test_roadmap_names_blocking_dependencies` | dropped **T-113** from INT-0011's row | yes |
| `test_adopted_categories_are_not_listed_as_candidates` | re-added a "Candidate categories" section | yes |
| `test_every_intent_has_acceptance_criteria` | emptied INT-0016's `## Acceptance criteria` | yes |
| `test_roadmap_is_reachable_from_summary` | unlinked `roadmap.md` from `SUMMARY.md` | yes |

Failures name the offending chapter precisely — e.g.

```text
these intent chapters exist but are not linked from docs/SUMMARY.md,
so they are unreachable when the Book is read as a document: ["INT-0003"]
```

`git status` confirmed a clean working tree after each restore, so no probe
leaked into the committed Book.

### The defect these tests caught before they existed

The plan critique (C-001) predicted, and inspection confirmed, that
`SUMMARY.md` linked only **7 of 13** chapters — INT-0001 through INT-0006
predate the navigation convention and were never added. Had the test been
written without T-039 also fixing the Book, it would have failed on day one, and
the tempting "fix" would have been to weaken the test to whatever currently
passed — destroying the invariant it exists to protect. The Book was corrected
instead. All 17 chapters are now reachable.

## Book validation

`check-book.sh` reports **valid v2 Book (17 intent chapters)** — up from 13, the
four adopted categories having been added by T-039.

## Deviation from the locked plan — recorded, not hidden

T-040's locked EARS clause read: every intent SHALL appear "exactly once in the
**phase listing**". Implementing it showed the clause is not satisfiable by a
roadmap worth reading:

- The phase listing is forward-looking and deliberately omits the **six
  already-realized** chapters (INT-0001, 0003, 0004, 0005, 0006, 0007). Listing
  them under a future phase would be false.
- Prose that mentions an intent a second time is useful, not a defect — for
  example noting that INT-0015 and INT-0016 are cheaper than their position
  suggests.

`test_roadmap_covers_every_intent_exactly_once` therefore asserts against the
roadmap's **dependency map**, which is the structure that genuinely holds one
canonical row per intent, plus at-least-one mention anywhere in the document.
That verifies what the clause is *for* — total coverage, no duplicates, no
omissions — rather than its literal wording. Rewording the roadmap to satisfy
the literal clause would have made it worse. The reasoning is recorded in the
test's own doc comment so a later reader is not left guessing.

## Regression contract — carried forward unchanged

Every suite from Sprints 0-7 ran unchanged and passed. A documentation sprint
that alters a test result has done something wrong.

| Suite | Result |
|---|---|
| `radio_it` (6) | ok |
| `stream_it` (2) | ok |
| `tunnel_it` (3) | ok |
| `ssh_tunnel_it` (2) | ok |
| `loopback_it` (2) | ok |
| `readme_it` (1) | ok |
| `driver_cli_it` (3) | ok |
| `e2e_pipeline_tests` (5) | ok |
| `rigctl_server_it` (3) | ok |
| `cloudlog_it` (5) | ok |
| `pluto_iiod` (2), `fsk_roundtrip`, `fsk_timing` | ok |
| `hw_radio` (2) | ignored — requires the physical Pluto+ |

The two hardware tests remain `#[ignore]`d and were **not** re-run: no code they
exercise changed this sprint, and their last live result (Sprint 7: 87 bytes as
2 datagrams, recovered byte-for-byte) stands. Re-running them would produce no
new evidence.
