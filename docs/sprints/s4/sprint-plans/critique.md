# Plan Critique — Sprint 4

## Concerns

### C-001: `WRITEBUF` response framing is unverified at plan time
- **Where:** `build-plan.md` T-023 / `research-report.md` §5
- **Quote:** "`WRITEBUF` was deliberately **not** probed during research because writing a TX buffer is the action that could emit"
- **Failure mode:** hidden-dep
- **Why it matters:** The EARS clause promises a specific command/response shape that has not been confirmed against the live daemon. If iiod's `WRITEBUF` reply differs (e.g. an extra echoed line, as `READBUF` has), the transport will desynchronize the connection — exactly the class of bug that broke the first live RX attempt in Sprint 2.
- **Suggested response:** fix-in-plan (already reflected) — the framing is settled empirically during Build **with `loopback=1` engaged and TX attenuation at −89.75 dB**, so the discovery step itself cannot radiate. The mock-iiod replay test then pins whatever framing proves correct, and the Sprint 2 precedent (trailing-newline handling) is the first thing to check on desync.

### C-002: `test_pluto_tx_gain_range` asserts on a disconnected driver
- **Where:** `test-plan.md` T-024 unit tests
- **Quote:** "−100.0 and +1.0 rejected with a config error (no device write attempted while disconnected)"
- **Failure mode:** weak-assertion
- **Why it matters:** Validating only the range check on a disconnected driver does not prove the attribute is written correctly when connected, so a wrong attribute name or direction token would pass unit tests.
- **Suggested response:** fix-in-plan — the connected write path is covered by `test_pluto_iiod_tx_replay` (mock server observes the actual `WRITE ... OUTPUT voltage0 hardwaregain` frame) and by the live loopback test, which sets maximum attenuation through this exact path. The unit test covers the guard; the integration test covers the write.

### C-003: The live loopback test must restore device state even on failure
- **Where:** `build-plan.md` T-026 / `test-plan.md` E2E
- **Quote:** "`exit_loopback_test_mode()` and `teardown` restore prior device state"
- **Failure mode:** flake-risk
- **Why it matters:** A panic mid-test (e.g. a failed assertion) would leave the physical radio with `loopback=1` and −89.75 dB attenuation set, silently breaking the next RX run against real hardware.
- **Suggested response:** fix-in-plan — restore must not depend on the happy path: perform assertions after restoring, or restore via a guard whose `Drop` runs on unwind. Build will use the assert-after-restore ordering (simplest and panic-safe) so device state is always returned.

### C-004: Sprint advances INT-0002 but cannot realize it
- **Where:** `build-plan.md` Intents / `INT-0002` Acceptance criteria #3, #6
- **Failure mode:** intent-drift
- **Why it matters:** Even with TX working, INT-0002's multi-vendor (#3) and hotplug/overflow-recovery (#6) criteria remain unmet, and over-the-air transmit is deliberately unproven.
- **Suggested response:** defer-with-rationale — INT-0002 stays `active` at Loop. This sprint's honest claim is the **transmit half of criteria 1 and 2, verified under internal loopback**; over-the-air, multi-vendor (T-101), and hotplug remain carried forward. No criterion is silently marked met.

## Confidence
proceed-with-caveats
