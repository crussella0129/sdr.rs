# Sprint 6 Integration Tests

- **Tested head:** `2821e3fc480b7eaf6fde937dfc12252ff8ea8680`
- **Result:** pass (4 passed, 0 failed). No radio required.

## `RadioLink` over `MockSdr` loopback (INT-0008)

### Regression contract — carried over from Sprint 5 **unchanged**
These three were the guard on replacing a working receive path, and they earned
their keep: the first switch to timing recovery broke two of them (see below).

- `test_radiolink_datagram_roundtrip_over_mock`: PASS.
- `test_radiolink_recovers_from_sample_offset`: PASS.
- `test_radiolink_noise_returns_none`: PASS.

### New
- `test_radiolink_recovers_under_clock_drift`: the transmitted IQ is resampled so the receiver's clock differs from the transmitter's, then re-injected; the datagram is still recovered byte-for-byte at ±0.1%. This is the case the removed sample-phase search could not handle **at all**, so it is the test that justifies the change. PASS.

## What the contract caught
Switching to timing recovery initially failed two of the three carried-over
tests. Diagnosis (rather than rebaselining) found two real issues:

1. **Frames need trailing flush symbols.** The timing loop consumes a symbol
   settling at the start of a burst, shifting its output stream so the frame's
   final CRC byte was truncated — every payload failed to decode while the old
   fixed-count path succeeded. A measurement across four payloads showed
   `fixed=true, timing=false, timing+pad=true` in every case. Fixed with a
   2-byte `TRAILER`; standard postamble practice.
2. **The ±0.5% drift figure did not hold at frame length.** See the test report.
