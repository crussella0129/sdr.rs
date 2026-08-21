# INT-0002 — Hardware Driver Subsystem and PlutoSDR Integration

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0002
- **State:** realized
- **Work evidence:** [T-003 build plan](../sprints/s0/sprint-plans/build-plan.md#t-003-sdr-hardware-abstraction-plutosdr-iio-client-sigmf-and-mock-drivers)
- **Completion evidence:** [T-003 completion](../work/completed-tasks.md#t-003-sprint-0)
- **Code evidence:** [sdr-hardware](../../crates/sdr-hardware/src/lib.rs)
- **Test evidence:** [Sprint 0 test report](../sprints/s0/sprint-tests/test-report.md)
- **Documentation evidence:** [README.md](../../README.md)

## Intent
Provide a unified, cross-platform hardware abstraction layer (`SdrDriver`) supporting physical SDR hardware and file-based captures. Deliver first-class native support for PlutoSDR / Pluto+ via Industrial I/O (IIO) over Gigabit Ethernet and USB, alongside drivers for HackRF, RTL-SDR, and SigMF / WAV / Raw IQ recording and replay engines.

Non-goals for this intent: DSP demodulation algorithms (covered by INT-0003) and GUI visualization (covered in higher-level application layers).

## Acceptance criteria
1. `SdrDriver` trait abstracts device discovery, configuration (center frequency, sample rate, analog bandwidth, gain modes, AGC), and continuous asynchronous RX/TX buffer streaming.
2. PlutoSDR driver communicates over network (`ip:192.168.1.10` / `ip:192.168.2.1`) or USB, controlling AD9361/AD9363 registers and streaming complex 16-bit IQ samples without buffer overruns at standard rates.
3. File-based driver accurately reads and writes SigMF compliant archive pairs (JSON metadata + raw dataset) and RIFF/WAV files with sample-accurate timestamping.
4. Hotplug detection, clean device teardown, and buffer starvation / overflow recovery are handled gracefully without panics.

## Rationale
Real-world SDR workflows depend on hardware interoperability. The Pluto+ / AD936x platform offers high RF bandwidth (up to 56 MHz) and full-duplex transceiver capabilities over Ethernet, making native Rust support essential for laboratory, field, and remote operations.

## Alternatives
- Rely solely on SoapySDR dynamic C bindings: Rejected as sole driver strategy to eliminate external C shared library dependencies on embedded and Windows targets, while retaining SoapySDR as an optional fallback backend.
- USB-only transport for Pluto: Rejected because Pluto+ features dedicated Gigabit Ethernet allowing higher data rates and network isolation.

## Consequences
- Requires socket-level networking protocols and USB bulk streaming handling with asynchronous multi-threaded I/O pipelines.
- Unit testing requires mock hardware backends to run deterministically in CI environments where physical RF radios are absent.

## Transition history
- 2026-08-19: created as `proposed`.
- 2026-08-19: moved to `planned` for Sprint 0 execution under T-003.
- 2026-08-19: transitioned to `active` upon starting Build Phase.
- 2026-08-19: transitioned to `realized` in Sprint 0 under T-003.
