# Sprint 7 Research Report — SSH-over-radio stream bridge (T-103)

Make the project's headline capability real: pipe a bidirectional byte stream
through the radio link so an OpenSSH `ProxyCommand` can run over it. This also
settles a standing honesty debt — Sprint 2's audit found INT-0006's acceptance
criterion 4 claims an SSH handshake, but the tunnel has no runnable bridge.

## Intents Reviewed
- [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) — **selected**; primary, and **due a correction**. Criterion 4 states the stream bridge "pipes bidirectional byte streams … successfully completing an end-to-end simulated SSH handshake". Sprint 2's whole-corpus review found `StreamTunnel` has real packetize/ingest logic but **no runtime**: the CLI constructs it, prints "Ready for OpenSSH ProxyCommand", and exits. The intent is `realized` on an unmet criterion (§4.1).
- [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) — **selected**; the bridge runs over `RadioLink`, so its datagram limits (§4.2) bound what the bridge can carry. Criterion 1 gains a stream-oriented consumer.
- [INT-0005](../../../intents/INT-0005-regulatory-band-compliance.md) — **selected** (context): the existing `tunnel` command already gates encrypted payloads through the compliance database; the runtime must keep that gate rather than drop it.

## 1. Sprint Goal
Deliver a working stream bridge: byte stream ⇄ MTU-sized datagrams ⇄ `RadioLink`,
exposed as a runnable `sdr-cli tunnel` suitable for `ssh -o ProxyCommand=…`.
Verify byte-stream fidelity in CI over `MockSdr`, confirm a real OpenSSH client's
bytes traverse the link, and correct INT-0006 criterion 4 to match reality.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| crates/sdr-protocols/src/tunnel.rs | high | `StreamTunnel`: `packetize` (chunk to MTU → ARQ frames) and `ingest_frame` (CRC + ARQ → rx buffer). Real logic, **never driven by anything**. Predates `RadioLink` and duplicates its ARQ layer without modulation — superseded for the radio path (§4.3). |
| crates/sdr-cli/src/main.rs (`Tunnel`) | high | Prints station/frequency/compliance status, constructs a `StreamTunnel`, announces readiness, exits. No I/O loop. This is what "no runnable bridge" means concretely. |
| crates/sdr-mesh/src/radio.rs | high | `RadioLink` — the real radio path (modulation + timing recovery, Sprint 6). `recv_datagram` reads `rx_chunk` samples and extracts **one** frame; `rx_chunk` defaults to 16 384 samples. Both facts bound the bridge (§4.2). |
| crates/sdr-mesh/src/node.rs | high | `MeshInterface` (`send_datagram`/`recv_datagram`) — the seam the bridge should pump, so it works over `MockSdr` in CI and `PlutoSdr` live. |
| crates/sdr-mesh/src/kiss.rs | medium | Datagram framing already applied inside `RadioLink`; the bridge works above it and needs no framing of its own. |
| crates/sdr-core/src/compliance.rs | medium | `check_compliance(..., is_encrypted=true)` — the gate the current command performs and the runtime must preserve. |
| crates/sdr-mesh/tests/radio_it.rs | medium | The `MockSdr` loopback harness pattern the bridge tests should reuse. |

## 3. External Sources
- **In-repo measurement and the local toolchain (primary authority).** The
  questions are properties of this codebase and this machine, measured directly
  (§4). No external sources were required.

## 4. Measured findings

### 4.1 INT-0006 criterion 4 is unmet — the bridge does not exist
Confirmed by reading the handler: `Commands::Tunnel` prints status, evaluates
compliance, constructs a `StreamTunnel`, and returns. Nothing reads stdin,
nothing writes stdout, no socket is opened, and `StreamTunnel::packetize` is
never called outside its own unit tests. Sprint 1 realized the intent on a
*simulated* handshake; the criterion's plain reading is not satisfied. This
sprint should make it true **and** record the correction, in the same pattern as
INT-0002 (Sprint 2) and INT-0004 (Sprint 2).

### 4.2 `RadioLink` has two hard limits that bound any stream on top of it
Measured with `MockSdr` in loopback:

| probe | result |
|---|---|
| **A.** strict ping-pong (send one, receive one, ×5) | **5/5 recovered** |
| **B.** burst of 4 datagrams, then drain | **1 of 4 recovered** |
| **C.** single datagram, 64 B | OK |
| **C.** single datagram, 256 B / 512 B / 1 KiB / 2 KiB | **all fail (none recovered)** |
| **D.** two queued, `recv_datagram` twice | first OK, second `None` |

Two distinct causes:

1. **One frame per capture.** `recv_datagram` consumes `rx_chunk` samples and
   scans for a single sync word; any further frames in that capture are
   discarded with it (B and D). A byte stream produces back-to-back frames, so
   this must be fixed or the stream loses data whenever the sender outruns the
   receiver.
