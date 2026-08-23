# Test Critique — Sprint 3

## Concerns

### C-001: INT-0008 is only partially proven (by design — Phase A)
- **Where:** `INT-0008` Acceptance criteria #1 (real-`tun` half), #2, #4 / `e2e-tests.md`
- **Quote:** "a `tun` interface mode carries real IP" (#1); "Multi-hop routing works … `babeld`" (#2); "AREDN-compatible … subnet" (#4)
- **Failure mode:** intent-coverage
- **Why it matters:** These criteria have no executed test this sprint; claiming the intent realized would overstate coverage.
- **Suggested response:** defer-with-rationale — INT-0008 is phased and this sprint delivers Phase A only. The real `tun` device and `babeld`/AREDN interop need OS privileges and an AREDN neighbor (Phase B); on-air needs Pluto TX (Phase C, T-102). `e2e-tests.md` names each unlocker and INT-0008 stays `active` (not `realized`). Criteria #1 (framing/recovery) and #3 (the compliance gate) are fully proven.

### C-002: Loopback link does not model channel loss (ARQ retransmit path unexercised here)
- **Where:** `crates/sdr-mesh/tests/loopback_it.rs` (`LoopbackLink` delivers every frame)
- **Failure mode:** integration-drift (potential)
- **Why it matters:** The loopback proves datagram framing/transit but not recovery under packet loss; ARQ retransmission is exercised only by its own `sdr-protocols` unit tests, not through the mesh layer.
- **Suggested response:** defer-with-rationale — Phase A's promise is datagram framing + compliance gating over a working link; ARQ loss-recovery is already covered at its own layer (INT-0006, Sprint 1) and end-to-end loss behavior belongs with the on-air Phase C link. Not a Phase A gap.

## Confidence
proceed-with-caveats
