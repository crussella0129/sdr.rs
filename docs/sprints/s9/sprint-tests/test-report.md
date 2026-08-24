# Sprint 9 — Test Report

- **Verdict:** pass, with one caveat that materially bounds the claim
- **Critique:** [critique.md](critique.md) — `proceed-with-caveats`, four concerns addressed
- **Date:** 2026-08-24
- **Tested head:** `454e821d2c91cd05de40ed9f6867efd845322290`
- **Canonical runner:** `cargo test --workspace` — **33 suites, 0 failed**
- **Lint:** `cargo clippy --workspace --all-targets` — 0 errors

## What this sprint set out to fix

Research found that **the retransmission half of ARQ did not exist anywhere in
this codebase**: no frame was retained, `max_retries` was never read, received
ACKs were discarded, and `RadioLink` bypassed the ARQ receive path entirely. The
test asserting otherwise had two identical branches and simulated no loss.

INT-0006 criterion 2 — "retransmits dropped packets under simulated RF packet
loss up to 30%" — was unmet and had never been met, and I had marked the intent
`realized` partly on that test's name. The intent was re-opened to `active`.

## Criterion 2 is now met, in the way the criterion specifies

The criterion asks for delivery under **simulated** loss, and that is exactly
what is now verified:

```text
test arq_reliability::test_arq_delivers_all_payloads_at_30pct_loss ... ok
```

Twenty-four payloads, 30% of frames dropped, every payload delivered exactly
once and in order — through real framing, real CRC-32, real sequence numbers and
real retransmission.

**The loss is proven real rather than assumed.** Disabling retransmission made
all four reliability tests fail, each naming the payload that never arrived
("payload 2 was never acknowledged at 30% loss"). Disabling duplicate
suppression instead failed only the ACK-loss test, leaving the others green —
the tests discriminate between mechanisms rather than passing as a block.

## EARS clause coverage

| Task | EARS clause | Executed test | Result |
|---|---|---|---|
| T-042 | WHEN T1 expires without an ACK THEN return for retransmission | `test_arq_retransmits_after_timeout` | pass |
| T-042 | WHEN T1 has not expired THEN return nothing | `test_arq_no_retransmission_before_timeout` | pass |
| T-042 | WHEN a matching ACK arrives THEN clear and never retransmit | `test_arq_ack_stops_retransmission` | pass |
| T-042 | WHEN retried `max_retries` times THEN abandon and report permanent failure | `test_arq_gives_up_after_max_retries` | pass |
| T-042 | WHEN a duplicate arrives THEN suppress the payload but still ACK | `test_arq_duplicate_suppressed_but_acked` | pass |
| T-043 | WHEN 30% of frames drop THEN deliver every payload exactly once, in order | `test_arq_delivers_all_payloads_at_30pct_loss` | pass |
| T-043 | WHEN 10% of frames drop THEN the same | `test_arq_delivers_all_payloads_at_10pct_loss` | pass |
| T-043 | WHEN an ACK is lost THEN do not deliver twice | `test_arq_ack_loss_does_not_duplicate_payload` | pass |
| T-043 | WHEN the same seed is used THEN the result is identical | `test_arq_lossy_channel_is_deterministic` | pass |
| T-044 | WHEN data is received over the radio THEN queue an ACK | `test_radiolink_acks_received_data` | pass |
| T-044 | WHEN a duplicate is received THEN do not deliver twice | `test_radiolink_suppresses_duplicate_frames` | pass |
| T-044 | WHEN `service` runs after T1 THEN retransmit | `test_radiolink_retransmits_unacked_frame` | pass |

Every clause maps to an executed test whose **failure path was demonstrated** —
the standard this sprint set itself, since it exists because a test asserted
more than its body delivered.

## The caveat that bounds this sprint's claim

**The tunnel and stream paths are not protected by any of this.**

`set_reliable` and `service` are called only from `radio_it.rs`. `sdr-cli tunnel`
constructs a `RadioLink` and never enables reliability, so every `StreamBridge`
and every SSH session over the radio still runs **fire-and-forget with no
retransmission**. T-113 was raised so a dropped frame would not silently truncate
a stream; that mechanism now exists and is proven, but the path that motivated it
does not yet use it.

This is deferred deliberately, not overlooked. Enabling it means threading a time
source into the stdio and TCP pump loops and deciding how retransmission
interacts with a bidirectional byte pump — design work, and `main.rs` is outside
every locked task's touched paths. Backlog **T-117**.

So: **INT-0006 criterion 2 is met; INT-0011's prerequisite is only partly
discharged.** Anyone reading "delivered at 30% loss" should not conclude the SSH
tunnel is now reliable.

## Other caveats — stated, not implied away

1. **Not verified on hardware, and not claimed to be.** One radio in internal
   loopback hears its own transmission, so an ACK exchange is degenerate — the
   same self-negotiation that stalls the real OpenSSH client in `ssh_tunnel_it`.
   **The unlocker is a second radio.**
2. **The channel model is erasure, not corruption** (critique C-002). It drops
   whole frames — a fair model of what a CRC-checked link discards — but does
   not reproduce reordering, channel duplication, or a corrupt payload with a
   coincidentally valid CRC. "30% loss" here means 30% erasure.
3. **One seed per reliability test** (C-003). Determinism is the right default;
   one seed exercises one drop pattern. Multi-seed sweep is backlog **T-118**.
4. **The give-up path is untested under loss** (C-004). The retry bound is
   covered in isolation and the reliability tests deliberately raise it, so the
   composition — a link bad enough to exhaust retries, with the caller observing
   a permanent failure — is exercised nowhere. Backlog **T-118**.
5. **T1 = 500 ms is a reasoned default, not a measurement.** Half-duplex
   turnaround latency on a real link is unmeasured; the constant documents this.
6. **Stop-and-wait, not sliding window.** INT-0006's intent text mentions the
   latter; stop-and-wait is what a half-duplex channel supports naturally.
   Throughput under ARQ was not measured and is not claimed.
7. **The Pluto's cyclic buffer conflicts with stop-and-wait** (T-115). A buffer
   repeating one frame indefinitely cannot express "sent once, now listening".
8. **Nothing went on air.** Still gated on **T-108**.

## Regression contract — held, and it was genuinely at risk

Plan critique C-002 warned T-044 could break the contract *structurally*: the
tests run over `MockSdr` loopback, which echoes every ACK back onto the path
those tests cover. The six carried-over `radio_it` tests pass **unchanged**, for
two structural reasons rather than luck — ARQ is opt-in, and receiving never
transmits as a side effect. Every suite from Sprints 0-8 passes.

## Test evidence links

- [unit-tests.md](unit-tests.md)
- [integration-tests.md](integration-tests.md)
- [e2e-tests.md](e2e-tests.md)
- [critique.md](critique.md)
