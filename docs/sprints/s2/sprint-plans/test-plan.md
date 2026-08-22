Finalized - DO NOT EDIT

# Sprint 2 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0007](../../../intents/INT-0007-station-logging-cloudlog.md) | #4 README lists Cloudlog | T-011 / WHEN README renders THEN it SHALL contain a Cloudlog entry | test_readme_lists_cloudlog |
| [INT-0007](../../../intents/INT-0007-station-logging-cloudlog.md) | #1 push CAT to /api/radio | T-012 / WHEN push_radio THEN POST /api/radio, Ok on 200 | test_push_radio_ok |
| [INT-0007](../../../intents/INT-0007-station-logging-cloudlog.md) | #1/#3 auth handling | T-012 / WHEN 401 THEN Auth error, no panic | test_push_radio_401 |
| [INT-0007](../../../intents/INT-0007-station-logging-cloudlog.md) | #2 ADIF QSO to /api/qso | T-012 / WHEN log_qso THEN valid ADIF POST /api/qso | test_log_qso_adif |
| [INT-0007](../../../intents/INT-0007-station-logging-cloudlog.md) | #2 ADIF record shape | T-012 / (ADIF builder) | test_adif_record_format |
| [INT-0007](../../../intents/INT-0007-station-logging-cloudlog.md) | #3 key not logged | T-012 / WHEN key set THEN SHALL NOT log key | test_api_key_not_logged |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #2 real Pluto I/O — handshake | T-013 / WHEN connect THEN VERSION handshake + context XML | test_iiod_command_framing (+ live: hw_verify_pluto_connect) |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #2 real Pluto I/O — enumerate | T-013 / WHEN context parsed THEN enumerate ad9361-phy + cf-ad9361-lpc | test_iiod_context_parse |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #2 real Pluto I/O — control | T-013 / WHEN set_* THEN write ad9361-phy attr, errors as SdrError | test_iiod_attr_write_framing, test_iiod_error_surfaced |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #2 real Pluto I/O — stream | T-013 / WHEN read_samples THEN int16 IQ → Complex32, non-zero on hw | test_iiod_iq_int16_to_complex32 (+ live: hw_verify_pluto_rx) |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #2 defaults | T-013 / WHEN default PlutoSdr THEN ip:192.168.2.1 port 30431 | test_pluto_defaults_addr_port |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #3 multi-device / default builds C-lib-free | T-014 / WHEN no feature THEN build with Mock+Pluto only | test_list_devices_mock (+ CI default `cargo test`) |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #3 SoapySDR backend | T-014 / WHEN soapysdr feature THEN SoapySdr impls SdrDriver + enumerates | cargo check --features soapysdr (gated; documented) |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #4 CLI selects device | T-016 / WHEN --driver pluto THEN construct PlutoSdr | test_cli_record_driver_mock, test_cli_record_driver_pluto_unreachable |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #4 devices listing | T-016 / WHEN devices THEN print enumerated devices | test_cli_devices_lists |
| [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md) | #4 Rigctl over TCP — bind/accept | T-015 / WHEN rigctl starts THEN bind TCP + accept | test_rigctl_server_get_freq |
| [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md) | #4 Rigctl over TCP — respond | T-015 / WHEN client sends cmd THEN reply + hold until q | test_rigctl_server_set_and_get_mode, test_rigctl_server_dump_state |
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | zero-copy consequence (opt.) | T-017 / WHEN step repeats THEN reuse buffer, identical output | test_linear_pipeline_multi_step_correctness, test_linear_pipeline_buffer_reused |

## Unit Tests

### T-011 unit tests
- **Intent:** [INT-0007](../../../intents/INT-0007-station-logging-cloudlog.md)
- `test_readme_lists_cloudlog`: README content contains `magicbug/Cloudlog`. (Repo-level doc test.)

