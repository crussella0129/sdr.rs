Finalized - DO NOT EDIT

# Sprint 4 Build Plan

## Intents
- [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) — state: active; acceptance criteria advanced: 1 (continuous asynchronous RX/**TX** buffer streaming) and 2 (real Pluto I/O, transmit half). Verified under AD9361 internal loopback only — **no RF radiated** this sprint, so over-the-air transmit remains unproven by design. Criteria 3 (multi-vendor) and 6 (hotplug/overflow) remain carried forward.

## Schema Tree
- Sprint Goal: PlutoSDR transmit over iiod, verified with zero emission
  - iiod client
    - T-023: DEBUG direction, WRITEBUF, S16 conversion
  - PlutoSDR driver
    - T-024: TX path (config + start/write/stop)
    - T-025: loopback safety controls
  - Verification
    - T-026: mock-iiod CI regression + live loopback test

## Execution Sequence

### T-023: iiod client — DEBUG direction, WRITEBUF, and S16 TX conversion
- **Intent:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- **Touches:** crates/sdr-hardware/src/iiod.rs
- **Depends on:** (none)
- **Acceptance criterion:** INT-0002 #1 — the driver layer must support TX buffer streaming, which requires the `WRITEBUF` transport and correct sample encoding.
- **Success criterion (EARS):**
  - **WHEN** a debug-attribute command is built, **THEN** it **SHALL** use the `DEBUG` direction token (e.g. `READ <dev> DEBUG loopback`).
  - **WHEN** `Complex32` samples are converted for transmission, **THEN** they **SHALL** be encoded as interleaved little-endian int16 scaled by the S16 full scale (32768) and **SHALL** clamp out-of-range input to the int16 bounds rather than wrapping.
  - **WHEN** `write_buf` is called, **THEN** it **SHALL** emit `WRITEBUF <dev> <nbytes>` followed by the raw payload, and **SHALL** surface a negative iiod status as an `SdrError`.
- **Notes:** TX uses `le:S16/16` (full scale 32768) while RX uses `le:S12/16` (2048) — probed live; the existing `AD9361_RX_FULL_SCALE` must **not** be reused for TX. Mirrors the existing `read_buf`/`iq_bytes_to_complex32` pair.

### T-024: PlutoSdr transmit path
- **Intent:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- **Touches:** crates/sdr-hardware/src/pluto.rs
- **Depends on:** T-023
- **Acceptance criterion:** INT-0002 #1 (RX/TX streaming) and #2 (real Pluto I/O, transmit half).
- **Success criterion (EARS):**
  - **WHEN** `has_tx()` is called on a PlutoSDR driver, **THEN** it **SHALL** return `true`.
  - **WHEN** `write_samples` is called after `start_tx`, **THEN** the driver **SHALL** open the TX buffer when needed and emit the samples via `WRITEBUF`, returning the number of samples written.
  - **WHEN** `write_samples` is called without a preceding `start_tx`, **THEN** it **SHALL** return `Ok(0)` and **SHALL NOT** transmit.
  - **WHEN** `set_tx_gain` is called with a value outside −89.75…0 dB, **THEN** it **SHALL** return a configuration error and **SHALL NOT** write the attribute.
- **Notes:** TX device is `cf-ad9361-dds-core-lpc` (fallback `iio:device2`), TX_LO is `OUTPUT altvoltage1`, TX control channel is `OUTPUT voltage0`. TX `hardwaregain` is **attenuation** (0 dB = full output, −89.75 dB = maximum attenuation), range probed live. Mirrors `read_samples`' lazy-open bookkeeping.

### T-025: Loopback safety controls (no-emission contract in code)
- **Intent:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- **Touches:** crates/sdr-hardware/src/pluto.rs
- **Depends on:** T-023, T-024
- **Acceptance criterion:** INT-0002 #6 (graceful, safe device control) and the sprint's zero-emission verification constraint.
- **Success criterion (EARS):**
  - **WHEN** `enter_loopback_test_mode()` succeeds, **THEN** the device **SHALL** have `loopback=1` (AD9361-internal, RF section bypassed) and TX gain at maximum attenuation (−89.75 dB).
  - **WHEN** `exit_loopback_test_mode()` runs, **THEN** it **SHALL** restore the previously saved loopback mode and TX gain.
  - **WHEN** the loopback mode is represented in the API, **THEN** the radiating FPGA mode (`loopback=2`, RX→TX with the RF chain active) **SHALL NOT** be constructible.
- **Notes:** per ADI's AD9361 driver documentation, mode 1 bypasses the entire RF section; mode 2 actively transmits and is excluded from this sprint by construction, not merely by convention.

### T-026: TX verification — mock-iiod CI regression and live loopback test
- **Intent:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- **Touches:** crates/sdr-hardware/tests/pluto_iiod.rs, crates/sdr-hardware/tests/hw_pluto.rs
- **Depends on:** T-023, T-024, T-025
- **Acceptance criterion:** INT-0002 #2 — real hardware transmit evidence (under loopback), plus a hardware-free regression.
- **Success criterion (EARS):**
  - **WHEN** the driver transmits against the mock iiod server, **THEN** the server **SHALL** observe a well-formed `WRITEBUF` command carrying the expected interleaved S16 payload.
  - **WHEN** the live loopback test runs against the physical Pluto+, **THEN** the written TX samples **SHALL** be observed on the RX path as non-zero data, with the RF section bypassed and no emission.
- **Notes:** the live test is `#[ignore]`d so CI never requires a radio; it engages loopback and maximum attenuation before any buffer write and restores prior state on completion.
