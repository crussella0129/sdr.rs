# Sprint 10 Research Report

## Intents Reviewed
- [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) — **revised**; memory safety, lossless streaming, tag propagation, and anti-alias/SIMD/CORDIC claims were found unmet; state corrected to `active`.
- [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) — **revised**; real network-IIO work was confirmed, while USB/local transport, capture integrity, and transactional lifecycle gaps were made explicit; remains `active`.
- [INT-0003](../../../intents/INT-0003-modulation-demodulation.md) — **revised**; WFM stereo/RDS, SSB image rejection, GFSK deviation, and BER evidence were found incomplete; state corrected to `active`.
- [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md) — **revised**; Rigctl TCP is real, but LoRa/ADS-B/APRS/POCSAG and spectrum claims exceed the implementation; state corrected to `active`.
- [INT-0005](../../../intents/INT-0005-regulatory-band-compliance.md) — **revised**; bandwidth/duty-cycle evaluation and fail-closed input/enforcement are missing; state corrected to `active`.
- [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) — **revised**; Sprint 9's serialized ARQ proof stands, but reordered frames, GFSK scaling, driver short writes, and the fire-and-forget tunnel contradict the wider criteria; state corrected to `active`.
- [INT-0007](../../../intents/INT-0007-station-logging-cloudlog.md) — **revised**; the isolated client and mock tests work, but required receiver-state composition and truthful QSO validation do not; state corrected to `active`.
- [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) — **revised**; selected for reliability, streaming, and policy-bypass findings; remains `active`.
- [INT-0009](../../../intents/INT-0009-receiver-application.md) — **revised and selected**; the existing CLI false-success paths are recorded as starting gaps rather than application evidence; remains `proposed` pending Plan.
- [INT-0011](../../../intents/INT-0011-mesh-messaging-callsign.md) — **revised (dependency only)**; stale T-113 dependency replaced by ARQ correctness T-126 followed by tunnel integration T-117; remains `proposed` and is not built this sprint.

## 1. Sprint Goal

Perform an adversarial whole-corpus code review of everything currently built: run the available workspace tests, inspect safety/correctness/security/performance/maintainability, reproduce negative paths that happy-path tests miss, correct inaccurate Book claims, and turn every confirmed stub or incomplete behavior into owned intent/backlog work. The build recommendation is bounded to tractable critical/high defects and truthful CLI behavior; full decoder feature families, multi-vendor hardware, TUN/Babel, on-air operation, and current-law certification remain out of scope.

## 2. Existing Code Survey

