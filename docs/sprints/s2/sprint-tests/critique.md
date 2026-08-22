# Test Critique — Sprint 2

## Concerns

### C-001: INT-0002 acceptance criterion 3 (multi-vendor backend) is only partially proven
- **Where:** `INT-0002` Acceptance criteria #3 / `e2e-tests.md` "Live hardware E2E" / `unit-tests.md` T-014
- **Quote:** "at least the common SDRs are supported behind `SdrDriver` … Pluto+, RTL-SDR, HackRF, and Airspy at minimum"
- **Failure mode:** intent-coverage
- **Why it matters:** The sprint proves the enumeration seam (`list_devices`) and a real, live-verified PlutoSDR backend, but RTL-SDR/HackRF/Airspy via SoapySDR/seify are not implemented or tested (no host libraries or such devices available). Criterion 3 is therefore not fully satisfied.
- **Suggested response:** defer-with-rationale — INT-0002 stays `active` at Loop (not `realized`); criterion 3 (and criterion 6 hotplug/overflow) carry forward via backlog T-101. The Pluto-specific criteria (2, 4) are fully and independently proven, so the sprint still advances the intent honestly.

### C-002: INT-0002 criterion 6 (hotplug / buffer overflow recovery) is not exercised
- **Where:** `INT-0002` Acceptance criteria #6 / `integration-tests.md`
- **Quote:** "Hotplug detection, clean device teardown, and buffer starvation / overflow recovery … without panics"
- **Failure mode:** intent-coverage
- **Why it matters:** Teardown and graceful connect-failure are tested (`test_cli_record_driver_pluto_unreachable`, `teardown` in hw tests), but hotplug and overrun recovery are not.
- **Suggested response:** defer-with-rationale — requires sustained real-hardware streaming and physical unplugging; carried forward to the hardware follow-on. Not claimed as met.

### C-003: Process-spawning E2E tests reuse an ephemeral port after releasing it
- **Where:** `e2e-tests.md` Rigctl/CLI E2E (`free_port()` binds `:0`, drops, then the child re-binds)
- **Quote:** "Grab an unused TCP port by binding to port 0 and releasing it"
- **Failure mode:** flake-risk
- **Why it matters:** A brief TOCTOU window exists between releasing the probe socket and the child binding it; another process could claim the port.
- **Suggested response:** defer-with-rationale — the window is sub-millisecond on a developer/CI host and each test owns its own port; the connect helper retries for up to 10s. Acceptable; revisit only if flakes appear.

## Confidence
proceed-with-caveats
