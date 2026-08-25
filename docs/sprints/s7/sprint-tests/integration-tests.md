# Sprint 7 — Integration Test Results

- **Tested head:** `2d77c1bf064a7b2258e9ab6a3b48219271be8399`
- **Runner:** `cargo test --workspace --tests`
- **Date:** 2026-08-23

## Regression contract — carried over unchanged

The four `radio_it` tests from Sprints 5 and 6 ran **unchanged**. A behaviour
change in them is a failure, not a rebaseline. (This contract caught two real
defects in Sprint 6, which is why it is kept.)

| Test | Origin | Result |
|---|---|---|
| `test_radiolink_datagram_roundtrip_over_mock` | Sprint 5 | ok |
| `test_radiolink_recovers_from_sample_offset` | Sprint 5 | ok |
| `test_radiolink_recovers_under_clock_drift` | Sprint 6 | ok |
| `test_radiolink_noise_returns_none` | Sprint 5 | ok |

All four still pass after T-035 restructured the receive path from
"decode the first frame and discard the capture" to a scanning loop. Notably
`test_radiolink_datagram_roundtrip_over_mock` is the direct evidence for the
EARS clause *WHEN a capture contains one frame THEN behaviour SHALL be
unchanged*.

## T-035 — multiple frames per capture (INT-0008 criterion 1)

`crates/sdr-mesh/tests/radio_it.rs` — 6 passed.

| Test | EARS clause | Result |
|---|---|---|
| `test_radiolink_recovers_burst_of_datagrams` | WHEN a capture contains several valid frames THEN each SHALL be returned in turn | ok |
| `test_radiolink_oversized_frame_not_truncated` | WHEN a frame exceeds one capture THEN it SHALL be unrecovered, not silently truncated | ok |

`test_radiolink_recovers_burst_of_datagrams` is the research probe that measured
**1 of 4** recovered before this sprint; it now recovers 4 of 4, in order.

## T-036 — `StreamBridge` over the mock radio (INT-0006 criterion 4)

`crates/sdr-mesh/tests/stream_it.rs` — 2 passed.

| Test | EARS clause | Result |
|---|---|---|
| `test_stream_bridge_roundtrip_multi_chunk` | WHEN a stream longer than the MTU is sent THEN it SHALL be reassembled in order | ok |
| `test_stream_bridge_carries_ssh_like_banner` | (supporting) an SSH-shaped banner survives the full framing/modulation path | ok |

These exercise the real `RadioLink` path — KISS framing, CRC-32, FSK modulation,
demodulation with symbol-timing recovery — not a stub.

## T-037 — `sdr-cli tunnel` runtime (INT-0006 criterion 4)

`crates/sdr-cli/tests/tunnel_it.rs` — 3 passed. Each spawns the **real binary**.

| Test | EARS clause | Result |
|---|---|---|
| `test_cli_tunnel_stdio_pipes_bytes` | WHEN `tunnel --stdio` runs THEN stdin SHALL be transmitted and received bytes written to stdout, until EOF | ok |
| `test_cli_tunnel_reports_compliance` | WHEN the band prohibits encrypted payloads THEN the decision SHALL be surfaced before traffic | ok |
| `test_cli_tunnel_status_goes_to_stderr` | (added during build) status must not pollute the tunnelled stream on stdout | ok |

`test_cli_tunnel_status_goes_to_stderr` was **not in the locked plan**. It was
added because routing status to stdout would corrupt the byte stream an SSH
client reads — a defect the planned tests would not have caught.

## Unaffected suites re-verified

`loopback_it` (2), `readme_it` (1), `driver_cli_it` (3), `rigctl_server_it` (3),
`cloudlog_it` (5), `pluto_iiod` (2), `fsk_roundtrip`/`fsk_timing`, plus
`sdr-protocols` and `sdr-station` suites — all pass. No suite regressed.
