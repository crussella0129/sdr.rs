# Sprint 6 Unit Tests

- **Tested head:** `2821e3fc480b7eaf6fde937dfc12252ff8ea8680`
- **Runner:** `cargo test --workspace` — 100 passed, 0 failed, 4 ignored (hardware).

## T-031 — `GardnerClockRecovery` (INT-0001)
The component had existed since Sprint 0 with **no correctness coverage**: its
only test asserted the output was non-empty and roughly the right length, which
a broken timing-error detector would pass.

- `test_gardner_clock_recovery` (strengthened): recovered symbol **values** now
  compared against a **non-periodic** pseudo-random ±1 sequence. An alternating
  pattern was deliberately rejected — with period 2 a wrong lag still aligns, so
  mismatches would go unnoticed. PASS.
- `test_gardner_tracks_clock_drift`: the same sequence resampled to ±0.2%
  receiver clock offset still recovers exactly — the condition a fixed sample
  phase cannot handle at all. PASS.

## T-032 — `FskTimingDemod` (INT-0006)
- `test_timing_demod_aligned`: an FSK burst carrying the preamble and sync word demodulates bit-exact. PASS.
- `test_timing_demod_mid_symbol`: the same burst started 7 of 10 samples into the first symbol still demodulates bit-exact. The fixed-count `FskDemod` corrupts roughly half the bits in this case (Sprint 5 measured 37/71). PASS.
- `test_timing_demod_clock_drift`: bit-exact at ±0.1% and ±0.5% clock offset combined with a start offset, over an ~80-bit burst. PASS.

## Regression contract (Sprint 5, unchanged)
- `test_fsk_roundtrip_bit_exact`, `test_fsk_roundtrip_offset_degrades`: pass unmodified. The fixed-count demodulator's behaviour is deliberately unchanged; `FskTimingDemod` was added **alongside** it, since `PskDemod` still depends on the original.
