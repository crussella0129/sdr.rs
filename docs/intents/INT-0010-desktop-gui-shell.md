# INT-0010 — Desktop GUI Shell and Real-Time Visualization

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0010
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** [Roadmap](../roadmap.md) — phase and dependency placement
- **Review evidence:** [Sprint 8 research report](../sprints/s8/sprint-research/research-report.md)

## Intent
Deliver a native, cross-platform graphical front end for `sdr.rs` that is fast
and pleasant enough to be the primary way people use the suite, and whose
visualization layer is strong enough to carry the demanding cases — full-rate
waterfalls now, and eventually the 2D/3D rendering radio astronomy
([INT-0012](INT-0012-radio-astronomy-suite.md)) will want.

**Chosen direction: `egui` as the application shell, `wgpu` as the
visualization engine, `Bevy` deferred but not rejected.**

The decisive structural point is that **egui and Bevy both render through
`wgpu`.** Visualization written as `wgpu` render passes is therefore portable
between them, which makes this a reversible decision rather than a one-way door.
The intent requires that portability to be real, not incidental: the renderer
sits behind a trait, and no shader or GPU buffer logic may assume its host.

Scope:
1. An `egui` shell (control panels, menus, docking, settings) driving the same
   core APIs as `sdr-cli`.
2. A `wgpu`-backed spectrum and waterfall renderer, uploaded as textures through
   an `egui_wgpu` paint callback — **not** egui's shape painter, which cannot
   sustain full-rate waterfall throughput.
3. A documented decision record stating the specific conditions under which
   Bevy would be adopted, so the deferral is a standing judgment rather than
   inertia.

**Non-goals:** a web/WASM build (egui supports it, but hardware access does
not); mobile; and the receiver application logic itself
([INT-0009](INT-0009-receiver-application.md)), which this presents.

## Acceptance criteria
1. A native shell builds and runs on Windows, Linux and macOS, presenting a live
   spectrum and waterfall at **≥30 fps** at a stated sample rate and FFT size,
   with the frame rate measured and published rather than asserted.
2. The waterfall renders through a `wgpu` render pass invoked from an
   `egui_wgpu` callback; a test or benchmark demonstrates it sustains the stated
   rate where egui shape painting does not.
3. Every control action (tune, mode, gain, record, scan) invokes the same public
   core API the CLI uses. No signal-processing or session logic lives in widget
   callbacks — verified by the GUI crate depending only on public core APIs.
4. The renderer is reachable behind a trait with no `egui` types in its
   interface, demonstrated by rendering the same visualization through a second
   host (a headless `wgpu` target is sufficient).
5. A decision record names the concrete triggers for adopting Bevy and is
   revisited whenever INT-0012 advances.

## Rationale
Three pieces of evidence point the same way.

**SDR++ — the app named as the quality bar for
[INT-0009](INT-0009-receiver-application.md) — is built on Dear ImGui**, an
immediate-mode GUI. `egui` is the Rust analogue of that exact paradigm. A
receiver front end is a dense field of controls over rapidly-changing state,
which is the immediate-mode sweet spot: no retained widget tree to synchronize
against a stream that changes every frame.

**Bevy's strengths are mostly overhead here.** Its ECS, scheduler and asset
pipeline earn their cost when managing many entities in a 3D scene. A control
surface with a couple of large GPU-backed visualizations is not that. Adopting
an engine for a capability (INT-0012 imaging) that has not been designed yet
would be paying its cost years early.

**The expensive artifact survives either choice.** The hard, valuable work is
the `wgpu` visualization code, and it is portable. `bevy_egui` further means
that if astronomy later needs a real 3D scene graph, Bevy can host the existing
egui panels rather than forcing a rewrite. Deferring costs little; committing
early costs a lot.

This staging — ship on the pragmatic stack, optimize per-platform once the
surface is mature — matches the approach already adopted for cross-platform UI
work generally.

## Alternatives
- **Bevy as the primary shell now.** Rejected for the reasons above. Would be
  reconsidered if INT-0012's imaging turns out to need scene-graph management,
  many-entity simulation, or a real 3D camera rig — the documented triggers.
- **Tauri / web front end.** Rejected: the visualization is GPU- and
  throughput-bound, and a JS bridge sits directly in the hot path. The usual
  argument for a web stack (fast iteration on a familiar toolkit) does not
  outweigh that here, unlike in ordinary application UI.
- **egui's built-in painter for the waterfall.** Rejected on throughput
  grounds; recorded because it is the obvious first attempt and criterion 2
  exists to prevent it being taken.
- **Native per-platform toolkits.** Rejected: triples the surface for a
  single-developer project.

## Consequences
- A `wgpu` dependency and a GPU-capable test environment; headless CI cannot
  verify frame rate, so criterion 1 needs an explicit measurement route.
- Immediate mode redraws continuously, which costs power. Idle-state throttling
  becomes a real requirement, not a nicety.
- Keeping the GUI free of logic (criterion 3) constrains the core APIs to stay
  genuinely public and usable — a discipline that also benefits the CLI.
- Deferring Bevy means accepting a possible future migration cost; criterion 4
  exists to keep that cost bounded.

## Transition history
- 2026-08-23: created as `proposed` (Sprint 8 roadmap). Direction chosen —
  `egui` + `wgpu`, Bevy deferred — on the evidence recorded under Rationale.
