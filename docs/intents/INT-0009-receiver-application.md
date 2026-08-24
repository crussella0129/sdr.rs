# INT-0009 — Receiver Application: Monitoring, Recording, and Playback

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0009
- **State:** planned
- **Work evidence:** [Sprint 10 build plan — T-133](../sprints/s10/sprint-plans/build-plan.md)
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** [Roadmap](../roadmap.md) — phase and dependency placement
- **Review evidence:** [Sprint 10 research report](../sprints/s10/sprint-research/research-report.md) — existing CLI/file-path composition audit; [Sprint 8 research report](../sprints/s8/sprint-research/research-report.md)

## Intent
Deliver the everyday receiver application — the "regular old radio monitoring
and recording" that most users will open `sdr.rs` to do — at feature parity with
SDR++, which is the explicit quality bar for this category.

Scope:
1. **Multi-VFO reception.** Several independent tuners demodulating concurrently
   from one capture stream, each with its own mode, bandwidth, squelch and sink.
2. **Frequency management.** Named bookmarks, band-plan metadata, import/export,
   and a scanner that steps or sweeps a range and stops on squelch break.
3. **Recording and playback.** Baseband IQ to SigMF and demodulated audio to
   WAV; a recorded session replays through the same DSP path that produced it.
4. **Audio routing.** Device output, file, and network sinks.

This intent owns the *application* layer only. The DSP primitives
([INT-0001](INT-0001-core-dsp-pipeline.md)), demodulators
([INT-0003](INT-0003-modulation-demodulation.md)), decoders and spectrum
([INT-0004](INT-0004-protocol-decoders-spectrum.md)) and drivers
([INT-0002](INT-0002-hardware-drivers-pluto.md)) already exist and are composed
here, not reimplemented.

**Non-goals:** transmit operation (INT-0002/INT-0006); the GUI itself
([INT-0010](INT-0010-desktop-gui-shell.md)), which presents this application but
is a separate desired outcome; and protocol decoders beyond INT-0004's set.

"Parity with SDR++" is a floor, not a ceiling. Where a capability is cheap in
Rust and absent from SDR++ — reproducible SigMF provenance, scriptable headless
capture, compliance-aware tuning via
[INT-0005](INT-0005-regulatory-band-compliance.md) — it belongs here.

## Acceptance criteria
1. Two or more VFOs demodulate concurrently from a single capture stream, each
   with independent mode, bandwidth, squelch and audio sink, verified by a test
   that recovers distinct known signals from one synthetic wideband capture.
2. A frequency manager persists named bookmarks with band-plan metadata across
   restarts, and a scanner sweeps a configured range and halts on squelch break.
3. Baseband IQ records to SigMF and demodulated audio to WAV; replaying a
   recorded session through the same DSP path reproduces the original
   demodulated output within a stated tolerance.
4. A published per-feature parity audit against SDR++'s module list states, for
   every feature, whether `sdr.rs` has it, lacks it, or deliberately declines it
   with a reason.

## Rationale
Everything built so far is reachable only through `sdr-cli` subcommands. The
DSP, drivers, decoders and spectrum analysis are real and tested, but there is
no application that composes them into the thing a person sits down and uses.
This category is the widest audience and the most direct measure of whether the
suite is worth adopting; it is also the honest prerequisite for the GUI, which
otherwise would have no coherent application layer to present.

SDR++ is named as the bar because the user identified it as a great app and
because its module list is a concrete, auditable parity target rather than a
vague aspiration.

## Alternatives
- **GNU Radio-style flowgraph runtime as the application layer.** Rejected as
  the primary target: maximum flexibility, but it makes the common case (open
  the app, tune, listen, record) far harder than a purpose-built receiver.
  Revisit as a separate intent if scripted signal-processing graphs are wanted.
- **Ship the GUI first and let the application emerge behind it.** Rejected —
  it produces logic trapped in widget callbacks, which is precisely what
  INT-0010 criterion 3 forbids.

## Consequences
- Multi-VFO forces a decision about channelization strategy (per-VFO decimation
  chains vs a polyphase channelizer); the latter scales better and is the likely
  requirement once VFO counts grow.
- Recording provenance implies stable SigMF metadata conventions, which become a
  compatibility surface once users have archives.
- A published parity audit is a standing obligation: it must be revisited when
  SDR++ adds modules, or it becomes stale and misleading.

## Transition history
- 2026-08-23: created as `proposed` (Sprint 8 roadmap).
- 2026-08-24: revised after Sprint 10's audit (remains `proposed`). The existing `sdr-cli demod --output` path is explicitly not evidence for criterion 3: it prints an export message without creating a file, truncates SigMF input to 100,000 samples, and has no playback command. Those are starting gaps for this intent, not realized receiver functionality.
- 2026-08-24: moved to `planned` for Sprint 10's bounded T-133 receiver-output slice. This plans real mono-WAV audio export plus explicit rejection of unsupported FSK file output and unknown modes; multi-VFO, scanning, full-capture streaming, and replay remain T-129 and the intent cannot be realized by this slice.
