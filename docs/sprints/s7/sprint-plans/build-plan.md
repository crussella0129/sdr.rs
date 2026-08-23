Finalized - DO NOT EDIT

# Sprint 7 Build Plan

## Intents
- [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) — state: active (re-opened this sprint); acceptance criterion 4 covered. The criterion was **restated before this plan locked**: it previously claimed "an end-to-end simulated SSH handshake" while no runtime existed to pipe anything. It now states what is demonstrable — a runnable stream bridge carrying bidirectional byte streams over the radio link, with a real OpenSSH client exchanging protocol version strings. Criteria 1–3 remain satisfied and unchanged.
- [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) — state: active (context); criterion 1 gains a stream-oriented consumer, and `RadioLink` gains multi-frame recovery (T-035). No criterion of INT-0008 changes.

## Schema Tree
- Sprint Goal: make "SSH over radio" a runnable capability rather than a claim
  - Transport
    - T-035: extract every frame from a capture
    - T-036: `StreamBridge` — byte stream ⇄ datagrams
  - Runtime
    - T-037: runnable `sdr-cli tunnel` (stdio ProxyCommand + TCP)
  - Verification
    - T-038: real `ssh` client and live hardware

## Execution Sequence

### T-035: Extract every frame from a capture
- **Intent:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- **Touches:** crates/sdr-mesh/src/radio.rs, crates/sdr-mesh/tests/radio_it.rs
- **Depends on:** (none)
- **Acceptance criterion:** INT-0008 #1 — datagrams recovered without corruption. A capture holding several frames currently loses all but one, which a byte stream would hit immediately.
- **Success criterion (EARS):**
  - **WHEN** a capture contains several valid frames, **THEN** `recv_datagram` **SHALL** return each recovered datagram in turn rather than discarding all but the first.
  - **WHEN** a capture contains exactly one frame, **THEN** the recovered datagram **SHALL** be unchanged from current behaviour.
  - **WHEN** a frame occupies more samples than one capture, **THEN** it **SHALL** be reported as unrecovered rather than silently truncated.
- **Notes:** measured limits — a burst of 4 datagrams yields 1, and payloads beyond ~180 bytes fail because a frame cannot exceed `rx_chunk` samples (16 384 at `sps = 10`). Continue scanning the bit stream past each decoded frame, queueing into the existing `inbox`; raise the default `rx_chunk` so a full-MTU frame fits comfortably.

### T-036: `StreamBridge` — byte stream ⇄ datagrams
- **Intent:** [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md)
- **Touches:** crates/sdr-mesh/src/stream.rs, crates/sdr-mesh/src/lib.rs
- **Depends on:** T-035
- **Acceptance criterion:** INT-0006 #4 — split a stream into MTU-sized datagrams and reassemble it in order byte-for-byte.
- **Success criterion (EARS):**
  - **WHEN** a byte stream longer than the MTU is sent, **THEN** it **SHALL** be split into datagrams of at most the MTU and reassembled by the peer in the order sent.
  - **WHEN** arbitrary binary bytes are carried, **THEN** the reassembled stream **SHALL** equal the original byte-for-byte.
  - **WHEN** no datagram is available, **THEN** reading **SHALL** yield no bytes rather than blocking or erroring.
- **Notes:** pure and I/O-free, above `MeshInterface`, so it is testable without a radio and works over `MockSdr` and `PlutoSdr` alike. Framing stays `RadioLink`'s responsibility — deliberately **not** routed through `StreamTunnel`, which would frame every payload twice.

### T-037: Make `sdr-cli tunnel` runnable
- **Intent:** [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md)
- **Touches:** crates/sdr-cli/src/main.rs
- **Depends on:** T-036
- **Acceptance criterion:** INT-0006 #4 — a `sdr-cli tunnel` process actually pipes streams (stdin/stdout or TCP).
- **Success criterion (EARS):**
  - **WHEN** `tunnel --stdio` runs, **THEN** bytes arriving on stdin **SHALL** be transmitted over the link and received bytes **SHALL** be written to stdout, until stdin reaches EOF.
  - **WHEN** `tunnel --listen <port>` runs and a client connects, **THEN** the socket **SHALL** be pumped in both directions over the link.
  - **WHEN** the configured band prohibits encrypted payloads, **THEN** the command **SHALL** surface the compliance decision before carrying traffic.
- **Notes:** replaces the print-and-exit handler. A reader thread feeds a channel so the main loop can poll stdin and the radio without async plumbing around the blocking driver. The existing `check_compliance(..., is_encrypted=true)` gate is retained, not dropped.

### T-038: Real `ssh` client and live hardware
- **Intent:** [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md), [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- **Touches:** crates/sdr-cli/tests/ssh_tunnel_it.rs, crates/sdr-mesh/tests/hw_radio.rs
- **Depends on:** T-037
- **Acceptance criterion:** INT-0006 #4 — a real OpenSSH client exchanges protocol version strings across the link; INT-0008 #1 on real hardware for a stream.
- **Success criterion (EARS):**
  - **WHEN** a real `ssh` client is launched with the bridge as its `ProxyCommand`, **THEN** it **SHALL** receive a protocol version string across the link.
  - **WHEN** the `ssh` binary is unavailable, **THEN** the test **SHALL** skip cleanly rather than fail.
  - **WHEN** a multi-chunk byte stream is carried over the Pluto+, **THEN** it **SHALL** be recovered byte-for-byte.
- **Notes:** with no local `sshd`, the peer banner the client sees is its own echoed by the loopback — a valid `SSH-2.0-…` string, so the client proceeds and the exchange proves bidirectional transit. A complete session is out of scope and stated as such. The live test uses the standing internal-loopback configuration and restores device state before assertions.
