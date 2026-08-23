Finalized - DO NOT EDIT

# Sprint 6 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | #4 timing loops hold without slippage | T-031 / WHEN alternating PAM processed THEN recovered symbols match | test_gardner_clock_recovery |
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | #4 under a clock offset | T-031 / WHEN input resampled THEN symbols still match | test_gardner_tracks_clock_drift |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #3 bursts demodulable (aligned) | T-032 / WHEN aligned THEN bits equal transmitted | test_timing_demod_aligned |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #3 bursts demodulable (mid-symbol) | T-032 / WHEN starting mid-symbol THEN bits still equal | test_timing_demod_mid_symbol |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #3 under a clock offset | T-032 / WHEN clock differs up to ±0.5% THEN bits still equal | test_timing_demod_clock_drift |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 single-pass recovery | T-033 / WHEN valid frame THEN recovered in one pass, no phase loop | test_radiolink_datagram_roundtrip_over_mock, test_radiolink_recovers_from_sample_offset |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 without a shared clock | T-033 / WHEN clock offset present THEN datagram still byte-for-byte | test_radiolink_recovers_under_clock_drift |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 negative path | T-033 / WHEN no valid frame THEN Ok(None), no panic | test_radiolink_noise_returns_none |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 on real hardware | T-034 / WHEN live test runs with timing recovery THEN identical datagram recovered | hw_verify_mesh_datagram_over_radio (live) |

## Unit Tests

### T-031 unit tests
- **Intent:** [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md)
- `test_gardner_clock_recovery` (strengthened): an alternating ±1 PAM sequence at 4 samples/symbol is recovered with the correct **values**, compared against the transmitted sequence allowing a bounded start-up lag. Replaces the previous length-only assertion.
- `test_gardner_tracks_clock_drift`: the same sequence resampled to simulate a receiver clock offset is still recovered correctly.

### T-032 unit tests
- **Intent:** [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md)
- `test_timing_demod_aligned`: an FSK burst carrying the packet preamble and sync word demodulates bit-exact.
- `test_timing_demod_mid_symbol`: the same burst started 7 samples in (of 10 per symbol) still demodulates bit-exact — the case the fixed-count `FskDemod` fails with ~half the bits wrong.
- `test_timing_demod_clock_drift`: recovered bit-exact at ±0.1% and ±0.5% clock offset.

## Integration Tests
### `RadioLink` over `MockSdr` loopback (INT-0008)
- **Intents:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- **Regression contract — these three carry over from Sprint 5 unchanged** and must pass without modification; a behaviour change in them is a failure, not a rebaseline:
  - `test_radiolink_datagram_roundtrip_over_mock`
  - `test_radiolink_recovers_from_sample_offset`
  - `test_radiolink_noise_returns_none`
- `test_radiolink_recovers_under_clock_drift` (new): the transmitted IQ is resampled to simulate a receiver whose clock differs from the transmitter's, then re-injected; the datagram is still recovered byte-for-byte. This is the case the removed sample-phase search could not handle at all, so it is the test that justifies the change.

## End-to-End Tests
- **Status:** possible — live hardware E2E performed by the agent; CI stays hardware-free.
- `hw_verify_mesh_datagram_over_radio` (live, agent-run): the Sprint 5 hardware test re-run with the phase search removed, against the physical Pluto+ at `192.168.2.1:30431` under the standing internal-loopback configuration (maximum attenuation, DDS silenced). Device state restored before assertions.
- **Not-yet-possible (named unlockers):**
  - Behaviour under channel noise / BER characterization → needs a real channel; the internal loopback is noiseless, so this sprint cannot settle it. Carried forward.
  - Two separate radios exchanging datagrams → needs a second radio, and over-the-air operation (T-108) which requires explicit go-ahead.
  - Multi-hop routing → Phase B (T-107), `tun`/babeld.
