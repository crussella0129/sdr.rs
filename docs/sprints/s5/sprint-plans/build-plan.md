Finalized - DO NOT EDIT

# Sprint 5 Build Plan

## Intents
- [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) — state: active; acceptance criterion advanced: 1 (IP datagrams framed over the link layer and recovered without corruption) — moving from the in-process `LoopbackLink` to a **real radio-backed** implementation, verified under AD9361 internal loopback with **internal loopback**. Criteria 2 (babeld multi-hop), the `tun` half of 1, and 4 (AREDN addressing) remain Phase B. Over-the-air remains out of scope (backlog T-108).
- [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) — state: realized (context, unchanged); supplies `ArqTransceiver`/`PacketFramer` and the FSK modulator. T-027 adds regression coverage for the modulator↔demodulator pair without altering any acceptance criterion.

## Schema Tree
- Sprint Goal: first true IP-over-radio transit, verified with loopback testing
  - DSP foundation
    - T-027: FSK modulator ↔ demodulator round-trip regression
  - Mesh radio link
    - T-028: bit-level frame synchronizer
    - T-029: `RadioLink` — radio-backed `MeshInterface` (+ CI over `MockSdr`)
  - Hardware verification
    - T-030: live datagram over the Pluto internal loopback

## Execution Sequence

### T-027: FSK modulator ↔ demodulator round-trip regression
- **Intent:** [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md)
- **Touches:** crates/sdr-demod/tests/fsk_roundtrip.rs
- **Depends on:** (none)
- **Acceptance criterion:** INT-0006 #3 — the modulator produces demodulable bursts; this pins the property the mesh now relies on.
- **Success criterion (EARS):**
  - **WHEN** a known byte pattern is FSK-modulated and then demodulated sample-aligned, **THEN** the recovered bits **SHALL** equal the transmitted bits exactly (zero bit errors).
  - **WHEN** the same IQ stream is demodulated starting from a large sample offset, **THEN** the recovered bits **SHALL** contain errors, documenting why the receiver must align.
- **Notes:** makes the Sprint 5 research measurement permanent (0/72 errors aligned; 37/71 at offset 7). Guards the DSP pair the whole mesh path now depends on.

### T-028: Bit-level frame synchronizer
- **Intent:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- **Touches:** crates/sdr-mesh/src/framesync.rs, crates/sdr-mesh/src/lib.rs
- **Depends on:** (none)
- **Acceptance criterion:** INT-0008 #1 — recovering a frame from a demodulated bit stream is a precondition for recovering the datagram.
- **Success criterion (EARS):**
  - **WHEN** a bit stream contains the sync word (`D3 91 D3 91`) at an arbitrary **bit** offset, **THEN** the synchronizer **SHALL** return bytes beginning at that sync word, packed MSB-first.
  - **WHEN** the bit stream contains no sync word, **THEN** it **SHALL** return `None`.
- **Notes:** pure and dependency-free; reuses `SYNC_WORD` from `sdr_protocols::packet`. Produces exactly the form `PacketFramer::decode` already scans for.

### T-029: `RadioLink` — a radio-backed `MeshInterface`
- **Intent:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- **Touches:** crates/sdr-mesh/src/radio.rs, crates/sdr-mesh/src/lib.rs, crates/sdr-mesh/Cargo.toml
- **Depends on:** T-027, T-028
- **Acceptance criterion:** INT-0008 #1 — datagrams framed over the radio link and recovered without corruption.
- **Success criterion (EARS):**
  - **WHEN** a datagram is sent through `RadioLink`, **THEN** it **SHALL** be emitted as modulated IQ through the driver's `write_samples`.
  - **WHEN** received IQ contains a valid frame at any sample phase, **THEN** `recv_datagram` **SHALL** recover the original datagram byte-for-byte.
  - **WHEN** received IQ contains no valid frame, **THEN** `recv_datagram` **SHALL** return `Ok(None)` and **SHALL NOT** error or panic.
- **Notes:** generic over `SdrDriver` so `MockSdr` serves CI and `PlutoSdr` serves hardware. Composes existing parts only — `kiss`, `ArqTransceiver`, `PacketFramer`, `FskModulator`, `FskDemod`. The sample-phase search is self-checking: CRC-32 validates the candidate phase.

### T-030: Live datagram over the Pluto internal loopback
- **Intent:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- **Touches:** the hardware test file (`#[ignore]`d, CI-excluded)
- **Depends on:** T-029
- **Acceptance criterion:** INT-0008 #1 on **real hardware**.
- **Success criterion (EARS):**
  - **WHEN** a datagram is sent through `RadioLink` on the physical Pluto+ with internal loopback engaged, **THEN** the identical datagram **SHALL** be recovered from the received IQ, with the RF section bypassed and the signal kept internal.
- **Notes:** uses the Sprint 4 safety helpers (`enter_loopback_test_mode`: RF bypassed, −89.75 dB, DDS silenced) and `set_tx_cyclic(true)` so the frame repeats; reads ≥ 2× the frame length. Device state restored **before** assertions, per Sprint 4 critique C-003.
