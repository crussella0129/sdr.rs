# Plan Critique — Sprint 10

## Concerns

### C-001: T-135 conflicts with authoritative INT-0002 status
- **Where:** `INT-0002-hardware-drivers-pluto.md` Acceptance criterion #1 / `build-plan.md` T-135 and Honest Completion Boundary
- **Quote:** “`SdrDriver` trait abstracts ... continuous asynchronous RX/TX buffer streaming. **(met — trait exists and is well-factored.)**”
- **Failure mode:** intent-drift
- **Why it matters:** T-135 exists because the abstraction falsely reports unsupported TX operations as successful, yet its owning criterion remains marked met and its transition history does not record that correction. INT-0002 is also omitted from the plan’s non-terminal completion boundary.
- **Suggested response:** fix-in-plan — revise INT-0002 criterion #1/transition history to record the default-TX defect and T-135 boundary, and include INT-0002 in the honest completion boundary.

### C-002: Transmission-plan numeric domains are undefined
- **Where:** `build-plan.md` T-128 Success criterion clause 1 / `test-plan.md` T-128 Clauses 1–2
- **Quote:** “WHEN jurisdiction is unknown or any plan number is non-finite/non-positive/out of range”
- **Failure mode:** EARS-vague
- **Why it matters:** Neither units nor valid ranges are specified. In particular, zero or negative EIRP in dBm is valid low power, while duty cycle could mean a fraction or percentage; generic “non-positive/out of range” validation can therefore produce incorrect compliance decisions.
- **Suggested response:** fix-in-plan — name every field and unit, define its exact domain, and add boundary tests distinguishing valid zero/negative dBm from invalid values and percentage duty cycle from fractional duty cycle.

### C-003: Transactional radio tests do not cover the quantified I/O matrix
- **Where:** `build-plan.md` T-126 clause 4 / `test-plan.md` Intent Traceability row “exact radio writes” and Transactional radio I/O
- **Quote:** “WHEN an initial, ACK, or retry write is zero, short, or errors”
- **Failure mode:** plan-test-mismatch
- **Why it matters:** The test plan explicitly covers zero/short initial writes, but only generically “failed” ACK and retry writes. It does not commit to zero and short outcomes for those state-sensitive paths, and the unrelated broadcast-peer test is incorrectly listed as exact-write verification.
- **Suggested response:** fix-in-plan — specify a table-driven Initial/ACK/Retry × Zero/Short/Error matrix, or narrow the EARS clause to the combinations actually tested; remove the broadcast-peer test from the exact-write row.

### C-004: T-133 still claims bitstream-export coverage while rejecting it
- **Where:** `build-plan.md` Intents and T-133 Acceptance criterion / `INT-0009-receiver-application.md` Transition history
- **Quote:** “the demodulated-audio/bitstream export slice of INT-0009 #3”
- **Failure mode:** intent-drift
- **Why it matters:** INT-0009 criterion #3 authorizes WAV audio output, and the revised implementation correctly rejects FSK file output. The remaining “bitstream export” claims contradict both that intent boundary and the named rejection test.
- **Suggested response:** fix-in-plan — change the plan and INT-0009 transition entry to describe an audio-export slice plus explicit rejection of unsupported FSK export.

### C-005: Book dependency regression has no EARS trace
- **Where:** `test-plan.md` Lint, Format, and Book Gates / `build-plan.md` T-126
- **Quote:** “`test_roadmap_names_blocking_dependencies` is updated to assert INT-0011's real T-126 -> T-117 dependency”
- **Failure mode:** plan-test-mismatch
- **Why it matters:** This is a named planned test, but no T-126 EARS clause or Intent Traceability row defines its expected outcome.
- **Suggested response:** fix-in-plan — add a dependency-consistency EARS clause and trace row for this test.

### C-006: Formatter gate is known to fail
- **Where:** `sprint-research/research-report.md` §2; `sprint-plans/test-plan.md` §Lint, Format, and Book Gates; `sprint-plans/build-plan.md` §Verification Strategy.
- **Quote:** “`cargo +1.93.0 fmt --all -- --check` finds five pre-existing formatting diffs,” while the same command is declared a gate and cleanup remains owned by T-106.
- **Failure mode:** hidden-dep
- **Why it matters:** The declared gate cannot pass without silently absorbing deferred T-106 work.
- **Suggested response:** fix-in-plan — include the bounded T-106 baseline or define an attainable no-new-drift gate.

