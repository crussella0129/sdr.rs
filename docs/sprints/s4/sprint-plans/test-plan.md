Finalized - DO NOT EDIT

# Sprint 4 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #1 TX transport (DEBUG token) | T-023 / WHEN debug attr command built THEN DEBUG token | test_iiod_debug_direction_token |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #1 TX sample encoding | T-023 / WHEN Complex32 converted THEN interleaved LE int16 × 32768, clamped | test_iiod_complex32_to_iq_bytes, test_iiod_tx_scale_clamps |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #1 TX buffer transport | T-023 / WHEN write_buf THEN WRITEBUF + payload, errno → SdrError | test_iiod_writebuf_framing, test_iiod_writebuf_error_surfaced |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #1 TX capability advertised | T-024 / WHEN has_tx THEN true | test_pluto_has_tx |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #2 TX streaming | T-024 / WHEN write_samples after start_tx THEN WRITEBUF, count returned | test_pluto_iiod_tx_replay (+ live: hw_verify_pluto_tx_loopback) |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #2 no transmit when inactive | T-024 / WHEN write_samples without start_tx THEN Ok(0), no transmit | test_pluto_write_samples_inactive_is_noop |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #2 TX gain validation | T-024 / WHEN set_tx_gain out of −89.75…0 THEN error, no write | test_pluto_tx_gain_range |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #6 safe device control | T-025 / WHEN enter_loopback_test_mode THEN loopback=1 + max attenuation | test_loopback_mode_values (+ live: hw_verify_pluto_tx_loopback) |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #6 state restoration | T-025 / WHEN exit_loopback_test_mode THEN restore saved mode + gain | hw_verify_pluto_tx_loopback (live) |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | zero-emission constraint | T-025 / WHEN loopback represented THEN mode 2 not constructible | test_loopback_mode_excludes_rf |

## Unit Tests

### T-023 unit tests
- **Intent:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- `test_iiod_debug_direction_token`: `cmd_read_channel`/`cmd_write_channel_header` with `Direction::Debug` emit the `DEBUG` token.
- `test_iiod_complex32_to_iq_bytes`: `Complex32(1.0, -1.0)` → int16 `32767 / -32768` (interleaved LE); round-trip shape matches `iq_bytes_to_complex32`'s ordering.
- `test_iiod_tx_scale_clamps`: input beyond ±1.0 clamps to the int16 bounds rather than wrapping.
- `test_iiod_writebuf_framing`: `cmd_writebuf("iio:device2", 4096)` == `"WRITEBUF iio:device2 4096"`.
- `test_iiod_writebuf_error_surfaced`: a negative status maps to `SdrError::Hardware` (no panic).

### T-024 unit tests
- **Intent:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- `test_pluto_has_tx`: `has_tx()` is `true`.
- `test_pluto_tx_gain_range`: −89.75 and 0.0 accepted; −100.0 and +1.0 rejected with a config error (no device write attempted while disconnected).
- `test_pluto_write_samples_inactive_is_noop`: without `start_tx`, `write_samples` returns `Ok(0)`.

### T-025 unit tests
- **Intent:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- `test_loopback_mode_values`: `Disabled` → `"0"`, `InternalDigital` → `"1"`.
- `test_loopback_mode_excludes_rf`: the enum exposes exactly the internal-loopback modes; no variant maps to `"2"` (the FPGA RX→TX mode that transmits).

## Integration Tests
### PlutoSDR TX over mock iiod (INT-0002)
- **Intents:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- `test_pluto_iiod_tx_replay`: the existing in-process mock iiod server (extended to answer `WRITEBUF` and `DEBUG` attribute access) observes the driver's full TX sequence — connect, TX configuration writes, `OPEN` on the TX device, `WRITEBUF` with the expected interleaved S16 payload — and `write_samples` reports the sample count. No radio required.

## End-to-End Tests
- **Status:** possible — live hardware E2E performed by the agent under internal loopback (CI remains hardware-free; hardware tests are `#[ignore]`d).
- `hw_verify_pluto_tx_loopback` (live, agent-run, **loopback**): against the physical Pluto+ at `192.168.2.1:30431` — `enter_loopback_test_mode()` (sets `loopback=1`, RF section bypassed, and TX gain to −89.75 dB) → configure TX → `start_tx` → write a known IQ pattern → `start_rx` → read back → assert non-zero received samples → `exit_loopback_test_mode()` and `teardown` restore prior device state.
- **Not-yet-possible (named unlockers):**
  - Over-the-air transmit verification → requires the user's separate explicit go-ahead on band/power/antenna; would be gated through the `sdr-mesh` compliance DB (INT-0005/INT-0008). Deliberately excluded from this sprint.
  - Two-radio on-air link and mesh multi-hop → mesh **Phase C** (INT-0008), which this sprint unblocks but does not deliver.
