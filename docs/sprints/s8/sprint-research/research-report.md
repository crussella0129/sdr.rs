# Sprint 8 Research Report

## Intents Reviewed
- [INT-0009](../../../intents/INT-0009-receiver-application.md) — **created**; relevance: the "regular old radio monitoring/recording" category, at SDR++ parity; current state: `proposed`.
- [INT-0010](../../../intents/INT-0010-desktop-gui-shell.md) — **created**; relevance: the GUI front end and the egui-vs-Bevy decision this sprint was asked to settle; current state: `proposed`.
- [INT-0011](../../../intents/INT-0011-mesh-messaging-callsign.md) — **created**; relevance: chat/file transfer over the mesh with callsign identity; current state: `proposed`.
- [INT-0012](../../../intents/INT-0012-radio-astronomy-suite.md) — **created**; relevance: the radio astronomy suite including imaging; current state: `proposed`.
- [INT-0013](../../../intents/INT-0013-ml-signal-analysis.md) — **created**; relevance: ML decoding and signal analysis, and how to serve a model; current state: `proposed`.
- [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) — **selected**; relevance: INT-0011 builds directly on it; the work resumed after this sprint continues here; current state: `active`.
- [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) — **selected**; relevance: transport beneath INT-0011, and source of the T-113 ARQ prerequisite; current state: `realized`.
- [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) — **selected**; relevance: INT-0012's clock-stability needs will press on it; current state: `active`.

No existing chapter's desired outcome changed, so no revisions or state
transitions were made to previously realized intents.

## 1. Sprint Goal

Establish the trajectory of the whole project rather than advance a single
feature. Concretely: name the categories of functionality `sdr.rs` is eventually
meant to cover, give each a durable intent chapter so it is semantic authority
rather than a note in a chat log, and settle the one architectural question that
blocks several of them — whether the front end is built on `egui` or `Bevy`.
Also answer the open question of how ML models should be served. The sprint
produces Book structure and decisions; no feature code. Work then resumes on the
mesh where Sprint 7 left off.

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| `README.md` | high | A raw reference list, not a project description. Already collects `MiniRadioTelescope` and `eht-imaging` (astronomy/imaging) and `cupy`/`cuda-oxide` (GPU) — the astronomy category was anticipated from the start, not added now. |
| `docs/intents/` (8 chapters) | high | 6 `realized`, 2 `active`. Covers DSP, drivers, modulation, decoders, compliance, tunnel, logging, mesh. **No chapter covers an application or a GUI** — the gap this sprint fills. |
| `crates/sdr-cli/src/main.rs` | high | 728 LOC. The only entry point. Every capability is reachable solely as a subcommand; there is no session or application layer for a GUI to present. |
| `crates/sdr-core/src/traits.rs` | high | The `Result`/stream seam the GUI must drive; determines whether INT-0010 criterion 3 (no logic in widgets) is achievable without refactoring. |
| `crates/sdr-hardware/src/driver.rs` | high | `SdrDriver` — the abstraction all categories share. Multi-VFO (INT-0009) and long integration (INT-0012) both press on it. |
| `crates/sdr-spectrum/src/lib.rs` | high | 392 LOC — the smallest crate, and the direct input to the waterfall. FFT/CFAR exist; nothing renders. |
| `crates/sdr-dsp/` | high | 1170 LOC. FIR/SIMD, NCO, resamplers, Costas, Gardner. The primitives INT-0009's channelizer and INT-0012's integrator build on. |
| `crates/sdr-demod/` | medium | 983 LOC. WFM/NFM/AM/SSB/CW + OOK/FSK/PSK. INT-0009 composes these; INT-0013's classifier is the baseline-beating target. |
| `crates/sdr-mesh/src/{radio,stream,node}.rs` | high | 939 LOC. `RadioLink`, `StreamBridge`, `MeshInterface` — exactly the seam INT-0011 messaging sits on. |
| `crates/sdr-mesh/src/policy.rs` | high | The dual-mode compliance gate. INT-0011's authentication-vs-confidentiality distinction extends this, and must not weaken it. |
| `crates/sdr-protocols/src/packet.rs` | high | ARQ exists and passes at 30% simulated loss but is **not wired into the receive path** (T-113) — the hard prerequisite for INT-0011's reliable messaging. |
| `crates/sdr-hardware/src/sigmf.rs` | medium | Recording provenance for INT-0009 criterion 3 and INT-0012 criterion 4. |
| `crates/sdr-station/` | low | 304 LOC. Cloudlog/ADIF. INT-0013 criterion 6 constrains what may be logged from a model-derived decode. |
| `docs/work/tasks.md` | high | 13 backlog items. T-113 (ARQ) and T-107 (Phase B `tun`) are prerequisites for INT-0011; T-101 (multi-vendor) gates INT-0009's hardware breadth. |

