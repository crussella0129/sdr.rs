# Sprint 2 Test Report

- **Tested head:** `3af00bf3e1433aa11992bc634f03b99efcd9f4a7`
- **Runner:** `cargo test --workspace` (CI-safe; 2 `#[ignore]`d hardware tests run separately by the agent against the attached Pluto+).
- **Result:** all suites pass — 0 failed. `cargo clippy --workspace --all-targets` reports 0 errors.
- **Critique verdict:** proceed-with-caveats (see `critique.md`).

## Suite results
| Suite | Kind | Result |
|-------|------|--------|
| sdr-core lib | unit | 8 passed |
| sdr-dsp lib | unit | 7 passed |
| sdr-demod lib | unit | 7 passed |
| sdr-protocols lib | unit | 6 passed |
| sdr-spectrum lib | unit | 2 passed |
| sdr-hardware lib | unit | 14 passed |
| sdr-station lib | unit | 4 passed |
| sdr-station cloudlog_it | integration | 5 passed |
| sdr-hardware pluto_iiod | integration | 1 passed |
| sdr-cli rigctl_server_it | e2e | 3 passed |
| sdr-cli driver_cli_it | e2e | 3 passed |
| sdr-cli e2e_pipeline_tests | e2e (regression) | 5 passed |
| sdr-hardware hw_pluto | live hardware (agent) | 2 passed |

## Intent verification

| Intent | Criteria | Verdict |
|--------|----------|---------|
| [INT-0007](../../../intents/INT-0007-station-logging-cloudlog.md) | 1 (/api/radio), 2 (/api/qso ADIF), 3 (config/no key leak), 4 (README) | **Verified** — unit + integration tests all green; ready to realize. |
| [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md) | 4 (Rigctl over TCP) | **Verified** — e2e server tests prove bind/accept/respond/keep-open; criteria 1–3 unchanged from Sprint 0. Ready to realize. |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | 2 (real Pluto I/O), 4 (CLI selects) | **Verified on real hardware** (live hw_pluto + CLI capture). |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | 3 (multi-vendor RTL/HackRF/Airspy), 6 (hotplug/overflow) | **Partial / carried forward** — enumeration seam delivered; SoapySDR backend and hotplug recovery deferred to backlog T-101/T-102. INT-0002 stays `active`. |
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | zero-copy consequence (T-017) | **Verified** — behavior-preserving optimization; intent already realized. |

## Conclusion
The sprint's CI-verifiable slice is fully green and the PlutoSDR path is proven
against the physical radio. INT-0007 and INT-0004 are ready to be realized at
Loop; INT-0002 is materially advanced (real Pluto RX + CLI selection) but
remains `active` because its multi-vendor and hotplug criteria are carried
forward. No re-architecture failure; proceed to Loop.
