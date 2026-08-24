# INT-0002 — Hardware Driver Subsystem and PlutoSDR Integration

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0002
- **State:** active
- **Work evidence:** [Sprint 10 build plan — T-135](../sprints/s10/sprint-plans/build-plan.md), [T-003 build plan](../sprints/s0/sprint-plans/build-plan.md#t-003-sdr-hardware-abstraction-plutosdr-iio-client-sigmf-and-mock-drivers), [T-013 build plan](../sprints/s2/sprint-plans/build-plan.md#t-013-real-pluto-driver--pure-rust-iiod-network-client), [T-014 build plan](../sprints/s2/sprint-plans/build-plan.md#t-014-device-enumeration-api--optional-soapysdr-backend), [T-016 build plan](../sprints/s2/sprint-plans/build-plan.md#t-016-cli---driver-wiring--devices-subcommand)
- **Review evidence:** [Sprint 10 research report](../sprints/s10/sprint-research/research-report.md) — lifecycle, transport, and capture-integrity audit; [Sprint 2 research report](../sprints/s2/sprint-research/research-report.md) — audit found the original driver layer was simulated only.
- **TX work evidence:** [Sprint 4 build plan — T-023..T-026](../sprints/s4/sprint-plans/build-plan.md)
- **TX test evidence:** [Sprint 4 test report](../sprints/s4/sprint-tests/test-report.md)
- **Completion evidence:** [T-003 completion](../work/completed-tasks.md#t-003-sprint-0)
- **Code evidence:** [sdr-hardware](../../crates/sdr-hardware/src/lib.rs)
- **Test evidence:** [Sprint 0 test report](../sprints/s0/sprint-tests/test-report.md), [Sprint 2 test report](../sprints/s2/sprint-tests/test-report.md)
- **Documentation evidence:** [README.md](../../README.md)

## Intent
Provide a unified, cross-platform hardware abstraction layer (`SdrDriver`) supporting physical SDR hardware and file-based captures. Deliver first-class native support for PlutoSDR / Pluto+ via Industrial I/O (IIO) over Gigabit Ethernet and USB, alongside drivers for HackRF, RTL-SDR, and SigMF / WAV / Raw IQ recording and replay engines.

Non-goals for this intent: DSP demodulation algorithms (covered by INT-0003) and GUI visualization (covered in higher-level application layers).

## Acceptance criteria
1. `SdrDriver` trait abstracts device discovery, configuration (center frequency, sample rate, analog bandwidth, gain modes, AGC), and continuous asynchronous RX/TX buffer streaming. **(partial — the trait boundary is well-factored and real TX backends override it, but the default TX methods report successful no-ops/zero writes for an RX-only backend; Sprint 10 T-135 makes unsupported TX fail explicitly. The MultiTone mock also ignores its requested frequencies/sample rate; deferred T-137 owns that test-backend defect.)**
2. **PlutoSDR driver performs real I/O:** connects over network IIO (`ip:...`) or USB to a physical Pluto / Pluto+, configures AD9361/AD9363, and streams non-zero complex IQ samples verified against the attached radio. **(partial — real network-iiod RX and internal-loopback TX are hardware-verified; USB/local URIs are parsed and advertised but rejected at connect time.)**
3. **Multi-device compatibility:** at least the common SDRs are supported behind `SdrDriver` via a feature-gated real backend (e.g. `seify`/`soapysdr`/`desperado`) — Pluto+, RTL-SDR, HackRF, and Airspy at minimum — with `list_devices()` enumeration. Pure DSP crates (`sdr-core`, `sdr-dsp`) must remain free of hardware C-lib dependencies. **(not met — real network Pluto plus `MockSdr` exist; RTL-SDR, HackRF, and Airspy backends do not.)**
4. **CLI actually selects the chosen device:** `--driver` routes to the real backend rather than always using `MockSdr`. **(met for network Pluto and mock; unsupported transports must fail explicitly rather than being advertised as working.)**
5. File-based driver accurately reads and writes SigMF compliant archive pairs (JSON metadata + raw dataset) and RIFF/WAV files with sample-accurate timestamping. **(partial — valid round trips work, but truncated component samples are silently accepted, timestamps are not populated, and raw-IQ/file replay is not exposed as an `SdrDriver`.)**
6. Hotplug detection, clean device teardown, and buffer starvation / overflow recovery are handled gracefully without panics. **(not met — close failures are discarded, failed buffer I/O can desynchronize local/open state, loopback restoration is not transactional, and hotplug/overflow recovery remains unverified.)**

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
- 2026-08-23: **transmit path delivered in Sprint 4 (remains `active`).** A pure-Rust iiod TX path (T-023/T-024) with loopback safety controls (T-025) was verified on the physical Pluto+ with **internal loopback**: under AD9361 internal digital loopback (RF section bypassed) at maximum attenuation, all 4096 written samples were accepted and returned on RX at peak |amp| 0.7071 — exactly the transmitted pattern's magnitude. Criterion 1 (RX/**TX** streaming) and the transmit half of criterion 2 are met *under loopback*; **over-the-air transmit is deliberately unverified** (backlog T-108, needs explicit go-ahead) and criterion 3 (multi-vendor, T-101) remains carried forward, so the intent stays `active`. Testing on hardware also caught three defects that unit and mock tests missed: the two-phase `WRITEBUF` exchange, DDS tone generators enabled by default, and one-shot TX buffers draining before observation. See [Sprint 4 test report](../sprints/s4/sprint-tests/test-report.md).
- 2026-08-22: **advanced in Sprint 2 (remains `active`).** Criteria 1, 2, 4, 5 now met and verified: a pure-Rust iiod client (T-013) performs real RX I/O — proven live against the physical Pluto+ at `192.168.2.1:30431` (4096/4096 non-zero IQ at 95.83 MHz) — and the CLI selects it (T-016), also confirmed live. Criterion 3 (multi-vendor RTL/HackRF/Airspy via SoapySDR/seify) and criterion 6 (hotplug/overflow recovery) are carried forward as backlog T-101/T-102 and are **not** claimed met; the intent stays `active`. See [Sprint 2 test report](../sprints/s2/sprint-tests/test-report.md).
- 2026-08-24: **revised by Sprint 10 audit (remains `active`).** Stale criterion annotations that still called the now-real network driver a zero-streaming stub were corrected. The review also made the remaining gaps explicit: USB/local transport is parse-only, truncated SigMF/WAV component samples are silently accepted, capture timestamps are absent, and Pluto buffer/loopback cleanup is not transactional.
- 2026-08-24: Sprint 10 Plan review corrected criterion 1's blanket “met” annotation (remains `active`). The abstraction exists, but an RX-only implementation inherits TX methods that falsely return success; T-135 owns the bounded fail-explicit correction without claiming hardware breadth or lifecycle completion.
- 2026-08-24: Sprint 10's adversarial Plan re-screen recorded the remaining `MockSdr` MultiTone configuration defect as T-137 rather than allowing it to disappear from the audit inventory.