### T-012 unit tests
- **Intent:** [INT-0007](../../../intents/INT-0007-station-logging-cloudlog.md)
- `test_push_radio_ok`: mock server returns 200 → `Ok`; asserts POST path `/index.php/api/radio` and JSON keys `{key,radio,frequency,mode,power,timestamp}`.
- `test_push_radio_401`: mock server returns 401 → `Err(Auth)`, no panic.
- `test_log_qso_adif`: mock server captures body → path `/index.php/api/qso`, `type:"adif"`, `string` present.
- `test_adif_record_format`: ADIF builder emits `<CALL:..>`, `<QSO_DATE:..>`, `<BAND:..>`, `<MODE:..>`, `<EOR>`.
- `test_api_key_not_logged`: capture log output during a call → key value absent.
- Stubs: `tiny_http` mock server bound to `127.0.0.1:0`.

### T-013 unit tests
- **Intent:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- `test_iiod_context_parse`: fixture context XML → `DeviceInfo` list contains `ad9361-phy` and `cf-ad9361-lpc` with channel counts.
- `test_iiod_iq_int16_to_complex32`: interleaved `[i0,q0,i1,q1,...]` int16 → `Complex32` with correct scaling/order.
- `test_iiod_command_framing`: `VERSION`/`PRINT`/`OPEN`/`READBUF` command bytes match the iiod line protocol.
- `test_iiod_attr_write_framing`: attribute write for LO freq/rate/gain emits the correct `WRITE ad9361-phy ...` frame.
- `test_iiod_error_surfaced`: an iiod error response maps to `SdrError` (no panic).
- `test_pluto_defaults_addr_port`: default `PlutoSdr` endpoint is `ip:192.168.2.1`, port `30431`.
- Stubs: captured iiod byte fixtures; no radio required.

### T-014 unit tests
- **Intent:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- `test_list_devices_mock`: `list_devices()` on the default build returns the mock/pluto entries and needs no C library.
- Gated: `cargo check --features soapysdr` compiles the `SoapySdr` backend (documented; not part of default CI because it needs SoapySDR host libs).

### T-015 unit tests
- **Intent:** [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md)
- `test_rigctl_server_get_freq`: server on `127.0.0.1:0`; client sends `f\n` → receives the frequency line.
- `test_rigctl_server_set_and_get_mode`: `M USB 2800\n` then `m\n` → `USB` + width; connection stays open.
- `test_rigctl_server_dump_state`: `\dump_state\n` → multi-line dump ending `RPRT 0`.

### T-016 unit tests
- **Intent:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- `test_cli_record_driver_mock`: `record --driver mock` produces a capture using the mock driver.
- `test_cli_record_driver_pluto_unreachable`: `record --driver pluto` with an unreachable endpoint exits non-zero with a clear error, no panic.
- `test_cli_devices_lists`: `devices` prints at least the mock device.

### T-017 unit tests
- **Intent:** [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md)
- `test_linear_pipeline_multi_step_correctness`: many `step()` calls over a source produce the same output as a single-shot reference.
- `test_linear_pipeline_buffer_reused`: the reusable buffer field retains capacity across `step()` calls (no per-call reallocation).

## Integration Tests
### Station logging integration
- **Intents:** [INT-0007](../../../intents/INT-0007-station-logging-cloudlog.md)
- `test_cloudlog_radio_then_qso`: against one mock server, `push_radio` then `log_qso` both succeed and hit the correct endpoints in order.

### Hardware driver integration
- **Intents:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- `test_pluto_iiod_replay`: a scripted iiod mock socket answers `VERSION`/`PRINT`/`OPEN`/`READBUF`; `PlutoSdr` connects, enumerates, and reads a synthetic IQ buffer end-to-end (no physical radio).

## End-to-End Tests
- **Status:** possible
- `test_cli_e2e_record_demod` (existing, extended): `record --driver mock` → `info` → `demod` still passes with the new driver-selection code path.
- **Live hardware E2E (performed by the agent, recorded as INT-0002 evidence, not CI):**
  `hw_verify_pluto_connect` (connect + enumerate against `192.168.2.1:30431`) and
  `hw_verify_pluto_rx` (`record --driver pluto` → non-zero IQ → `info`). These
  prove INT-0002 #2/#6 on real hardware; they gate INT-0002 realization and are
  not run in CI (no radio in CI). SoapySDR device E2E and Pluto TX are unlocked
  by a follow-on hardware sprint with those devices attached.
