# Plan Critique — Sprint 8

## Concerns

### C-001: the SUMMARY test would fail on day one against pre-existing chapters
- **Where:** `test-plan.md` Integration Tests / `build-plan.md` T-039
- **Quote:** "enumerates `docs/intents/INT-*.md` and asserts each is linked from `docs/SUMMARY.md`"
- **Failure mode:** hidden-dep
- **Why it matters:** Verified on disk: **13 chapters exist, but `SUMMARY.md`
  links only 7** — INT-0001 through INT-0006 were never added, because the
  navigation convention post-dates them. T-039's scope covers only the four new
  chapters, so the test as planned fails immediately on six pre-existing ones.
  That is either a red suite at the end of a documentation sprint, or — worse —
  the temptation to weaken the test to whatever currently passes, which would
  destroy the invariant the test exists to protect.
- **Suggested response:** fix-in-plan — T-039 must also add the six missing
  links. The test is right; the Book is wrong, and the fix belongs in this
  sprint since it is the one establishing the invariant.

### C-002: T-040's dependency clause has no test
- **Where:** `build-plan.md` T-040 EARS / `test-plan.md` Intent Traceability
- **Quote:** "WHEN an intent has a blocking dependency, THEN the roadmap **SHALL** name that dependency."
- **Failure mode:** plan-test-mismatch
- **Why it matters:** Two tests cover T-040 —
  `test_roadmap_covers_every_intent_exactly_once` and
  `test_roadmap_is_reachable_from_summary` — and neither checks that
  dependencies are named. The dependency map is the roadmap's most useful
  content and the part most likely to rot when an intent is added; leaving its
  EARS clause unverified means the clause is decoration.
- **Suggested response:** fix-in-plan — add
  `test_roadmap_names_blocking_dependencies` asserting the load-bearing edges
  the plan itself names (INT-0011 ← T-113, INT-0010 ← INT-0009, INT-0015 ←
  INT-0008, INT-0012 ← INT-0002, INT-0017 ← T-108) appear in the dependency
  table.

### C-003: an EARS clause claims a validation `check-book.sh` does not perform
- **Where:** `build-plan.md` T-039 EARS 1
- **Quote:** "it **SHALL** have an intent chapter that `check-book.sh` validates, carrying a unique `INT-NNNN` id and **non-empty acceptance criteria**"
- **Failure mode:** EARS-vague
- **Why it matters:** Checked directly: `check-book.sh` contains no acceptance-
  criteria validation. The clause names a guarantee that nothing provides, so a
  chapter could ship with an empty `## Acceptance criteria` section and the
  sprint would still report the clause satisfied. Asserting verification that
  does not exist is the precise failure this project has spent three sprints
  correcting in its intents.
- **Suggested response:** fix-in-plan — either narrow the clause to what
  `check-book.sh` actually does, or provide the missing check. Provide it: a
  test asserting every chapter has a non-empty `## Acceptance criteria` section
  is three lines and makes the clause true.

### C-004: sprint-advanced intents are not moved to `planned`
- **Where:** `build-plan.md` "Intent states are deliberately not advanced" / phase contract
- **Quote:** "INT-0009..INT-0017 stay `proposed` through this sprint."
- **Failure mode:** intent-drift
- **Why it matters:** The Plan phase contract says to "move sprint-advanced
  `proposed` or `deferred` intents to `planned`, attach task or plan Work
  evidence, and append the actual transition." Nine chapters are touched by this
  sprint and none transitions, which on its face is the contract being skipped.
- **Suggested response:** **reject** — the critique is wrong because these
  intents are *authored* by the sprint, not *advanced* by it. No task moves any
  of them toward its acceptance criteria; T-039 creates chapters and T-040
  orders them. Marking them `planned` would assert that implementation is
  scheduled, contradicting the distinction the roadmap rests on — that naming a
  category is intent, and only intent state schedules work. Confirmed that
  `finalize-plan.sh` enforces no state transition, so this is a judgment about
  honesty rather than a tooling workaround. The reasoning is already recorded in
  the build plan rather than left implicit.

## Resolutions (primary agent)

| Concern | Response | Action |
|---|---|---|
| C-001 | fix-in-plan | T-039 scope extended to add the six missing `SUMMARY.md` links (INT-0001..INT-0006). The invariant is kept strict; the Book is corrected to meet it. |
| C-002 | fix-in-plan | `test_roadmap_names_blocking_dependencies` added to T-041 and to the traceability map. |
| C-003 | fix-in-plan | `test_every_intent_has_acceptance_criteria` added to T-041; T-039's EARS clause reworded to name that test rather than overclaiming `check-book.sh`. |
| C-004 | reject | Rationale above; recorded in the build plan so a later reader sees the deviation was deliberate. |

Book-integrity tests rise from four to six. Re-screened after the changes: every
EARS clause now maps to a named test, and every planned test traces to a clause.

## Confidence
proceed-with-caveats
