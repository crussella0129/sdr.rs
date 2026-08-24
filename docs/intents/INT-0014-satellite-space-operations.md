# INT-0014 — Satellite and Space Operations

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0014
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none
- **Review evidence:** [Sprint 8 research report](../sprints/s8/sprint-research/research-report.md)

## Intent
Work satellites end to end: know when one is overhead, follow it in frequency as
it moves, and turn what comes down into something useful.

Scope:
1. **Pass prediction.** Orbit propagation from TLE/OMM element sets, producing
   AOS/LOS times, azimuth/elevation tracks and a pass schedule for an observer.
2. **Doppler correction.** Continuous tuning correction across a pass, so a
   narrowband downlink stays demodulable from horizon to horizon.
3. **Weather-satellite imagery.** APT (NOAA) and LRPT (Meteor-M) decode to
   images, with the geometry corrections that make them usable.
4. **Telemetry.** Beacon and telemetry decode for amateur satellites and
   cubesats.
5. **Rotator control.** Az/el pointing output, so the same track that predicts
   a pass can drive an antenna.

**Non-goals:** transmit — satellite uplink and full-duplex operation are gated
on the same on-air constraint as everything else (T-108) and are not claimed
here; and commercial/proprietary downlink formats.

This category shares real machinery with
[INT-0012](INT-0012-radio-astronomy-suite.md): both need an observer location,
time, a coordinate transform and a pointing solution. That overlap should be
built once, in one place, rather than twice — the shared component is the
observer/ephemeris layer, and whichever intent lands first owns it.

## Acceptance criteria
1. Pass prediction from a TLE/OMM set produces AOS/LOS and an az/el track whose
   agreement with an independent, established propagator is quantified and
   published — not asserted.
2. Doppler correction tracks a full pass such that a narrowband downlink remains
   demodulable from AOS to LOS, verified on a synthetic pass with a known
   Doppler profile and on at least one recorded real pass.
3. An APT or LRPT downlink decodes to a recognizable image from a recorded IQ
   capture, so the result is reproducible without waiting for a satellite.
4. Az/el output drives a rotator through an existing control protocol
   (Hamlib `rotctl`), verified against a simulator.
5. Predictions and decoded products carry the element set, epoch and observer
   location used, so a result can be reproduced or shown to be stale.

## Rationale
Satellite work is one of the most common reasons people buy an SDR, and it is
the category where the existing pieces of this suite come closest to already
sufficing: the demodulators, recording and Hamlib integration all exist. What is
missing is the orbital layer — prediction, Doppler, pointing — which is
self-contained and well-specified.

It also earns its place by shared infrastructure. The observer/time/coordinate
machinery it needs is the same INT-0012 needs for drift scans, and building it
once for both is cheaper than either alone.

Criterion 5 exists because TLEs go stale quickly; a prediction without its epoch
is unreproducible and quietly wrong within days.

## Alternatives
- **Defer to Gpredict/SatNOGS for prediction and drive `sdr.rs` from them.** A
  legitimate and probably correct *first* integration — they are mature and
  well-tested. Rejected as the endpoint because Doppler correction wants to sit
  inside the receive chain, not be bolted on over a control socket. Interop
  remains valuable and should be offered either way.
- **Implement orbit propagation from scratch.** Rejected: SGP4 is a precise,
  standardized model where a well-tested implementation should be borrowed, not
  reinvented. Criterion 1's comparison against an established propagator exists
  to keep whatever is used honest.
- **Treat this as a plugin on INT-0009.** Partly true — it will present through
  the receiver application — but the orbital layer is substantial enough, and
  shared with INT-0012, to warrant its own chapter.

## Consequences
- TLE/OMM ingestion means a network fetch and a staleness policy; element sets
  must be cached with their epoch and flagged when old.
- Doppler correction inside the receive chain constrains the tuning API: it must
  accept continuous frequency updates without gaps or clicks.
- The observer/ephemeris layer is shared with INT-0012, creating a real
  dependency between two otherwise independent categories. Whichever is built
  first should expose it deliberately rather than burying it.
- LRPT decoding needs error correction (Viterbi/Reed-Solomon) not currently in
  the suite.

## Transition history
- 2026-08-23: created as `proposed` (Sprint 8, T-039) — adopted from the
  candidate list.
