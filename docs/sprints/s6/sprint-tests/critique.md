# Test Critique — Sprint 6

## Concerns

### C-001: A published research figure was wrong and had to be corrected mid-build
- **Where:** `sprint-research/research-report.md` §4.3–4.4 vs `crates/sdr-mesh/src/radio.rs`
- **Quote (research):** "0 errors through **±0.5% clock drift**"
- **Failure mode:** evidence-drift
- **Why it matters:** The research report measured drift on a short (~80-bit) burst with best-lag bit comparison. At frame length (~256 bits), where every bit must survive for CRC-32 to pass, the real limit is about **±0.1%** — five times tighter. The plan, the build plan and the code comments all inherited the optimistic number. Had the drift test been written to the research figure without checking, it would have failed; had it been written loosely, the wrong figure would have shipped as documentation.
- **Suggested response:** fix-in-plan (done) — the frame-level limit was measured directly, a five-point loop-gain sweep confirmed tuning does **not** widen it (so the limit is structural, not configuration), and the claim was corrected in `radio.rs`, `fsk.rs`, the drift test and the completion record. The research report is left as written with the discrepancy documented here rather than edited retroactively, so the correction is visible. Widening the tolerance is backlog T-112.

### C-002: Drift is verified only in simulation
- **Where:** `integration-tests.md` `test_radiolink_recovers_under_clock_drift`
- **Failure mode:** intent-coverage
- **Why it matters:** The clock offset is produced by linear-interpolation resampling, which is a clean model: no noise, no jitter, no phase noise. Two real radios differ in all of those as well as rate.
- **Suggested response:** defer-with-rationale — and stated in the e2e record: the live test cannot show drift because the loopback shares a clock, and no second radio is available. The claim made is "tolerates a clock offset in simulation to ±0.1%", not "works between two radios". That remains gated on T-108 and a second device.

### C-003: The trailer changes what goes on the wire
- **Where:** `crates/sdr-mesh/src/radio.rs` (`TRAILER`)
- **Failure mode:** hidden-dep
- **Why it matters:** Every transmitted frame now carries two extra bytes. A receiver built against the previous format is unaffected (the header is length-prefixed, so trailing bytes are ignored), but the change is silent on the wire and worth recording rather than leaving to be rediscovered.
- **Suggested response:** defer-with-rationale — documented at the constant with the reason, and harmless by construction. Both ends of this link are built from the same crate, and there is no external peer implementation to break.

### C-004: `FskDemod` and `FskTimingDemod` now coexist with no guidance beyond doc comments
- **Where:** `crates/sdr-demod/src/fsk.rs`
- **Failure mode:** granularity
- **Why it matters:** Two demodulators with overlapping purpose invites picking the wrong one; `FskDemod` silently produces garbage on an unaligned stream.
- **Suggested response:** defer-with-rationale — the module doc states the distinction and when to prefer each, and `FskDemod` is retained deliberately because `PskDemod` and the Sprint 5 regression depend on its behaviour. Consolidating them would be a larger refactor than this sprint's goal.

## Confidence
proceed-with-caveats
