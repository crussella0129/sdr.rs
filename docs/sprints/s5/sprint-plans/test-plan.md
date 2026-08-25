Finalized - DO NOT EDIT

# Sprint 5 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #3 modulator produces demodulable bursts | T-027 / WHEN modulated + demodulated aligned THEN bits equal exactly | test_fsk_roundtrip_bit_exact |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #3 (alignment sensitivity) | T-027 / WHEN demodulated from a large offset THEN errors occur | test_fsk_roundtrip_offset_degrades |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 frame recovery from bits | T-028 / WHEN sync word at any bit offset THEN bytes from that word | test_framesync_finds_sync_at_bit_offsets |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 negative path | T-028 / WHEN no sync word THEN None | test_framesync_absent_returns_none |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 bit packing order | T-028 / WHEN packing THEN MSB-first | test_framesync_packs_msb_first |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 datagram emitted as IQ | T-029 / WHEN send_datagram THEN modulated IQ via write_samples | test_radiolink_datagram_roundtrip_over_mock |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 datagram recovered | T-029 / WHEN valid frame at any sample phase THEN datagram byte-for-byte | test_radiolink_datagram_roundtrip_over_mock, test_radiolink_recovers_from_sample_offset |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 negative path | T-029 / WHEN no valid frame THEN Ok(None), no panic | test_radiolink_noise_returns_none |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 on real hardware | T-030 / WHEN sent on the physical Pluto+ under loopback THEN identical datagram recovered, under internal loopback | hw_verify_mesh_datagram_over_radio (live) |

## Unit Tests

### T-027 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md)
- `test_fsk_roundtrip_bit_exact`: modulate `AA AA D3 91 01 02 FF 00 5A` at 1 MSPS / 100 kHz deviation / 10 sps, demodulate aligned → **zero** bit errors across all 72 bits.
- `test_fsk_roundtrip_offset_degrades`: demodulating the same IQ from a large sample offset produces a non-zero error count — the documented motivation for the receiver's alignment search.

### T-028 unit tests
- **Intent:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- `test_framesync_finds_sync_at_bit_offsets`: for every bit offset 0..8, a stream with the sync word embedded yields bytes starting at `D3 91 D3 91`.
- `test_framesync_absent_returns_none`: a stream with no sync word yields `None`.
- `test_framesync_packs_msb_first`: recovered bytes match the MSB-first packing the modulator uses.

### T-029 unit tests
- **Intent:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- (Behaviour is exercised end-to-end by the integration tests below; `RadioLink` holds no logic worth isolating from the driver it drives.)

## Integration Tests
### `RadioLink` over `MockSdr` loopback (INT-0008)
- **Intents:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- `test_radiolink_datagram_roundtrip_over_mock`: with `MockSdr::enable_loopback()`, a datagram sent through `RadioLink` is recovered byte-for-byte from the looped-back IQ — the full KISS → ARQ → modulate → sample → demodulate → framesync → deframe path, no radio.
- `test_radiolink_recovers_from_sample_offset`: the same exchange with a deliberate sample offset injected into the received IQ. The mock loopback is sample-exact and would otherwise always succeed at phase 0, so this is what genuinely exercises the phase search.
- `test_radiolink_noise_returns_none`: random/silent IQ yields `Ok(None)` — no panic, no spurious datagram.

## End-to-End Tests
- **Status:** possible — live hardware E2E performed by the agent under internal loopback; CI stays hardware-free (`#[ignore]`d).
- `hw_verify_mesh_datagram_over_radio` (live, agent-run, **loopback**): against the physical Pluto+ at `192.168.2.1:30431`. Engages `enter_loopback_test_mode()` (RF section bypassed, −89.75 dB, DDS silenced) and `set_tx_cyclic(true)`; sends a datagram through `RadioLink`, reads ≥ 2× the frame length, and recovers the identical datagram. Device state restored **before** assertions so a failure cannot strand the radio.
- **Not-yet-possible (named unlockers):**
  - Over-the-air datagram transit → backlog **T-108**, requires the user's explicit go-ahead on band/power/antenna.
  - Two-radio link and multi-hop routing → mesh **Phase B/C** (T-107), needing `tun`/babeld and a second radio.
  - Operation over a channel with clock drift → symbol-timing recovery (Gardner/M&M) carry-forward; the digital loopback has no clock offset, so the phase search suffices here.
