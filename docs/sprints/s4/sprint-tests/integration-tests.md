# Sprint 4 Integration Tests

- **Tested head:** `ab285092657e049991e0cd70f379aaec04d3de35`
- **Result:** pass (2 passed, 0 failed). No radio required.

## PlutoSDR TX over mock iiod (INT-0002)
`crates/sdr-hardware/tests/pluto_iiod.rs` — the in-process mock iiod server,
extended this sprint to answer `WRITEBUF` (two-phase) and `DEBUG`/attribute
reads, with command and payload capture for assertions.

- `test_pluto_iiod_tx_replay`: drives `start_tx` → `write_samples` and asserts
  the observed exchange:
  - `OPEN` issued on the **TX** device (`iio:device2`);
  - a well-formed `WRITEBUF iio:device2 16` (4 samples × 4 bytes);
  - TX tuning written to `OUTPUT altvoltage1 frequency` (the TX LO, not the RX LO);
  - the captured payload decodes as interleaved LE S16 at full scale —
    `i16::MAX`, `i16::MIN`, `16384` for `(1.0, −1.0)` and `0.5`;
  - `write_samples` reports all 4 samples accepted. PASS.
- `test_pluto_iiod_replay` (existing RX regression): unaffected by the TX
  changes. PASS.

The mock server's `WRITEBUF` handler mirrors the real daemon's two-phase
behaviour (header ack, then payload, then bytes-written), so this test would now
catch a regression to the single-status implementation that failed on hardware.
