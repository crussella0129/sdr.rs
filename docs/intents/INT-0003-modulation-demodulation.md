# INT-0003 — Universal Modulation and Demodulation Engine

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0003
- **State:** active
- **Review evidence:** [Sprint 10 research report](../sprints/s10/sprint-research/research-report.md) — signal-path audit found missing stereo/RDS behavior, incorrect SSB separation, and incorrect GFSK deviation scaling.
- **Work evidence:** [T-004 build plan](../sprints/s0/sprint-plans/build-plan.md#t-004-sdr-demod-analog-and-digital-demodulation-pipelines)
- **Completion evidence:** [T-004 completion](../work/completed-tasks.md#t-004-sprint-0)
- **Code evidence:** [sdr-demod](../../crates/sdr-demod/src/lib.rs)
- **Test evidence:** [Sprint 0 test report](../sprints/s0/sprint-tests/test-report.md)
- **Documentation evidence:** [README.md](../../README.md)

## Intent
Implement a comprehensive collection of analog and digital modulation and demodulation pipelines in Rust. Analog support covers Wideband FM (with stereo multiplex MPX decoding and RDS subcarrier filtering), Narrowband FM (with CTCSS sub-audible tone squelch), AM (envelope detection and synchronous carrier tracking), SSB (Upper/Lower Sideband with Weaver / Phasing method), and CW beat tone detection. Digital modulation includes OOK/ASK, FSK/GFSK, and PSK/QPSK with carrier synchronization (Costas Loop) and symbol timing clock recovery (Gardner / Mueller & Müller).

Non-goals for this intent: Higher-level packet protocol parsing (such as ADS-B or LoRaWAN framing, covered in INT-0004).

## Acceptance criteria
1. WFM demodulator recovers broadcast audio with SNR > 40 dB on clean synthetic FM carriers and separates L+R / L-R stereo channels. **(not met — the current WFM path is a mono discriminator/de-emphasis filter; stereo MPX/pilot/RDS is absent.)**
2. NFM demodulator provides configurable hysteresis squelch and accurate frequency discriminator output. **(implemented at a basic unit-tested level; RF-vector accuracy remains unmeasured.)**
3. AM/SSB demodulators extract clear baseband audio from DSB-AM and single-sideband suppressed carrier transmissions. **(partial — AM has basic synthetic coverage; the current `I ± Q` SSB path does not reject the opposite sideband.)**
4. FSK/GFSK demodulators slice binary streams with bit error rate (BER) matching theoretical curves under additive white Gaussian noise (AWGN). **(not met — aligned FSK round trips work, but GFSK deviation is divided by samples-per-symbol and no theoretical AWGN BER curve is measured.)**
5. Symbol synchronization and Costas loop lock onto modulated carrier phase within 500 symbol intervals. **(partial — clean BPSK Costas and constrained Gardner tests pass, but invalid/non-finite samples-per-symbol and loop-gain configurations are not rejected; PSK round-trip coverage remains absent. Configuration guards are owned by T-136.)**

## Rationale
Modulation and demodulation form the bridge between raw digitized RF samples and human-interpretable audio or binary data streams. Providing standard, highly optimized demodulator blocks enables immediate compatibility with commercial, amateur, and utility radio signals.

## Alternatives
- Relying on external soundcard audio pipes: Rejected because native in-pipeline demodulation enables multi-channel multi-VFO listening simultaneously across wideband spectrum.
- Fixed-point arithmetic only: Rejected; modern 64-bit and SIMD architectures execute 32-bit floating-point (f32) DSP at equal or superior speed while preserving high dynamic range.

## Consequences
- Requires comprehensive synthetic test vector generation to verify demodulation accuracy across varied SNR levels.
- Algorithms must balance computational complexity with latency for real-time audio output.

## Transition history
- 2026-08-19: created as `proposed`.
- 2026-08-19: moved to `planned` for Sprint 0 execution under T-004.
- 2026-08-19: transitioned to `active` upon starting Build Phase.
- 2026-08-19: transitioned to `realized` in Sprint 0 under T-004.
- 2026-08-24: **re-opened to `active` after Sprint 10's audit.** Sprint 0 realized an initial demodulator surface, not the stated acceptance boundaries: WFM stereo/RDS is absent, SSB does not separate sidebands, GFSK deviation is wrong by the samples-per-symbol factor, and BER/PSK claims lack evidence. The desired modulation outcome remains unchanged.
- 2026-08-24: Sprint 10's adversarial Plan re-screen recorded Gardner's invalid-configuration paths under T-136 so they remain explicit while that bounded correction is deferred.
