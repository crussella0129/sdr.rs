Finalized - DO NOT EDIT

# Sprint 9 Build Plan

**Goal: real ARQ (T-113).**

## Context

T-113 was scoped as "wire the existing ARQ layer into `RadioLink`'s receive
path". Research found there is nothing to wire. **The retransmission half of ARQ
does not exist anywhere in this codebase:**

- `ArqTransceiver` retains no transmitted frame, so retransmission is impossible
  in principle.
- `max_retries` is assigned `5` in `new()` and never read anywhere.
- `PacketType::Ack => Ok((None, None))` — a received ACK is discarded.
- No timer, no unacked buffer, no backoff.
- `RadioLink::extract_payloads` calls `PacketFramer::decode` **directly**,
  bypassing `process_rx_frame` — the radio path has no sequence checking, no
  duplicate suppression and no ACK generation at all.
- `node.rs:71` generates an ACK and discards it (`_ack`).
- `test_arq_retransmission_lossy_channel` has two character-for-character
  identical branches, simulates no loss, and performs no retransmission.

INT-0006 has been re-opened to `active`: criterion 2 is unmet and never was met.
That corrects an error I made closing Sprint 7, where I marked the intent
realized on the strength of that test's *name*.

This sprint builds the real thing.

## Design decision: time is a parameter, not a clock read

Retransmission is defined by timeouts, so the state machine must never read the
system clock — tests would have to sleep, making them slow and flaky, and
timeout paths would be effectively untestable.

**Every ARQ method that depends on time takes an explicit `now_ms: u64`.** The
caller supplies it: the CLI passes real monotonic time, tests pass whatever they
like and advance it instantly. No `Clock` trait, no trait objects, no
`SystemTime::now()` anywhere in the state machine. This is the single most
important decision in the sprint and everything else follows from it.

Parameter shape follows AX.25 (see research): a T1 retransmission timeout, an N2
retry limit, and backoff — with values chosen locally rather than inheriting
AX.25's 3000 ms, which targets a much slower channel.

## Intent states

INT-0006 was re-opened `realized -> active` during research (criterion 2 unmet).
INT-0008 is already `active`. Both are preserved as `active` here with no
duplicate transition entry, per the phase contract.

## Tasks

### T-042: Complete `ArqTransceiver` with retransmission
- **Intent:** INT-0006 (criterion 2)
- **Touches:** `crates/sdr-protocols/src/packet.rs`, `crates/sdr-protocols/src/tunnel.rs` (call site)
- **Depends on:** nothing
- **API shape:** prefer an **additive** change — keep `create_data_frame` and
  `process_rx_frame`'s existing signatures working so the four existing call
  sites (`radio.rs:175`, `node.rs:71`/`:86`, `tunnel.rs:36`) are not disturbed.
  `StreamTunnel` already handles its ACK correctly and there is no reason to
  break it (plan critique C-003).
- Retain each transmitted frame with its sequence number, send time and retry
  count. Clear it when a matching ACK arrives — fixing the discarded-ACK arm.
  Add `due_retransmissions(now_ms)` returning frames whose T1 has expired,
  incrementing their retry count and re-arming the timer with backoff. Enforce
  `max_retries`: past N2 the frame is abandoned and surfaced to the caller as a
  **permanent failure**, never silently dropped. Queue generated ACKs for the
  caller to transmit rather than returning them to be discarded.
- **EARS:**
  - WHEN a data frame is transmitted and no ACK arrives before T1 expires, THEN
    `due_retransmissions` **SHALL** return that frame for retransmission.
  - WHEN a matching ACK arrives, THEN the frame **SHALL** be cleared and
    **SHALL NOT** be retransmitted thereafter.
  - WHEN a frame has been retransmitted `max_retries` times without an ACK,
    THEN it **SHALL** be abandoned and reported as a permanent failure.
  - WHEN a duplicate data frame arrives, THEN its payload **SHALL** be suppressed
    and an ACK **SHALL** still be produced.
  - WHEN T1 has **not** expired, THEN `due_retransmissions` **SHALL** return
    nothing — so a retransmit-everything implementation cannot pass the timeout
    clause vacuously.
- **Tests:** `test_arq_retransmits_after_timeout`, `test_arq_ack_stops_retransmission`,
  `test_arq_gives_up_after_max_retries`, `test_arq_duplicate_suppressed_but_acked`,
  `test_arq_no_retransmission_before_timeout`.

### T-043: Seeded lossy channel and reliability under 30% loss
- **Intent:** INT-0006 (criterion 2)
- **Touches:** `crates/sdr-protocols/src/lib.rs` (tests), new test module
- **Depends on:** T-042
- A channel model that drops frames at a configured rate using a **seeded** RNG,
  so a loss test is reproducible rather than flaky. Drive two `ArqTransceiver`s
  through it with deterministic time and assert every payload arrives, in order,
  exactly once — at 0%, 10% and **30%** loss, the figure INT-0006 criterion 2
  names and has never had.
