# INT-0006 — Bidirectional Packet Radio Transceiver and SSH Tunneling

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0006
- **State:** realized
- **Work evidence:** [T-008 build plan](../sprints/s1/sprint-plans/build-plan.md#t-008-packet-framing-crc-32-sequence-numbering-and-arq-retransmission), [T-009 build plan](../sprints/s1/sprint-plans/build-plan.md#t-009-continuous-phase-gfskfsk-packet-modulator-and-sdrdriver-tx-streaming-pipeline), [T-010 build plan](../sprints/s1/sprint-plans/build-plan.md#t-010-stream-tunnel-proxy-bridge-for-ssh-and-sdr-cli-bandstunnel-commands)
- **Completion evidence:** [T-008 completion](../work/completed-tasks.md#t-008-sprint-1), [T-009 completion](../work/completed-tasks.md#t-009-sprint-1), [T-010 completion](../work/completed-tasks.md#t-010-sprint-1)
- **Code evidence:** [packet.rs](../../crates/sdr-protocols/src/packet.rs), [modulator.rs](../../crates/sdr-demod/src/modulator.rs), [tunnel.rs](../../crates/sdr-protocols/src/tunnel.rs)
- **Test evidence:** [Sprint 1 test report](../sprints/s1/sprint-tests/test-report.md)
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
1. Packet radio engine frames binary data with preamble, sync word, sequence counter, payload, and CRC-32, and extracts payload without corruption.
2. ARQ layer automatically acknowledges received frames (ACK) and retransmits dropped packets under simulated RF packet loss up to 30%.
3. Modulator generates compliant continuous-phase FSK / GFSK / LoRa baseband IQ bursts with clean spectral rolloff.
4. Terminal / stream bridge pipes bidirectional byte streams (stdin/stdout or TCP proxy socket) through packet radio frames, successfully completing an end-to-end simulated SSH handshake and session transfer.

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
