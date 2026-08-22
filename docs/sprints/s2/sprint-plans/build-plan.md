Finalized - DO NOT EDIT

# Sprint 2 Build Plan

## Intents
- [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) — state: active; acceptance criteria covered: 2 (real Pluto I/O), 3 (multi-device backend), 4 (CLI selects device), 6 (teardown/error handling).
- [INT-0007](../../../intents/INT-0007-station-logging-cloudlog.md) — state: planned; acceptance criteria covered: 1 (/api/radio push), 2 (/api/qso ADIF), 3 (config/no key leak), 4 (README lists Cloudlog).
- [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md) — state: active (reopened); acceptance criteria covered: 4 (Rigctl responds over TCP).
- [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) — state: realized; non-semantic optimization only (T-017 removes a per-iteration allocation in `LinearPipeline`; no acceptance criterion changed).

## Schema Tree
- Sprint Goal: close the library→application gaps found in review (real hardware, station logging, runtime servers)
  - Station logging (INT-0007)
    - T-011: README lists Cloudlog
    - T-012: Cloudlog client (`/api/radio`, `/api/qso`)
  - Real hardware (INT-0002)
    - T-013: pure-Rust iiod Pluto+ driver
    - T-014: device enumeration + optional SoapySDR backend
    - T-016: CLI `--driver` wiring + `devices` subcommand
  - Runtime servers (INT-0004)
    - T-015: Rigctl TCP server
  - Core hardening (INT-0001)
    - T-017: `LinearPipeline` buffer reuse

## Execution Sequence

### T-011: Add Cloudlog to the README reference catalog
- **Intent:** [INT-0007](../../../intents/INT-0007-station-logging-cloudlog.md)
- **Touches:** README.md
- **Depends on:** (none)
- **Acceptance criterion:** INT-0007 #4 — README reference catalog includes Cloudlog.
- **Success criterion (EARS):**
  - **WHEN** the README reference list is rendered, **THEN** it **SHALL** contain a Cloudlog entry linking `https://github.com/magicbug/Cloudlog` with a one-line description.
- **Notes:** documentation task; verified by content inspection (`test_readme_lists_cloudlog` greps the file).

### T-012: Cloudlog station-logging client
- **Intent:** [INT-0007](../../../intents/INT-0007-station-logging-cloudlog.md)
- **Touches:** crates/sdr-station/Cargo.toml, crates/sdr-station/src/lib.rs, crates/sdr-station/src/cloudlog.rs, crates/sdr-station/src/adif.rs, Cargo.toml (workspace members + deps)
- **Depends on:** (none)
- **Acceptance criterion:** INT-0007 #1, #2, #3.
- **Success criterion (EARS):**
  - **WHEN** `CloudlogClient::push_radio(state)` is called, **THEN** it **SHALL** POST JSON `{key,radio,frequency,mode,power,timestamp}` to `<base>/index.php/api/radio` and return `Ok` on HTTP 200.
  - **WHEN** the API responds HTTP 401, **THEN** `push_radio` **SHALL** return an `Auth` error and **SHALL NOT** panic.
  - **WHEN** `CloudlogClient::log_qso(contact)` is called, **THEN** it **SHALL** serialize a valid ADIF record and POST `{key,station_profile_id,type:"adif",string}` to `<base>/index.php/api/qso`.
  - **WHEN** an API key is configured, **THEN** the client **SHALL NOT** write the key value to any log output.
- **Notes:** new crate isolates the HTTP dep (`ureq` + `serde_json`) from pure-DSP crates. Mirror a small `RadioState`/mode-string input to avoid an `sdr-spectrum` dependency.

### T-013: Real Pluto+ driver — pure-Rust iiod network client
- **Intent:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- **Touches:** crates/sdr-hardware/src/pluto.rs, crates/sdr-hardware/src/iiod.rs (new), crates/sdr-hardware/src/lib.rs
- **Depends on:** (none)
- **Acceptance criterion:** INT-0002 #2 (real Pluto network/USB I/O), #6 (teardown/error handling).
- **Success criterion (EARS):**
  - **WHEN** `connect()` is called against a reachable iiod endpoint, **THEN** it **SHALL** complete the iiod `VERSION` handshake and retrieve the device-context XML.
  - **WHEN** the device-context XML is parsed, **THEN** it **SHALL** enumerate `ad9361-phy` and `cf-ad9361-lpc` with their channels into `DeviceInfo`.
  - **WHEN** `set_frequency`/`set_sample_rate`/`set_gain` are called, **THEN** the driver **SHALL** write the matching `ad9361-phy` attribute over iiod and surface an iiod failure as `SdrError`.
  - **WHEN** `read_samples()` is called after `start_rx()`, **THEN** it **SHALL** return interleaved int16 IQ converted to `Complex32` (non-zero against real hardware).
  - **WHEN** a `PlutoSdr` is created with defaults, **THEN** its endpoint **SHALL** be `ip:192.168.2.1` on iiod port `30431`.
