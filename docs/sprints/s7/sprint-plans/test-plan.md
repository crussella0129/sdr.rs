Finalized - DO NOT EDIT

# Sprint 7 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 several frames per capture | T-035 / WHEN capture holds several frames THEN each returned in turn | test_radiolink_recovers_burst_of_datagrams |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 single-frame behaviour unchanged | T-035 / WHEN one frame THEN unchanged | test_radiolink_datagram_roundtrip_over_mock (carried over) |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 oversized frame | T-035 / WHEN frame exceeds a capture THEN unrecovered, not truncated | test_radiolink_oversized_frame_not_truncated |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #4 MTU chunking | T-036 / WHEN stream longer than MTU THEN split and reassembled in order | test_stream_bridge_chunks_to_mtu, test_stream_bridge_roundtrip_multi_chunk |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #4 byte-for-byte fidelity | T-036 / WHEN arbitrary binary carried THEN equal byte-for-byte | test_stream_bridge_binary_safe |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #4 empty read | T-036 / WHEN no datagram available THEN no bytes, no error | test_stream_bridge_empty_read |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #4 stdio pump | T-037 / WHEN tunnel --stdio runs THEN stdin transmitted, received written to stdout | test_cli_tunnel_stdio_pipes_bytes |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #4 compliance gate retained | T-037 / WHEN band prohibits encryption THEN decision surfaced | test_cli_tunnel_reports_compliance |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #4 real client exchanges protocol bytes | T-038 / WHEN real ssh runs with the bridge THEN receives a protocol version string | test_ssh_client_banner_traverses_bridge |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #4 graceful skip | T-038 / WHEN ssh unavailable THEN skip cleanly | test_ssh_client_banner_traverses_bridge |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 stream on real hardware | T-038 / WHEN stream carried over Pluto+ THEN recovered byte-for-byte | hw_verify_stream_over_radio (live) |

## Unit Tests

### T-035 unit tests
- **Intent:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- `test_radiolink_oversized_frame_not_truncated`: a datagram whose frame exceeds one capture yields `None` rather than a corrupt partial payload — failing loudly rather than silently returning wrong bytes.

### T-036 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md)
- `test_stream_bridge_chunks_to_mtu`: a stream several times the MTU produces the expected number of datagrams, none exceeding the MTU.
- `test_stream_bridge_binary_safe`: all 256 byte values, including KISS-significant `0xC0`/`0xDB` and NULs, survive unchanged.
- `test_stream_bridge_empty_read`: with nothing received, reading yields zero bytes and no error.

## Integration Tests

### `RadioLink` over `MockSdr` loopback (INT-0008)
- **Regression contract — carried over unchanged** from Sprints 5 and 6; a behaviour change in these is a failure, not a rebaseline (this contract caught two real defects in Sprint 6):
  - `test_radiolink_datagram_roundtrip_over_mock`
  - `test_radiolink_recovers_from_sample_offset`
  - `test_radiolink_recovers_under_clock_drift`
  - `test_radiolink_noise_returns_none`
- `test_radiolink_recovers_burst_of_datagrams` (new): four datagrams sent back-to-back and then drained are **all** recovered, in order. This is the measured failure (1 of 4) that motivates T-035.

### `StreamBridge` over `MockSdr` loopback (INT-0006)
- `test_stream_bridge_roundtrip_multi_chunk`: a byte stream spanning several MTUs is written into the bridge, carried over the radio path, and reassembled byte-for-byte in order.

### CLI runtime (INT-0006)
- `test_cli_tunnel_stdio_pipes_bytes`: spawns the real `sdr-cli tunnel --stdio` binary, writes bytes to its stdin, and observes them returned on stdout through the loopback — proving the command is a working pipe rather than a status printer.
- `test_cli_tunnel_reports_compliance`: on a band where encrypted payloads are prohibited, the command surfaces the compliance decision.

## End-to-End Tests
- **Status:** possible — a real OpenSSH client in CI, and live hardware performed by the agent.
- `test_ssh_client_banner_traverses_bridge`: launches `ssh -v` with `-o ProxyCommand="…tunnel --stdio…"` and asserts from its verbose output that a remote protocol version was received. Skips cleanly when `ssh` is absent. **What this shows:** a real SSH client's protocol bytes make a round trip through the radio bridge. **What it does not show:** a completed session — with no local `sshd`, the peer banner the client sees is its own, echoed by the loopback.
- `hw_verify_stream_over_radio` (live, agent-run, `#[ignore]`d): a multi-chunk byte stream over the Pluto+ under the standing internal-loopback configuration, recovered byte-for-byte, with device state restored before assertions.
- **Not-yet-possible (named unlockers):**
  - A complete SSH session (key exchange, auth, shell) → requires an SSH **server**; `sshd` is not installed on this machine. Backlog.
  - Reliability under packet loss → the receive path validates CRC-32 but performs no ARQ retransmission, so a dropped frame would silently truncate a stream. Invisible in a lossless loopback; needs a lossy-channel model and a retransmit layer. Backlog.
  - Two separate radios, and any on-air operation → second device plus T-108 (explicit go-ahead).
