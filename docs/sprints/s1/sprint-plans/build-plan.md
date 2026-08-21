Finalized - DO NOT EDIT

# Sprint 1 Build Plan

## Intents
- [INT-0005](../../../intents/INT-0005-regulatory-band-compliance.md) — state: planned; acceptance criteria covered: 1, 2, 3
- [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) — state: planned; acceptance criteria covered: 1, 2, 3, 4

## Schema Tree
- Sprint 1: Regulatory Band Compliance & Packet Radio SSH Tunneling
  - Regulatory Advisor Engine (INT-0005)
    - T-007: Jurisdictional Regulatory Compliance database and transmission advisor
  - Packet Radio Link Layer & Modulator (INT-0006)
    - T-008: Packet framing, CRC-32, sequence numbering, and ARQ retransmission
    - T-009: Continuous-phase GFSK/FSK packet modulator and SdrDriver TX streaming pipeline
  - Tunnel Bridge & CLI Verification (INT-0005, INT-0006)
    - T-010: Stream tunnel proxy bridge for SSH and sdr-cli bands/tunnel commands

## Execution Sequence

### T-007: Jurisdictional Regulatory Compliance database and transmission advisor
- **Intent:** [INT-0005](../../../intents/INT-0005-regulatory-band-compliance.md)
- **Touches:** crates/sdr-core/src/compliance.rs, crates/sdr-core/src/lib.rs
- **Depends on:** (none)
- **Acceptance criterion:** Database catalogs ISM, SRD, and Amateur bands across US, EU, UK, AU, and Global; compliance checker validates frequency, power, bandwidth, and encryption legality.
- **Success criterion (EARS):**
  - **WHEN** querying recommended bands for jurisdiction US with encrypted data, **THEN** the advisor **SHALL** return 902–928 MHz ISM (Part 15) and exclude Amateur 2m/70cm bands.
  - **WHEN** evaluating an encrypted transmission in an Amateur band (e.g. 144.2 MHz or 433.0 MHz amateur), **THEN** the compliance checker **SHALL** return a non-compliance warning citing encryption prohibition.
  - **WHEN** querying EU regulations for 868 MHz, **THEN** the advisor **SHALL** return exact ERP limits (25 mW to 500 mW) and duty cycle limits (1% to 10%).
- **Notes:** Structured with rich metadata including EIRP, ERP, channel width, license type, and regulatory citations.

### T-008: Packet framing, CRC-32, sequence numbering, and ARQ retransmission
- **Intent:** [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md)
- **Touches:** crates/sdr-protocols/src/packet.rs, crates/sdr-protocols/src/lib.rs
- **Depends on:** (none)
- **Acceptance criterion:** Packet framer generates packets with preamble, sync word, sequence counter, payload length, and CRC-32; ARQ layer retransmits dropped frames across lossy channels.
- **Success criterion (EARS):**
  - **WHEN** `PacketFramer` encodes a byte payload, **THEN** decoding the framed bytes **SHALL** verify CRC-32 and extract the exact original payload.
  - **WHEN** `ArqTransceiver` transfers data across a channel with simulated 25% packet drops, **THEN** the ARQ engine **SHALL** deliver 100% of payload bytes in exact sequential order without duplicates.
- **Notes:** Implements Stop-and-Wait ARQ with configurable ACK timeouts and sequence numbering.

### T-009: Continuous-phase GFSK/FSK packet modulator and SdrDriver TX streaming pipeline
- **Intent:** [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md)
- **Touches:** crates/sdr-demod/src/modulator.rs, crates/sdr-demod/src/lib.rs, crates/sdr-hardware/src/driver.rs, crates/sdr-hardware/src/mock.rs, crates/sdr-hardware/src/pluto.rs, crates/sdr-hardware/src/lib.rs
- **Depends on:** T-008
- **Acceptance criterion:** GFSK modulator synthesizes continuous-phase baseband IQ bursts; SdrDriver supports TX sample streaming; MockSdr supports simulated full-duplex loopback.
- **Success criterion (EARS):**
  - **WHEN** binary packet bits are passed to `GfskModulator`, **THEN** the output IQ burst **SHALL** maintain continuous phase with Gaussian transition smoothing.
  - **WHEN** `MockSdr` is in loopback mode, **THEN** samples written via `write_samples()` **SHALL** be readable via `read_samples()`.
- **Notes:** Integrate with `PlutoSdr` TX buffer preparation and `SdrDriver` TX controls.

### T-010: Stream tunnel proxy bridge for SSH and sdr-cli bands/tunnel commands
- **Intent:** [INT-0005](../../../intents/INT-0005-regulatory-band-compliance.md), [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md)
- **Touches:** crates/sdr-protocols/src/tunnel.rs, crates/sdr-cli/src/main.rs, crates/sdr-cli/tests/e2e_pipeline_tests.rs
- **Depends on:** T-007, T-008, T-009
- **Acceptance criterion:** `sdr-cli bands` command queries legal frequency bands by jurisdiction; `StreamTunnel` transports bidirectional byte streams over packet radio with simulated SSH session verification.
- **Success criterion (EARS):**
  - **WHEN** `sdr-cli bands --jurisdiction US --encrypted` is executed, **THEN** it **SHALL** print legal ISM bands with EIRP limits and regulatory citations.
  - **WHEN** `StreamTunnel` passes simulated SSH client-server traffic, **THEN** all data bytes **SHALL** be exchanged bidirectionally across the packet radio channel with zero data corruption.
- **Notes:** Full end-to-end integration tests in `crates/sdr-cli/tests/e2e_pipeline_tests.rs`.
