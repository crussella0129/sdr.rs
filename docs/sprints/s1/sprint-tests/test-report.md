# Sprint 1 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|---------------|--------|------------------------|
| [INT-0005](../../../intents/INT-0005-regulatory-band-compliance.md) | Jurisdictional database catalogs ISM/SRD/Amateur bands across US, EU, UK, AU, Global; compliance API validates frequency, power, and encryption legality | T-007 / `test_regulatory_compliance_ism_bands`, `test_regulatory_compliance_amateur_encryption_rejection`; T-010 / `test_e2e_ssh_over_radio_tunnel_and_compliance` | pass | Test evidence links this report; eligible for realized after completion evidence |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | Packet framing with CRC-32, ARQ retransmissions, continuous-phase GFSK modulator, TX driver loopback, and bidirectional SSH stream tunnel | T-008 / `test_packet_framing_crc32`, `test_arq_retransmission_lossy_channel`; T-009 / `test_gfsk_modulator_continuous_phase`, `test_mock_sdr_tx_loopback`; T-010 / `test_stream_tunnel_bidirectional`, `test_e2e_ssh_over_radio_tunnel_and_compliance` | pass | Test evidence links this report; eligible for realized after completion evidence |

## Summary
- Unit tests: 33 passed / 0 failed / 33 total
- Integration tests: 5 passed / 0 failed / 5 total
- E2E tests: 5 passed / 0 failed / 5 total
- CI status: not-configured

## CI Confirmation
- **Head SHA:** 3764fe6
- **CI run:** CI not configured — local confirmations only
- **Conclusion:** success
- **Confirmations:** Local canonical test runner (`cargo test --workspace`) passed 38/38 tests.

## Failures
None.

## Technical Debt Identified
None.

## Coverage Observations
Comprehensive test coverage for jurisdictional band advisor, encryption legality verification, packet radio framing, ARQ state machines, GFSK continuous-phase modulation, hardware TX streaming, and bidirectional SSH stream proxy tunneling.
