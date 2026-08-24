# INT-0001 — Core DSP Pipeline and SIMD Engine

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0001
- **State:** active
- **Review evidence:** [Sprint 10 research report](../sprints/s10/sprint-research/research-report.md) — whole-corpus audit found memory unsafety, silent pipeline loss, missing tag propagation, and unverified numerical/performance claims.
- **Work evidence:** [Sprint 10 build plan — T-119/T-120](../sprints/s10/sprint-plans/build-plan.md), [T-001 build plan](../sprints/s0/sprint-plans/build-plan.md#t-001-cargo-workspace-setup-and-sdr-core-streaming-architecture), [T-002 build plan](../sprints/s0/sprint-plans/build-plan.md#t-002-sdr-dsp-filtering-simd-convolution-nco-resamplers-and-synchronization)
- **Completion evidence:** [T-001 completion](../work/completed-tasks.md#t-001-sprint-0), [T-002 completion](../work/completed-tasks.md#t-002-sprint-0)
- **Code evidence:** [sdr-core](../../crates/sdr-core/src/lib.rs), [sdr-dsp](../../crates/sdr-dsp/src/lib.rs)
- **Test evidence:** [Sprint 0 test report](../sprints/s0/sprint-tests/test-report.md)
- **Documentation evidence:** [README.md](../../README.md)

## Intent
Provide a high-throughput, memory-safe, zero-copy Digital Signal Processing (DSP) core and stream abstraction in Rust. The engine provides complex sample representations (`Complex32`, `Complex64`, `Complex<i16>`), sample-stream metadata tagging (frequency, sample rate, burst markers), circular lock-free sample buffers, numerically controlled oscillators (NCO), windowed FIR/IIR filter design and polyphase resamplers, Hilbert transforms, and SIMD-accelerated vector operations (AVX2, AVX-512, ARM NEON).

Non-goals for this intent: hardware-specific USB/IIO drivers (covered by INT-0002) and protocol-specific payload decoders (covered by INT-0004).

## Acceptance criteria
1. Core streaming traits (`Source`, `Sink`, `Block`, `Stream`) permit composable pipeline execution with lock-free bounded ring buffers. **(not met — the current shared-reference ring buffer mutates storage without `UnsafeCell` and does not enforce SPSC ownership; `LinearPipeline` can silently discard partially consumed/written samples.)**
2. FIR filter implementation with SIMD acceleration passes frequency response validation against reference filter coefficients across low-pass, high-pass, and band-pass configurations. **(partial — scalar FIR behavior is tested, but empty taps can reach modulo-by-zero; no SIMD path or complete low/high/band-pass reference validation exists. Invalid-construction handling is owned by T-136.)**
3. Rational and polyphase resampling engines correctly resample IQ streams across arbitrary rate transitions with anti-aliasing rejection > 60 dB. **(not met — a 100:1 decimation probe measured only about 5.2 dB rejection because filter sizing ignores the decimation factor.)**
4. NCO and CORDIC mixing modules accurately shift center frequencies and maintain phase continuity across buffer boundaries without phase slippage. **(partial — NCO phase continuity is tested; the implementation uses `sin_cos`, not a CORDIC path.)**
5. In-band stream tags propagate synchronously with sample offsets across processing blocks. **(not met — tag types exist, but `LinearPipeline` never calls `Source::get_tags` or `Sink::put_tags`.)**

## Rationale
SDR applications require deterministic real-time processing of high-rate IQ streams (tens of MSPS). Implementing a modular DSP core in idiomatic Rust guarantees memory safety and thread safety without sacrificing the raw performance provided by C++ engines like GNU Radio and SDR++, while avoiding garbage collection pauses.

## Alternatives
- C++ with Rust FFI: Considered wrapping GNU Radio or liquid-dsp, but rejected to achieve a native, portable, memory-safe Rust codebase with seamless cross-compilation.
- Purely dynamic graph framework: Considered dynamic message passing for every sample, but rejected due to high CPU overhead compared to batched block-based buffer processing.

## Consequences
- Requires careful use of SIMD intrinsics and cache-aligned memory layouts to meet multi-million sample/second real-time deadlines.
- Demands rigorous unit and benchmark testing of numerical accuracy and precision against standard DSP reference models.

## Transition history
- 2026-08-19: created as `proposed`.
- 2026-08-19: moved to `planned` for Sprint 0 execution under T-001 and T-002.
- 2026-08-19: transitioned to `active` upon starting Build Phase.
- 2026-08-19: transitioned to `realized` in Sprint 0 under T-001 and T-002.
- 2026-08-24: **re-opened to `active` after Sprint 10's whole-corpus audit.** The Sprint 0 realization was an error of fact: the public ring-buffer API is unsound under its advertised concurrent use, the pipeline drops partial consumption/writes and all tags, empty FIR taps can panic, the resampler misses the stated rejection by a wide margin at large decimation, and the SIMD/CORDIC claims have no implementation. This follows the Book's established correction precedent for a factually incorrect terminal state; the desired outcome is unchanged. T-136 owns the deferred FIR configuration guard.
