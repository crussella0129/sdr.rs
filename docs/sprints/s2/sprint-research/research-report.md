# Sprint 2 Research Report

Review-and-gap sprint: audit the entire `sdr.rs` corpus for optimality and
missing capability, verify hardware compatibility (Pluto+ and other common
SDRs), and evaluate Cloudlog as a station-logging integration target.

## Intents Reviewed
- [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) — **revised**;
  the hardware intent was marked `realized` in Sprint 0 but the audit found its
  acceptance criteria are only simulated (no real device I/O, single-vendor).
  Re-opened to `active`; acceptance criteria sharpened to require verified
  Pluto+ streaming plus a multi-device backend. Primary intent this sprint
  advances.
- [INT-0007](../../../intents/INT-0007-station-logging-cloudlog.md) — **created**;
  station logging and Cloudlog integration (push CAT frequency/mode/power to
  `/api/radio`; export contacts/decodes as ADIF to `/api/qso`).

## 1. Sprint Goal
Perform a whole-corpus review of `sdr.rs` to (a) judge whether the existing
architecture and implementations are optimal, (b) enumerate what is missing
against the reference ecosystem in `README.md`, (c) confirm real compatibility
with the user's PlutoSDR / Pluto+ (known-good under SDR++) and other common
SDRs, and (d) assess Cloudlog (`magicbug/Cloudlog`) for integration and decide
whether to add it to the README reference catalog. Produce an honest gap
analysis and a bounded, verifiable build plan; defer physical-radio-verified
work that cannot be proven in CI to a hardware-in-the-loop follow-on.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| crates/sdr-hardware/src/driver.rs | high | `SdrDriver` trait is well-designed (freq/rate/bw/gain/gain-mode, RX+TX, `DeviceInfo`). Good abstraction; the seam a real backend can plug into. |
| crates/sdr-hardware/src/pluto.rs | high | **Stub.** `connect()` only sets a flag; `read_samples()` fills zeros ("offline/simulated mode"). No libiio, no socket, no USB. Only param validation is real. |
| crates/sdr-hardware/src/mock.rs | high | Only functional driver. Deterministic tone/loopback generator used everywhere in the CLI. |
| crates/sdr-hardware/src/lib.rs | high | Exports only `PlutoSdr` + `MockSdr`. No RTL-SDR/HackRF/Airspy/Lime/Soapy. Tests assert URI parsing + mock streaming, not real I/O. |
| crates/sdr-hardware/src/sigmf.rs | medium | Real SigMF read/write (JSON meta + raw dataset). File path works. |
| crates/sdr-hardware/src/wav.rs | medium | Real RIFF/WAV IQ read/write. File path works. |
| crates/sdr-cli/src/main.rs | high | `Record` **ignores `--driver`** and always uses `MockSdr`. `Rigctl`/`Tunnel` construct objects, print a banner, and exit — no server, no proxy loop. File-in/file-out only. |
| crates/sdr-spectrum/src/rigctl.rs | high | Correct Hamlib command **parser** (f/F/m/M/v/`\dump_state`), but there is **no TCP listener** anywhere. "Rigctl server on port 4532" is not a server. |
| crates/sdr-protocols/src/tunnel.rs | high | Real ARQ packetize/ingest logic, but no stdin/stdout or TCP proxy runtime binds it to an actual SSH `ProxyCommand`. |
| crates/sdr-protocols/src/packet.rs | medium | Solid framing/CRC-32/seq/ARQ. Unit-tested and genuinely functional. |
| crates/sdr-core/src/traits.rs | high | Clean `Block`/`Source`/`Sink` traits. `LinearPipeline::step()` heap-allocates `in_buf` every iteration — contradicts the "pre-allocated ring buffer / zero-copy" design claim. |
| crates/sdr-demod/src/wfm.rs | medium | **Mono** quadrature discriminator + de-emphasis only. No 19 kHz stereo pilot PLL and no RDS, despite Sprint 0 research describing both. |
| crates/sdr-core/src/compliance.rs | medium | Real jurisdictional band/compliance database (Sprint 1). Functional and tested. |
| crates/sdr-core/src/sample.rs | medium | Sample types incl. `cs8` (HackRF) and `cs16`/`cf32` — the type layer already anticipates multi-device formats the drivers do not yet produce. |
| README.md | high | Mission + 16 reference links (ingested Sprint 0). No Cloudlog entry yet; user asked to add it. |
| docs/intents/INT-0002-hardware-drivers-pluto.md | high | Marked `realized` on URI-parse + mock tests; acceptance criteria (real Pluto streaming, HackRF/RTL drivers) are not met. Primary target for revision. |

