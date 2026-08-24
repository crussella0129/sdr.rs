Finalized - DO NOT EDIT

# Sprint 10 Build Plan

## Sprint Goal

Convert Sprint 10's whole-corpus audit into a bounded correctness recovery:
remove the core memory-unsafety and silent stream loss, make the advertised
stop-and-wait link structurally and transactionally correct, make transmit
policy fail closed, replace the visible demod-export stub with real output, and
restore a deterministic full-workspace verification baseline.

This is not a claim that every audited gap fits one sprint. Every confirmed
stub or incomplete path is owned by a durable Book task (T-119 through T-137,
plus retained earlier tasks); the explicit deferral table below keeps the
unselected work visible.

## Intents

- [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) — state: `active`; acceptance criteria covered: #1 bounded lock-free buffering/lossless composition and #5 one-to-one tag propagation.
- [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) — state: `active`; acceptance criterion covered: #1 truthful TX capability behavior for backends that do not implement transmission.
- [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md) — state: `active`; acceptance criterion covered: #4 deterministic verification of the already-functional Rigctl TCP server.
- [INT-0005](../../../intents/INT-0005-regulatory-band-compliance.md) — state: `active`; acceptance criteria covered: #2 full transmission-plan inputs and #3 invalid-jurisdiction refusal.
- [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) — state: `active`; acceptance criteria covered: #2 stop-and-wait ordering/source semantics and #4 exact-write/policy refusal on the radio path.
- [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) — state: `active`; acceptance criteria covered: #1 non-corrupt transactional radio delivery and #3 a non-bypassable application TX gate.
- [INT-0009](../../../intents/INT-0009-receiver-application.md) — state: `planned`; acceptance criterion covered: the demodulated-audio-to-WAV slice of #3 plus explicit rejection of unsupported FSK file export. Playback, provenance replay, multi-VFO, scanning, and application composition remain open.

## Schema Tree

- Audit-driven correctness recovery
  - Core stream safety
    - T-119: proven bounded lock-free ring storage
    - T-120: lossless partial-progress and honest tag propagation
  - Link correctness
    - T-135: unsupported HAL transmitter operations fail explicitly
    - T-126: one-slot, source-validated, transactional stop-and-wait ARQ
  - Transmit safety
    - T-128: typed full-plan compliance and fail-closed application TX
  - Truthful application surfaces
    - T-133: real demod output or explicit error
    - T-134: race-free Rigctl process verification
  - Attainable verification
    - T-106: normalize the five known rustfmt hunks

## Design Decisions

### Proven queue rather than handwritten unsafe storage

T-119 wraps `crossbeam_queue::ArrayQueue<T>` (current compatible release
`0.3.13`) behind the existing
`RingBuffer<T>` API. The current API does not enforce one producer and one
consumer, so safety cannot rest on that undocumented runtime promise. The
proven queue remains bounded and lock-free even if callers exceed SPSC use.
The observable usable-capacity rule remains `next_power_of_two(max(2, n)) - 1`.
`sdr-core` will forbid unsafe code so this defect cannot silently return there.

### Preserve work; reject invented tag positions

T-120 keeps the public `Block::process` and `LinearPipeline::step` signatures.
The runner retains unconsumed input and unwritten output, validates every
reported count, and returns zero only after terminal EOF and one successful
flush. `Source::get_tags` is clarified to return absolute input-stream offsets.
A default `Block::map_tags` keeps offsets exact for one-to-one blocks; a
rate-changing block with tags must override mapping or return an explicit error.
Guessing an offset would be another form of silent corruption.

### Genuine stop-and-wait with an I/O commit boundary

T-126 chooses one outstanding frame rather than adding a receive window. This
matches the existing intent, half-duplex model, and seeded loss harness. Initial
sends and retries use prepare/commit operations: sequence, timeout, retry, ACK,
and abandonment state changes only after the driver accepts the complete IQ
frame. A future sequence is neither delivered nor ACKed; only the last delivered
sequence is treated as an ACK-worthy duplicate. Header source validation is
peer pinning, not cryptographic authentication.

### One complete transmission plan, evaluated before and during TX