### C-007: Confirmed audit defects disappear from ownership
- **Where:** `sprint-research/research-report.md` §1 and §2; `docs/work/tasks.md`; `build-plan.md` §Explicit Deferrals From the Audit.
- **Quote:** The audit records empty FIR taps, invalid Gardner configuration, and a MultiTone mock that ignores requested frequencies/sample rate.
- **Failure mode:** missing-risk
- **Why it matters:** These confirmed defects had no persistent task or explicit deferral.
- **Suggested response:** defer-with-rationale — create named backlog items tied to suitable intents.

### C-008: Tunnel-derived compliance inputs are not verified
- **Where:** `build-plan.md` T-128; `test-plan.md` T-128 binary and policy composition.
- **Quote:** “The continuous tunnel derives modem bandwidth, uses 100% duty cycle,” without tests asserting either derived value.
- **Failure mode:** plan-test-mismatch
- **Why it matters:** Manually constructed plans can pass while CLI wiring supplies constants or wrong values.
- **Suggested response:** fix-in-plan — add named derivation and pre-driver refusal tests for bandwidth and duty.

### C-009: Demod writer failures lack an EARS clause and test
- **Where:** `build-plan.md` T-133 implementation and EARS/test list; `test-plan.md` T-133 artifact coverage.
- **Quote:** T-133 promises to propagate writer/finalization errors, but named tests cover only success or validation before creation.
- **Failure mode:** plan-test-mismatch
- **Why it matters:** Output-open, write, or finalization failures could still report success.
- **Suggested response:** fix-in-plan — add measurable failure EARS and real/injected writer failure tests.

### C-010: Stable task inventory remains inconsistent with the build plan
- **Where:** `build-plan.md` introduction and T-128; `docs/work/tasks.md` T-128/T-135.
- **Quote:** Ownership stopped at T-134 despite selected T-135, and stable T-128 omitted required cross-crate migrations.
- **Failure mode:** hidden-dep
- **Why it matters:** Resume and dependency accounting would be unreliable.
- **Suggested response:** fix-in-plan — include T-135 in the ownership wording and synchronize T-128 touches.

## Resolutions (primary agent)

| Concern | Resolution |
|---|---|
| C-001 | INT-0002 criterion #1 and transition history now record the false-success TX defaults; T-135 evidence and INT-0002's honest non-terminal boundary are explicit. |
| C-002 | T-128 defines exact Hz/dBm/percentage-point fields and domains; finite zero/negative EIRP is valid, and boundary tests distinguish percent semantics. |
| C-003 | T-126 now requires and traces all nine Initial/ACK/Retry × Zero/Short/Error cases; the broadcast-peer test is no longer misclassified as write coverage. |
| C-004 | T-133 and INT-0009 now claim audio-to-WAV only and explicitly reject unsupported FSK file export. |
| C-005 | T-126 has an EARS clause and named Book regression requiring T-126 then T-117 and rejecting stale T-113. |
| C-006 | T-106 is selected as a bounded final task owning the five known rustfmt hunks, making the global formatter gate attainable. |
| C-007 | T-136 owns FIR/Gardner validation and T-137 owns MultiTone configuration fidelity; both are tied to intents, roadmap dependencies, and the explicit deferral table. |
| C-008 | The plan fixes the Carson formula, forces 100% tunnel duty, unit-tests exact derivation, and independently tests bandwidth and duty refusals before unreachable-driver construction. |
| C-009 | T-133 now has writer-failure EARS plus invalid-target and injected `Write + Seek` write/finalize failures; success may print only after finalization. |
| C-010 | Ownership wording now covers T-119 through T-137, and persistent T-128 touches match every build-plan migration. |

The final adversarial re-screen re-read the current plans, research, task
inventory, roadmap, and cited intents. It reported no remaining concern:
`(none — plan is executable as written)`.

## Confidence
clean
