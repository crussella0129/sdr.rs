# Sprint 7 — Test Report

- **Verdict:** pass, with caveats recorded below
- **Critique:** [critique.md](critique.md) — `proceed-with-caveats`, all five concerns addressed
- **Date:** 2026-08-23
- **Canonical runner:** `cargo test --workspace` — **32 suites, 0 failed**
- **Lint:** `cargo clippy --workspace --all-targets` — 0 errors
- **CI authority:** no hosted CI on this repository; the local workspace runner
  is the canonical suite. Stated plainly rather than implying a green badge.
- **Live hardware:** `cargo test -p sdr-mesh --test hw_radio -- --ignored --nocapture --test-threads=1` — 2 passed against the physical Pluto+

## What this sprint set out to correct

INT-0006 has been named "SSH over radio" since Sprint 1 and was marked
`realized` on criterion 4 in that sprint. Sprint 2's audit found no runtime
existed: `Commands::Tunnel` printed a readiness banner and exited, and
`StreamTunnel::packetize` was never called outside its own unit tests. The
intent was re-opened to `active` and criterion 4 restated to what is actually
demonstrable.

This sprint built the missing runtime and verified it. The headline result:

```text
debug1: Executing proxy command: exec "…\sdr-cli.exe" tunnel --stdio --driver mock
debug1: Local version string SSH-2.0-OpenSSH_10.3
debug1: Remote protocol version 2.0, remote software version OpenSSH_10.3
```

A real OpenSSH client completes the SSH version exchange over the radio link.

## EARS clause coverage

| Task | EARS clause | Executed test | Result |
|---|---|---|---|
| T-035 | WHEN a capture contains several valid frames THEN each SHALL be returned in turn | `test_radiolink_recovers_burst_of_datagrams` | pass |
| T-035 | WHEN a capture contains one frame THEN behaviour SHALL be unchanged | `test_radiolink_datagram_roundtrip_over_mock` (carried over) | pass |
| T-035 | WHEN a frame exceeds one capture THEN it SHALL be unrecovered, not truncated | `test_radiolink_oversized_frame_not_truncated` | pass |
| T-036 | WHEN a stream longer than the MTU is sent THEN it SHALL be split and reassembled in order | `test_stream_bridge_chunks_to_mtu`, `test_stream_bridge_roundtrip_multi_chunk` | pass |
| T-036 | WHEN arbitrary binary bytes are carried THEN the stream SHALL be equal byte-for-byte | `test_stream_bridge_binary_safe` | pass |
| T-036 | WHEN no datagram is available THEN no bytes and no error | `test_stream_bridge_empty_read` | pass |
| T-037 | WHEN `tunnel --stdio` runs THEN stdin SHALL be transmitted and received bytes written to stdout | `test_cli_tunnel_stdio_pipes_bytes` | pass |
| T-037 | WHEN the band prohibits encrypted payloads THEN the decision SHALL be surfaced | `test_cli_tunnel_reports_compliance` | pass |
| T-038 | WHEN a real `ssh` client runs with the bridge as its ProxyCommand THEN it SHALL receive a protocol version string | `test_ssh_proxycommand_exchanges_version` | pass |
| T-038 | WHEN a byte stream is carried over the Pluto+ THEN it SHALL be recovered byte-for-byte | `hw_verify_stream_over_radio` (live) | pass |
| T-038 | WHEN `ssh` is unavailable THEN skip cleanly | `test_ssh_client_banner_traverses_bridge` | **not executed** — see caveats |

## Intent verification

### INT-0006 criterion 4 — **verified**

> "a `sdr-cli tunnel` process pipes bidirectional byte streams (stdin/stdout,
> the OpenSSH `ProxyCommand` contract, or a TCP proxy socket) through the radio
> link, splitting the stream into MTU-sized datagrams and reassembling it in
> order byte-for-byte. A real OpenSSH **client** launched with the bridge as its
> `ProxyCommand` exchanges protocol version strings across the link."

All three transports are exercised, each against the real binary:
`--stdio` (`test_cli_tunnel_stdio_pipes_bytes`), `ProxyCommand`
(`test_ssh_proxycommand_exchanges_version`), and the TCP socket
(`test_ssh_client_banner_traverses_bridge`).

The criterion names `ProxyCommand` explicitly. During the build phase that form
appeared broken on Windows and only the TCP mode was verified; re-tested against
the finished runtime, ProxyCommand works — the earlier failure was the
incomplete tunnel, not Windows. The test the criterion names was added rather
than amending the criterion to match the test that happened to pass.

### INT-0008 criterion 1 — **partially verified**

> "IP datagrams are framed over the `packet.rs` link layer and recovered without
> corruption between two nodes (loopback/mock), **and a `tun` interface mode
> carries real IP on at least one supported OS**."

The **framing and recovery half is verified**, including on real hardware.
The **`tun` half is not addressed by this sprint at all** — no `tun` interface
is created and no real IP is carried. That is Mesh Phase B (backlog T-107).

INT-0008 must therefore remain `active`. Marking it realized here would repeat
precisely the failure Sprint 2's audit found in INT-0002, INT-0004 and INT-0006.

## Caveats — stated, not implied away

1. **No SSH session is verified.** There is no `sshd` on this machine, so the
   peer version string the client receives is **its own**, echoed by the
   loopback. Key exchange, authentication and a shell are unverified — backlog
   **T-114**. Both ssh tests say so in their doc comments.
2. **Ordering is demonstrated on a lossless channel only.** `MockSdr` and the
   Pluto internal loopback cannot drop or reorder. The receive path validates
   CRC-32 but performs no ARQ retransmission, so on a lossy channel a dropped
   frame would silently truncate a stream — backlog **T-113**.
3. **The graceful-skip clause never executed.** `ssh` is present here, so that
   branch was not taken in any recorded run. Recorded as *not executed* rather
   than verified; a container without `ssh` would exercise it.
4. **Streaming over one radio is ping-pong, not continuous.** A cyclic transmit
   buffer holds one frame and repeats it, so chunks are written and read back
   one at a time and repeats are discarded. This is a property of driving a
   single radio's cyclic DMA buffer, not a defect in `StreamBridge` — backlog
   **T-115**.
5. **Hardware tests must run serially.** One radio allows one open buffer;
   parallel runs fail with iiod errno 16 (EBUSY). Documented in the test module;
   a driver-level guard is backlog **T-116**.

## Defects this phase found

- **`StreamBridge::pump` could hang forever.** Its unbounded
  `while let Some(..)` drain never returns against a transmitter repeating a
  frame from a cyclic buffer. Found by the live hardware test, which hung. This
  was a genuine deployment bug, not a test artifact — a real peer transmitting
  continuously would have triggered it. Fixed by bounding the drain at
  `PUMP_MAX_DATAGRAMS`, with `clear_inbound` added to discard repeats.
- **`--listen` gave no readiness signal.** The TCP mode bound its port silently,
  leaving both the test and a human operator guessing. It now reports
  `Listening on TCP port {port}` on stderr. This removed a fixed-sleep race in
  the test suite: runtime fell from ~20 s to ~0.35 s, stable across 3/3 runs.

## Regression contract

The seven `radio_it` and `hw_radio` tests carried forward from Sprints 5 and 6
ran **unchanged** and all pass. This contract caught two real defects in
Sprint 6, which is why it is maintained rather than rebaselined.

## Test evidence links

- [unit-tests.md](unit-tests.md)
- [integration-tests.md](integration-tests.md)
- [e2e-tests.md](e2e-tests.md)
- [critique.md](critique.md)
