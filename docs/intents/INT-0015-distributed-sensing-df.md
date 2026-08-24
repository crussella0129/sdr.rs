# INT-0015 — Distributed Sensing and Direction Finding

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0015
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** [Roadmap](../roadmap.md) — phase and dependency placement
- **Review evidence:** [Sprint 8 research report](../sprints/s8/sprint-research/research-report.md)

## Intent
Turn several `sdr.rs` stations into one instrument: locate a transmitter by
combining observations from receivers in different places.

Scope:
1. **Time-difference-of-arrival (TDOA) geolocation.** Three or more receivers
   capture the same signal; the differences in arrival time produce hyperbolic
   position lines whose intersection locates the emitter.
2. **Time synchronization across nodes.** The measurement's entire accuracy
   rests on this — see below.
3. **Distributed capture coordination.** Starting a synchronized capture across
   nodes, collecting the results, and correlating them.
4. **Presentation.** Position estimates with an **uncertainty region**, never a
   bare point.
5. **Single-site direction finding** (pseudo-Doppler or phase interferometry) as
   the simpler, non-distributed sibling.

**Non-goals:** covert or unattended surveillance tooling, and tracking
individuals. This exists for the legitimate uses direction finding has always
served — interference hunting, fox hunting, beacon and emitter
characterization — and the intent says so deliberately rather than leaving the
purpose unstated.

### The constraint that governs this category

**TDOA is a timing measurement, not a radio measurement.** Radio waves travel
about 300 m per microsecond, so a 1 µs synchronization error is a ~300 m
position error — and clock error enters the result directly, not as noise that
averages away. Everything here follows from that:

- Node clock discipline (GPS/PPS, or a shared reference) is the *primary*
  engineering problem; the DSP is comparatively easy.
- The achievable accuracy must be **derived from the measured timing
  uncertainty**, not asserted from a specification sheet.
- A position estimate without an uncertainty region is not a result. Criterion 4
  exists to prevent shipping confident dots on a map.

## Acceptance criteria
1. Correlating captures from three or more synthetic receivers with known
   positions and a known emitter recovers that emitter's location within an
   error bound derived from the simulated timing uncertainty.
2. Node time synchronization is **measured** — the actual inter-node timing
   uncertainty is reported with its method (GPS/PPS discipline or equivalent) —
   and the resulting position uncertainty follows from that measurement rather
   than from a datasheet claim.
3. A synchronized capture is coordinated across three or more real nodes over
   the mesh ([INT-0008](INT-0008-mesh-networking-aredn.md)), with the captures
   collected and correlated end to end.
4. Every position estimate is presented with an uncertainty region; no interface
   reports a position without one.
5. Single-site direction finding produces a bearing with a stated angular
   uncertainty, verified against a signal from a known direction.

## Rationale
This is the category closest to emerging on its own. The suite already has
multi-node networking ([INT-0008](INT-0008-mesh-networking-aredn.md)),
timestamped IQ recording with SigMF provenance, and DSP primitives including
correlation. What is missing is time discipline and the correlation pipeline —
and the mesh that exists for messaging is precisely the fabric a distributed
sensor array needs. Few SDR suites can offer this because few have the
networking layer already built.

The capability is genuinely useful: interference hunting is a real, frequent
problem for which amateur and professional operators currently improvise.

Criterion 2 is written as a measurement rather than a target because the
temptation here is to quote a GPS module's specification and present derived
positions as though that specification were achieved end to end.

## Alternatives
- **Single-site direction finding only** (pseudo-Doppler, phased arrays).
  Simpler, no synchronization problem, no networking — and included here as
  criterion 5 rather than dismissed. Rejected as the whole scope because it
  yields a bearing, not a position, and does not use the distributed capability
  that makes this project well placed.
- **Power-difference (RSSI) trilateration.** Far easier — no timing
  requirement — but propagation-dependent and unreliable in the cluttered
  environments where interference hunting actually happens. Reasonable as a
  coarse first estimate to seed TDOA, not as the method.
- **Angle-of-arrival with phased arrays at each node.** Powerful and
  complementary, but multiplies per-node hardware cost. Revisit after TDOA.
- **KerberosSDR/KrakenSDR-style coherent multi-channel receivers.** A different
  architecture (coherent channels in one box) solving a related problem. Worth
  supporting as *hardware* under
  [INT-0002](INT-0002-hardware-drivers-pluto.md); does not replace the
  distributed case.

## Consequences
- A hard dependency on time synchronization hardware (GPS/PPS or a common
  reference). This is a real cost imposed on the user and must be stated in
  documentation, not discovered.
- Correlation requires moving substantial IQ between nodes — a poor fit for the
  radio mesh's own bandwidth, so the coordination path and the data path may
  need to differ (radio for control, conventional network for bulk capture).
- Depends on **T-101** (multi-vendor hardware): a distributed array of one
  supported device type is an unrealistic deployment.
- Uncertainty reporting is a presentation constraint that reaches into
  [INT-0010](INT-0010-desktop-gui-shell.md) — the map view must render regions,
  not pins.
- The non-goals above are a standing constraint on how this is documented and
  presented, not a one-time disclaimer.

## Transition history
- 2026-08-23: created as `proposed` (Sprint 8, T-039) — adopted from the
  candidate list; identified in research as the strongest candidate because the
  mesh already provides the networked sensor fabric it requires.
