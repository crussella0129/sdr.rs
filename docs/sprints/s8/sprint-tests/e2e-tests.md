# Sprint 8 — End-to-End Test Results

- **Tested head:** `3cb6098916c71dd46c051570f868595a0dd55f5e`
- **Date:** 2026-08-23

## Status: not applicable — and why that is not a cop-out

This sprint changed **no runtime behaviour**. No code path executes differently
before and after it. There is no user-facing flow to drive end to end, because
the deliverable is the Project Book: four intent chapters, a roadmap, and
navigation.

Where an E2E test would normally sit, the
[Book-integrity tests](integration-tests.md) do the equivalent job for a
documentation deliverable: they exercise the **real artifacts on disk**, not a
model of them, and they fail when the artifacts are wrong. Their negative
capability was verified by breaking an invariant and observing the failure.

Declaring "E2E impossible" while observable behaviour existed would be the
cop-out this section screens for. Nothing observable was skipped.

## What this sprint deliberately does not verify

The sprint's output is a set of claims about *intent*. Most of those claims are
not testable, and this report does not pretend otherwise.

| Claim | Why it is not verified here | Unlocked by |
|---|---|---|
| The roadmap's **ordering is correct** | Human judgment, not a fact about the code. The user reorders at any sprint boundary. | Nothing — this is a decision, not a measurement. |
| The nine `proposed` categories are **worth building** | Same — a product judgment. | Nothing. |
| INT-0010's **≥30 fps** waterfall | No prototype exists; it is an unverified claim written into the chapter and labelled as such. Headless CI cannot measure frame rate. | An egui/wgpu prototype on a GPU-capable machine (Phase 3). |
| INT-0012's accumulator numerics and **Pluto clock stability** | Unmeasured. Both are the kind of problem that silently yields plausible wrong answers, which is why INT-0012 criterion 1 (noise floor falling as √t) is written to expose them. | The astronomy spike (Phase 4). |
| INT-0013's **classifier accuracy and latency** | No model, no dataset, no measurement. | INT-0013 implementation (Phase 5). |
| INT-0015's achievable **TDOA position accuracy** | Depends on measured inter-node timing uncertainty, which needs hardware. | Time-sync hardware + INT-0015 (Later). |
| INT-0017's **level accuracy** | Requires transmitting into a calibrated fixture. | Transmit authorization (**T-108**) + calibration standards. |

Every one of these is recorded in its own intent chapter as an open claim rather
than presented as established. That is the point of writing them down: a figure
in a `proposed` chapter is a target to be measured, not evidence.

## Hardware

Not exercised. The two `#[ignore]`d Pluto+ tests were not re-run because no code
they touch changed. Their last live result stands from Sprint 7 — 87 bytes
carried as 2 datagrams over the internal loopback, recovered byte-for-byte.

**Nothing in this sprint went on air.** That remains gated on **T-108** and the
user's explicit authorization.
