# Sprint 9 — Unit Test Results

- **Tested head:** `454e821d2c91cd05de40ed9f6867efd845322290`
- **Runner:** `cargo test --workspace --lib`
- **Date:** 2026-08-24
- **CI authority:** no hosted CI on this repository; the local workspace runner
  is the canonical suite.

## T-042 — `ArqTransceiver` state machine

`crates/sdr-protocols/src/packet.rs`, module `arq_tests` — **5 passed**.

Every test drives time explicitly through `now_ms`. Nothing sleeps, nothing
reads the system clock, and the whole module completes in under a millisecond.
That is the point of the design decision: a state machine that reads the clock
itself can only be tested by sleeping, which is how timeout coverage ends up
slow, flaky, and eventually skipped.

| Test | EARS clause | Result |
|---|---|---|
| `test_arq_retransmits_after_timeout` | WHEN T1 expires without an ACK THEN the frame SHALL be returned for retransmission | ok |
| `test_arq_no_retransmission_before_timeout` | WHEN T1 has not expired THEN nothing SHALL be returned | ok |
| `test_arq_ack_stops_retransmission` | WHEN a matching ACK arrives THEN the frame SHALL be cleared and SHALL NOT be retransmitted | ok |
| `test_arq_gives_up_after_max_retries` | WHEN retried `max_retries` times THEN the frame SHALL be abandoned and reported as a permanent failure | ok |
| `test_arq_duplicate_suppressed_but_acked` | WHEN a duplicate data frame arrives THEN its payload SHALL be suppressed and an ACK SHALL still be produced | ok |

### Negative capability verified for all five

This sprint exists because a test's name asserted more than its body delivered,
so every test here was checked by breaking the behaviour and observing the
failure.

| Behaviour broken | Tests that failed |
|---|---|
| ACK arm reverted to discarding (the original defect) | `test_arq_ack_stops_retransmission` |
| T1 comparison removed (retransmit unconditionally) | `test_arq_no_retransmission_before_timeout` |
| Frame retention removed | `test_arq_retransmits_after_timeout`, `test_arq_gives_up_after_max_retries` |
| Duplicate suppression removed | `test_arq_duplicate_suppressed_but_acked` |

Each restored cleanly to 5 passed afterwards.

Note that `test_arq_no_retransmission_before_timeout` exists specifically so the
timeout test cannot pass vacuously: an implementation that retransmits
everything unconditionally satisfies "retransmits after timeout" while being
useless. The pair only means something together.

## Full workspace unit tally

| Crate | Passed | Failed |
|---|---|---|
| sdr-core | 8 | 0 |
| sdr-dsp | 25 | 0 |
| sdr-demod | 8 | 0 |
| sdr-hardware | 25 | 0 |
| sdr-mesh | 14 | 0 |
| sdr-protocols | **14** (5 new + 4 reliability + 5 pre-existing) | 0 |
| sdr-spectrum | 2 | 0 |
| sdr-station | 4 | 0 |

## Lint

`cargo clippy --workspace --all-targets` — **0 errors**. Pre-existing warnings
remain backlog T-105; this sprint added none.
