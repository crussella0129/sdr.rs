# Test Critique — Sprint 4

## Concerns

### C-001: Loopback proves the digital path, not the RF chain
- **Where:** `e2e-tests.md` `hw_verify_pluto_tx_loopback` / `INT-0002` criterion 2
- **Quote:** "`loopback=1` (AD9361-internal digital — the entire RF section is bypassed)"
- **Failure mode:** intent-coverage
- **Why it matters:** The very property that makes this test safe — bypassing the RF section — also means the mixer, PA and antenna path are unexercised. Sample transport, encoding, buffer handling and device control are proven on real hardware; the RF chain itself is not.
- **Suggested response:** defer-with-rationale — this is the verification depth the user explicitly chose. The report claims only what was shown ("verified under internal loopback"), INT-0002 stays `active`, and over-the-air verification is tracked as backlog **T-108** requiring separate go-ahead. No over-claim.

### C-002: The cyclic buffer means the transmitter is left running until closed
- **Where:** `crates/sdr-hardware/src/pluto.rs` `set_tx_cyclic` / `write_samples`
- **Failure mode:** flake-risk (operational)
- **Why it matters:** A cyclic buffer repeats indefinitely. On real hardware with the RF path active, a caller who writes cyclically and forgets `stop_tx`/`teardown` would transmit continuously — a genuine operational hazard beyond this sprint's loopback context.
- **Suggested response:** fix-in-plan (partially addressed) — `stop_tx` and `teardown` both close the TX buffer, `set_tx_cyclic` defaults to `false`, and the live test closes in the restore block. Remaining residual risk is a caller who drops the driver without teardown; a `Drop` impl closing the buffer is worth a follow-up but is not required for this sprint's claim. Recorded for the backlog rather than silently ignored.

### C-003: `test_pluto_tx_gain_range` still exercises only the disconnected guard
- **Where:** `unit-tests.md` T-024 (carried over from the plan critique)
- **Failure mode:** weak-assertion
- **Why it matters:** The unit test proves the range check, not that the attribute write is well-formed.
- **Suggested response:** reject (the critique is now stale) — the connected write path is covered twice over: `test_pluto_iiod_tx_replay` asserts the observed `WRITE … OUTPUT voltage0 hardwaregain` frame against a mock server, and the live test sets −89.75 dB through this exact path and the resulting attenuation was confirmed on the device. Adequately covered.

## Confidence
proceed-with-caveats
