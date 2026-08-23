# Sprint 4 Unit Tests

- **Tested head:** `ab285092657e049991e0cd70f379aaec04d3de35`
- **Runner:** `cargo test --workspace` — 87 passed, 0 failed, 3 ignored (hardware).
- **Scope:** PlutoSDR transmit path. Pre-existing suites unchanged and green.

## T-023 — iiod TX transport (INT-0002)
- `test_iiod_debug_direction_token`: `READ <dev> DEBUG loopback` / `WRITE <dev> DEBUG loopback <len>` — the `DEBUG` token replaces the direction and no channel name appears. PASS.
- `test_iiod_writebuf_framing`: `cmd_writebuf("iio:device2", 4096)` == `"WRITEBUF iio:device2 4096"`. PASS.
- `test_iiod_writebuf_error_surfaced`: negative status → `SdrError::Hardware`, no panic. PASS.
- `test_iiod_complex32_to_iq_bytes`: `(1.0, −1.0)` → int16 `32767 / −32768`; `0.5` → `16384` (S16 full scale 32768, interleaved LE). PASS.
- `test_iiod_tx_scale_clamps`: ±5.0 saturates to the int16 bounds rather than wrapping polarity. PASS.
- `test_iiod_command_framing` (extended): `OPEN … CYCLIC` emitted when cyclic is requested, plain `OPEN` otherwise. PASS.

## T-024 — PlutoSdr TX path (INT-0002)
- `test_pluto_has_tx`: `has_tx()` is `true`. PASS.
- `test_pluto_tx_gain_range`: −89.75, −20.0 and 0.0 accepted; −100.0 and +1.0 rejected with a config error and no device write. PASS.
- `test_pluto_write_samples_inactive_is_noop`: without `start_tx`, `write_samples` returns `Ok(0)` — an accidental write cannot key the transmitter. PASS.
- `test_pluto_tx_defaults_to_max_attenuation`: a freshly constructed driver sits at −89.75 dB with TX inactive. PASS.

## T-025 — Loopback safety controls (INT-0002)
- `test_loopback_mode_values`: `Disabled` → `"0"`, `InternalDigital` → `"1"`. PASS.
- `test_loopback_mode_excludes_rf`: no variant maps to `"2"` — the radiating FPGA RX→TX mode is not constructible through this API. PASS.