T-128 replaces the positional compliance query with `TransmissionPlan`:
jurisdiction; center frequency in Hz (finite, `> 0`); occupied bandwidth in Hz
(finite, `> 0`); EIRP in dBm (any finite value, including zero or negative);
duty cycle in percent (finite, `0 < value <= 100`); and encryption status. The entire
occupied span must fit the matched catalog band, and encoded bandwidth/duty
limits become refusals rather than prose warnings. The tunnel derives occupied
bandwidth with the binary-FSK Carson estimate
`2 * deviation_hz + sample_rate / samples_per_symbol`, assumes 100% duty cycle
because no limiter exists, requires the operator to state EIRP, and uses a
policy-checked interface so refusal cannot fall through to driver startup or
later sends.

This changes code mechanics only. No catalog row, citation, or present-day law
is updated or certified in this sprint; `None` means only “this offline catalog
encodes no limit,” not that no legal constraint exists.

## Execution Sequence

### T-119: Replace the unsound ring buffer with proven bounded lock-free storage

- **Intent:** [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md)
- **Touches:** `Cargo.toml`, `crates/sdr-core/Cargo.toml`, `crates/sdr-core/src/buffer.rs`, `crates/sdr-core/src/lib.rs`, `Cargo.lock`
- **Depends on:** (none)
- **Acceptance criterion:** INT-0001 #1, limited to memory-safe bounded lock-free buffering; zero-copy and throughput claims are not advanced.
- **Implementation:** preserve the type, constructor, shared alias, method signatures, FIFO behavior, and usable-capacity contract over `ArrayQueue<T>`; remove raw-pointer mutation/manual unsafe trait impls; document concurrent availability values as snapshots and `clear` as requiring producer quiescence; add `#![forbid(unsafe_code)]` to `sdr-core`.
- **Success criterion (EARS):**
  - **WHEN** one producer and one consumer move more than ten capacities through concurrent wraparound, **THEN** `RingBuffer` **SHALL** deliver every accepted value exactly once and in FIFO order.
  - **WHEN** a batch is larger than available capacity or a read is larger than queued data, **THEN** `RingBuffer` **SHALL** return the exact partial count without overwriting or inventing values.
  - **WHEN** a quiescent buffer is cleared and reused, **THEN** it **SHALL** be empty and preserve its capacity and FIFO behavior.
  - **WHEN** `RingBuffer<T>` is shared for a `T: Send`, **THEN** its public type **SHALL** satisfy the compile-time `Send + Sync` contract without local unsafe code.
- **Tests:** `test_ring_buffer_concurrent_spsc_preserves_order_without_loss`, `test_ring_buffer_capacity_and_partial_io`, `test_ring_buffer_clear_is_safe_and_reusable`, `test_ring_buffer_send_sync_for_send_samples`; retain the existing FIFO/wraparound regressions.

### T-120: Make `LinearPipeline` lossless under partial progress and honest about tags

- **Intent:** [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md)
- **Touches:** `crates/sdr-core/src/traits.rs`
- **Depends on:** T-119
- **Acceptance criterion:** INT-0001 #1 composable lossless execution and #5 exact tag propagation for mappings a block can define.
- **Implementation:** retain pending input/output and absolute input/output cursors; never read another source chunk while old input/output remains; validate `consumed`, `produced`, and sink acceptance; retain unwritten output after an error; flush exactly once at EOF. Add a defaulted `Block::map_tags` that maps one-to-one only and requires rate-changing blocks to opt into an exact mapping.
- **Success criterion (EARS):**
  - **WHEN** a block consumes only part of a source chunk, **THEN** `LinearPipeline` **SHALL** retain and process the remainder exactly once before reading the source again.
  - **WHEN** a sink accepts a positive partial write or returns an error after partial progress, **THEN** `LinearPipeline` **SHALL** retain the unwritten suffix and deliver it in order without re-running the block.
  - **WHEN** a block or sink reports an impossible count or zero progress on pending data, **THEN** `LinearPipeline` **SHALL** return an explicit contract error without discarding pending samples.
  - **WHEN** source tags cross a one-to-one block under partial consumption, **THEN** the sink **SHALL** receive each tag exactly once at the same absolute output offset; a rate-changing block without an explicit mapper **SHALL** error rather than guess.
  - **WHEN** the source reaches EOF after all pending work drains, **THEN** `LinearPipeline` **SHALL** flush the sink exactly once and only then return zero.
