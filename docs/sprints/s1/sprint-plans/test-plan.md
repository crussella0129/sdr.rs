Finalized - DO NOT EDIT

# Sprint 1 Test Plan

## Intent Coverage
- [INT-0005](../../../intents/INT-0005-regulatory-band-compliance.md) — Acceptance criteria: 1, 2, 3 -> covered by T-007, T-010
- [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) — Acceptance criteria: 1, 2, 3, 4 -> covered by T-008, T-009, T-010

## Unit Tests
- `crates/sdr-core`:
  - `test_regulatory_compliance_ism_bands`: Query US 902–928 MHz, EU 868 MHz, 2.4 GHz, 5.8 GHz ISM bands; verify EIRP limits, duty cycle, and encryption permission.
  - `test_regulatory_compliance_amateur_encryption_rejection`: Verify that checking compliance for encrypted transmission on Amateur 2m/70cm bands returns an explicit violation error.
- `crates/sdr-protocols`:
  - `test_packet_framing_crc32`: Encode payload with preamble, sync word, headers, and CRC-32; verify framing and decode roundtrip.
  - `test_arq_retransmission_lossy_channel`: Transmit 50 packet frames through simulated 25% loss channel; verify 100% in-order data recovery via ARQ.
  - `test_stream_tunnel_bidirectional`: Run bidirectional stream tunnel exchanging client and server byte sequences.
- `crates/sdr-demod`:
  - `test_gfsk_modulator_continuous_phase`: Synthesize GFSK burst from binary bits; verify phase continuity and Gaussian envelope shaping.
- `crates/sdr-hardware`:
  - `test_sdr_driver_tx_streaming`: Test `MockSdr` TX streaming buffer writes and loopback RX retrieval.

## Integration & E2E Tests
- `crates/sdr-cli/tests/e2e_pipeline_tests.rs`:
  - `test_e2e_ssh_over_radio_tunnel`: End-to-end simulation of an SSH protocol session across packet radio:
    1. SSH client proxy initiates connection -> packets framed and modulated with GFSK -> transmitted via SDR TX.
    2. SDR RX receives IQ -> demodulates GFSK -> verifies CRC-32 -> ARQ acknowledges -> delivers payload to SSH daemon.
    3. Bidirectional data transfer verified with zero packet loss or byte corruption.
  - `test_e2e_regulatory_advisor_cli`: Test CLI `bands` command with US and EU filters.

## Canonical Runner
- `cargo test --workspace`
