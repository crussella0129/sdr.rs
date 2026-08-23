# Sprint 7 — Unit Test Results

- **Tested head:** `2d77c1bf064a7b2258e9ab6a3b48219271be8399`
- **Runner:** `cargo test --workspace --lib`
- **Date:** 2026-08-23
- **CI authority:** no hosted CI on this repository; the local workspace runner is
  the canonical suite. Recorded as such rather than implying a green CI badge.

## Result

All library unit suites pass. No failures, no ignored unit tests.

## T-036 — `StreamBridge` (INT-0006 criterion 4)

`crates/sdr-mesh/src/stream.rs`, suite `sdr-mesh --lib`: 14 passed.

| Test | EARS clause | Result |
|---|---|---|
| `stream::tests::test_stream_bridge_chunks_to_mtu` | WHEN a stream longer than the MTU is sent THEN it SHALL be split into MTU-sized datagrams | ok |
| `stream::tests::test_stream_bridge_binary_safe` | WHEN arbitrary binary bytes are carried THEN the stream SHALL be equal byte-for-byte | ok |
| `stream::tests::test_stream_bridge_empty_read` | WHEN no datagram is available THEN no bytes and no error | ok |
| `stream::tests::test_stream_bridge_read_respects_max` | (supporting) a bounded read leaves the remainder buffered | ok |

`test_stream_bridge_binary_safe` covers all 256 byte values, including the
KISS-significant `0xC0`/`0xDB` and NUL — the bytes most likely to be eaten by a
framing bug.

## T-035 — frame sync helpers (INT-0008 criterion 1)

| Test | Result |
|---|---|
| `framesync::tests::test_framesync_finds_sync_at_bit_offsets` | ok |
| `framesync::tests::test_framesync_packs_msb_first` | ok |
| `framesync::tests::test_framesync_absent_returns_none` | ok |

## Compliance gate (unchanged this sprint, re-verified)

`policy::tests::{test_gate_open_allowed, test_gate_encrypted_refused,
test_gate_encrypted_ism_allowed, test_gate_uncataloged_refused}` — 4 passed.
The dual-mode rule (encrypted permitted on ISM, refused on amateur) still holds
after the tunnel became a live pipe.

## Full workspace unit tally

| Crate | Passed | Failed |
|---|---|---|
| sdr-core | 8 | 0 |
| sdr-dsp | 25 | 0 |
| sdr-demod | 8 | 0 |
| sdr-hardware | 25 | 0 |
| sdr-mesh | 14 | 0 |
| sdr-protocols | 6 | 0 |
| sdr-spectrum | 2 | 0 |
| sdr-station | 4 | 0 |

## Lint

`cargo clippy --workspace --all-targets` — no `error`-level diagnostics.
Pre-existing warnings (`map_or` simplification, `io::Error::other`) remain and
are tracked as backlog T-105; this sprint added none.
