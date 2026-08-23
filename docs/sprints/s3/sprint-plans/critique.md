# Plan Critique — Sprint 3

## Concerns

### C-001: Several INT-0008 acceptance criteria have no Phase A work
- **Where:** `INT-0008` Acceptance criteria #1 (tun half), #2, #4 / `build-plan.md` Intents
- **Quote:** "a `tun` interface mode carries real IP on at least one supported OS" (#1); "Multi-hop routing works … via `babeld`" (#2); "Addressing uses routable IP (AREDN-compatible … subnet …)" (#4)
- **Failure mode:** intent-drift
- **Why it matters:** An acceptance criterion with no planned work could read as an unproven promise if the intent were claimed realized.
- **Suggested response:** defer-with-rationale — INT-0008 is explicitly **phased** and this sprint advances only Phase A. The real `tun` device and `babeld`/AREDN interop need OS privileges and an AREDN neighbor (Phase B); addressing follows there; on-air is Phase C (gated on Pluto TX, T-102). The `test-plan.md` E2E section names each unlocker, and INT-0008 stays `active` at Loop (not `realized`). No criterion is silently dropped.

### C-002: `test_gate_encrypted_refused` depends on a cataloged encryption-prohibited band
- **Where:** `test-plan.md` T-020 / `build-plan.md` T-020
- **Quote:** "encrypted on a cataloged band where encryption is prohibited → `Refuse`"
- **Failure mode:** weak-assertion (potential)
- **Why it matters:** If `REGULATORY_BANDS` contains no entry with `encryption_permitted=false` in range, that test could not exercise the true refusal path (only the uncataloged-frequency path).
- **Suggested response:** tighten-assertion at build time — inspect `crates/sdr-core/src/compliance.rs` `REGULATORY_BANDS`; if an amateur/encryption-prohibited band exists, assert the in-band refusal; otherwise document that `test_gate_uncataloged_refused` carries the refusal path and add an amateur band as a small INT-0005 follow-up. Verified during T-020.

### C-003: Two-node loopback must stay deterministic
- **Where:** `test-plan.md` `test_two_node_datagram_roundtrip`
- **Failure mode:** flake-risk
- **Why it matters:** A threaded/timed link would introduce nondeterminism.
- **Suggested response:** fix-in-plan — `LoopbackLink` is a synchronous in-memory queue (no threads, no timers); the test drives send→pump→recv deterministically. No flake surface.

## Confidence
proceed-with-caveats
