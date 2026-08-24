# Sprint 9 — Integration Test Results

- **Tested head:** `454e821d2c91cd05de40ed9f6867efd845322290`
- **Runner:** `cargo test --workspace --tests`
- **Date:** 2026-08-24
- **Result:** **33 suites, 0 failed**

## T-043 — reliability over a channel that actually drops frames

`crates/sdr-protocols/src/lib.rs`, module `arq_reliability` — **4 passed**.

| Test | EARS clause | Result |
|---|---|---|
| `test_arq_delivers_all_payloads_at_30pct_loss` | WHEN 30% of frames are dropped THEN every payload SHALL be delivered exactly once, in order | ok |
| `test_arq_delivers_all_payloads_at_10pct_loss` | WHEN 10% of frames are dropped THEN the same guarantee SHALL hold | ok |
| `test_arq_ack_loss_does_not_duplicate_payload` | WHEN an ACK is lost THEN the payload SHALL NOT be delivered twice | ok |
| `test_arq_lossy_channel_is_deterministic` | WHEN the same seed is used THEN the result SHALL be identical | ok |

`test_arq_delivers_all_payloads_at_30pct_loss` is **the criterion-2 test** — the
one INT-0006 has claimed since Sprint 1 and never had.

### The loss is proven real, not assumed

The deleted test's failure mode was passing without exercising anything, so the
first question asked of its replacement was whether the loss actually happens.
Disabling retransmission in the state machine made **all four** tests fail, each
naming the exact payload that never arrived:

```text
assertion `left == right` failed: payload 2 was never acknowledged at 30% loss
assertion `left == right` failed: payload 0 was never acknowledged at 40% loss
assertion `left == right` failed: payload 4 was never acknowledged at 10% loss
assertion `left == right` failed: payload 6 was never acknowledged at 30% loss
```

That establishes two things at once: frames are genuinely being dropped, and
delivery genuinely depends on retransmission working.

Disabling **duplicate suppression** instead failed
`test_arq_ack_loss_does_not_duplicate_payload` *alone*, leaving the other three
green — which is exactly right, since only the ACK-dropping scenario re-delivers
a frame the receiver already has. The tests discriminate between the two
mechanisms rather than passing as a block.

### What was deleted

`test_arq_retransmission_lossy_channel` is **gone**, not repaired. Its two
branches were character-for-character identical, it simulated no loss, and it
performed no retransmission — yet its name was cited as evidence that criterion
2 was met. A repaired version under the same name would have preserved exactly
that confusion. A comment at the old site records what happened and points to
the replacement.

## T-044 — ARQ on the mesh receive path

`crates/sdr-mesh/tests/radio_it.rs` — **9 passed** (6 carried over + 3 new), over
`MockSdr` loopback exercising the real framing, modulation and demodulation path.

| Test | EARS clause | Result |
|---|---|---|
| `test_radiolink_acks_received_data` | WHEN a data frame is received over the radio THEN an ACK SHALL be queued | ok |
| `test_radiolink_suppresses_duplicate_frames` | WHEN a duplicate frame is received THEN its payload SHALL NOT be delivered twice | ok |
| `test_radiolink_retransmits_unacked_frame` | WHEN `service` is called after T1 expires THEN the frame SHALL be retransmitted | ok |

**Negative capability:** removing ACK queueing failed
`test_radiolink_acks_received_data`; removing frame retention failed both
`test_radiolink_retransmits_unacked_frame` and
`test_radiolink_suppresses_duplicate_frames`. The six contract tests stayed green
throughout, confirming they genuinely do not depend on the new paths.

## Regression contract — held, and it was genuinely at risk

Plan critique C-002 warned that T-044 could break the contract *structurally*
rather than through a defect: the tests run over `MockSdr` loopback, which echoes
everything back to the sender, so every ACK emitted would return as a received
frame on the very path those tests cover.

It held. The six carried-over `radio_it` tests pass **unchanged**, for two
structural reasons rather than luck:

- **ARQ is opt-in** (`set_reliable(true)`). A fire-and-forget link behaves
  exactly as before, so existing callers see no change in what goes on the
  channel.
- **Receiving never transmits as a side effect.** ACKs are queued during decode
  and leave only in `service()`, so `recv_datagram` cannot inject frames into a
  capture another assertion depends on.

| Suite | Result |
|---|---|
| `radio_it` — the 6 carried-over tests | ok, unchanged |
| `stream_it` (2), `loopback_it` (2), `readme_it` (1) | ok |
| `tunnel_it` (3), `ssh_tunnel_it` (2), `book_it` (6) | ok |
| `driver_cli_it` (3), `e2e_pipeline_tests` (5), `rigctl_server_it` (3) | ok |
| `cloudlog_it` (5), `pluto_iiod` (2), `fsk_roundtrip`, `fsk_timing` | ok |
| `hw_radio` (2) | ignored — requires the physical Pluto+ |

`node.rs`'s ACK fix is covered by `loopback_it`, which continues to pass: ACKs
produce no payload, so the datagram inbox is unaffected.

The two hardware tests were **not** re-run. No code they exercise changed in a
way they cover — `RadioLink` defaults to fire-and-forget, which is the mode they
use — and Sprint 7's live result stands.
