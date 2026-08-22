# Plan Critique — Sprint 2

## Concerns

### C-001: T-017 is planned work against an already-`realized` intent
- **Where:** `build-plan.md` T-017 / `INT-0001-core-dsp-pipeline.md` (state: realized)
- **Quote:** "state: realized; non-semantic optimization only (T-017 removes a per-iteration allocation in `LinearPipeline`)"
- **Failure mode:** intent-drift
- **Why it matters:** Attaching sprint work to a realized intent can blur whether an acceptance criterion re-opened. If the change altered behavior it would need the intent re-opened and re-verified.
- **Suggested response:** defer-with-rationale — the diff is behavior-preserving (identical output; only removes a heap allocation), changes no acceptance criterion, and aligns with INT-0001's stated zero-copy consequence. Test `test_linear_pipeline_multi_step_correctness` pins output equivalence. Intent stays realized; if any behavior changes during Build, re-open INT-0001 before landing.

### C-002: T-014 SoapySDR clause is verified by gated compilation, not a CI runtime test
- **Where:** `test-plan.md` Intent Traceability (INT-0002 #3 SoapySDR backend) / `build-plan.md` T-014
- **Quote:** "cargo check --features soapysdr (gated; documented)"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** An EARS clause without a runtime test is weaker verification; a broken Soapy path could pass default CI.
- **Suggested response:** defer-with-rationale — SoapySDR requires host C libraries and physical RTL/HackRF/Airspy hardware absent from CI and this machine. Default CI stays green and C-lib-free (the stronger guarantee in criterion 3); the Soapy backend is compiled behind its feature and its live device verification is explicitly unlocked by a follow-on hardware sprint. Not a blocker.

### C-003: T-014 bundles the enumeration API and the SoapySDR backend
- **Where:** `build-plan.md` T-014
- **Quote:** "Device enumeration API + optional SoapySDR backend"
- **Failure mode:** granularity
- **Why it matters:** Two changes in one task can produce an incoherent diff and complicate review/revert.
- **Suggested response:** defer-with-rationale — both serve exactly one acceptance criterion (INT-0002 #3, multi-device behind `SdrDriver`) and share the `DeviceInfo`/`list_devices()` surface; keeping them together yields a coherent "multi-device seam" diff. Build may split the commit if the Soapy module grows.

### C-004: T-013 is large and the riskiest task (full iiod client)
- **Where:** `build-plan.md` T-013
- **Quote:** "largest/riskiest task ... If the streaming buffer path proves too deep for the sprint ..."
- **Failure mode:** granularity / missing-risk
- **Why it matters:** A single task spanning handshake, XML parsing, attribute I/O, and buffer streaming could stall the sprint or tempt faked evidence.
- **Suggested response:** defer-with-rationale — it is one coherent unit (the Pluto driver) and the plan already carries an explicit fallback (land handshake + enumeration + attribute I/O + a live connection proof; stage `READBUF` streaming with fixture tests as backlog) and an explicit "do not fake IQ" guard. Live verification against `192.168.2.1:30431` and the `test_pluto_iiod_replay` integration test bound the risk.

## Confidence
proceed-with-caveats
