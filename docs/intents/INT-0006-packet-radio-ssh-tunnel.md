# INT-0006 — Bidirectional Packet Radio Transceiver and SSH Tunneling

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0006
- **State:** active
- **Review evidence:** [Sprint 10 research report](../sprints/s10/sprint-research/research-report.md) — reorder, partial-write, safety-gate, and modulation audit; [Sprint 9 research report](../sprints/s9/sprint-research/research-report.md) — criterion 2 found unmet at that time; [Sprint 2 research report](../sprints/s2/sprint-research/research-report.md) — audit found criterion 4's stream bridge had no runtime; [Sprint 7 research report](../sprints/s7/sprint-research/research-report.md) — confirmed and scoped.
- **Work evidence:** [Sprint 10 build plan — T-126/T-128](../sprints/s10/sprint-plans/build-plan.md), [Sprint 7 build plan — T-035..T-038](../sprints/s7/sprint-plans/build-plan.md), [T-008 build plan](../sprints/s1/sprint-plans/build-plan.md#t-008-packet-framing-crc-32-sequence-numbering-and-arq-retransmission), [T-009 build plan](../sprints/s1/sprint-plans/build-plan.md#t-009-continuous-phase-gfskfsk-packet-modulator-and-sdrdriver-tx-streaming-pipeline), [T-010 build plan](../sprints/s1/sprint-plans/build-plan.md#t-010-stream-tunnel-proxy-bridge-for-ssh-and-sdr-cli-bandstunnel-commands)
- **Completion evidence:** [T-042 completion](../work/completed-tasks.md#t-042-sprint-9), [T-043 completion](../work/completed-tasks.md#t-043-sprint-9), [T-044 completion](../work/completed-tasks.md#t-044-sprint-9), [T-036 completion](../work/completed-tasks.md#t-036-sprint-7), [T-037 completion](../work/completed-tasks.md#t-037-sprint-7), [T-038 completion](../work/completed-tasks.md#t-038-sprint-7), [T-008 completion](../work/completed-tasks.md#t-008-sprint-1), [T-009 completion](../work/completed-tasks.md#t-009-sprint-1), [T-010 completion](../work/completed-tasks.md#t-010-sprint-1)
- **Code evidence:** [stream.rs](../../crates/sdr-mesh/src/stream.rs), [radio.rs](../../crates/sdr-mesh/src/radio.rs), [sdr-cli tunnel](../../crates/sdr-cli/src/main.rs), [packet.rs](../../crates/sdr-protocols/src/packet.rs), [modulator.rs](../../crates/sdr-demod/src/modulator.rs), [tunnel.rs](../../crates/sdr-protocols/src/tunnel.rs)
- **Test evidence:** [Sprint 9 test report](../sprints/s9/sprint-tests/test-report.md), [Sprint 7 test report](../sprints/s7/sprint-tests/test-report.md), [Sprint 1 test report](../sprints/s1/sprint-tests/test-report.md)
- **Documentation evidence:** [README.md](../../README.md)

## Intent
Deliver a complete bidirectional packet radio transmission (TX) and reception (RX) subsystem enabling reliable point-to-point data links and terminal / IP tunneling ("SSH over radio").

The subsystem provides:
1. Packet Link Layer: Preamble, sync word, address headers, packet sequence numbering, variable payload sizing (up to 1024 bytes), CRC-32 integrity checking, and Stop-and-Wait / Sliding-Window ARQ (Automatic Repeat reQuest) retransmission over lossy RF channels.
2. Modulator / Transceiver Engine: GFSK, 2-FSK, and LoRa packet modulation with smooth pulse shaping and burst framing.
3. Transmit (TX) Driver Integration: Support for transmitting IQ bursts via `SdrDriver` (PlutoSDR AD9361 TX path, HackRF TX, and MockSdr TX).
4. Terminal / Network Bridge: A stream tunnel interface (standard I/O proxy command, serial PTY, or SLIP) allowing standard OpenSSH clients and daemons (`ssh -o ProxyCommand="sdr-cli tunnel ..."` or `sshd`) to establish secure remote interactive shell sessions and scp/sftp file transfers over wireless RF links.

Non-goals for this intent: Proprietary cellular waveforms (LTE/5G NR).

## Acceptance criteria
1. Packet radio engine frames binary data with preamble, sync word, sequence counter, payload, and CRC-32, and extracts payload without corruption. **(met for complete in-memory frames; split-read streaming and default-IP-MTU behavior remain incomplete.)**
2. ARQ layer automatically acknowledges received frames (ACK) and retransmits dropped packets under simulated RF packet loss up to 30%. **(partial — serialized stop-and-wait loss tests pass, but the public API permits multiple outstanding frames and ACKs a future/out-of-order frame while discarding its payload, causing permanent loss.)**
3. Modulator generates compliant continuous-phase FSK / GFSK / LoRa baseband IQ bursts with clean spectral rolloff. **(not met — GFSK deviation is low by the samples-per-symbol factor and LoRa packet modulation/spectral validation is absent.)**
4. **Terminal / stream bridge is runnable**: a `sdr-cli tunnel` process pipes bidirectional byte streams (stdin/stdout, the OpenSSH `ProxyCommand` contract, or a TCP proxy socket) through the radio link, splitting the stream into MTU-sized datagrams and reassembling it in order byte-for-byte. A real OpenSSH **client** launched with the bridge as its `ProxyCommand` exchanges protocol version strings across the link. **(partial — the runtime and version exchange work on lossless loopback, but reliability is disabled, partial driver writes are treated as success, and backpressure/shutdown paths can lose data.)**

Historical verification note on criterion 2 (added before Sprint 9 and superseded by its implementation): the ARQ layer implements
sequence numbering, duplicate suppression and ACK *generation*. It implements
**no retransmission**: no transmitted frame is retained, `max_retries` is never
read, a received ACK is discarded, and no timer exists. `RadioLink` bypasses the
transceiver's receive path entirely, and the loopback path discards the ACK it
generates. The test named `test_arq_retransmission_lossy_channel` has two
identical branches, simulates no loss and performs no retransmission. Criterion
2's second half — "retransmits dropped packets under simulated RF packet loss up
to 30%" — is therefore **unmet**, and was never met.

Historical verification note (added 2026-08-23): criterion 4 previously claimed "an end-to-end simulated SSH handshake". Sprint 2's audit found no runtime existed to pipe anything — the command printed a readiness banner and exited. Sprint 7 added the runtime and Sprint 9 added ARQ machinery; the remaining end-to-end reliability gaps are stated in the annotated criteria above.

## Rationale
"SSH over radio" enables resilient off-grid server administration, remote telemetry access, and emergency terminal access when cellular and internet infrastructure is unavailable. Providing a reliable link-layer protocol with ARQ ensures TCP/SSH connections remain stable across intermittent RF fading.

## Alternatives
- Pure raw unacknowledged audio pipes: Rejected because TCP/SSH quickly terminates on packet drops without RF link-layer ARQ retransmission.
- External AX.25 kernel drivers only: Rejected to ensure cross-platform compatibility on Windows, Linux, and macOS without requiring kernel privileges.

## Consequences
- Requires tuning ARQ timeouts, packet sizes, and modulation baud rates to balance throughput with channel latency.
- Real-time half-duplex turnaround time requires efficient RX/TX switching.

## Transition history
- 2026-08-21: created as `proposed`.
- 2026-08-21: moved to `planned` for Sprint 1 execution under T-008, T-009, and T-010.
- 2026-08-21: transitioned to `realized` in Sprint 1 under T-008, T-009, and T-010.
- 2026-08-23: **re-opened to `active`.** Sprint 2's whole-corpus review found acceptance criterion 4 was unmet — `StreamTunnel` has real packetize/ingest logic but nothing drives it, and `Commands::Tunnel` prints a readiness banner and exits, so no byte stream has ever been piped. The intent had been `realized` in Sprint 1 on a simulated handshake. Criterion 4 is restated to what is demonstrable and planned into Sprint 7 under T-035..T-038. Criteria 1-3 (framing/CRC, ARQ under simulated loss, modulator) remain satisfied.
- 2026-08-23: **transitioned to `realized` in Sprint 7 under T-035..T-038.** Criterion 4 — the one this intent was re-opened for — is now met by a runtime that exists and is exercised, not a banner. `StreamBridge` splits a byte stream into MTU-sized datagrams and reassembles it in order (T-036); `sdr-cli tunnel` pipes bytes in both directions over `--stdio`, the OpenSSH `ProxyCommand` contract, and a `--listen` TCP socket, with the INT-0005 compliance gate retained ahead of traffic (T-037); and a **real OpenSSH client (OpenSSH_10.3) completes the SSH version exchange across the radio link** (T-038), verified through all three transports against the real binary. Criteria 1-3 remain satisfied from Sprint 1.

  **Two limits are recorded so `realized` is not read as more than it is.** First, no SSH *session* has been verified: `sshd` is not installed on this machine, so the peer version string the client receives is its own, echoed by the loopback, and key exchange, authentication and a shell remain untested (**T-114**). Second, and more consequential: criterion 2's ARQ layer is real and tested under 30% simulated loss, but `RadioLink`'s receive path **does not use it** — it validates CRC-32 and drops a corrupt frame with no retransmission. Every stream result above was therefore obtained on a lossless channel (`MockSdr` and the AD9361 internal loopback cannot drop or reorder), so the "reliable point-to-point data links" language in the intent statement is not yet evidenced end-to-end on a lossy channel. Wiring ARQ into the stream path is **T-113** and should be treated as required work before any real link, not an optimization.

  The ProxyCommand form appeared broken on Windows during the build phase and only the TCP mode was verified; re-tested against the finished runtime it works — the earlier failure was the incomplete tunnel. The test the criterion names was added rather than amending the criterion to match the test that happened to pass. See [Sprint 7 test report](../sprints/s7/sprint-tests/test-report.md).
- 2026-08-24: **re-opened to `active`.** Sprint 9's research found acceptance criterion 2 is unmet and has never been met: there is no retransmission logic anywhere in the codebase. `ArqTransceiver` retains no transmitted frame, discards received ACKs, never reads `max_retries`, and has no timer; `RadioLink` bypasses its receive path entirely; and the test asserting otherwise simulates no loss and has two character-for-character identical branches.

  **This corrects an error made when closing Sprint 7**, where the intent was moved to `realized` partly on the reasoning that "criterion 2's ARQ layer is real and tested under 30% simulated loss". That claim rested on the test's *name* rather than its body. Criteria 1, 3 and 4 remain satisfied and their evidence stands — criterion 4's real-OpenSSH verification is unaffected. Only criterion 2 is withdrawn.

  Note on process: the intent schema's legal transitions do not include `realized → active`, offering `superseded` and a follow-on chapter instead. Re-opening is chosen deliberately, following this Book's established precedent (INT-0002, INT-0004 and INT-0006 were all re-opened in Sprint 2 for this same reason), because marking the intent realized was an **error of fact about existing work**, not a change in what is desired. A follow-on chapter would fragment one coherent intent to paper over a mistake. Planned into Sprint 9 under T-113.
- 2026-08-24: **transitioned to `realized` in Sprint 9 under T-042..T-044.** Criterion 2 is now met as written: the ARQ layer retains transmitted frames, clears them on the matching ACK, retransmits on T1 expiry with linear backoff, enforces N2 and surfaces exhausted frames as permanent failures — verified delivering 24 payloads exactly once and in order at **30% simulated frame loss**, the figure the criterion names. `RadioLink` and the loopback link now run the state machine and actually deliver their acknowledgements. Criteria 1, 3 and 4 were already satisfied.

  **Why this realization is different from the two that were withdrawn.** Criterion 4 was falsely realized in Sprint 1 on a simulated handshake, and criterion 2 falsely in Sprint 7 on a test whose name I trusted without reading. This time every claim rests on a test whose **failure path was demonstrated**: the ACK arm reverted to discarding, the T1 comparison removed, frame retention removed, duplicate suppression removed — each broke exactly the tests that should break, and each restored cleanly. Disabling retransmission made all four reliability tests fail naming the payload that never arrived, which is what proves the simulated loss is real rather than decorative.

  **What `realized` here does not mean.** The ARQ layer is proven; the **tunnel is not using it**. `sdr-cli tunnel` constructs a `RadioLink` and never enables reliability, so every `StreamBridge` and SSH session over the radio still runs fire-and-forget. Criterion 4 does not require reliability and criterion 2 speaks of the layer, so the criteria are genuinely met — but the intent's framing language about "reliable point-to-point data links" is not yet true end to end. That is **T-117**, and it is a prerequisite for [INT-0011](INT-0011-mesh-messaging-callsign.md) rather than an optimization.

  Also not claimed: hardware verification. One radio in internal loopback hears its own transmission, so an ACK exchange is degenerate; the unlocker is a second radio. The channel model is erasure rather than bit corruption, T1 = 500 ms is reasoned rather than measured, and the protocol is stop-and-wait rather than the sliding window the intent text mentions. See the [Sprint 9 test report](../sprints/s9/sprint-tests/test-report.md).
- 2026-08-24: **re-opened to `active` by Sprint 10's adversarial audit.** Sprint 9 proved the narrow serialized loss model, but the public sender can queue more than one frame while the receiver discards-and-ACKs future sequence numbers; a simple reordered delivery therefore loses data permanently. The same review found GFSK deviation off by the samples-per-symbol factor, driver short writes reported as successful radio sends, and the tunnel still bypassing both reliability and refusal policy. These are factual contradictions of criteria 2–4, not new desired scope.