**Total: 9 crates, ~8,680 LOC, 8 intents.** The decisive structural finding is
that the project is a well-tested library set with a CLI, and **nothing yet
composes it into an application** — which is why INT-0009 and INT-0010 are
separate intents and why INT-0009 must precede the GUI.

## 3. External Sources

- [SDR++ (AlexandreRouma/SDRPlusPlus)](https://github.com/AlexandreRouma/SDRPlusPlus) — the named parity bar for INT-0009. Its module list (20+ source drivers, sinks, decoders, frequency manager, rigcontrol server, recording) is the concrete audit target for criterion 4. **Decisively: it is built on Dear ImGui** — an immediate-mode GUI — which is the single strongest piece of evidence in the egui/Bevy decision.
- [egui (emilk/egui)](https://github.com/emilk/egui) — the Rust immediate-mode GUI, the same paradigm as Dear ImGui. Documents the supported ways to combine with 3D: a 3D library beneath egui, or rendering a scene to a texture and displaying it — both of which keep the visualization portable.
- [bevy_egui](https://docs.rs/bevy_egui/latest/bevy_egui/) — egui integration for Bevy. Establishes that adopting Bevy later can *host* existing egui panels rather than requiring a rewrite, which is what makes the deferral reversible.
- [ort (ONNX Runtime for Rust)](https://ort.pyke.io/) — mature in-process inference, production-proven (Bloop semantic search, Magika file-type detection, Wasmtime). The recommended primary backend for INT-0013's signal path; `candle` is the pure-Rust second implementation criterion 2 requires.
- [RTL-SDR — radio astronomy](https://www.rtl-sdr.com/rtl-sdr-for-budget-radio-astronomy/) — survey of amateur practice: hydrogen-line detection, meteor scatter, pulsar observation, and interferometry with spaced dishes. Confirms the capability ladder in INT-0012 and that current practice is a patchwork of flowgraphs and scripts rather than an integrated suite.

## 4. Risks, Unknowns, Dependencies

- **Risk — scope inflation is now the dominant project risk.** Five new intents
  take the Book from 8 chapters to 13, spanning receiver software, a GUI, mesh
  messaging, observational astronomy and ML. Any one is a project. Mitigated by
  keeping all five `proposed` (described, *not* accepted into executable work),
  so scheduling stays a deliberate per-sprint act rather than an implied
  commitment.
- **Risk — a GUI built on the current surface would trap logic in widgets.**
  There is no application layer today, so a front end started now would grow
  session and DSP logic inside callbacks. INT-0010 criterion 3 forbids it, which
  is precisely why INT-0009 should precede it.
- **Risk — astronomy failing silently.** Naive `f32` accumulation stalls as sums
  grow, producing plausible spectra that are wrong. INT-0012 criterion 1 (noise
  floor must fall as √t across two decades) exists specifically to make that
  failure detectable rather than invisible.
- **Risk — ML presenting guesses as facts.** Addressed by INT-0013 criteria 4
  and 6 (documented provenance; output as inference-with-confidence).
- **Unknown — frame rate on the target machines.** INT-0010 claims ≥30 fps at a
  stated sample rate and FFT size, unverified until a prototype exists.
  Headless CI cannot measure it, so criterion 1 needs an explicit measurement
  route.
- **Unknown — Pluto clock stability over long integrations.** INT-0012's
  integrations run minutes to hours; whether the internal oscillator suffices,
  or an external reference is required, is unmeasured and presses on INT-0002.
- **Unknown — ONNX Runtime cross-compilation burden.** A C++ dependency with
  platform-specific binaries; the size of that cost across Windows/Linux/macOS
  is untested, and is why a second pure-Rust backend is required rather than
  optional.
- **Dependency — T-113 (ARQ in the receive path) gates INT-0011.** Reliable
  messaging is unachievable while frames are dropped without retransmission.
  This is a prerequisite, not parallel work.
- **Dependency — T-107 (Phase B `tun`) gates the IP-native options** for
  INT-0011 and remains the unmet half of INT-0008 criterion 1.
- **Dependency — T-101 (multi-vendor drivers) bounds INT-0009's parity claim.**
  SDR++ supports 20+ sources; `sdr.rs` currently has Pluto and mock.

## 5. Recommended Approach

**Primary — the front end: `egui` as the shell, `wgpu` as the visualization
engine, Bevy deferred but not rejected.**

Three findings converge:

1. **SDR++, the explicitly named quality bar, is built on Dear ImGui.** `egui`
   is the Rust analogue of that exact paradigm. A receiver front end is a dense
   field of controls over state that changes every frame — the immediate-mode
   sweet spot, with no retained widget tree to synchronize against a live
   stream.
2. **Both egui and Bevy render through `wgpu`,** so the expensive artifact — the
   spectrum/waterfall render passes — is portable between them. This is a
   reversible decision, and INT-0010 criterion 4 requires that portability to be
   demonstrated rather than assumed.
3. **`bevy_egui` means later adoption hosts the existing panels** instead of
   forcing a rewrite. Deferring costs little; committing early pays for an ECS,
   scheduler and asset pipeline that a control surface does not use.

The concrete engineering constraint: the waterfall must render through an
`egui_wgpu` paint callback, **not** egui's shape painter, which cannot sustain
full-rate throughput. Criterion 2 exists to stop that obvious first attempt.

Bevy is revisited when INT-0012's imaging arrives with a real need for scene-graph
management, many-entity rendering or a 3D camera rig — named as explicit
triggers so the deferral stays a live judgment rather than inertia.

**Primary — ML serving: split by data rate, not by convenience.** In-process
inference (`ort` primary, `candle` as the pure-Rust second backend) for anything
in the sample path; an optional user-supplied OpenAI-compatible local endpoint
for the language-shaped work — explanation, summarization, report generation.
The localhost-server instinct in the original framing is correct; it is simply
correct for the *assistant*, not the decoder. Putting HTTP and serialization
inside a loop that runs at the sample rate is the error to avoid.

**Primary — sequencing.** Categories are chapters, not a schedule. All five are
created `proposed` so that scheduling remains an explicit act. The dependency
structure the survey exposes suggests a natural order — finish the mesh work in
flight (T-113 ARQ, then INT-0011 messaging, which it gates); build INT-0009's
application layer before INT-0010's GUI so logic has a home outside widgets;
begin INT-0012 and INT-0013 as independent spikes, since neither blocks the
others and both carry unmeasured unknowns better discovered early.

**Alternative considered — Bevy as the primary shell.** Rejected on the evidence
above, and recorded in INT-0010 with the conditions that would reverse it.

**Alternative considered — fewer, broader intents** (e.g. one "applications"
chapter). Rejected: the Book's contract is one chapter per distinct desired
outcome, and these five have genuinely different acceptance criteria,
dependencies and risks. Collapsing them would hide exactly the dependency
structure that makes the roadmap useful.

**Rationale.** The project has strong, well-tested internals and no application.
Naming the categories as durable intents converts a set of ambitions into
auditable acceptance criteria, and settling the front-end question unblocks the
largest piece of future work while keeping the costly decision reversible.

## Artifacts
- [INT-0009](../../../intents/INT-0009-receiver-application.md) — receiver application (created)
- [INT-0010](../../../intents/INT-0010-desktop-gui-shell.md) — GUI shell, with the egui/wgpu decision and Bevy triggers (created)
- [INT-0011](../../../intents/INT-0011-mesh-messaging-callsign.md) — mesh messaging and callsign identity (created)
- [INT-0012](../../../intents/INT-0012-radio-astronomy-suite.md) — radio astronomy suite (created)
- [INT-0013](../../../intents/INT-0013-ml-signal-analysis.md) — ML signal analysis (created)
- `docs/README.md` — Project Book updated with the durable category taxonomy
