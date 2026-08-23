Finalized - DO NOT EDIT

# Sprint 6 Build Plan

## Intents
- [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) — state: active; acceptance criterion 1 strengthened: datagram recovery stops depending on transmitter and receiver sharing one clock. This is the prerequisite for attempting criterion 2 (multi-hop across separate nodes) honestly. Criteria 2 and 4 and the `tun` half of 1 remain Phase B (T-107); over-the-air remains T-108.
- [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) — state: realized (context); acceptance criterion 4 covers synchronization loops. `GardnerClockRecovery` has existed since Sprint 0 with no correctness coverage; T-031 supplies it. No criterion changes and no state change.
- [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) — state: realized (context); gains a timing-recovering demodulator alongside the existing `FskDemod`. No criterion changes.

## Schema Tree
- Sprint Goal: replace the sample-phase search with real symbol-timing recovery
  - DSP foundation
    - T-031: real test coverage for `GardnerClockRecovery`
    - T-032: `FskTimingDemod` (discriminator → Gardner → slice)
  - Mesh receiver
    - T-033: switch `RadioLink` to timing recovery
  - Hardware verification
    - T-034: live re-verification on the Pluto+

## Execution Sequence

### T-031: Real test coverage for `GardnerClockRecovery`
- **Intent:** [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md)
- **Touches:** crates/sdr-dsp/src/lib.rs
- **Depends on:** (none)
- **Acceptance criterion:** INT-0001 #4 — synchronization loops maintain timing without slippage. The component has carried no correctness evidence until now.
- **Success criterion (EARS):**
  - **WHEN** a known alternating PAM sequence is processed by the loop, **THEN** the recovered symbols **SHALL** match the transmitted sequence (allowing a bounded start-up lag), not merely be non-empty.
  - **WHEN** the input is resampled to simulate a receiver clock offset, **THEN** the recovered symbols **SHALL** still match the transmitted sequence.
- **Notes:** the existing `test_gardner_clock_recovery` asserts only output length; a broken timing-error detector would pass it. Strengthened before anything new depends on the component.

### T-032: `FskTimingDemod` — discriminator → Gardner → slice
- **Intent:** [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md)
- **Touches:** crates/sdr-demod/src/fsk.rs, crates/sdr-demod/src/lib.rs
- **Depends on:** T-031
- **Acceptance criterion:** INT-0006 #3 — the modulator's bursts must be demodulable; this makes them demodulable without assuming symbol alignment.
- **Success criterion (EARS):**
  - **WHEN** an FSK burst is demodulated sample-aligned, **THEN** the recovered bits **SHALL** equal the transmitted bits.
  - **WHEN** the burst starts mid-symbol, **THEN** the recovered bits **SHALL** still equal the transmitted bits (allowing a bounded lag).
  - **WHEN** the receiver clock differs from the transmitter's by up to ±0.5%, **THEN** the recovered bits **SHALL** still equal the transmitted bits.
- **Notes:** Gardner's TED assumes a linear modulation, so the frequency discriminator must run **first** — FSK is constant-envelope and offers the TED nothing on raw IQ. Added **alongside** `FskDemod`, which `PskDemod` and the Sprint 5 round-trip regression still use.

### T-033: Switch `RadioLink` to timing recovery
- **Intent:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- **Touches:** crates/sdr-mesh/src/radio.rs, crates/sdr-mesh/tests/radio_it.rs
- **Depends on:** T-032
- **Acceptance criterion:** INT-0008 #1 — datagrams recovered without corruption, now without requiring a shared clock.
- **Success criterion (EARS):**
  - **WHEN** received IQ contains a valid frame, **THEN** `recv_datagram` **SHALL** recover the datagram in a single demodulation pass, with no candidate-phase loop.
  - **WHEN** the received stream carries a clock offset within the supported range, **THEN** the datagram **SHALL** still be recovered byte-for-byte.
  - **WHEN** no valid frame is present, **THEN** `recv_datagram` **SHALL** return `Ok(None)` without erroring or panicking.
- **Notes:** `sync_to_frame` + CRC-32 stay, supplying frame alignment and validation and absorbing the loop's 0–1 symbol start-up lag. The module doc describing the phase search (and naming its own replacement) is updated.

### T-034: Live re-verification on the Pluto+
- **Intent:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- **Touches:** crates/sdr-mesh/tests/hw_radio.rs
- **Depends on:** T-033
- **Acceptance criterion:** INT-0008 #1 on real hardware, with the phase search removed.
- **Success criterion (EARS):**
  - **WHEN** the live test runs against the physical Pluto+ with timing recovery in place, **THEN** the identical datagram **SHALL** be recovered.
- **Notes:** uses the standing internal-loopback configuration (maximum attenuation, DDS silenced) and restores device state before assertions. A cyclic buffer's wrap discontinuity may briefly unlock the loop; capturing ≥ 2× the frame length keeps a complete frame clear of it.