- **Notes:** std `TcpStream` only, zero C deps (fulfills INT-0002's Windows/no-C-lib goal). Fixes the wrong port (50901→30431) and default address (192.168.1.10→192.168.2.1) found in review. RX-first; TX deferred. Live verification performed against the attached Pluto+ at `192.168.2.1:30431`.

### T-014: Device enumeration API + optional SoapySDR backend
- **Intent:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- **Touches:** crates/sdr-hardware/src/driver.rs, crates/sdr-hardware/src/soapy.rs (new, feature-gated), crates/sdr-hardware/src/lib.rs, crates/sdr-hardware/Cargo.toml
- **Depends on:** T-013 (shares `DeviceInfo` shape)
- **Acceptance criterion:** INT-0002 #3 (multi-device compatibility behind `SdrDriver`, pure crates stay C-lib-free).
- **Success criterion (EARS):**
  - **WHEN** the `soapysdr` feature is enabled, **THEN** a `SoapySdr` type **SHALL** implement `SdrDriver` and enumerate devices via SoapySDR.
  - **WHEN** no hardware feature is enabled (default), **THEN** `sdr-hardware` **SHALL** build and test with only `MockSdr` + `PlutoSdr` and no C-library dependency.
- **Notes:** the SoapySDR backend is the documented path for RTL-SDR/HackRF/Airspy; live-verifying those is a follow-on (no such hardware attached).

### T-015: Rigctl TCP server
- **Intent:** [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md)
- **Touches:** crates/sdr-cli/src/main.rs (Rigctl command → tokio TCP listener wrapping `RigctlHandler`)
- **Depends on:** (none)
- **Acceptance criterion:** INT-0004 #4 — Rigctl server responds to `f`/`F`/`m`/`M`/VFO queries **over TCP**.
- **Success criterion (EARS):**
  - **WHEN** the `rigctl` command starts, **THEN** it **SHALL** bind a TCP listener on the configured port and accept connections.
  - **WHEN** a connected client sends a command line, **THEN** the server **SHALL** reply with `RigctlHandler` output and **SHALL** keep the connection open until it receives `q`.
- **Notes:** reuses the existing `RigctlHandler` engine unchanged; only adds the missing network runtime. Listener loop lives in `sdr-cli` (already depends on tokio) to keep `sdr-spectrum` dependency-light.

### T-016: CLI `--driver` wiring + `devices` subcommand
- **Intent:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- **Touches:** crates/sdr-cli/src/main.rs
- **Depends on:** T-013, T-014
- **Acceptance criterion:** INT-0002 #4 — CLI actually selects the chosen device.
- **Success criterion (EARS):**
  - **WHEN** `record --driver pluto` is invoked, **THEN** the CLI **SHALL** construct a `PlutoSdr` rather than a `MockSdr`.
  - **WHEN** `record --driver pluto` is invoked and the radio is unreachable, **THEN** the CLI **SHALL** exit with a clear error and **SHALL NOT** panic.
  - **WHEN** `devices` is invoked, **THEN** the CLI **SHALL** print the enumerated devices.
- **Notes:** current `Record` ignores `--driver` and always uses `MockSdr` (review finding).

### T-017: Reuse `LinearPipeline` input buffer across steps
- **Intent:** [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md)
- **Touches:** crates/sdr-core/src/traits.rs
- **Depends on:** (none)
- **Acceptance criterion:** non-semantic optimization consistent with INT-0001's zero-copy consequence; no acceptance criterion changed.
- **Success criterion (EARS):**
  - **WHEN** `step()` is called repeatedly, **THEN** it **SHALL** reuse a preallocated input buffer field rather than heap-allocating a new `Vec` each call, producing identical output to the prior implementation.
- **Notes:** small, isolated diff; behavior-preserving.
