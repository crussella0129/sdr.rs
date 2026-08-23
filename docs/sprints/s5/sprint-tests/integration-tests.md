# Sprint 5 Integration Tests

- **Tested head:** `19bd39e5e1ed338ee7282695e8c38e4ddc8905a8`
- **Result:** pass (3 passed, 0 failed). No radio required.

## `RadioLink` over `MockSdr` loopback (INT-0008)
`crates/sdr-mesh/tests/radio_it.rs` — the full mesh radio path with a mock
driver standing in for hardware.

- `test_radiolink_datagram_roundtrip_over_mock`: a datagram containing bytes that require KISS escaping (`0xC0`, `0xDB`) plus a plausible IPv4 header start is sent through `RadioLink` and recovered **byte-for-byte** from the looped-back IQ — exercising KISS → ARQ frame → FSK modulate → sample → demodulate → framesync → CRC-32 → KISS decode. PASS.
- `test_radiolink_recovers_from_sample_offset`: the modulated IQ is drained from the mock and re-injected shifted by half a symbol plus one, so the receiver starts **mid-symbol**. The datagram is still recovered, proving the sample-phase search genuinely works. This test exists because the mock loopback is sample-exact and would otherwise always succeed at phase 0, leaving the search unexercised (plan critique C-002). PASS.
- `test_radiolink_noise_returns_none`: silence yields `Ok(None)` — no error, no panic, and no datagram invented where none was transmitted. PASS.
