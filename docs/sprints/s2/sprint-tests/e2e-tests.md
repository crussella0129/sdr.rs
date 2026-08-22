# Sprint 2 End-to-End Tests

- **Tested head:** `3af00bf3e1433aa11992bc634f03b99efcd9f4a7`
- **Status:** possible — CI E2E runs green; live hardware E2E performed by the agent against the attached Pluto+.

## Rigctl TCP server E2E (INT-0004, T-015)
`crates/sdr-cli/tests/rigctl_server_it.rs` — each test launches the real
`sdr-cli rigctl --port <ephemeral>` binary and drives it over loopback TCP:

- `test_rigctl_server_get_freq`: bind + accept; `f` → `144000000`. PASS.
- `test_rigctl_server_set_and_get_mode`: `M USB 2800` → `RPRT 0`, then `m` → `USB` / `2800` on the still-open connection (proves it holds the connection open). PASS.
- `test_rigctl_server_dump_state`: `\dump_state` → multi-line dump ending `RPRT 0`. PASS.

## CLI device selection E2E (INT-0002, T-016)
`crates/sdr-cli/tests/driver_cli_it.rs` — runs the real CLI binary:

- `test_cli_record_driver_mock`: `record --driver mock` exits 0 and writes the capture file. PASS.
- `test_cli_record_driver_pluto_unreachable`: `record --driver ip:127.0.0.1:1` exits non-zero with no panic in stderr (graceful failure / negative path). PASS.
- `test_cli_devices_lists`: `devices` exits 0 and lists the mock device. PASS.

## Existing pipeline E2E (regression)
`crates/sdr-cli/tests/e2e_pipeline_tests.rs`: 5 tests pass — the record → info →
demod path is unaffected by the new driver-selection code.

## Live hardware E2E (performed by the agent; not run in CI)
Executed against the physical Pluto+ at `192.168.2.1:30431`
(`cargo test -p sdr-hardware --test hw_pluto -- --ignored`):

- `hw_verify_pluto_connect`: real iiod VERSION handshake + context enumeration. PASS.
- `hw_verify_pluto_rx`: tuned to 95.83 MHz FM, read 4096 samples — **4096/4096 non-zero**, peak |amp| ≈ 0.82. PASS.
- CLI live confirmation: `sdr-cli devices` enumerated `PlutoSDR / AD936x [ip:192.168.2.1]`; `sdr-cli record --driver pluto --freq 95830000 --rate 3000000 --samples 8192` captured 8192 real IQ samples to WAV, re-read by `sdr-cli info`.

This live evidence proves INT-0002 criteria 2 (real Pluto streaming) and 4 (CLI
selects the device) on real hardware. SoapySDR/RTL/HackRF/Airspy device E2E and
Pluto TX are unlocked by the backlog hardware follow-on (T-101, T-102).