## 3. External Sources
- [Cloudlog (magicbug/Cloudlog)](https://github.com/magicbug/Cloudlog) — Self-hosted amateur-radio logging web app (PHP 7.4+/CodeIgniter 3, MySQL, Docker). Logs HF–microwave contacts, QSL management, ADIF import, companion CAT/automation tooling.
- [Cloudlog API (wiki)](https://github.com/magicbug/Cloudlog/wiki/API) — JSON API: `POST /index.php/api/qso` `{key, station_profile_id, type:"adif", string:<ADIF>}` to log; `POST /index.php/api/radio` `{key, radio, frequency, mode, power?, timestamp}` for live CAT state. API-key auth (401 on missing/invalid). Wavelog (a fork) exposes a compatible API.
- [soapysdr (crates.io)](https://crates.io/crates/soapysdr) — Rust bindings to SoapySDR HAL; wraps RTL-SDR, HackRF, USRP, LimeSDR, BladeRF, Airspy, PlutoSDR. Needs SoapySDR + per-device modules installed on the host (C shared libs).
- [desperado (crates.io)](https://crates.io/crates/desperado) — Feature-gated Rust HAL: pure-Rust RTL-SDR (`rs_rtl`), Airspy (`rs_spy`), HackRF (`rs_hackrf`), Adalm-Pluto, plus SoapySDR for LimeSDR/BladeRF. Reduces host C-lib dependence.
- [seify (FutureSDR/seify)](https://github.com/FutureSDR/seify) — Rusty SDR HAL: Soapy backend (all Soapy frontends) plus growing pure-Rust drivers, typed + dynamic-dispatch devices, vendored `rusb`/libusb for zero-install USB. Closest analogue to the role `SdrDriver` should play.

Research-scope note: the 16 reference codebases enumerated in `README.md`
(GNU Radio, SDR++, Pluto+ setup, cuda-oxide, GNSS-SDR, CuPy, MiniRadioTelescope,
RadioLib, HackRF, URH, radio-ML, Hamlib, eht-imaging, gr-lora, RadioSniffer,
50-things-SDR) were ingested in the Sprint 0 research report and are treated as
established context here. This sprint's new external research is scoped to the
delta the goal introduces: Cloudlog (a new reference the user asked to add) and
the Rust SDR-hardware ecosystem that determines how real Pluto+/multi-device
support is actually built. The five sources above stay within the phase budget.

## 4. Audit Findings — Optimality and Gaps

**Verdict: `sdr.rs` is a high-quality DSP _library_ with a clean, well-factored
trait architecture — but it is not yet a working SDR _application_.** The
signal-processing math, protocol framing, compliance DB, and file (SigMF/WAV)
paths are real and tested. The parts that touch the outside world — radios,
network servers, and the streaming runtime — are stubs or one-shot simulations,
and several intents were marked `realized` on tests that only exercise
parsing/mock behavior.

### 4.1 Hardware compatibility (the user's Pluto+ ask) — largest gap
- No real device I/O exists. `PlutoSdr` does not link libiio and streams zeros;
  the CLI never selects it (always `MockSdr`). So `sdr.rs` **cannot currently
  talk to the Pluto+** the user has working under SDR++, nor any other radio.
- No RTL-SDR / HackRF / Airspy / LimeSDR / BladeRF support of any kind.
- The `SdrDriver` trait, `DeviceInfo`, and multi-format sample types are the
  right seams — the gap is backends, not architecture.
- Path forward (Rust-native, honoring the global "prefer Rust" preference):
  adopt a real backend behind `SdrDriver`, feature-gated so the pure-DSP crates
  stay dependency-light. Candidates: **seify** (Soapy + pure-Rust, vendored USB,
  zero-install — best fit), **soapysdr** (broadest coverage, host C libs),
  **desperado** (pure-Rust RTL/Airspy/HackRF/Pluto). For the user's Ethernet
  Pluto+ specifically, a network-IIO path (`industrial-io`/libiio or seify's
  Soapy-Pluto) is directly verifiable against their hardware.

### 4.2 Missing runtime / servers
- Rigctl: parser is correct; **no TCP server** binds `--port`. Small, fully
  CI-verifiable fix (tokio listener around `RigctlHandler`).
- Tunnel: ARQ logic is real; **no stdin/stdout or TCP proxy loop** wires it to
  `ssh -o ProxyCommand`. End-to-end "SSH over radio" is unproven.
- No live/continuous streaming runtime, no real-time audio out, no spectrum
  UI/waterfall surface — everything is file→file or single-shot.

### 4.3 Capability overstatement (doc vs. code)
- WFM described as stereo + RDS; code is mono discriminator only.
- INT-0002 (`realized`) requires real Pluto streaming and HackRF/RTL drivers —
  none implemented. Honesty correction needed in the Book.

### 4.4 Efficiency / correctness nits
- `LinearPipeline::step()` allocates a fresh input buffer every iteration
  (against the stated zero-copy/ring-buffer design). Reuse a preallocated buffer.

### 4.5 Cloudlog assessment
- Good fit and low-risk to integrate. `sdr.rs` already models rig state
  (`RigState`/rigctl), so a small HTTP client can push `/api/radio`
  (frequency/mode/power) live and export decoded/manual contacts as ADIF to
  `/api/qso`. Fully unit-testable against a mock HTTP server (no radio needed).
  Recommend adding Cloudlog to the README catalog and creating INT-0007.

## 5. Risks, Unknowns, Dependencies
- **Risk — unverifiable-in-CI hardware claims.** Real Pluto+/USB streaming can
  only be proven with the physical radio. *Mitigation:* split work — land
  CI-verifiable pieces now (driver-selection wiring, backend behind a feature
  flag with device enumeration, rigctl TCP server, Cloudlog client with a mock
  server); reserve on-air RX/TX verification for a hardware-in-the-loop sprint
  the user runs with the Pluto+ attached. Do not mark hardware intents
  `realized` on simulation alone.
- **Dependency — external backends.** `soapysdr` needs host C libraries;
  `seify`/`desperado` reduce that but add USB/driver surface. Keep all of it
  behind Cargo features so `sdr-dsp`/`sdr-core` stay pure.
- **Unknown — Cloudlog API drift.** `/api/qso` and `/api/radio` shapes verified
  from the project wiki/community tools; confirm against the user's own Cloudlog
  instance/version before trusting field names in production.
- **Scope risk.** "Review everything + make it optimal" is unbounded. This
  sprint delivers the audit + a small, verifiable slice; the rest becomes
  tracked backlog and follow-on intents, surfaced for the user to prioritize.

## 6. Recommended Approach
1. **Audit deliverable** — publish the whole-corpus review (this report's
   findings) as a durable artifact the user can read and act on.
2. **README** — add Cloudlog to the reference catalog (explicit user ask).
3. **Honesty correction** — revise INT-0002 to reflect simulated-only status,
   sharpen its acceptance criteria toward real multi-device I/O, and re-open it
   `active`.
4. **CI-verifiable slice (candidate build tasks; finalized in Plan):**
   - Wire the CLI `--driver` flag to actually construct `PlutoSdr` vs `MockSdr`.
   - Introduce a feature-gated real-hardware backend (`seify` or `soapysdr`)
     behind `SdrDriver`, with `list_devices()` enumeration; keep it optional.
   - Add a real tokio Rigctl **TCP server** around `RigctlHandler`.
   - Add a Cloudlog client module (`/api/radio`, `/api/qso`) tested against a
     mock HTTP server; new INT-0007.
5. **Hardware-in-the-loop follow-on (separate sprint, user runs it):** verify
   real Pluto+ RX (and TX) end-to-end with the attached radio; only then move
   INT-0002 toward `realized`.
6. **Backlog** — record remaining gaps (WFM stereo/RDS, tunnel proxy runtime,
   streaming/UI, `LinearPipeline` allocation) as `(backlog)` tasks with intent
   IDs for future sprints.

## 7. Artifacts
- `docs/sprints/s2/sprint-research/research-report.md` — this report.
- `docs/intents/INT-0002-hardware-drivers-pluto.md` — revised (reality + sharpened criteria).
- `docs/intents/INT-0007-station-logging-cloudlog.md` — created (Cloudlog integration).
- Reviewed source: `crates/sdr-hardware/src/{driver,pluto,mock,lib}.rs`,
  `crates/sdr-cli/src/main.rs`, `crates/sdr-spectrum/src/rigctl.rs`,
  `crates/sdr-protocols/src/tunnel.rs`, `crates/sdr-core/src/traits.rs`,
  `crates/sdr-demod/src/wfm.rs`.