| File | Relevance | Notes |
|---|---|---|
| `Cargo.toml` | high | Nine-crate workspace and dependency boundary; pure DSP remains separate from HTTP/hardware dependencies. |
| `crates/sdr-core/src/buffer.rs` | critical | Shared-reference writes bypass `UnsafeCell`; unrestricted callers defeat the claimed SPSC invariant. |
| `crates/sdr-core/src/traits.rs` | critical | Partial block consumption/sink writes and all tags are silently discarded. |
| `crates/sdr-core/src/compliance.rs` | critical | Bandwidth/duty-cycle fields are not evaluated; invalid/non-finite inputs can fail open. |
| `crates/sdr-dsp/src/resample.rs` | high | Filter length scales with interpolation, not worst-case decimation; 100:1 probe rejected an aliased tone by only about 5.2 dB. |
| `crates/sdr-dsp/src/fir.rs` | medium | Scalar FIR works; empty taps can reach modulo-by-zero and SIMD/reference claims are absent. |
| `crates/sdr-dsp/src/clock_recovery.rs` | medium | Useful constrained drift tests exist; invalid SPS/gain configurations are not rejected. |
| `crates/sdr-demod/src/modulator.rs` | high | GFSK zero-stuff/filter normalization divides requested deviation by samples per symbol. |
| `crates/sdr-demod/src/ssb.rs` | high | `I ± Q` does not reject the opposite sideband; existing test checks only finiteness. |
| `crates/sdr-demod/src/wfm.rs` | high | Functional mono discriminator/de-emphasis, but no stereo MPX/pilot/RDS path. |
| `crates/sdr-demod/src/psk.rs` | medium | No independent round trip; QPSK output ordering conflicts with its documented mapping. |
| `crates/sdr-protocols/src/packet.rs` | critical | Multiple outstanding frames plus discard-and-ACK of unexpected sequence numbers causes permanent reordered-frame loss. |
| `crates/sdr-protocols/src/lora.rs` | high | Exact-boundary symbol demod works; payload path is Gray-map/pack only with hard-coded SNR and vacuous short CRC. |
| `crates/sdr-protocols/src/adsb.rs` | high | Known callsign/CRC works; altitude bits are extracted incorrectly and position/velocity are absent. |
| `crates/sdr-protocols/src/aprs.rs` | high | Byte-level AX.25 parser only; no Bell-202/NRZI/HDLC receive path and malformed minimum frame is accepted. |
| `crates/sdr-spectrum/src/fft.rs` | medium | FFT peak path works; constructor panics on invalid size and >60 fps is unmeasured. |
| `crates/sdr-spectrum/src/cfar.rs` | high | Training cells are averaged in logarithmic dB rather than linear power, so the implementation is not CA-CFAR. |
| `crates/sdr-spectrum/src/rigctl.rs` | medium | Command parser is real; state is per-client and not connected to a receiver/driver. |
| `crates/sdr-hardware/src/driver.rs` | high | Default TX methods return successful zero writes, allowing unsupported TX to masquerade as success. |
| `crates/sdr-hardware/src/pluto.rs` | high | Network iiod RX/TX is real; USB/local is parse-only and open/close/loopback state changes are not transactional. |
| `crates/sdr-hardware/src/sigmf.rs` | high | Valid round trips work; any component read error is treated as EOF and timestamps remain absent. |
| `crates/sdr-hardware/src/wav.rs` | high | Valid round trips work; truncated/odd component data is silently shortened. |
| `crates/sdr-hardware/src/mock.rs` | medium | Deterministic tone/loopback are valuable; MultiTone ignores requested frequencies/sample rate. |
| `crates/sdr-mesh/src/radio.rs` | critical | Zero/short driver writes count as successful sends; decoder state is discarded across reads; ACK/retry state is consumed before I/O commits. |
| `crates/sdr-mesh/src/stream.rs` | high | Fire-and-forget burst emission conflicts with stop-and-wait; queues/backpressure/shutdown are unbounded or underspecified. |
| `crates/sdr-mesh/src/node.rs` | high | Policy refusal exists here, but the CLI tunnel bypasses this wrapper and constructs `RadioLink` directly. |
| `crates/sdr-mesh/src/kiss.rs` | medium | Correct escaping/framing on bounded tests; pre-delimiter accumulation is unbounded. |
| `crates/sdr-station/src/cloudlog.rs` | high | Mock HTTP paths work; profile/import semantics can produce false success and no app uses the client. |
| `crates/sdr-station/src/adif.rs` | medium | Serializer exists but accepts invalid dates/times/callsigns/modes/non-finite values. |
| `crates/sdr-cli/src/main.rs` | critical | `demod --output` claims export without writing; invalid jurisdiction defaults to US; tunnel warns then transmits; several invalid modes/configurations succeed or panic. |
| `crates/sdr-cli/tests/rigctl_server_it.rs` | high | Free-port release/spawn race made the full suite flaky under parallel execution. |
| `crates/sdr-cli/tests/e2e_pipeline_tests.rs` | medium | Good cross-crate smoke tests; several names/assertions prove only shapes or simulated handshakes, not advertised capabilities. |
| `docs/work/tasks.md` | high | T-103/T-113 were stale; replaced with concrete T-119–T-137 audit findings while preserving existing hardware/gated backlog. |
| `docs/roadmap.md` | high | State counts and ARQ dependencies were stale; repaired from intent authority. |

### Review dimensions

- **Security / safety — needs immediate work.** The ring buffer is memory-unsound under its public concurrent API; remote control/tunnel listeners and policy ownership need hardening; no secret was found hard-coded.
- **Correctness — critical.** Reproduced silent data loss, false-success file output, truncated-capture acceptance, ARQ reordered-frame loss, wrong DSP scaling/sideband behavior, wrong ADS-B altitude, and incomplete compliance evaluation.
- **Performance — unverified.** Hot paths reuse some buffers and deterministic DSP primitives are compact, but SIMD/60-fps/anti-alias claims are not benchmarked or met; receive polling repeatedly allocates and loses streaming state.
- **Maintainability — needs work.** Crate boundaries and test seams are good, but constructors rely on panics/silent defaults, partial behavior is presented through complete-sounding APIs, workspace formatting drifts, and Clippy reports warnings across multiple crates.

### Functionality positively confirmed

