# INT-0017 — Test and Measurement Instrumentation

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0017
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none
- **Review evidence:** [Sprint 8 research report](../sprints/s8/sprint-research/research-report.md)

## Intent
Use the radio as a bench instrument. An SDR with transmit is most of a signal
generator, a scalar network analyzer and a noise-figure meter already; what is
missing is the measurement discipline around it.

Scope:
1. **Signal generator.** Calibrated CW, swept, modulated and noise output at a
   commanded frequency and level.
2. **Scalar network analyzer.** Swept transmission and return-loss measurement
   for filters, antennas and cables — SWR, insertion loss, return loss.
3. **Noise-figure measurement.** Y-factor method using a noise source, for
   characterizing LNAs and receive chains.
4. **Power and spectrum measurement.** Channel power, occupied bandwidth,
   harmonic and spurious measurement, referenced to a calibration.
5. **Calibration.** Open/short/load and through references, with every result
   carrying its calibration state.

**Non-goals:** vector (phase-coherent) network analysis, which needs coherent
reference channels most SDRs do not provide; and any claim of laboratory-grade
absolute accuracy.

### The constraint that governs this category

**A measurement without a calibration and an uncertainty is not a measurement.**
Bench instruments are trusted precisely because they state their accuracy. An
SDR-based instrument that reports "SWR 1.4" with no calibration state and no
error bound invites a decision it cannot support. Every criterion below is
written to force the calibration and its uncertainty to travel with the number.

This category is also **transmit-gated**: it cannot be exercised meaningfully
without transmitting, so it inherits
[INT-0005](INT-0005-regulatory-band-compliance.md)'s compliance gate and the
standing constraint that on-air operation requires explicit authorization
(T-108). Measurements into a load or a shielded fixture are the normal mode; a
sweep radiating from an antenna is not.

## Acceptance criteria
1. The signal generator produces output at a commanded frequency and level, with
   level accuracy **measured against a reference** across a stated frequency
   range and published as an accuracy figure with its uncertainty.
2. A scalar sweep of a component with known characteristics (a calibration
   standard or a characterized filter) reproduces its response within a stated
   tolerance, after an open/short/load calibration.
3. Noise figure by the Y-factor method reproduces the known noise figure of a
   characterized amplifier within a stated uncertainty.
4. Every reported measurement carries its calibration state and uncertainty; an
   uncalibrated measurement is labelled as such and never presented as
   calibrated.
5. Every transmitting measurement passes the INT-0005 compliance gate before
   output is enabled, and the default configuration assumes a terminated
   fixture rather than an antenna.

## Rationale
This is the category that most directly repays the work already done on
transmit. The Pluto TX path, the compliance gate, the spectrum analyzer and the
DSP primitives all exist; what is missing is calibration and the measurement
framing around them.

It is also the smallest, best-bounded category on the list. Its acceptance
criteria are unusually crisp because measurement correctness is checkable
against known standards — a characterized filter either reproduces its response
or it does not, with no judgment involved.

The honesty requirements in criteria 1-4 are the point of the category rather
than caveats on it. SDR-based instrumentation is genuinely useful at the
hobbyist bench and genuinely misleading when it presents uncalibrated numbers
with instrument-like confidence.

## Alternatives
- **Support external instruments (NanoVNA, TinySA) rather than becoming one.**
  Sensible and complementary — those are inexpensive, purpose-built and already
  calibrated. Rejected as a *replacement* because the SDR is present anyway and
  the marginal capability is large; integration remains worth offering.
- **Vector network analysis.** Rejected as a non-goal: it requires phase-coherent
  reference channels beyond most supported hardware. Revisit only for devices
  that genuinely provide them.
- **Skip calibration and present relative measurements only.** Rejected —
  relative sweeps are useful, but shipping them without calibration state is how
  a plot gets read as an absolute measurement. Criterion 4 permits uncalibrated
  results only when labelled.

## Consequences
- Requires transmit, so this category cannot progress past simulation until the
  on-air constraint (T-108) is lifted by explicit authorization. Fixture-based
  measurement into a load is the path that does not require it.
- Calibration standards are user hardware, imposing a documented equipment
  requirement.
- Level accuracy depends on the specific device's transmit chain, so calibration
  is per-device and per-frequency and cannot be a single constant.
- Depends on **T-101** (multi-vendor hardware) for the accuracy claims to
  generalize beyond one device.
- Carrying calibration state and uncertainty through to presentation constrains
  [INT-0010](INT-0010-desktop-gui-shell.md): a measurement display must show
  its calibration state, not just a number.

## Transition history
- 2026-08-23: created as `proposed` (Sprint 8, T-039) — adopted from the
  candidate list.
