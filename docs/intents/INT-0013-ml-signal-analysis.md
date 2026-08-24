# INT-0013 — ML-Assisted Signal Classification, Decoding, and Analysis

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0013
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** [Roadmap](../roadmap.md) — phase and dependency placement
- **Review evidence:** [Sprint 8 research report](../sprints/s8/sprint-research/research-report.md)

## Intent
Use machine learning where it genuinely beats hand-written DSP — identifying
what a signal *is*, and decoding waveforms that resist analytic demodulation —
and use a language model where language is actually the task.

### The split this intent is built on

The question of how to serve a model has two different answers, and running them
together is the main design error to avoid:

| | Signal path | Assistant path |
|---|---|---|
| **Task** | classify modulation, detect/segment signals, decode noisy or unknown waveforms | explain a signal, summarize a session, assist identification, generate reports |
| **Data rate** | sample rate — continuous, real time | occasional, human-paced |
| **Deployment** | **in-process inference**, no network hop | **optional local endpoint** (OpenAI-compatible, e.g. a local server) |
| **Required?** | yes, when the feature is used | no — entirely optional, never in the DSP path |

Serving a classifier over localhost HTTP would put serialization and a network
round trip inside a loop that runs at the sample rate; that is the wrong place
for a server. Conversely, embedding a large language model in-process to answer
occasional questions would impose a heavyweight dependency for a feature used
once a minute, when many users already run a local endpoint.

So: **in-process for the sample path, local server for the language path.** The
localhost-server instinct is right — it is just right for the assistant, not the
decoder.

Scope:
1. Modulation and signal-type classification from IQ.
2. Signal detection and segmentation in wideband captures — finding what is
   present before deciding what it is.
3. ML-assisted decoding for waveforms where analytic demodulation is brittle.
4. An **optional** assistant against a user-supplied local endpoint.
5. A pluggable inference backend, so no single runtime is load-bearing.

**Non-goals:** training infrastructure (models are trained out-of-band);
shipping a model whose provenance cannot be documented; and any feature that
becomes unavailable when no model is installed — ML augments this suite, it does
not gate it.

## Acceptance criteria
1. A classifier identifies modulation type from IQ with **published accuracy on
   a stated dataset**, running in-process with no network hop in the sample
   path.
2. The inference backend sits behind a trait with **at least two working
   implementations** (e.g. ONNX Runtime via `ort`, and `candle` for a pure-Rust
   build), selectable at runtime and covered by the same tests.
3. Classification sustains a **stated sample rate on stated hardware**, measured
   and published — not asserted. A classifier that cannot keep up is a
   correctness failure, not a performance note.
4. Every shipped or recommended model has documented provenance: source,
   training data, licence, and a reproducible path to retrain it. No opaque
   weights.
5. The assistant connects to a user-supplied OpenAI-compatible local endpoint,
   is fully optional, and its absence disables only assistant features. A test
   asserts the DSP path never calls it.
6. Model output is presented as **inference with a confidence**, never as
   ground truth — a classification is a hypothesis a human can override.

## Rationale
Signal classification is a genuine ML win: modulation recognition from raw IQ is
a well-studied problem where learned models outperform hand-crafted feature
detectors, especially at low SNR and with unknown signals. Wideband detection
and segmentation are similarly a good fit — "what is present in this capture" is
tedious to hand-code and natural to learn.

Rust is well positioned here. In-process inference avoids the Python round trip
entirely, and ONNX Runtime is the mature path for models exported from the
PyTorch ecosystem, with `candle` available where a pure-Rust build matters more
than raw throughput. Requiring two backends (criterion 2) keeps the abstraction
honest and prevents the project being captive to one runtime's platform support.

Criteria 4 and 6 exist because ML features in radio software frequently ship as
unexplained black boxes that state confident answers. For a tool used to make
decisions about real signals — and potentially regulatory ones — an unsourced
model presenting guesses as facts is a defect, not a feature.

## Alternatives
- **Serve everything from a local HTTP model server.** Rejected for the signal
  path for the latency and serialization reasons above; adopted for the
  assistant path, where it is the better answer.
- **Python sidecar for inference.** Rejected: reintroduces the interpreter
  dependency and IPC that native inference exists to avoid, and complicates
  distribution of a cross-platform binary.
- **`burn` as the inference framework.** A strong Rust-native option that
  unifies training and inference. Not selected initially because this intent
  explicitly does not own training, which is where burn's main advantage lies.
  A reasonable third backend under criterion 2's trait.
- **Hand-written classifiers only.** Rejected as a ceiling, not a starting
  point — they remain the correctness baseline any model must beat, and that
  comparison is how criterion 1's accuracy claim stays meaningful.

## Consequences
- ONNX Runtime is a C++ dependency with platform-specific binaries, complicating
  cross-compilation; the `candle` backend is the mitigation and a reason
  criterion 2 requires two.
- Model files are large and unsuitable for git; distribution needs a separate
  fetch-and-verify path.
- Published accuracy figures are a standing obligation — they must be
  re-measured when a model or dataset changes, or they become misleading.
- Real-time inference competes with DSP for CPU/GPU; the budget in criterion 3
  must be measured under realistic concurrent load, not in isolation.
- An ML-assisted decode is evidence, not proof. Anything feeding
  [INT-0007](INT-0007-station-logging-cloudlog.md) logging or a compliance
  decision must record that it was model-derived.

## Transition history
- 2026-08-23: created as `proposed` (Sprint 8 roadmap).
