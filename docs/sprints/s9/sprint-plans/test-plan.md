Finalized - DO NOT EDIT

# Sprint 9 Test Plan

## The standard this sprint is held to

Sprint 9 exists because a test called `test_arq_retransmission_lossy_channel`
simulated no loss and performed no retransmission, and because I then marked
INT-0006 `realized` on the strength of that name. Every test below is therefore
subject to one additional requirement beyond passing: **its failure path must be
demonstrated** by breaking the behaviour and observing the test go red. A test
that cannot fail is what produced this sprint.

## Intent Traceability

| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #2 retransmits dropped packets | T-042 / WHEN T1 expires without an ACK THEN the frame SHALL be returned for retransmission | `test_arq_retransmits_after_timeout` |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #2 ACK stops retransmission | T-042 / WHEN a matching ACK arrives THEN the frame SHALL be cleared and SHALL NOT be retransmitted | `test_arq_ack_stops_retransmission` |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #2 bounded retries | T-042 / WHEN retried `max_retries` times THEN the frame SHALL be abandoned and reported as a permanent failure | `test_arq_gives_up_after_max_retries` |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #2 automatic acknowledgement | T-042 / WHEN a duplicate data frame arrives THEN its payload SHALL be suppressed and an ACK SHALL still be produced | `test_arq_duplicate_suppressed_but_acked` |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #2 no premature retransmit | T-042 / WHEN T1 has not expired THEN nothing SHALL be returned | `test_arq_no_retransmission_before_timeout` |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | **#2 under 30% loss** | T-043 / WHEN 30% of frames are dropped THEN every payload SHALL be delivered exactly once, in order | `test_arq_delivers_all_payloads_at_30pct_loss` |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #2 reproducibility | T-043 / WHEN the same seed is used THEN the result SHALL be identical | `test_arq_lossy_channel_is_deterministic` |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #2 lost ACKs do not duplicate | T-043 / WHEN an ACK is lost THEN the payload SHALL NOT be delivered twice | `test_arq_ack_loss_does_not_duplicate_payload` |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 recovery without corruption | T-044 / WHEN a data frame is received over the radio THEN an ACK SHALL be queued | `test_radiolink_acks_received_data` |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 recovery without corruption | T-044 / WHEN a duplicate frame is received THEN its payload SHALL NOT be delivered twice | `test_radiolink_suppresses_duplicate_frames` |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #2 on the radio path | T-044 / WHEN `service` is called after T1 expires THEN the frame SHALL be retransmitted | `test_radiolink_retransmits_unacked_frame` |

Criterion 2 is the whole point of the sprint and is the only intent criterion
whose state changes. Criteria 1, 3 and 4 of INT-0006 are unaffected and are
re-verified only as regression.

## Unit Tests

### T-042 — `ArqTransceiver` state machine (`crates/sdr-protocols/src/packet.rs`)

All five drive time explicitly (`now_ms`), so timeout behaviour is exercised
instantly and deterministically — no sleeping, no wall clock.

- `test_arq_retransmits_after_timeout` — a frame sent at t=0 with no ACK is
  returned by `due_retransmissions` once T1 has passed.
- `test_arq_no_retransmission_before_timeout` — the negative case: nothing is
  returned at t < T1. Guards against a retransmit-everything implementation
  passing the previous test vacuously.
- `test_arq_ack_stops_retransmission` — after the matching ACK, the frame is
  never returned again however far time advances.
- `test_arq_gives_up_after_max_retries` — the frame is abandoned after N2
  attempts and surfaced as a permanent failure, not silently dropped.
- `test_arq_duplicate_suppressed_but_acked` — a repeated sequence number yields
  no payload but still yields an ACK (the lost-ACK case).

## Integration Tests

### T-043 — reliability over a lossy channel (`crates/sdr-protocols`)

Two transceivers exchanging payloads through a **seeded** drop model with
deterministic time.

- `test_arq_delivers_all_payloads_at_30pct_loss` — **the criterion-2 test.**
  Every payload delivered exactly once and in order at 30% frame loss.
- `test_arq_delivers_all_payloads_at_10pct_loss` — the same at a milder rate,
  so a regression that only appears under heavy loss is still localized.
- `test_arq_lossy_channel_is_deterministic` — the same seed produces the same
  drop sequence, so a failure is reproducible rather than a flake.
- `test_arq_ack_loss_does_not_duplicate_payload` — drops **ACKs** specifically,
  which forces retransmission of an already-delivered frame; the receiver must
  suppress the duplicate. This is the failure mode that corrupts a byte stream
  and it deserves its own test rather than being folded into the loss rate.

**`test_arq_retransmission_lossy_channel` is deleted**, not amended. Its name
claims what its body never did, and leaving it would preserve the exact
confusion that caused this sprint.

### T-044 — ARQ on the mesh receive path (`crates/sdr-mesh/tests/radio_it.rs`)

Over `MockSdr` loopback, exercising the real framing/modulation path.

- `test_radiolink_acks_received_data` — a received data frame produces an ACK.
- `test_radiolink_suppresses_duplicate_frames` — the same frame twice yields one
  datagram.
- `test_radiolink_retransmits_unacked_frame` — `service(now_ms)` past T1
  retransmits.

**Regression contract — carried forward unchanged:** the six existing `radio_it`
tests and the two `hw_radio` tests. This contract caught two real defects in
Sprint 6 and validated T-035 in Sprint 7. A change in their results is a
failure, not a rebaseline — and it matters especially here, because T-044 alters
the receive path they cover.

## End-to-End Tests

- **Status:** possible in simulation; **not possible on hardware**, with a named
  reason rather than a shrug.
- The 30% loss test is the end-to-end proof for criterion 2: real framing, real
  CRC, real sequence numbers, real retransmission, over a channel that actually
  drops frames.
- **Not-yet-possible (named unlockers):**
  - **ARQ verified on real hardware** → requires **two radios**. One device in
    internal loopback hears its own transmission, making an ACK exchange
    degenerate — the same self-negotiation that stalls the SSH client in
    `ssh_tunnel_it`. A second device is the unlocker; nothing in this sprint may
    imply hardware ARQ coverage.
  - **Stop-and-wait over the Pluto's cyclic buffer** → **T-115**. A buffer that
    repeats one frame indefinitely cannot express "sent once, now listening".
  - **A measured T1** → requires half-duplex turnaround latency measured on
    hardware. The chosen value is a reasoned default and must be labelled as one.

## Lint and Format

`cargo fmt` and `cargo clippy --workspace --all-targets` clean of errors.
Pre-existing warnings remain backlog T-105; this sprint adds none.
