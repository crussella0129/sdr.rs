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

### F. Satellite and space operations
Pass prediction from orbital elements, Doppler correction across a pass,
weather-satellite imagery (APT/LRPT), telemetry, and rotator pointing. Shares
its observer/ephemeris layer with category C — that machinery should be built
once, not twice.
→ [INT-0014](intents/INT-0014-satellite-space-operations.md).

### G. Distributed sensing and direction finding
Locating a transmitter by combining observations from receivers in different
places: TDOA geolocation, plus single-site direction finding. Category B already
builds the networked sensor fabric this needs, which is what makes it
practical here. Accuracy is governed by **time synchronization**, not by radio
performance — 1 µs of clock error is ~300 m of position error.
→ [INT-0015](intents/INT-0015-distributed-sensing-df.md).

### H. Propagation monitoring and beacon reporting
Unattended weak-signal monitoring (WSPR, FT8/FT4), beacon watching, spot
reporting to the established aggregators, and a **local** propagation record
that remains useful with no network connection.
→ [INT-0016](intents/INT-0016-propagation-beacon-reporting.md).

### I. Test and measurement
The radio as a bench instrument: signal generator, scalar network analyzer,
noise-figure meter, power and spectrum measurement. Every result carries its
calibration state and uncertainty — an uncalibrated number presented with
instrument-like confidence is the failure mode this category exists to avoid.
Transmit-gated, so it inherits the compliance gate.
→ [INT-0017](intents/INT-0017-test-and-measurement.md).

### Cross-cutting
Hardware support ([INT-0002](intents/INT-0002-hardware-drivers-pluto.md)),
regulatory and band compliance
([INT-0005](intents/INT-0005-regulatory-band-compliance.md)) — which gates every
transmitting category — and station logging
([INT-0007](intents/INT-0007-station-logging-cloudlog.md)).

Sequencing and dependencies between all of these live in the
[roadmap](roadmap.md).
