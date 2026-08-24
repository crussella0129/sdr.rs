# Project Book

This directory is the canonical Sprint Loops Book: project intent, executable
work, realization evidence, and sprint provenance live here together.

## What `sdr.rs` is meant to become

A comprehensive, cross-platform SDR suite in Rust. "Comprehensive" is defined by
the categories below — each one a distinct area of functionality with its own
intent chapter, acceptance criteria and risks. They are deliberately broad; the
list is expected to grow.

A category being listed here means it is **intended**, not scheduled. Intent
chapters carry the state (`proposed`, `planned`, `active`, `realized`), and
scheduling is an explicit per-sprint act. This section is the map, not the plan.

### A. Radio monitoring, recording and playback
The everyday receiver — multi-VFO reception, frequency management and scanning,
IQ and audio recording with provenance, and playback through the same DSP path.
The quality bar is **feature parity with SDR++**, treated as a floor rather than
a ceiling.
→ [INT-0009](intents/INT-0009-receiver-application.md), building on
[INT-0001](intents/INT-0001-core-dsp-pipeline.md),
[INT-0003](intents/INT-0003-modulation-demodulation.md),
[INT-0004](intents/INT-0004-protocol-decoders-spectrum.md).

### B. Networking over radio — mesh, SSH, and messaging
IP-over-radio meshing with AREDN/Babel interoperability, SSH tunnelling, and on
top of them a messaging service in the shape people already know: channels and
direct messages, presence, and resumable file transfer — addressed by
**callsign**, with sender authentication available even on bands where
confidentiality is prohibited.
→ [INT-0008](intents/INT-0008-mesh-networking-aredn.md),
[INT-0006](intents/INT-0006-packet-radio-ssh-tunnel.md),
[INT-0011](intents/INT-0011-mesh-messaging-callsign.md).

### C. Radio astronomy
A complete observational chain rather than a spectrum display pointed at the
sky: long-integration spectrometry, calibration in physical units with stated
uncertainty, drift scans with sky coordinates, transient and pulsar work, and
imaging. The hydrogen line at 1420.405 MHz is the reference target.
→ [INT-0012](intents/INT-0012-radio-astronomy-suite.md).

### D. ML-assisted decoding and signal analysis
Models where they beat hand-written DSP — modulation classification, wideband
detection and segmentation, decoding waveforms that resist analytic treatment —
plus an optional assistant for the language-shaped work. Split by data rate:
**in-process inference in the sample path, an optional local endpoint for the
assistant.**
→ [INT-0013](intents/INT-0013-ml-signal-analysis.md).

### E. Desktop application and visualization
A native cross-platform front end: `egui` for the shell, `wgpu` for
high-throughput spectrum and waterfall rendering, with Bevy deferred — not
rejected — against documented triggers, most likely arising from category C's
imaging work. Both render through `wgpu`, so the visualization code is portable
between them.
→ [INT-0010](intents/INT-0010-desktop-gui-shell.md).

### Cross-cutting
Hardware support ([INT-0002](intents/INT-0002-hardware-drivers-pluto.md)),
regulatory and band compliance
([INT-0005](intents/INT-0005-regulatory-band-compliance.md)) — which gates every
transmitting category — and station logging
([INT-0007](intents/INT-0007-station-logging-cloudlog.md)).

### Candidate categories, not yet adopted
Recorded so they are not lost, and so adding one is a deliberate decision:

- **Satellite and space operations** — pass prediction, Doppler correction,
  weather-satellite imagery (APT/LRPT), telemetry. Shares tracking, pointing and
  Doppler machinery with category C.
- **Distributed sensing and direction finding** — TDOA geolocation across
  multiple receivers. Notable because category B already builds the networked
  sensor fabric this needs; the capability is close to emergent.
- **Propagation and beacon reporting** — WSPR/RBN-style monitoring and
  reporting, extending INT-0007.
- **Test and measurement** — using the radio as a signal generator, scalar
  network analyzer or noise-figure meter.