- Network-iiod Pluto RX and internal-loopback TX protocol paths are real; valid SigMF/WAV round trips work; physical-radio tests remain deliberately ignored by the normal suite.
- Basic sample conversions, tags as data structures, scalar FIR/NCO/Hilbert, constrained Gardner/FSK timing, packet CRC framing, serialized stop-and-wait loss simulation, FFT peak location, Rigctl TCP commands, mock tunnel byte piping, Cloudlog mock HTTP, and ADIF serialization have meaningful tests.
- `cargo +1.93.0 clippy --workspace --all-targets` completes with warnings but no error-level diagnostics. `cargo +1.93.0 fmt --all -- --check` finds five pre-existing formatting diffs.
- Subsystem runs passed: signal/protocol/spectrum **44 tests**; hardware **27 non-HIL tests** with 3 HIL ignored; CLI **22 tests**; mesh/station unit and integration suites passed when loopback socket permissions were available.
- A full workspace run reached the Rigctl suite and exposed a parallel-only TCP reset; the same three Rigctl tests passed alone. This is retained as T-132 rather than reported as a green full-suite baseline.

## 3. External Sources

- None used to establish the implementation findings. They are repository-specific and were reproduced from code/tests. In particular, Sprint 10 did **not** certify the regulatory catalog against current law; current primary legal sources must be reviewed before INT-0005 can return to `realized`.

## 4. Risks, Unknowns, Dependencies

- **Risk — unsafe code:** continuing to expose the current `RingBuffer` as `Sync` risks undefined behavior even when ordinary tests stay green.
- **Risk — false confidence from happy paths:** many tests use self-generated vectors, lossless loopback, complete reads, and full writes; they cannot detect independent-standard or backpressure failures.
- **Risk — compliance fail-open:** an invalid jurisdiction, ignored bandwidth/duty cycle, or a caller bypassing `MeshNode` can result in a transmit path proceeding after a non-compliant decision. No on-air test is authorized.
- **Risk — scope:** completing LoRa, APRS, POCSAG, WFM stereo/RDS, multi-vendor hardware, TUN/Babel, or a receiver application would each exceed this correction sprint; they are durable T-101/T-104/T-107/T-123/T-124/T-129 work.
- **Unknown — physical boundaries:** one Pluto in internal digital loopback cannot prove two-radio ARQ, over-the-air behavior, hotplug, true clock separation, or concurrent device access.
- **Unknown — current legal data:** the code-shape audit proves fields/inputs are ignored; it does not prove each catalog row's present-day jurisdictional accuracy.
- **Dependency — toolchain:** the default Rust 1.95 sysroot on this WSL environment is missing its standard-library files despite rustup metadata; installed Rust 1.93.0 is viable and is the verification toolchain for this sprint.
- **Dependency — sequence:** repair ARQ semantics (T-126) before enabling it in tunnel streams (T-117); repair core soundness/loss (T-119/T-120) before making new high-throughput pipeline claims.

## 5. Recommended Approach

Primary: treat Sprint 10 as a correctness-and-honesty recovery sprint. Plan a bounded build around (1) eliminating the critical ring-buffer unsoundness; (2) preventing pipeline and radio short-write/reordering loss; (3) replacing the visible CLI false-success and fail-open input/policy paths with explicit errors/real output; (4) stabilizing the Rigctl integration harness; and (5) applying formatter/Clippy refactors only where behavior is protected. Add adversarial tests first or alongside each fix. Leave broad feature completion in T-121–T-125/T-127/T-129–T-131 unless it is a small prerequisite to a selected correction.

Alternative considered: produce only a review report and backlog. Rejected because the audit found a critical unsafe abstraction and several small, reproducible high-severity failures whose safe corrections fit one sprint.

Rationale: this preserves working, well-tested bounded paths and narrows claims to evidence. It also avoids a broad cosmetic rewrite before safety and data-loss semantics are fixed.

## Artifacts

- `docs/intents/INT-0001-*.md` through `INT-0011-*.md` (reviewed subset) — authoritative state/acceptance corrections and dependency notes.
- `docs/work/tasks.md` — stale T-103/T-113 removed; T-119–T-137 record every confirmed stub or incomplete path with owning intents, including the FIR/Gardner/MultiTone findings added during adversarial Plan accounting.
- `docs/roadmap.md` — navigation/state/dependency view repaired from intent authority.
- Verification commands and observed failures are summarized above; no standalone binary/log artifact was saved.

## Budget Override

The user explicitly requested review of **everything built so far** across nine crates, including stub discovery and intent reconciliation. Safety, driver, DSP, protocol, application, and cross-crate composition claims cannot be assessed honestly within twenty files, so the survey exceeded that cap using three parallel read-only subsystem audits plus targeted adversarial probes. The expansion stayed within the requested whole-corpus goal, used no more than zero external sources for implementation claims, and stopped short of unrelated proposed feature families.