2. **Datagram size is capped by `rx_chunk`.** A frame occupies
   `frame_bytes × 8 × samples_per_symbol` samples. At the default
   `rx_chunk = 16 384` and `sps = 10`, a frame larger than ~204 bytes cannot fit
   in one capture, so ~180 bytes of payload is the ceiling — matching 64 B
   passing and 256 B failing.

Both are addressable: extract **all** frames from a capture, and chunk the
stream to an MTU that fits comfortably. Chunking is wanted regardless, since a
stream must be split for transmission anyway.

### 4.3 `StreamTunnel` is superseded for the radio path
It performs chunking + ARQ framing, which `RadioLink` already does internally
(via `ArqTransceiver` and `PacketFramer`). Routing the bridge through
`StreamTunnel` **and** `RadioLink` would frame every payload twice. The bridge
should sit above `MeshInterface` and let `RadioLink` own framing; `StreamTunnel`
stays as-is for non-radio use rather than being deleted mid-sprint.

### 4.4 A real OpenSSH client is available; a server is not
`ssh.exe` and `ssh-keygen.exe` ship with Windows (`C:\WINDOWS\System32\OpenSSH`);
**`sshd` is absent and the service is not installed**. Consequently:
- The real `ssh` client **can** be launched with `-o ProxyCommand=…`, and its
  protocol banner (`SSH-2.0-…`) will traverse the bridge — genuine evidence that
  a real SSH client's bytes cross the radio link.
- A **complete** SSH session (key exchange, auth, shell) cannot be verified
  locally without a server. That limit must be stated, not implied away.

### 4.5 Reliability under loss is not provided by the current path
`RadioLink::recv_datagram` validates CRC-32 but decodes frames with
`PacketFramer::decode` directly — it does not run `ArqTransceiver::process_rx_frame`,
so there is no sequence checking, duplicate suppression, or ACK/retransmit on the
receive side. In a lossless loopback this is invisible; over a real channel a
dropped frame would silently truncate the stream. SSH sits on TCP precisely
because it needs reliable, ordered bytes, so this is a genuine gap to name.

## 5. Risks, Unknowns, Dependencies
- **Risk — over-claiming "SSH over radio".** The honest deliverable is a working
  ProxyCommand-compatible bridge verified over a lossless loopback with a real
  client's bytes traversing it. *Mitigation:* state the two limits explicitly
  (no server locally, no ARQ retransmit) and correct INT-0006 criterion 4 to
  what is actually demonstrated rather than restating the old claim.
- **Risk — the multi-frame fix regressing Sprint 5/6 behaviour.** `RadioLink` is
  now covered by seven tests. *Mitigation:* carry them over unchanged as a
  regression contract, as in Sprint 6 — that contract caught two real defects.
- **Unknown — throughput.** Each datagram currently costs a full capture. A
  stream of MTU-sized chunks may be slow; this sprint should measure rather than
  assert. Not a correctness question.
- **Constraint — the compliance gate must survive.** The current command warns
  when encrypted payloads are illegal on the chosen band; the runtime must keep
  that check before carrying SSH traffic.
- **Constraint — live testing stays on internal loopback**, per the standing
  choice; anything on-air remains T-108 pending explicit go-ahead.
- **Dependency — none new.** `sdr-cli` already depends on `sdr-mesh`'s crates;
  stdin/stdout and TCP come from `std` and the existing tokio runtime.

## 6. Recommended Approach
1. **Extract every frame from a capture** in `RadioLink::recv_datagram`
   (continue scanning the bit stream past each decoded frame, queueing results
   into the existing inbox). Fixes probes B and D.
2. **Add a `StreamBridge`** above `MeshInterface`: chunk outbound bytes to a
   configurable MTU sized to fit one capture, and reassemble inbound datagrams
   into a byte stream. Pure and testable without any I/O.
3. **Make `sdr-cli tunnel` runnable**: a stdin/stdout mode (the ProxyCommand
   contract) and a TCP-listen mode, both pumping through the bridge, with the
   existing compliance gate retained.
4. **Verify** — CI: multi-chunk bidirectional byte stream over `MockSdr`,
   including binary data and sizes spanning several MTUs; plus a test that
   launches the real `ssh` client with the bridge as its ProxyCommand and
   asserts its banner traverses. Live: a byte stream over the Pluto+ internal
   loopback.
5. **Correct INT-0006 criterion 4** to describe what is demonstrated, recording
   the transition rather than silently editing.
6. **Out of scope:** ARQ retransmit/ordering under loss (name as backlog), a
   full SSH session (no local server), `tun`-based routing (Phase B), throughput
   optimization, over-the-air.

## 7. Artifacts
- `docs/sprints/s7/sprint-research/research-report.md` — this report.
- Measurements in §4.2 produced by a temporary probe
  (`crates/sdr-mesh/tests/scratch_stream.rs`), run and then removed; the table
  above is the durable record.
