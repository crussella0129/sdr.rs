# Test Critique — Sprint 8

## Concerns

### C-001: the traceability table presents Book structure as intent acceptance
- **Where:** `test-plan.md` Intent Traceability
- **Quote:** "| [INT-0014](...) | chapter exists, valid, reachable | T-039 / WHEN adopted THEN valid chapter ... |"
- **Failure mode:** intent-coverage
- **Why it matters:** The column is headed *Acceptance criterion*, but "chapter
  exists, valid, reachable" is **not** one of INT-0014's acceptance criteria.
  Its real criteria are things like "pass prediction agrees with an independent
  propagator" — none of which this sprint verifies, correctly, because the
  chapter is `proposed`. A reader skimming the table could conclude INT-0014 has
  verified acceptance criteria when it has none. That is the precise confusion
  Sprint 2's audit had to unwind in INT-0002, INT-0004 and INT-0006.
- **Suggested response:** tighten-assertion — the report must say plainly that
  **no intent acceptance criterion is verified by this sprint**, and that what
  was verified is Book structure. The four chapters stay `proposed` with no test
  evidence attached.

### C-002: only one of six tests had its failure path demonstrated
- **Where:** `integration-tests.md` (as first written)
- **Quote:** "Removing INT-0003's link from `SUMMARY.md` ... test result: FAILED"
- **Failure mode:** negative-path
- **Why it matters:** Five of the six guards were reported passing with no
  evidence they can fail. For tests whose entire value is catching future drift,
  an undemonstrated failure path is an assumption, not a result — a typo in a
  section heading could render one permanently green.
- **Suggested response:** fix — demonstrate all six.

### C-003: two assertions are string-shaped and would miss a rename
- **Where:** `book_it.rs` — `test_adopted_categories_are_not_listed_as_candidates`, `test_every_intent_has_acceptance_criteria`
- **Quote:** "`!readme.contains(\"Candidate categories\")`"
- **Failure mode:** weak-assertion
- **Why it matters:** Renaming the section to "Possible categories" would pass,
  and a chapter whose acceptance criteria read "TBD" satisfies the non-empty
  check. Both tests verify a *shape*, not the substance they gesture at.
- **Suggested response:** defer-with-rationale — the substance ("are these
  acceptance criteria any good?") is human judgment and not mechanizable. The
  shape checks catch the realistic failure, which is omission and drift, not
  deliberate evasion. Recorded as a known limit rather than papered over.

### C-004: the Book tests assume the repository layout
- **Where:** `book_it.rs` `repo_root()`
- **Quote:** "`Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(\"..\").join(\"..\")`"
- **Failure mode:** flake-risk
- **Why it matters:** The tests resolve `docs/` two levels above the crate
  manifest. If `sdr-cli` were ever packaged, vendored or built outside the
  workspace, `docs/` would be absent and the suite would fail for a reason
  unrelated to the Book.
- **Suggested response:** defer-with-rationale — these are workspace-internal
  integration tests asserting workspace-wide facts, and the crate is not
  published. Failure would be loud and immediately diagnosable from the panic
  message, which names the missing path.

## Resolutions (primary agent)

Re-run after the evidence changed. The verdict is unchanged.

| Concern | Response | Outcome |
|---|---|---|
| C-001 | tighten-assertion | **Accepted.** `test-report.md` states explicitly that this sprint verifies **no** intent acceptance criterion, and that the nine `proposed` chapters gain no test evidence. The traceability table's column is what it is, but the report is unambiguous. |
| C-002 | fix | **Fixed.** All six invariants were broken deliberately and each corresponding test observed to fail, then restored to 6 passed with a clean `git status`. The full matrix is in [integration-tests.md](integration-tests.md). |
| C-003 | defer-with-rationale | **Accepted as a known limit.** The shape checks catch omission and drift, which is the realistic failure mode; substance is human judgment. |
| C-004 | defer-with-rationale | **Accepted.** Workspace-internal tests on an unpublished crate; failure is loud and names the path. |

Post-resolution verification at head `3cb6098`: `cargo test --workspace` green
(**33 suites**, 0 failed), `cargo clippy --workspace --all-targets` 0 errors,
`check-book.sh` valid with 17 chapters.

## Confidence
proceed-with-caveats
