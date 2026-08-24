# INT-0012 — Radio Astronomy Suite

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0012
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none
- **Review evidence:** [Sprint 8 research report](../sprints/s8/sprint-research/research-report.md)

## Intent
Deliver a complete amateur radio astronomy suite: not a spectrum display pointed
at the sky, but the full observational chain — integrate, calibrate, record with
provenance, reduce, and image.

Scope, roughly in order of difficulty:
1. **Long-integration spectrometry.** Accumulate power spectra over minutes to
   hours with configurable resolution. This is the core instrument; the hydrogen
   line at **1420.405 MHz** is the reference target.
2. **Calibration.** Bandpass and gain calibration against a reference
   (hot/cold load, or on-source/off-source), producing output in physical units
   with a stated uncertainty.
3. **Drift scans.** Timestamped spectra tagged with sky coordinates derived from
   observer location and time, letting the Earth's rotation sweep the beam.
4. **Transient and periodic detection.** Meteor scatter detection; pulsar
   dedispersion and folding.
5. **Imaging.** Mapping from accumulated drift scans; interferometry between
   multiple receivers as the stretch goal.

**Non-goals initially:** VLBI-grade correlation and absolute flux calibration
against catalog standards. Both are real astronomy rather than an amateur suite,
and neither should be implied by anything shipped here.

### The constraint that governs this whole category

Radio astronomy is **integration-limited, not sensitivity-limited**: the signals
are far below the noise floor and only emerge from long coherent accumulation.
Everything in this intent follows from that. Numerical stability over billions of
accumulated samples matters more than throughput; timing and frequency stability
dominate the error budget; and any result must carry its uncertainty, because a
detection without an error bar is not a detection. An architecture that treats
astronomy as "a spectrum analyzer with a longer average" will produce confident
nonsense.

## Acceptance criteria
1. A long-integration spectrometer accumulates power spectra with configurable
   integration time and frequency resolution, and its noise floor falls as
   **√t** across at least two decades of integration time — the measurable proof
   that integration is actually coherent and numerically sound.
2. Bandpass and gain calibration against a reference produces spectra in
   physical units with a **stated uncertainty**, verified against a synthetic
   source of known strength.
3. Drift-scan mode records timestamped spectra tagged with sky coordinates
   computed from observer location and time, with the coordinate transform
   verified against an independent ephemeris.
4. Observations export to an interoperable format (**FITS**, alongside SigMF for
   raw IQ) that standard astronomy tooling reads without bespoke conversion.
5. A hydrogen-line detection is demonstrated with published setup, integration
   time, calibration and uncertainty — the end-to-end proof the chain works.
6. Accumulated drift scans render an integrated sky map.

## Rationale
This is the category with the largest gap between "an SDR can technically do it"
and "software exists that makes it practical". Amateur H-line work today is a
patchwork of GNU Radio flowgraphs, Python scripts and manual reduction. A suite
that carries an observation from capture through calibration to a publishable
map — with provenance and uncertainty intact — is genuinely missing.

It is also where a fast native language earns its keep: long integrations,
dedispersion and imaging are numerically heavy, and Rust plus GPU acceleration
is a strong fit.

The references already collected in the project README —
`UPennEoR/MiniRadioTelescope` and `achael/eht-imaging` — indicate this was
always intended to be a first-class category rather than a side feature, and
`cupy`/`cuda-oxide` indicate GPU acceleration was anticipated for it.

## Alternatives
- **Defer to GNU Radio flowgraphs plus Python reduction.** This is the status
  quo the intent exists to improve on. Rejected as an endpoint, but the tooling
  is a valuable correctness oracle: agreeing with an established pipeline is
  strong evidence, and criterion 4's FITS export makes that comparison possible.
- **Wrap existing astronomy libraries rather than implement.** Sound for
  coordinate transforms and FITS I/O, where correctness is subtle and standards
  are fixed — prefer a well-tested crate over hand-rolling. Not sound for the
  integration and calibration core, which is the actual contribution.
- **Treat astronomy as a plugin on the receiver application.** Rejected: the
  calibration, provenance and uncertainty requirements reach too deep into the
  capture path to bolt on afterwards.

## Consequences
- Numerical care is a hard requirement: naive `f32` accumulation will silently
  stall as the sum grows, so accumulator precision and strategy must be an
  explicit, tested decision (criterion 1 is designed to catch getting it wrong).
- Long integrations expose clock stability, which will surface hardware limits
  in [INT-0002](INT-0002-hardware-drivers-pluto.md) — a Pluto's internal
  oscillator may not suffice, implying external reference support.
- FITS and coordinate handling add dependencies in a domain where correctness is
  subtle; these should be borrowed, not invented.
- Imaging and 3D visualization are the most likely trigger for adopting Bevy
  under [INT-0010](INT-0010-desktop-gui-shell.md); that decision record must be
  revisited when this intent becomes active.
- Meaningful verification needs real sky data. Synthetic sources prove the
  arithmetic; only an actual observation proves the instrument.

## Transition history
- 2026-08-23: created as `proposed` (Sprint 8 roadmap).