- **Tests:** `test_linear_pipeline_partial_block_consumption_preserves_remainder`, `test_linear_pipeline_does_not_read_next_chunk_while_input_pending`, `test_linear_pipeline_partial_sink_writes_are_retried_in_order`, `test_linear_pipeline_sink_error_retains_unwritten_output`, `test_linear_pipeline_invalid_progress_returns_error`, `test_linear_pipeline_source_tags_forwarded_exactly_once`, `test_linear_pipeline_rate_changing_block_requires_tag_mapper`, `test_linear_pipeline_flushes_sink_once_at_eof`; retain the buffer-reuse and normal multi-step regressions.

### T-135: Make unsupported `SdrDriver` transmitter operations fail explicitly

- **Intent:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- **Touches:** `crates/sdr-hardware/src/driver.rs`
- **Depends on:** (none)
- **Acceptance criterion:** INT-0002 #1's truthful TX abstraction; this does not add a new hardware backend or repair Pluto lifecycle state.
- **Implementation:** keep `has_tx() == false` as the capability signal, but make the default `start_tx`, `stop_tx`, and `write_samples` methods return an `SdrError::Hardware` unsupported-transmitter error rather than successful no-ops or `Ok(0)`. Real TX implementations (`MockSdr`, `PlutoSdr`) already override the methods.
- **Success criterion (EARS):**
  - **WHEN** an RX-only `SdrDriver` uses the default TX methods, **THEN** `start_tx`, `stop_tx`, and `write_samples` **SHALL** each return an explicit unsupported-transmitter error and **SHALL NOT** report success.
- **Tests:** `test_sdr_driver_default_tx_methods_fail_explicitly`; retain MockSdr and Pluto TX regressions.

### T-126: Enforce source-validated, one-slot, transactionally committed stop-and-wait ARQ

- **Intents:** [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md), [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md); prerequisite evidence for [INT-0011](../../../intents/INT-0011-mesh-messaging-callsign.md)
- **Touches:** `crates/sdr-protocols/src/packet.rs`, `crates/sdr-protocols/src/lib.rs`, `crates/sdr-mesh/src/radio.rs`, `crates/sdr-mesh/tests/radio_it.rs`, `crates/sdr-cli/tests/book_it.rs`
- **Depends on:** T-135, so every lower-level unsupported/short-write outcome is explicit before ARQ commits against it.
- **Acceptance criterion:** INT-0006 #2 and #4; INT-0008 #1 framing/recovery half. TUN, stream-runtime reliability, and arbitrary read-boundary handling remain T-107/T-117/T-130.
- **Implementation:** replace the vector of unacknowledged frames with one prepared/awaiting-ACK slot; make a second tracked send fail without advancing sequence; prepare, commit, or abort an initial send; peek then commit retries; drain ACKs only after a full write; reject zero/short writes as errors; pin data/ACK sources to the configured peer and reject reliable broadcast peers; ACK only a newly expected or exactly last-delivered sequence. Update the stale Book dependency regression from T-113 to T-126 followed by T-117.
- **Success criterion (EARS):**
  - **WHEN** a second tracked frame is requested before the first is acknowledged, **THEN** `ArqTransceiver` **SHALL** return explicit busy/backpressure without advancing the transmit sequence or touching the driver.
  - **WHEN** a future frame arrives before the expected frame, **THEN** the receiver **SHALL NOT** deliver or ACK it; after the missing frame arrives and the future frame is retransmitted, both **SHALL** be delivered exactly once in order.
  - **WHEN** reliable mode has no concrete peer or data/ACK arrives from an address other than that peer, **THEN** the link **SHALL** refuse the configuration/frame without changing receive, acknowledgement, retry, or driver state.
  - **WHEN** an initial, ACK, or retry write is zero, short, or errors, **THEN** `RadioLink` **SHALL** return an error and preserve/abort state so no sequence, ACK, timer, or retry budget is falsely committed.
  - **WHEN** all writes complete, **THEN** existing deterministic 30%-loss, ACK-loss, duplicate-suppression, and radio roundtrip contracts **SHALL** remain green with exact-once delivery.
  - **WHEN** the Book's INT-0011 blocking-dependency contract is evaluated, **THEN** it **SHALL** name corrected ARQ T-126 followed by reliable-stream T-117 and **SHALL NOT** reference removed T-113.
