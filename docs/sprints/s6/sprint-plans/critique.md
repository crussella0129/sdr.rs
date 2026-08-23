# Plan Critique — Sprint 6

## Concerns

### C-001: Removing the phase search deletes a working fallback
- **Where:** `build-plan.md` T-033
- **Quote:** "Replace the `for phase in 0..sps` loop in `extract_payload` with a **single** demodulation pass"
- **Failure mode:** hidden-dep
- **Why it matters:** The phase search is currently the only thing making the live hardware test pass. Replacing rather than layering means a regression in timing recovery takes the whole radio path down, including the one end-to-end result the project has on hardware.
- **Suggested response:** fix-in-plan — the three Sprint 5 `radio_it` tests are carried over **unchanged as a regression contract**, so any loss of capability fails CI before hardware is touched, and T-034 re-verifies live. Keeping both paths would be the more conservative choice, but it would leave dead code whose failure mode is silent fallback: the receiver would appear to work while timing recovery was broken, which is worse than a loud failure. Replace, with the contract as the guard.

### C-002: Drift tolerance is asserted at ±0.5% but degradation begins at ±1%
- **Where:** `test-plan.md` T-032 `test_timing_demod_clock_drift`
- **Failure mode:** weak-assertion
- **Why it matters:** Testing only inside the passing range documents where it works but not where it stops, so a future change that quietly narrows the range to ±0.2% would still pass.
- **Suggested response:** fix-in-plan — the drift test asserts bit-exact recovery at ±0.5% (the supported edge). The measured breakdown at ±1–2% is recorded in the research report rather than asserted as a test, because pinning a *failure* threshold would codify current loop gains as a contract and make legitimate tuning look like a regression. The supported range is the claim; the boundary is documentation.

### C-003: `GardnerClockRecovery` is shared with `PskDemod`
- **Where:** `build-plan.md` T-031 / `crates/sdr-demod/src/psk.rs`
- **Failure mode:** hidden-dep
- **Why it matters:** T-031 only strengthens tests, but the component now has a second consumer with different expectations (PSK feeds raw IQ; FSK feeds discriminator output). A future change tuned for one could break the other silently, since PSK has no round-trip coverage of its own.
- **Suggested response:** defer-with-rationale — this sprint changes no Gardner *code*, only its tests, so no existing behaviour moves. The strengthened tests protect both consumers. PSK's missing round-trip coverage is a real pre-existing gap, but fixing it is not this sprint's goal; it should be recorded as backlog rather than absorbed here.

### C-004: The live test cannot distinguish "timing recovery works" from "the loopback is easy"
- **Where:** `build-plan.md` T-034
- **Failure mode:** intent-coverage
- **Why it matters:** The internal loopback shares a clock, so the hardware test would pass even if drift handling were broken — it re-verifies integration, not the new capability.
- **Suggested response:** defer-with-rationale — and stated plainly: the drift capability is proven in CI (where an offset can be injected deliberately), while the hardware test proves the swap did not break real-device integration. Neither is claimed to do the other's job, and the test report must not present the live pass as evidence of drift tolerance.

## Confidence
proceed-with-caveats
