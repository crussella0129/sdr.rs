# Sprint 2 Integration Tests

- **Tested head:** `3af00bf3e1433aa11992bc634f03b99efcd9f4a7`
- **Result:** all integration suites pass.

## Station logging integration (INT-0007)
`crates/sdr-station/tests/cloudlog_it.rs` — client driven against a local
`tiny_http` mock server bound to `127.0.0.1:0` (no radio, no external network):

- `test_push_radio_ok`: `push_radio` → `POST /index.php/api/radio` with JSON `{key,radio,frequency,mode,timestamp}`; HTTP 200 → `Ok`. PASS.
- `test_push_radio_401`: HTTP 401 → `Err(CloudlogError::Auth)`, no panic (negative path). PASS.
- `test_log_qso_adif`: `log_qso` → `POST /index.php/api/qso` with `type:"adif"`, `station_profile_id`, ADIF `string` containing the callsign and `<EOR>`. PASS.
- `test_cloudlog_radio_then_qso`: both endpoints hit in order over one server. PASS.

## Hardware driver integration (INT-0002)
`crates/sdr-hardware/tests/pluto_iiod.rs` — the `PlutoSdr` driver driven against
an in-process mock iiod server that replays the real protocol framing (including
the trailing newline after text payloads, and none after `READBUF`):

- `test_pluto_iiod_replay`: `start_rx` performs VERSION + PRINT (+ context parse) + a WRITE tuning sequence, then `read_samples` performs OPEN + READBUF and converts 32 bytes → 8 non-zero `Complex32`. PASS. This exercises the same command sequencing verified live against the physical radio, without a radio.
