# Roadmap

Ordering and dependencies for the whole project. The categories themselves live
in the [Project Book](README.md); the durable meaning of each lives in its
intent chapter.

## This is ordering, not commitment

A phase listing schedules nothing. **Intent state is what schedules work:** a
chapter at `proposed` is intended, not planned; it becomes `planned` when a
sprint takes it up and `active` when work begins. An intent can be reordered,
deferred or abandoned at any sprint boundary without this page being wrong,
because this page records a recommended sequence derived from dependencies — not
a promise, a schedule, or an estimate.

There are deliberately **no dates and no effort figures here.** None would be
evidence-backed, and inventing them would make this page misleading in exactly
the way the project's intent chapters are written to avoid.

## Where things stand

| | Count | Chapters |
|---|---|---|
| `realized` | 0 | — |
| `active` | 8 | INT-0001 … INT-0008 |
| `planned` | 1 | INT-0009 |
| `proposed` | 8 | INT-0010 … INT-0017 |

The suite today spans nine crates: DSP, drivers, demodulators, decoders,
spectrum analysis, compliance, packet radio, an SSH tunnel over radio, a mesh
transport, and station logging. Substantial bounded paths are tested, but the
Sprint 10 adversarial audit re-opened earlier terminal claims where safety,
numerical accuracy, negative paths, or application composition were not
actually evidenced. `sdr-station` is library-only, and the largest structural
gap remains that **nothing composes these internals into one receiver
application**, which is what sets the order below.

## Phases

### Phase 1 — Mesh reliability and messaging *(in flight)*
Finish what Sprint 7 started.

- **T-126 then T-117 — correct ARQ semantics, then enable them in the stream.**
  Sprint 9 added retransmission and duplicate suppression, but Sprint 10 found
  that reordered frames can be discarded and ACKed and that the tunnel still
  runs fire-and-forget. This is **required work before any real link**, not an
  optimization.
- **[INT-0011](intents/INT-0011-mesh-messaging-callsign.md)** — chat, file
  transfer and callsign identity, once T-126/T-117 make delivery reliable.
- **T-107** — the `tun` device, which completes the unmet half of
  [INT-0008](intents/INT-0008-mesh-networking-aredn.md) criterion 1.

### Phase 2 — Application layer
- **[INT-0009](intents/INT-0009-receiver-application.md)** — multi-VFO
  reception, frequency management and scanning, recording and playback. Parity
  with SDR++ as a floor.

This precedes the GUI deliberately. Without an application layer, front-end work
grows session and DSP logic inside widget callbacks — which INT-0010's own
acceptance criteria forbid.

### Phase 3 — Desktop GUI
- **[INT-0010](intents/INT-0010-desktop-gui-shell.md)** — `egui` shell with
  `wgpu` visualization; Bevy deferred against documented triggers.

### Phase 4 — Radio astronomy
- **[INT-0012](intents/INT-0012-radio-astronomy-suite.md)** — long-integration
  spectrometry, calibration, drift scans, imaging.

Worth spiking **early**, out of phase order: its two unknowns — accumulator
numerics and Pluto clock stability — are cheap to measure and expensive to
discover late. Both are the kind of problem that silently produces
plausible-looking wrong answers.

### Phase 5 — Machine learning
- **[INT-0013](intents/INT-0013-ml-signal-analysis.md)** — classification and
  ML-assisted decoding in-process; an optional local endpoint for the assistant.

### Later
- **[INT-0014](intents/INT-0014-satellite-space-operations.md)** — satellite and
  space operations.
- **[INT-0015](intents/INT-0015-distributed-sensing-df.md)** — distributed
  sensing and direction finding.
- **[INT-0016](intents/INT-0016-propagation-beacon-reporting.md)** — propagation
  and beacon reporting.
- **[INT-0017](intents/INT-0017-test-and-measurement.md)** — test and
  measurement.

Two of these are cheaper than their position suggests, and could move up if
priorities shift. INT-0016 reuses INT-0007's logging and HTTP infrastructure
almost directly and needs no transmit. INT-0015 is the one most nearly emergent
from what already exists, since the mesh is the sensor fabric it needs — its
blocker is time synchronization hardware, not software.

### Continuous
- **[INT-0002](intents/INT-0002-hardware-drivers-pluto.md)** — hardware breadth
  (**T-101**, multi-vendor). Several categories' claims only generalize once
  more than one device is supported.
- **[INT-0005](intents/INT-0005-regulatory-band-compliance.md)** — compliance,
  which gates every transmitting category and is active under **T-128** after
  the audit found bandwidth/duty-cycle and bypass gaps.

## Dependency map

What blocks what. "—" means nothing beyond the realized core.

