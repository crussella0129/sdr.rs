# Plan Critique — Sprint 5

## Concerns

### C-001: The sample-phase search is a stand-in for real symbol-timing recovery
- **Where:** `build-plan.md` T-029 / `research-report.md` §5
- **Quote:** "for each candidate sample phase `0..sps`"
- **Failure mode:** intent-drift (risk of over-claiming)
- **Why it matters:** Brute-forcing the phase works because the digital loopback has *no clock offset between transmitter and receiver* — they share the same clock. On a real over-the-air link between two radios, clocks drift and a fixed phase would slip mid-frame; the search would not save it. Presenting this as a general receiver would overstate the capability.
- **Suggested response:** defer-with-rationale — the sprint claims only "datagram recovered under internal loopback", and `clock_recovery.rs` (Gardner/M&M) is recorded as the principled replacement, carried forward for a real channel. The test plan's E2E section names this explicitly as an unlocker. No criterion is marked met on the strength of the search alone.

### C-002: `MockSdr` loopback is sample-exact, so CI would not exercise the phase search
- **Where:** `test-plan.md` `test_radiolink_datagram_roundtrip_over_mock`
- **Failure mode:** weak-assertion
- **Why it matters:** The mock returns exactly the samples written, so the receiver always succeeds at phase 0. A CI suite containing only that test would pass even if the phase search were broken or absent — and the hardware path (where alignment is arbitrary) would then fail.
- **Suggested response:** fix-in-plan (already reflected) — `test_radiolink_recovers_from_sample_offset` injects a deliberate offset so the search is genuinely covered in CI. Without it the mock test is necessary but not sufficient.

### C-003: A cyclic transmit buffer can yield a wrapped/partial frame
- **Where:** `build-plan.md` T-030
- **Quote:** "`set_tx_cyclic(true)` so the frame repeats"
- **Failure mode:** flake-risk
- **Why it matters:** The receiver starts capturing at an arbitrary point in a repeating buffer, so the first frame in the window may be truncated; a receiver that only examined the start of the capture would intermittently fail.
- **Suggested response:** fix-in-plan — capture ≥ 2× the frame length so at least one complete frame is present, and let the sync-word search locate it rather than assuming position zero. Already specified in T-030.

### C-004: T-027 adds tests to a `realized` intent (INT-0006)
- **Where:** `build-plan.md` Intents / T-027
- **Failure mode:** intent-drift
- **Why it matters:** Adding work against a realized intent can blur whether a criterion re-opened.
- **Suggested response:** defer-with-rationale — T-027 adds *regression coverage only*; it changes no source, no acceptance criterion, and no state. INT-0006 stays `realized`. The coverage exists because the mesh now depends on that DSP pair, which previously had no round-trip test.

## Confidence
proceed-with-caveats