- **Tests:** `test_arq_rejects_second_outstanding_send_without_advancing_sequence`, `test_arq_reordered_future_frame_is_not_acked_or_delivered`, `test_arq_duplicate_last_delivered_is_acked_without_redelivery`, `test_arq_sequence_wrap_preserves_duplicate_classification`, `test_arq_wrong_source_data_is_ignored_without_state_change`, `test_arq_wrong_source_ack_cannot_clear_outstanding_frame`, `test_arq_retry_state_changes_only_after_commit`, `test_radiolink_stop_and_wait_blocks_second_driver_write`, `test_radiolink_initial_write_failures_do_not_commit`, `test_radiolink_ack_write_failures_remain_queued`, `test_radiolink_retry_write_failures_remain_due`, `test_radiolink_reliable_mode_rejects_broadcast_peer`, `test_roadmap_names_blocking_dependencies`; retain the seeded loss and non-reliable radio suites.

### T-128: Make transmit compliance input-complete and fail closed

- **Intents:** [INT-0005](../../../intents/INT-0005-regulatory-band-compliance.md), [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md), [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- **Touches:** `crates/sdr-core/src/compliance.rs`, `crates/sdr-core/src/lib.rs`, `crates/sdr-mesh/src/policy.rs`, `crates/sdr-mesh/src/node.rs`, `crates/sdr-mesh/tests/loopback_it.rs`, `crates/sdr-cli/src/main.rs`, `crates/sdr-cli/tests/tunnel_it.rs`, `crates/sdr-cli/tests/ssh_tunnel_it.rs`, `crates/sdr-cli/tests/e2e_pipeline_tests.rs`
- **Depends on:** T-126 for the final radio/tunnel regression run; the compliance implementation itself is independent.
- **Acceptance criterion:** INT-0005 #2/#3, INT-0006 #4's pre-transmit gate, and INT-0008 #3.
- **Implementation:** introduce one typed `TransmissionPlan`; implement `FromStr` for jurisdiction with an error; require finite positive center-frequency/occupied-bandwidth Hz, finite EIRP dBm (zero and negative are valid), and finite duty-cycle percent in `(0, 100]`; check occupied span, encoded bandwidth, duty cycle, power, and encryption. Route `MeshPolicy`, `MeshNode`, `bands --check-freq`, and `tunnel` through it. A pure tunnel-plan helper derives binary-FSK occupied bandwidth as `2 * deviation_hz + sample_rate / samples_per_symbol`, forces 100% duty cycle, requires stated EIRP, and is unit-testable independently of driver construction. The command evaluates that plan before driver construction and remains wrapped by a policy-checked interface for every datagram.
- **Success criterion (EARS):**
  - **WHEN** jurisdiction is unknown; center-frequency Hz or occupied-bandwidth Hz is non-finite or `<= 0`; EIRP dBm is non-finite; or duty-cycle percent is non-finite, `<= 0`, or `> 100`, **THEN** the API and CLI **SHALL** refuse explicitly without substituting US defaults, while finite zero/negative EIRP **SHALL** remain valid input.
  - **WHEN** occupied bandwidth exceeds the catalog limit, the occupied span crosses a band edge, or duty cycle exceeds an encoded limit, **THEN** compliance **SHALL** return `NonCompliant` with the violated constraint.
  - **WHEN** the tunnel constructs its transmission plan, **THEN** occupied bandwidth **SHALL** equal `2 * deviation_hz + sample_rate / samples_per_symbol` and duty cycle **SHALL** equal 100%; each derived value **SHALL** independently refuse a restricted plan before driver construction.
  - **WHEN** the tunnel plan is refused, **THEN** the command **SHALL** exit nonzero before driver construction and every policy-wrapped interface **SHALL** emit zero frames.
  - **WHEN** a finite plan is exactly at encoded bandwidth/duty/power limits and encryption is permitted, **THEN** compliance **SHALL** allow it while preserving the existing amateur-encryption refusal.
- **Tests:** `test_jurisdiction_from_str_rejects_unknown`, `test_compliance_rejects_non_finite_inputs`, `test_compliance_rejects_invalid_frequency_and_bandwidth`, `test_compliance_rejects_invalid_duty_cycle_range`, `test_compliance_accepts_zero_and_negative_eirp`, `test_compliance_uses_percentage_duty_cycle_units`, `test_compliance_rejects_bandwidth_over_catalog_limit`, `test_compliance_rejects_occupied_span_crossing_band_edge`, `test_compliance_rejects_duty_cycle_over_catalog_limit`, `test_compliance_accepts_values_at_catalog_limits`, `test_compliance_preserves_power_and_encryption_refusals`, `test_node_policy_refusal_emits_no_frame`, `test_policy_checked_interface_refusal_emits_no_frame`, `test_cli_bands_rejects_unknown_jurisdiction`, `test_cli_tunnel_rejects_non_finite_plan`, `test_cli_tunnel_accepts_negative_eirp_input`, `test_tunnel_plan_derives_carson_bandwidth_and_full_duty`, `test_cli_tunnel_refuses_derived_bandwidth_before_driver_connection`, `test_cli_tunnel_refuses_continuous_duty_before_driver_connection`, `test_cli_tunnel_refuses_encrypted_amateur_band`, `test_cli_tunnel_policy_refusal_precedes_driver_connection`; migrate `test_cli_tunnel_stdio_pipes_bytes`, `test_cli_tunnel_status_goes_to_stderr`, `test_ssh_client_banner_traverses_bridge`, and `test_ssh_proxycommand_exchanges_version` to the required allowed EIRP input.

### T-133: Replace demod-export false success with typed, real output

- **Intent:** [INT-0009](../../../intents/INT-0009-receiver-application.md)
- **Touches:** `crates/sdr-cli/Cargo.toml`, `crates/sdr-cli/src/main.rs`, `crates/sdr-cli/tests/demod_output_it.rs`
- **Depends on:** T-128, because both edit the CLI command model and `main.rs`.
- **Acceptance criterion:** the demodulated-audio-to-WAV slice of INT-0009 #3, plus explicit rejection of unsupported FSK file export; full-capture streaming and replay remain T-129.
- **Implementation:** model demod results as audio or bits instead of using an empty audio vector as a sentinel; write finalized mono PCM WAV for audio modes; reject FSK file output as unsupported rather than inventing a format not named by INT-0009; validate mode/output compatibility before creating a file. Factor the WAV writer over a testable `Write + Seek` seam, propagate open/write/finalization errors, and print “Exported” only after successful finalization.
- **Success criterion (EARS):**
  - **WHEN** a valid IQ fixture is demodulated in an audio mode with a `.wav` output, **THEN** the CLI **SHALL** create a finalized mono WAV with the expected sample rate and nonzero sample count before reporting success.
  - **WHEN** FSK demodulation is given an output path, **THEN** the CLI **SHALL** return an explicit unsupported-output error and **SHALL NOT** create or claim a bitstream artifact.
  - **WHEN** the mode is unknown or the output type is incompatible, **THEN** the CLI **SHALL** exit nonzero and **SHALL NOT** create or claim an output artifact.
  - **WHEN** a WAV target cannot be opened or the writer/finalizer fails, **THEN** the CLI/helper **SHALL** return the underlying error and **SHALL NOT** print an export-success claim.
  - **WHEN** no output path is supplied, **THEN** the CLI **SHALL** report the typed result count without claiming a file export.
- **Tests:** `test_cli_demod_audio_writes_real_mono_wav`, `test_cli_demod_fsk_output_is_rejected_without_file`, `test_cli_demod_unknown_mode_fails_without_output`, `test_cli_demod_incompatible_extension_fails_without_output`, `test_cli_demod_wav_open_failure_returns_nonzero_without_success_claim`, `test_write_audio_wav_propagates_write_and_finalize_errors`, `test_cli_demod_without_output_reports_typed_count`.

### T-134: Remove the Rigctl process-test port race

- **Intent:** [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md)
- **Touches:** `crates/sdr-cli/tests/rigctl_server_it.rs`
- **Depends on:** (none)
- **Acceptance criterion:** INT-0004 #4's real TCP verification; this does not claim shared receiver/driver composition.
- **Implementation:** stop probing and releasing a “free” port; launch the real CLI with `--port 0`, read its existing readiness line, parse the bound loopback port, and connect only after readiness. Keep process teardown RAII-based and capture premature child exit as a useful test failure.
- **Success criterion (EARS):**
  - **WHEN** Rigctl integration tests start a server, **THEN** they **SHALL** connect to the exact ephemeral port reported after bind, with no release/spawn race.
  - **WHEN** several real Rigctl servers run concurrently, **THEN** each **SHALL** receive a unique port, answer its own query, and be reaped independently.
- **Tests:** the existing `test_rigctl_server_get_freq`, `test_rigctl_server_set_and_get_mode`, and `test_rigctl_server_dump_state` through the new helper; add `test_rigctl_servers_use_unique_ephemeral_ports`.

### T-106: Normalize the known formatter baseline

- **Intent:** [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) maintenance evidence only; no acceptance criterion advances.
- **Touches:** `crates/sdr-core/src/sample.rs`, `crates/sdr-demod/src/modulator.rs`, `crates/sdr-demod/src/psk.rs`, `crates/sdr-spectrum/src/rigctl.rs`
- **Depends on:** all selected behavior tasks, so the final formatter pass observes the complete sprint diff.
- **Implementation:** run the pinned formatter once, inspect the resulting five known layout-only hunks plus formatting in newly edited Rust, and retain no semantic change under this task.
- **Success criterion (EARS):**
  - **WHEN** `cargo +1.93.0 fmt --all -- --check` runs at the integrated boundary, **THEN** it **SHALL** exit zero with no deferred formatter baseline.
- **Tests:** the global format gate plus the affected `sdr-core`, `sdr-demod`, and `sdr-spectrum` test suites.

## Files Created / Modified

- Workspace manifests/lockfile and `sdr-core` buffer/pipeline modules (T-119/T-120).
- `sdr-hardware`'s default transmitter capability contract (T-135).
- `sdr-protocols` packet state, loss harness, and `sdr-mesh` radio tests/runtime (T-126).
- Compliance, mesh-policy/node, CLI command model, and their tests (T-128).
- CLI demod writer plus a focused binary integration suite (T-133).
- Rigctl binary integration harness (T-134).
- Four exact formatter-baseline files (T-106).
- Stable Book evidence/task/meta files required by Plan/Build/Test/Loop phases.

## Verification

- Toolchain: `cargo +1.93.0`; the default 1.95 sysroot is locally incomplete.
- `cargo +1.93.0 test -p sdr-core` after T-119/T-120.
- `cargo +1.93.0 test -p sdr-hardware` after T-135; physical HIL remains ignored.
- `cargo +1.93.0 test -p sdr-protocols -p sdr-mesh` after T-126.
- `cargo +1.93.0 test -p sdr-core -p sdr-mesh -p sdr-cli` after T-128/T-133/T-134, with loopback socket permission where required.
- `cargo +1.93.0 test --workspace` at the integrated boundary; physical-radio HIL tests remain ignored unless separately authorized/configured.
- `cargo +1.93.0 fmt --all -- --check` and `cargo +1.93.0 clippy --workspace --all-targets`; T-106 clears the five known format hunks, while broad pre-existing Clippy cleanup remains T-105.
- `bash` Book validators/router helpers at each phase boundary.
- Show the principal new negative tests failing against the pre-fix behavior where practical; a green name alone is not evidence.

## Explicit Deferrals From the Audit

| Work | Why it is not hidden in this sprint |
|---|---|
| T-121 resampler rejection; T-122 GFSK/SSB; T-123–T-125 decoder/spectrum completion | Each needs independent numerical/standard vectors and is separable from the selected safety spine. |
| T-136 FIR/Gardner input validation; T-137 truthful MultiTone configuration | The Plan re-screen made these medium-severity defects durable. They remain bounded follow-ups rather than displacing this sprint's critical safety and false-success corrections. |
| T-127 capture integrity and Pluto lifecycle/transport | Requires a distinct driver transaction design and hardware fault injection; network IIO's bounded happy path remains useful. |
| T-117 reliable tunnel runtime; T-130 arbitrary read boundaries/MTU/backpressure | Must follow corrected T-126 semantics; this sprint does not call a fire-and-forget tunnel reliable. |
| Remaining T-129 receiver work | Full-capture incremental processing, playback, provenance replay, multi-VFO, scanning, and parity remain explicitly open after T-133. |
| T-131 Cloudlog/application composition | The isolated HTTP client remains functional; shared receiver state and truthful import validation are a separate application slice. |
| Remaining T-132 Rigctl composition | T-134 fixes the flaky harness only; shared cross-client state, explicit operator bind policy, and real driver control remain open. |
| T-101/T-107/T-108/T-114–T-118 and broader proposed intents | Hardware breadth, TUN/Babel, two-radio/on-air proof, full SSH session, and messaging require named external or prior-task unlockers. No on-air transmission is authorized. |
| Current-law catalog certification | Must use current primary legal sources in a dedicated review; this sprint only makes the encoded data impossible to ignore. |

## Honest Completion Boundary

Even if every task passes, INT-0001, INT-0002, INT-0004, INT-0005, INT-0006,
INT-0008, and INT-0009 remain non-terminal because their other annotated criteria and
named backlog work remain open. Sprint 10 closes defects; it does not relabel a
partial suite as complete.
