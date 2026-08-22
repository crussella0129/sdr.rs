# INT-0002 — Hardware Driver Subsystem and PlutoSDR Integration

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0002
- **State:** active
- **Work evidence:** [T-003 build plan](../sprints/s0/sprint-plans/build-plan.md#t-003-sdr-hardware-abstraction-plutosdr-iio-client-sigmf-and-mock-drivers), [T-013 build plan](../sprints/s2/sprint-plans/build-plan.md#t-013-real-pluto-driver--pure-rust-iiod-network-client), [T-014 build plan](../sprints/s2/sprint-plans/build-plan.md#t-014-device-enumeration-api--optional-soapysdr-backend), [T-016 build plan](../sprints/s2/sprint-plans/build-plan.md#t-016-cli---driver-wiring--devices-subcommand)
- **Review evidence:** [Sprint 2 research report](../sprints/s2/sprint-research/research-report.md) — audit found the driver layer is simulated only.
- **Completion evidence:** [T-003 completion](../work/completed-tasks.md#t-003-sprint-0)
- **Code evidence:** [sdr-hardware](../../crates/sdr-hardware/src/lib.rs)
- **Test evidence:** [Sprint 0 test report](../sprints/s0/sprint-tests/test-report.md), [Sprint 2 test report](../sprints/s2/sprint-tests/test-report.md)
- **Documentation evidence:** [README.md](../../README.md)

## Intent
Provide a unified, cross-platform hardware abstraction layer (`SdrDriver`) supporting physical SDR hardware and file-based captures. Deliver first-class native support for PlutoSDR / Pluto+ via Industrial I/O (IIO) over Gigabit Ethernet and USB, alongside drivers for HackRF, RTL-SDR, and SigMF / WAV / Raw IQ recording and replay engines.

Non-goals for this intent: DSP demodulation algorithms (covered by INT-0003) and GUI visualization (covered in higher-level application layers).

## Acceptance criteria
1. `SdrDriver` trait abstracts device discovery, configuration (center frequency, sample rate, analog bandwidth, gain modes, AGC), and continuous asynchronous RX/TX buffer streaming. **(met — trait exists and is well-factored.)**
2. **PlutoSDR driver performs real I/O:** connects over network IIO (`ip:...`) or USB to a physical Pluto / Pluto+, configures AD9361/AD9363, and streams non-zero complex IQ samples verified against the attached radio. *(not met — current `PlutoSdr` is a validation-only stub that streams zeros; no libiio/socket/USB.)*
3. **Multi-device compatibility:** at least the common SDRs are supported behind `SdrDriver` via a feature-gated real backend (e.g. `seify`/`soapysdr`/`desperado`) — Pluto+, RTL-SDR, HackRF, and Airspy at minimum — with `list_devices()` enumeration. Pure DSP crates (`sdr-core`, `sdr-dsp`) must remain free of hardware C-lib dependencies. *(not met — only `PlutoSdr` stub + `MockSdr` exist.)*
4. **CLI actually selects the chosen device:** `--driver` routes to the real backend rather than always using `MockSdr`. *(not met.)*
5. File-based driver accurately reads and writes SigMF compliant archive pairs (JSON metadata + raw dataset) and RIFF/WAV files with sample-accurate timestamping. **(met.)**
6. Hotplug detection, clean device teardown, and buffer starvation / overflow recovery are handled gracefully without panics. *(partial — teardown exists; overflow/hotplug untested against real hardware.)*

Verification note: criteria 2, 3, and 6 require hardware-in-the-loop
verification against the physical Pluto+; they must not be marked realized on
mock/simulated evidence alone.

## Rationale
Real-world SDR workflows depend on hardware interoperability. The Pluto+ / AD936x platform offers high RF bandwidth (up to 56 MHz) and full-duplex transceiver capabilities over Ethernet, making native Rust support essential for laboratory, field, and remote operations.

## Alternatives
- Rely solely on SoapySDR dynamic C bindings (`soapysdr` crate): broadest device coverage (RTL/HackRF/Airspy/Lime/Blade/USRP/Pluto) but requires host C shared libraries. Retained as an optional feature-gated backend, not the sole strategy.
- `seify` (FutureSDR Rust HAL): Soapy backend plus growing pure-Rust drivers with vendored `rusb`/libusb (zero-install USB), typed + dynamic dispatch. Preferred first real backend — closest fit to the `SdrDriver` seam and aligns with the Rust-first preference.
- `desperado` crate: pure-Rust RTL-SDR/Airspy/HackRF/Pluto drivers (+ Soapy for Lime/Blade). Candidate for reducing C-lib dependence on RX-only common devices.
- Hand-rolled native libiio/network-IIO client for Pluto+ (`industrial-io` crate): direct control of the AD936x over Gigabit Ethernet; directly verifiable against the user's Ethernet Pluto+.
- USB-only transport for Pluto: Rejected as the only path because Pluto+ features dedicated Gigabit Ethernet allowing higher data rates and network isolation; both transports should be supported.

## Consequences
- Requires socket-level networking protocols and USB bulk streaming handling with asynchronous multi-threaded I/O pipelines.
- Unit testing requires mock hardware backends to run deterministically in CI environments where physical RF radios are absent.

## Transition history
- 2026-08-19: created as `proposed`.
- 2026-08-19: moved to `planned` for Sprint 0 execution under T-003.
- 2026-08-19: transitioned to `active` upon starting Build Phase.
- 2026-08-19: transitioned to `realized` in Sprint 0 under T-003.
- 2026-08-21: **re-opened to `active`.** Sprint 2 whole-corpus review found the Sprint 0 `realized` transition was inaccurate: the `PlutoSdr` driver is a validation-only stub (streams zeros, no libiio/socket/USB), the CLI never selects it, and no multi-device backend exists. Acceptance criteria rewritten to require real, hardware-verified I/O and multi-device support. See [Sprint 2 research report](../sprints/s2/sprint-research/research-report.md).
- 2026-08-22: **advanced in Sprint 2 (remains `active`).** Criteria 1, 2, 4, 5 now met and verified: a pure-Rust iiod client (T-013) performs real RX I/O — proven live against the physical Pluto+ at `192.168.2.1:30431` (4096/4096 non-zero IQ at 95.83 MHz) — and the CLI selects it (T-016), also confirmed live. Criterion 3 (multi-vendor RTL/HackRF/Airspy via SoapySDR/seify) and criterion 6 (hotplug/overflow recovery) are carried forward as backlog T-101/T-102 and are **not** claimed met; the intent stays `active`. See [Sprint 2 test report](../sprints/s2/sprint-tests/test-report.md).
