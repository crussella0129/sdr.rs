# INT-0004 — Wireless Protocol Decoders and Spectrum Analysis

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0004
- **State:** active
- **Work evidence:** [Sprint 10 build plan — T-134](../sprints/s10/sprint-plans/build-plan.md), [T-005 build plan](../sprints/s0/sprint-plans/build-plan.md#t-005-sdr-protocols-decoders-and-sdr-spectrum-analysis), [T-006 build plan](../sprints/s0/sprint-plans/build-plan.md#t-006-sdr-cli-tool-and-end-to-end-integration-testing), [T-015 build plan](../sprints/s2/sprint-plans/build-plan.md#t-015-rigctl-tcp-server)
- **Review evidence:** [Sprint 10 research report](../sprints/s10/sprint-research/research-report.md) — decoder and spectrum audit; [Sprint 2 research report](../sprints/s2/sprint-research/research-report.md) — acceptance criterion 4 (Rigctl over TCP) was not met at that time.
- **Completion evidence:** [T-005 completion](../work/completed-tasks.md#t-005-sprint-0), [T-006 completion](../work/completed-tasks.md#t-006-sprint-0), [T-015 completion](../work/completed-tasks.md#t-015-sprint-2)
- **Code evidence:** [sdr-protocols](../../crates/sdr-protocols/src/lib.rs), [sdr-spectrum](../../crates/sdr-spectrum/src/lib.rs), [sdr-cli](../../crates/sdr-cli/src/main.rs)
- **Test evidence:** [Sprint 0 test report](../sprints/s0/sprint-tests/test-report.md), [Sprint 2 test report](../sprints/s2/sprint-tests/test-report.md)
- **Documentation evidence:** [README.md](../../README.md)

## Intent
Deliver a suite of wireless protocol decoders and real-time spectrum analysis tools in Rust. Protocol decoders include LoRa physical-layer CSS (Chirp Spread Spectrum) demodulation/decoding, ADS-B (1090 MHz Mode S flight tracking), APRS/AX.25 packet radio (1200 baud Bell 202 AFSK), and POCSAG paging. Spectrum analysis tools provide high-speed FFT power spectrum estimation, waterfall data generation, energy threshold signal detection (CFAR), and Hamlib/Rigctl TCP control server (port 4532) for external software integration.

Non-goals for this intent: Encrypted proprietary military or cellular trunking standards (e.g. TETRA encrypted TEA, encrypted P25).

## Acceptance criteria
1. LoRa decoder correctly detects preambles, compensates fractional frequency offsets, de-chirps baseband IQ symbols, de-interleaves, applies Hamming FEC decoding, and verifies payload CRC. **(not met — only exact-boundary symbol de-chirping, Gray demapping, and bit packing exist; FEC/header/whitening/preamble/CFO/SNR behavior is absent.)**
2. ADS-B decoder detects Mode S pulses at 1090 MHz, validates CRC-24 checksums, and decodes airborne position, velocity, and callsign messages. **(partial — one CRC/callsign vector passes, but altitude bit extraction is wrong and position, velocity, and capture scanning are absent.)**
3. Spectrum analyzer calculates windowed FFT power spectra with configurable averaging and peak detection rates > 60 fps. **(partial — windowed FFT/EMA/peak location exist; the frame-rate claim is unmeasured and the CA-CFAR implementation averages dB rather than linear power.)**
4. Embedded Rigctl server responds to standard Hamlib frequency (`f`, `F`), mode (`m`, `M`), and VFO queries over TCP. **(met at the protocol/server level; the server is not yet composed with a shared receiver/driver state.)**
5. APRS/AX.25 is decoded from Bell-202/NRZI/HDLC input and POCSAG paging bursts are decoded with integrity checks. **(not met — APRS is currently a byte-level AX.25 parser and no POCSAG implementation exists.)**

## Rationale
Real-world SDR utility relies heavily on automated protocol interpretation (telemetry, aviation, IoT) and spectrum observability. Integrating standard protocol decoders directly into the `sdr.rs` suite enables out-of-the-box monitoring without requiring dozens of disparate external tools.

## Alternatives
- Spawning Python scripts for each decoder: Rejected due to high IPC latency, memory footprint, and Python environment dependencies.
- Monolithic monolithic binary without modular crates: Rejected; decoders must be standalone modular crates/modules so embedded or minimal installations can exclude unneeded protocols.

## Consequences
- Requires sample test captures (IQ files) to establish regression tests for each protocol decoder.
- Protocol decoders must be robust against corrupted, truncated, and malformed RF bursts.

## Transition history
- 2026-08-19: created as `proposed`.
- 2026-08-19: moved to `planned` for Sprint 0 execution under T-005 and T-006.
- 2026-08-19: transitioned to `active` upon starting Build Phase.
- 2026-08-19: transitioned to `realized` in Sprint 0 under T-005 and T-006.
- 2026-08-21: **re-opened to `active`.** Sprint 2 review found acceptance criterion 4 (Rigctl server over TCP) was not met — `RigctlHandler` is a correct command parser but nothing binds a TCP socket. Criteria 1–3 (LoRa, ADS-B, spectrum) remain satisfied. Planned into Sprint 2 under T-015. See [Sprint 2 research report](../sprints/s2/sprint-research/research-report.md).
- 2026-08-22: transitioned to `realized` in Sprint 2 under T-015 — a real tokio TCP server now binds the port and serves `RigctlHandler` per connection, proven by e2e tests that drive the CLI binary over loopback (see [Sprint 2 test report](../sprints/s2/sprint-tests/test-report.md)).
- 2026-08-24: **re-opened to `active` after Sprint 10's audit.** The Rigctl correction remains valid, but the broader Sprint 0 realization was factually premature: the LoRa path is a skeleton behind a misleading FEC test name, ADS-B altitude extraction fails a known vector and lacks position/velocity, APRS has no RF demodulator, POCSAG is absent, and spectrum throughput/CFAR claims are unproven or incorrect.
