Finalized - DO NOT EDIT

# Sprint 3 Build Plan

## Intents
- [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) — state: planned; acceptance criteria covered this sprint: 3 (compliance gate, fully) and 1 (IP framed over the link + recovered, the non-`tun` half). Criteria 2 (babeld multi-hop) and the real-`tun` half of 1, and 4 (AREDN subnet addressing), carry forward to Phase B.

## Schema Tree
- Sprint Goal: Mesh Phase A — IP-over-radio framing + dual-mode compliance gate (CI-verifiable; no tun/babeld/TX)
  - New crate `sdr-mesh`
    - T-019: KISS datagram framing
    - T-020: dual-mode compliance gate
    - T-021: MeshInterface seam + two-node loopback
  - Docs
    - T-022: AREDN + Babel in README references

## Execution Sequence

### T-019: KISS datagram framing over the radio link
- **Intent:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- **Touches:** crates/sdr-mesh/Cargo.toml, crates/sdr-mesh/src/lib.rs, crates/sdr-mesh/src/kiss.rs, Cargo.toml (workspace members + dep)
- **Depends on:** (none)
- **Acceptance criterion:** INT-0008 #1 — IP datagrams framed over the `packet.rs` link layer and recovered without corruption.
- **Success criterion (EARS):**
  - **WHEN** an IP datagram is KISS-encoded, **THEN** it **SHALL** be `FEND`(0xC0)-delimited with `0xC0`/`0xDB` byte-stuffed, and decoding **SHALL** return the exact original bytes.
  - **WHEN** a byte stream carrying multiple or partial KISS frames is fed to the streaming decoder, **THEN** it **SHALL** yield each complete datagram exactly once and buffer partial input without loss.
- **Notes:** KISS is the amateur-radio standard for datagram boundaries over a byte link. New `sdr-mesh` crate keeps networking out of the pure-DSP crates; deps `sdr-core`, `sdr-protocols`.

### T-020: Dual-mode compliance gate
- **Intent:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- **Touches:** crates/sdr-mesh/src/policy.rs, crates/sdr-mesh/src/lib.rs
- **Depends on:** T-019 (same crate)
- **Acceptance criterion:** INT-0008 #3 — consult `check_compliance(..., is_encrypted)` before encrypted TX and select encrypted/open or refuse.
- **Success criterion (EARS):**
  - **WHEN** `want_encrypted` and `RegulatoryDatabase::check_compliance(jur, freq, power, true)` is `Compliant`, **THEN** `MeshPolicy::evaluate` **SHALL** return `Allow(Encrypted)`.
  - **WHEN** `want_encrypted` and that check is `NonCompliant`, **THEN** it **SHALL** return `Refuse` carrying the regulator's reasons and **SHALL NOT** authorize an encrypted transmission.
  - **WHEN** `!want_encrypted` and `check_compliance(jur, freq, power, false)` is `Compliant`, **THEN** it **SHALL** return `Allow(Open)`.
- **Notes:** direct reuse of `sdr_core::compliance::RegulatoryDatabase` (INT-0005). No new bands; tests use cataloged bands.

### T-021: MeshInterface seam + two-node loopback
- **Intent:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- **Touches:** crates/sdr-mesh/src/node.rs, crates/sdr-mesh/src/lib.rs, crates/sdr-mesh/tests/loopback_it.rs
- **Depends on:** T-019, T-020
- **Acceptance criterion:** INT-0008 #1 (end-to-end datagram transit) and #3 (gate enforced at the node).
- **Success criterion (EARS):**
  - **WHEN** node A sends an IP datagram to node B over the in-memory loopback link, **THEN** B **SHALL** receive the identical datagram (carried through `ArqTransceiver` frames and KISS-reassembled).
  - **WHEN** a node sends with `want_encrypted` on a frequency where encryption is prohibited, **THEN** the node **SHALL** refuse (emit no frame) and surface the compliance reasons.
- **Notes:** `trait MeshInterface { send_datagram / recv_datagram }` is the seam a real `tun` (Phase B) plugs into; `LoopbackLink` is the CI-verifiable in-memory implementation over `ArqTransceiver`.

### T-022: Add AREDN + Babel to README references
- **Intent:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- **Touches:** README.md
- **Depends on:** (none)
- **Acceptance criterion:** INT-0008 rationale — capture the mesh reference architecture in the project catalog.
- **Success criterion (EARS):**
  - **WHEN** the README reference list is rendered, **THEN** it **SHALL** include AREDN (`https://github.com/aredn/aredn`) and the Babel routing RFC (RFC 8966) with one-line mesh-relevance notes.
- **Notes:** documentation task; verified by a content check (`test_readme_lists_aredn`).
