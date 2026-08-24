Finalized - DO NOT EDIT

# Sprint 10 Test Plan

## Verification Standard

Sprint 10 was triggered by green happy-path tests coexisting with undefined
behavior, silent truncation, false success, and a flaky full-suite run. Tests
therefore assert the negative boundary as well as the success path: impossible
progress is rejected, a future frame is not ACKed, failed I/O does not commit
state, policy refusal emits nothing, and a claimed file exists and parses.

The viable local toolchain is Rust 1.93.0. The default 1.95 sysroot is missing
its installed target library on this machine and is not evidence about the
repository.

## Intent Traceability

| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|---|---|---|---|
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | #1 bounded lock-free FIFO | T-119 / WHEN producer and consumer wrap concurrently THEN every accepted value SHALL arrive exactly once in FIFO order | `test_ring_buffer_concurrent_spsc_preserves_order_without_loss` |
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | #1 bounded partial I/O | T-119 / WHEN a batch exceeds availability THEN the exact partial count SHALL be returned without corruption | `test_ring_buffer_capacity_and_partial_io`, existing FIFO/wraparound tests |
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | #1 reusable buffer | T-119 / WHEN a quiescent buffer is cleared THEN it SHALL be empty and reusable with unchanged capacity | `test_ring_buffer_clear_is_safe_and_reusable` |
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | #1 thread-safe public type | T-119 / WHEN shared for `T: Send` THEN the type SHALL satisfy `Send + Sync` without local unsafe code | `test_ring_buffer_send_sync_for_send_samples`; crate-level `forbid(unsafe_code)` compile check |
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | #1 partial block progress | T-120 / WHEN a block partially consumes a chunk THEN the remainder SHALL be processed once before another source read | `test_linear_pipeline_partial_block_consumption_preserves_remainder`, `test_linear_pipeline_does_not_read_next_chunk_while_input_pending` |
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | #1 partial/error sink progress | T-120 / WHEN a sink partially accepts or errors after progress THEN the unwritten suffix SHALL be retained in order without block replay | `test_linear_pipeline_partial_sink_writes_are_retried_in_order`, `test_linear_pipeline_sink_error_retains_unwritten_output` |
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | #1 explicit contracts | T-120 / WHEN a stage reports impossible or zero progress THEN the pipeline SHALL error without discarding pending samples | `test_linear_pipeline_invalid_progress_returns_error` |
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | #5 synchronous tags | T-120 / WHEN tags cross one-to-one partial processing THEN each SHALL arrive once at the same absolute offset; unmapped rate change SHALL error | `test_linear_pipeline_source_tags_forwarded_exactly_once`, `test_linear_pipeline_rate_changing_block_requires_tag_mapper` |
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | #1 terminal lifecycle | T-120 / WHEN EOF follows drained work THEN the sink SHALL flush exactly once before `step` returns zero | `test_linear_pipeline_flushes_sink_once_at_eof` |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | #1 truthful TX abstraction | T-135 / WHEN an RX-only driver uses default TX methods THEN each SHALL return an unsupported-transmitter error | `test_sdr_driver_default_tx_methods_fail_explicitly` |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md), [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #2 stop-and-wait / #1 ordered recovery | T-126 / WHEN a second tracked send precedes ACK THEN ARQ SHALL return busy without sequence advance or driver I/O | `test_arq_rejects_second_outstanding_send_without_advancing_sequence`, `test_radiolink_stop_and_wait_blocks_second_driver_write` |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #2 no reorder loss | T-126 / WHEN a future frame precedes expected THEN it SHALL not be delivered or ACKed and later SHALL deliver in order after retransmit | `test_arq_reordered_future_frame_is_not_acked_or_delivered`, `test_arq_duplicate_last_delivered_is_acked_without_redelivery`, `test_arq_sequence_wrap_preserves_duplicate_classification` |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #2 peer state integrity | T-126 / WHEN reliable mode lacks a concrete peer or a frame comes from another source THEN it SHALL be refused without state/driver change | `test_arq_wrong_source_data_is_ignored_without_state_change`, `test_arq_wrong_source_ack_cannot_clear_outstanding_frame`, `test_radiolink_reliable_mode_rejects_broadcast_peer` |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md), [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #4 exact radio writes / #1 non-corrupt delivery | T-126 / WHEN initial, ACK, or retry I/O is zero/short/error THEN state SHALL not be falsely committed | `test_arq_retry_state_changes_only_after_commit`, `test_radiolink_initial_write_failures_do_not_commit`, `test_radiolink_ack_write_failures_remain_queued`, `test_radiolink_retry_write_failures_remain_due` (each radio test covers Zero/Short/Error) |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md), [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #2 loss recovery / #1 radio regression | T-126 / WHEN writes complete THEN 30%-loss, ACK-loss, duplicate, and radio contracts SHALL remain green | existing `arq_reliability` tests and non-reliable `radio_it` suite |
| [INT-0011](../../../intents/INT-0011-mesh-messaging-callsign.md) | T-126 then T-117 prerequisite consistency | T-126 / WHEN the Book dependency contract is evaluated THEN it SHALL name T-126 followed by T-117 and not removed T-113 | `test_roadmap_names_blocking_dependencies` |
| [INT-0005](../../../intents/INT-0005-regulatory-band-compliance.md) | #2/#3 typed numeric inputs and jurisdiction | T-128 / WHEN jurisdiction is unknown; frequency/bandwidth Hz invalid; EIRP dBm non-finite; or duty percent outside `(0,100]` THEN API/CLI SHALL refuse, while finite zero/negative EIRP remains valid | `test_jurisdiction_from_str_rejects_unknown`, `test_compliance_rejects_non_finite_inputs`, `test_compliance_rejects_invalid_frequency_and_bandwidth`, `test_compliance_rejects_invalid_duty_cycle_range`, `test_compliance_accepts_zero_and_negative_eirp`, `test_compliance_uses_percentage_duty_cycle_units`, `test_cli_bands_rejects_unknown_jurisdiction`, `test_cli_tunnel_rejects_non_finite_plan`, `test_cli_tunnel_accepts_negative_eirp_input` |
| [INT-0005](../../../intents/INT-0005-regulatory-band-compliance.md) | #2 bandwidth/duty constraints | T-128 / WHEN bandwidth/span/duty exceeds an encoded limit THEN compliance SHALL name that violation | `test_compliance_rejects_bandwidth_over_catalog_limit`, `test_compliance_rejects_occupied_span_crossing_band_edge`, `test_compliance_rejects_duty_cycle_over_catalog_limit` |
| [INT-0005](../../../intents/INT-0005-regulatory-band-compliance.md), [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #2/#4 tunnel-derived policy inputs | T-128 / WHEN tunnel plan is built THEN bandwidth SHALL equal `2 * deviation + sample_rate / samples_per_symbol` and duty SHALL be 100%, with each able to refuse before driver construction | `test_tunnel_plan_derives_carson_bandwidth_and_full_duty`, `test_cli_tunnel_refuses_derived_bandwidth_before_driver_connection`, `test_cli_tunnel_refuses_continuous_duty_before_driver_connection` |
| [INT-0005](../../../intents/INT-0005-regulatory-band-compliance.md), [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md), [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #2/#4/#3 fail-closed TX | T-128 / WHEN a tunnel plan is refused THEN CLI SHALL exit before driver construction and wrapped interface SHALL emit zero frames | `test_node_policy_refusal_emits_no_frame`, `test_policy_checked_interface_refusal_emits_no_frame`, `test_cli_tunnel_refuses_encrypted_amateur_band`, `test_cli_tunnel_policy_refusal_precedes_driver_connection` |
| [INT-0005](../../../intents/INT-0005-regulatory-band-compliance.md) | #2 boundary correctness | T-128 / WHEN values equal encoded limits and encryption is permitted THEN plan SHALL pass while amateur encryption still refuses | `test_compliance_accepts_values_at_catalog_limits`, `test_compliance_preserves_power_and_encryption_refusals`, `test_cli_tunnel_stdio_pipes_bytes`, `test_cli_tunnel_status_goes_to_stderr`, `test_ssh_client_banner_traverses_bridge`, `test_ssh_proxycommand_exchanges_version` with explicit allowed EIRP |
| [INT-0009](../../../intents/INT-0009-receiver-application.md) | #3 demodulated audio export slice | T-133 / WHEN valid audio demod has `.wav` output THEN CLI SHALL finalize a mono WAV with correct rate and samples before success | `test_cli_demod_audio_writes_real_mono_wav` |
| [INT-0009](../../../intents/INT-0009-receiver-application.md) | #3 truthful demod-output boundary | T-133 / WHEN FSK has an output path THEN CLI SHALL reject the unsupported export and create no artifact | `test_cli_demod_fsk_output_is_rejected_without_file` |
| [INT-0009](../../../intents/INT-0009-receiver-application.md) | #3 truthful receiver surface | T-133 / WHEN mode/output type is invalid THEN CLI SHALL fail and create no artifact | `test_cli_demod_unknown_mode_fails_without_output`, `test_cli_demod_incompatible_extension_fails_without_output` |
| [INT-0009](../../../intents/INT-0009-receiver-application.md) | #3 truthful writer failure | T-133 / WHEN output open/write/finalize fails THEN error SHALL propagate and no export-success claim SHALL print | `test_cli_demod_wav_open_failure_returns_nonzero_without_success_claim`, `test_write_audio_wav_propagates_write_and_finalize_errors` |
| [INT-0009](../../../intents/INT-0009-receiver-application.md) | #3 truthful no-output surface | T-133 / WHEN no path is supplied THEN CLI SHALL report typed count without claiming export | `test_cli_demod_without_output_reports_typed_count` |
| [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md) | #4 real Rigctl TCP | T-134 / WHEN a server starts THEN tests SHALL use its post-bind reported ephemeral port | existing three `rigctl_server_it` command tests through the new helper |
| [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md) | #4 isolated concurrent TCP servers | T-134 / WHEN several servers run concurrently THEN each SHALL have a unique port, answer, and reap independently | `test_rigctl_servers_use_unique_ephemeral_ports` |
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | maintenance only; no criterion advancement | T-106 / WHEN the pinned workspace formatter check runs THEN it SHALL exit zero | `cargo +1.93.0 fmt --all -- --check` |

## Unit Tests

### T-119 — buffer contract (`sdr-core`)

- **Clause 1:** `test_ring_buffer_concurrent_spsc_preserves_order_without_loss` — two scoped threads transfer a long monotonic sequence through many wraps; neither loss, duplication, nor reordering is permitted.
- **Clause 2:** `test_ring_buffer_capacity_and_partial_io` plus existing FIFO/wraparound tests — preserve the `n -> next_power_of_two(max(2,n))-1` usable capacity and exact batch counts.
- **Clause 3:** `test_ring_buffer_clear_is_safe_and_reusable` — clear with the producer quiescent, then repeat wraparound I/O.
- **Clause 4:** `test_ring_buffer_send_sync_for_send_samples` — compile-time generic assertion; `#![forbid(unsafe_code)]` makes local unsafe a compiler failure.

Optional if the installed component is available: run the focused `sdr-core`
tests under Miri. Miri availability is not a completion gate because the queue
implementation is a dependency, not local unsafe code.

### T-120 — pipeline progress and tags (`sdr-core`)

- **Clause 1:** a scripted block consumes alternating small prefixes while the source records read calls; assert exact sink sequence and no premature source read.
- **Clause 2:** a sink accepts `[2, 1, rest]`, then a separate sink errors after one partial acceptance and succeeds on the next `step`; assert the block call count does not increase during retry.
- **Clause 3:** parameterized invalid `consumed > input`, `produced > output`, zero consumption, zero sink acceptance, and oversized sink acceptance cases all return `SdrError::Protocol`/`Config` while retaining state.
- **Clause 4:** absolute tags on different partial subranges arrive once; a rate-changing fixture with tags and no mapper errors. A mapper fixture then proves an explicit scaled offset.
- **Clause 5:** source EOF and repeated post-EOF `step` calls produce one flush total and stable zero returns.

### T-126 — protocol state (`sdr-protocols`)

- **Clauses 1–3:** exercise one-slot busy behavior, future/expected/retransmitted ordering, exact last-delivered duplicate handling, wrong-source data, wrong-source ACK, and sequence wrap.
- **Clause 4:** prepare/abort/commit and peek/commit retry tests inspect sequence, retry count, deadline, and abandonment before and after commit. The radio suite executes all nine `Initial/ACK/Retry × Zero/Short/Error` cases.
- **Clause 5:** retain seeded 0/10/30% data-loss and ACK-loss tests, updated to commit only transmissions the simulated channel accepts.
- **Clause 6:** the Book test asserts both T-126 and T-117 in INT-0011's dependency row and rejects stale T-113.

### T-135 — unsupported TX defaults (`sdr-hardware`)

- **Clause 1:** a minimal RX-only test driver omits all optional TX overrides;
  each default TX method must return the same explicit unsupported-hardware
  class, while `has_tx()` remains false. Existing MockSdr/Pluto overrides remain
  regression coverage.

### T-128 — compliance evaluation (`sdr-core`, `sdr-mesh`)

- **Clauses 1–2:** table-driven invalid numeric/range/span/bandwidth/duty cases and exact-limit positive boundaries. Inputs are Hz, dBm, and percentage points: finite `0`/negative dBm are positive tests; duty `1.0` means one percent and `0.01` means 0.01 percent.
- **Clause 3:** the pure tunnel-plan helper is checked against exact default and non-default modem values; it must derive Carson bandwidth and force 100% duty rather than accept caller substitutions.
- **Clause 4:** a counting `MeshInterface` proves both `MeshNode` and the policy-checked tunnel wrapper perform zero sends after refusal.
- **Clause 5:** preserve power and encrypted-amateur negative regressions alongside an encoded-limit positive plan.

### T-133 — WAV writer errors (`sdr-cli`)

- A `Write + Seek` fixture that fails during sample output and another that fails during finalization prove the underlying error is returned.
- The real binary targets a directory/non-creatable path and must exit nonzero without printing `Exported`; this covers the file-open boundary independently of the injected writer seam.

## Integration Tests

### T-126 — transactional radio I/O (`sdr-mesh/tests/radio_it.rs`)

A deterministic `ScriptedDriver` returns queued `Full`, `Zero`, `Short(n)`, or
`Error` outcomes and captures each write. Three table-driven tests cover the
complete `Initial/ACK/Retry × Zero/Short/Error` failure matrix and prove:

- a busy second tracked send never invokes the driver;
- zero/short/error initial writes fail and abort without sequence advance;
- zero/short/error queued ACK writes leave the identical ACK pending;
- zero/short/error due retries remain due at the same time and consume no N2 attempt;
- reliable mode rejects a broadcast peer; and
- the existing fire-and-forget MockSdr radio suite remains unchanged.

### T-128 — binary and policy composition

- `bands` rejects an unknown jurisdiction rather than printing US/FCC.
- An encrypted 145 MHz US tunnel returns nonzero.
- A US 915 MHz plan at a sample rate whose Carson estimate exceeds 500 kHz
  refuses for derived bandwidth before an unreachable driver is contacted.
- An EU 433 MHz plan (no encoded bandwidth limit) refuses because the
  continuous tunnel forces 100% duty above the encoded 10% limit; this isolates
  duty derivation from bandwidth.
- A refused tunnel using an unreachable driver reports policy refusal, not a
  connection attempt; this makes ordering observable.
- The allowed mock stdio/TCP/OpenSSH paths supply explicit EIRP and retain their
  byte-for-byte/version-exchange behavior. This remains fire-and-forget until
  T-117 and is not reliability evidence.

### T-133 — real CLI artifacts

`demod_output_it.rs` drives the built binary against small deterministic WAV or
SigMF fixtures. The audio artifact is reopened with a WAV reader and its channel
count/rate/sample count checked. FSK-with-output, unknown-mode, and incompatible
extension cases assert both nonzero exit and path nonexistence. A directory or
otherwise non-creatable WAV target asserts nonzero exit and no `Exported`
message; FSK without an output path may still report its recovered in-memory
bit count.

### T-134 — race-free Rigctl process harness

All tests start `sdr-cli rigctl --port 0`, wait for and parse the readiness
address, then connect. The concurrency regression launches multiple children at
once and verifies distinct ports plus independent `f` responses before RAII
teardown. No test reserves a port before the child binds it.

## End-to-End Tests

- **Status:** possible in software for the selected boundaries.
- `cargo +1.93.0 test --workspace` is the integrated gate. It must include the
  full real-binary CLI suites and pass without the previous parallel Rigctl TCP
  reset. HIL tests remain ignored by their existing gates.
- `test_cli_demod_audio_writes_real_mono_wav` is the selected receiver E2E:
  capture fixture -> real CLI -> parseable output artifact.
- `test_cli_tunnel_policy_refusal_precedes_driver_connection` is the selected
  safety E2E: invalid transmission plan -> real CLI -> refusal -> no driver I/O.
- Existing MockSdr tunnel and OpenSSH version-exchange tests remain regression
  coverage only; they do not prove reliability, key exchange, authentication,
  two-radio behavior, or a shell.
- **Not-yet-possible / named unlockers:** meaningful hardware ARQ requires two
  radios and corrected Pluto lifecycle T-127; over-the-air work requires the
  user's explicit T-108 authorization; reliable stream/SSH behavior requires
  T-117 after T-126; normal IP MTU/read boundaries require T-130; receiver
  playback/provenance requires T-129.

## Failure-Path Demonstration

Before accepting each correction, demonstrate at least the principal regression
against the pre-fix behavior or a targeted mutation where practical:

- T-119: the old source fails the crate-level unsafe prohibition.
- T-120: the old pipeline loses a scripted remainder/partial write and forwards no tags.
- T-135: an RX-only driver using the old defaults reports successful TX startup/shutdown and a zero-byte successful write.
- T-126: the old receiver ACKs-and-loses the future sequence; the old radio accepts zero writes.
- T-128: the old CLI treats an unknown jurisdiction as US and warns-then-proceeds on amateur encryption.
- T-133: the old command exits successfully while the named file does not exist.
- T-134: repeated full-suite runs are supplementary evidence; the structural proof is removal of the release/spawn window and use of the server's post-bind address.
- T-106: the pinned workspace formatter check fails on the five audited hunks before the final formatting pass and exits zero afterward.

## Lint, Format, and Book Gates

- `cargo +1.93.0 fmt --all -- --check`.
- T-106 explicitly owns and clears the five pre-existing formatter hunks; this
  is not deferred or treated as an allowed failing gate.
- `cargo +1.93.0 clippy --workspace --all-targets`; no new warnings in touched
  code. Existing broad cleanup remains T-105, but actionable warnings in edited
  lines are fixed rather than waived.
- Installed Sprint Loop `check-book.sh`, `check-substrate.sh`, and router helper.
- `test_roadmap_names_blocking_dependencies` is updated to assert
  INT-0011's real T-126 -> T-117 dependency instead of deleted T-113.

## Honest Test Boundary

Passing this plan proves the selected software contracts only. It does not
certify the regulatory catalog's current legal accuracy, decoder completeness,
RF spectral compliance, a physical two-radio link, full SSH, Cloudlog import,
receiver playback, TUN/Babel, or any ignored hardware path.