| Intent | Blocked by | Why |
|---|---|---|
| [INT-0001](intents/INT-0001-core-dsp-pipeline.md) Core DSP | **T-119–T-121**, **T-136** | Restore memory safety, lossless/tagged streaming, measured resampling rejection, and fail-explicit DSP construction. |
| [INT-0002](intents/INT-0002-hardware-drivers-pluto.md) Hardware | **T-101**, **T-127**, **T-137** | Multi-vendor breadth plus truthful transport, lifecycle, capture integrity, and configuration-faithful mocks. |
| [INT-0003](intents/INT-0003-modulation-demodulation.md) Modulation | INT-0001, **T-104**, **T-122**, **T-136** | Stereo/RDS, sideband rejection, GFSK scaling, input guards, and measured BER remain. |
| [INT-0004](intents/INT-0004-protocol-decoders-spectrum.md) Decoders & spectrum | INT-0001, INT-0003, **T-123–T-125** | Decoder completeness and spectrum claims need independent vectors/measurement. |
| [INT-0005](intents/INT-0005-regulatory-band-compliance.md) Compliance | **T-128** | Bandwidth/duty-cycle inputs and fail-closed enforcement gate every transmitter. |
| [INT-0006](intents/INT-0006-packet-radio-ssh-tunnel.md) Packet radio & SSH | INT-0002, INT-0003, **T-126**, **T-117** | Correct reorder semantics first, then enable reliable tunnel transport. |
| [INT-0007](intents/INT-0007-station-logging-cloudlog.md) Station logging | **T-131** | Compose the tested client with receiver state and validate real success semantics. |
| [INT-0008](intents/INT-0008-mesh-networking-aredn.md) Mesh | **T-107**, **T-126**, **T-128**, **T-130** | TUN/Babel plus reliable, policy-enforced, streaming-safe radio transport. |
| [INT-0009](intents/INT-0009-receiver-application.md) Receiver app | INT-0002 (**T-101/T-127**), **T-133**, **T-129** | T-133 removes false-success export first; parity still needs hardware breadth and full record/demod/playback composition. |
| [INT-0010](intents/INT-0010-desktop-gui-shell.md) GUI | **INT-0009** | Needs an application layer, or logic lands in widget callbacks. |
| [INT-0011](intents/INT-0011-mesh-messaging-callsign.md) Messaging | **T-126**, **T-117**, INT-0008 | Reliable delivery needs correct reordering semantics and tunnel integration. |
| [INT-0012](intents/INT-0012-radio-astronomy-suite.md) Astronomy | **INT-0002** (clock stability) | Long integrations may exceed the internal oscillator; may need an external reference. |
| [INT-0013](intents/INT-0013-ml-signal-analysis.md) ML | INT-0004 | The hand-written decoders are the baseline a model must beat. |
| [INT-0014](intents/INT-0014-satellite-space-operations.md) Satellite | INT-0012 (shared ephemeris layer) | Observer/time/coordinate machinery is common to both; build it once. |
| [INT-0015](intents/INT-0015-distributed-sensing-df.md) Distributed sensing | **INT-0008**, node time sync, **T-101** | The mesh is the sensor fabric; timing accuracy *is* the measurement. |
| [INT-0016](intents/INT-0016-propagation-beacon-reporting.md) Propagation | INT-0007 | Reuses logging, ADIF and credential handling. |
| [INT-0017](intents/INT-0017-test-and-measurement.md) Test & measurement | transmit + **T-108**, **T-101** | Cannot be exercised without transmitting; on-air needs explicit authorization. |

### The load-bearing edges

Five dependencies determine most of the ordering:

- **INT-0011 ← T-126/T-117** — messaging cannot be reliable before ARQ is
  correct under reordering and enabled in the stream runtime.
- **INT-0010 ← INT-0009** — the GUI needs something to present.
- **INT-0012 ← INT-0002** — long integration may outrun the Pluto's clock.
- **INT-0015 ← INT-0008** — distributed sensing rides the mesh, and its accuracy
  is bounded by inter-node time synchronization.
- **INT-0017 ← T-108** — measurement means transmitting, which requires
  explicit authorization.

## Standing constraints

- **On-air operation requires explicit authorization** (**T-108**). Everything
  verified so far uses internal loopback with the transmitter at maximum
  attenuation. This gates INT-0017 entirely and parts of INT-0011 and INT-0014.
- **Claims must be measured, not asserted.** Several chapters carry figures that
  are still unverified and labelled as such — INT-0010's ≥30 fps waterfall,
  INT-0013's classifier accuracy and latency, INT-0017's level accuracy. They
  become evidence when a sprint measures them, and not before.