- **Delete `test_arq_retransmission_lossy_channel`.** It is actively misleading
  and must not survive; its replacement carries the same responsibility honestly.
- **EARS:**
  - WHEN frames are exchanged over a channel dropping 30% of them, THEN every
    payload **SHALL** be delivered exactly once and in order.
  - WHEN the same seed is used, THEN the result **SHALL** be identical run to run.
  - WHEN frames are exchanged over a channel dropping 10% of them, THEN every
    payload **SHALL** be delivered exactly once and in order.
  - WHEN an **ACK** is lost, forcing retransmission of an already-delivered
    frame, THEN the payload **SHALL NOT** be delivered twice.
- **Tests:** `test_arq_delivers_all_payloads_at_30pct_loss`,
  `test_arq_delivers_all_payloads_at_10pct_loss`, `test_arq_lossy_channel_is_deterministic`,
  `test_arq_ack_loss_does_not_duplicate_payload`.

### T-044: Run ARQ on the mesh receive paths
- **Intent:** INT-0008, INT-0006
- **Touches:** `crates/sdr-mesh/src/radio.rs`, `crates/sdr-mesh/src/node.rs`
- **Depends on:** T-042
- Route decoded frames through `process_rx_frame` instead of consuming
  `PacketFramer::decode` output directly, so the radio path gains sequence
  checking, duplicate suppression and ACK generation. Add
  `RadioLink::service(now_ms)` which transmits queued ACKs and any due
  retransmissions. Fix `node.rs`'s discarded `_ack` so the loopback path
  actually delivers acknowledgements.
- `extract_payloads` still needs `PacketFramer::decode` to compute `consumed`
  for the scan cursor; the same frame slice is then handed to the state machine.
- **ACKs must never reach the datagram inbox** (plan critique C-002). The
  regression tests run over `MockSdr` loopback, which echoes everything back to
  the sender, so every ACK emitted returns as a received frame. The
  `PacketType::Ack` arm must consume it and produce no payload. If a
  regression-contract test fails after this change, that is a **defect to fix,
  not a baseline to move** — the contract exists precisely because this task
  alters the path those tests cover.
- **EARS:**
  - WHEN a data frame is received over the radio, THEN an ACK **SHALL** be
    queued for transmission.
  - WHEN a duplicate frame is received over the radio, THEN its payload
    **SHALL NOT** be delivered twice.
  - WHEN `service` is called after T1 expires with a frame unacknowledged, THEN
    that frame **SHALL** be retransmitted.
- **Tests:** `test_radiolink_acks_received_data`,
  `test_radiolink_suppresses_duplicate_frames`,
  `test_radiolink_retransmits_unacked_frame`, plus the six existing `radio_it`
  tests **unchanged** as the regression contract.

## Files created / modified
- `crates/sdr-protocols/src/packet.rs` — the state machine (T-042)
- `crates/sdr-protocols/src/lib.rs` — replace the misleading test (T-043)
- `crates/sdr-mesh/src/{radio,node}.rs` — wire the receive paths (T-044)

No new dependencies: the seeded RNG is a small xorshift in test code rather than
a crate, keeping the dependency surface unchanged for a test-only need.

## Verification
- `cargo test --workspace` — all suites green; 33 suites at Sprint 8 close.
- **Negative capability checked, not assumed.** Sprint 9 exists because a test
  asserted more than its body delivered. Every new ARQ test must be shown to
  fail when the behaviour is broken — the same discipline applied to the Sprint 8
  Book tests.
- `cargo fmt` + `cargo clippy --workspace --all-targets` clean.
- **Regression contract:** the six `radio_it` tests and both `hw_radio` tests
  carry forward unchanged.

## Honest limits to state, not imply away
- **No hardware verification of ARQ.** One radio in internal loopback hears its
  own transmission, so an ACK exchange is degenerate — the same self-negotiation
  that stalls the SSH client in `ssh_tunnel_it`. Meaningful hardware ARQ needs
  **two radios**, which do not exist here. Simulation is the verification route
  and the report must say so plainly rather than implying hardware coverage.
- **The cyclic transmit buffer conflicts with stop-and-wait** (T-115). A buffer
  that repeats one frame indefinitely cannot express "sent once, now listening".
  Radio-level ARQ on the Pluto needs one-shot transmission with controlled
  turnaround; that is not solved here.
- **Stop-and-wait, not sliding window.** INT-0006's intent text mentions
  sliding-window ARQ. Stop-and-wait is what a half-duplex channel supports
  naturally and is far simpler to verify; the window is a later option, recorded
  rather than silently omitted.
- **T1 is unmeasured.** Half-duplex turnaround latency on real hardware is
  unknown, so the chosen timeout is a reasoned default, not a measured one.

## Out of scope (→ backlog / later sprints)
- INT-0011 messaging — slips to a later sprint; T-113 is a full sprint now that
  retransmission must be built rather than connected.
- T-115 (cyclic buffer / one-shot transmission with turnaround).
- Sliding-window ARQ; measuring T1 on hardware; two-radio verification (T-108
  territory).
