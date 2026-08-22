# Sprint 2 Unit Tests

- **Tested head:** `3af00bf3e1433aa11992bc634f03b99efcd9f4a7`
- **Runner:** `cargo test --workspace`
- **Result:** all unit suites pass (0 failed).

## T-012 — Cloudlog client (INT-0007)
- `test_adif_record_format`: ADIF builder emits `<CALL:4>W1AW`, `<QSO_DATE:8>…`, `<MODE:3>SSB`, `<FREQ:9>14.074000`, `<EOR>`. PASS.
- `test_adif_omits_absent_optionals`: absent optionals not serialized. PASS.
- `test_api_key_not_logged`: redacted `Debug` never prints the key. PASS.
- `test_endpoint_urls_trim_trailing_slash`: `/index.php/api/radio` and `/api/qso` URLs built correctly. PASS.

## T-013 — Pluto iiod driver (INT-0002)
- `test_iiod_command_framing`: `READ`/`OPEN`/`READBUF` lines match the iiod protocol (mask zero-padded to 32-bit word). PASS.
- `test_iiod_attr_write_framing`: `WRITE` header framing. PASS.
- `test_iiod_error_surfaced`: negative iiod status (`-EINVAL`) → `SdrError::Hardware`, no panic. PASS.
- `test_iiod_iq_int16_to_complex32`: interleaved int16 → normalized `Complex32` (order + scale). PASS.
- `test_iiod_context_parse`: real Pluto context XML fixture → enumerates `ad9361-phy` + `cf-ad9361-lpc` (4 RX scan channels, id `iio:device3`). PASS.
- `test_pluto_defaults_addr_port`: default endpoint `ip:192.168.2.1` port `30431`. PASS.
- `test_pluto_uri_explicit_port_preserved`, `test_pluto_rejects_out_of_range_frequency`. PASS.

## T-014 — Device enumeration (INT-0002)
- `test_list_devices_mock`: `list_devices()` always includes the mock device (deterministic with or without a radio). PASS.

## T-015 — Rigctl engine (INT-0004)
- Covered by the existing `RigctlHandler` unit tests plus the e2e server tests (see e2e-tests.md); the parser was reused unchanged.

## T-017 — LinearPipeline buffer reuse (INT-0001)
- `test_linear_pipeline_multi_step_correctness`: multi-step output equals the single-shot reference (`0,2,…,18`). PASS.
- `test_linear_pipeline_buffer_reused`: input buffer's backing pointer is stable across steps (no per-call reallocation), capacity ≥ 64. PASS.

## T-018 — Clippy deny-error fixes (INT-0005, INT-0003)
- Existing compliance and SSB unit tests continue to pass after the behavior-preserving fixes; `cargo clippy --workspace` reports 0 errors.
