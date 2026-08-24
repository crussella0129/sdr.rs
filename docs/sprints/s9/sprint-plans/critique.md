# Plan Critique — Sprint 9

## Concerns

### C-001: the test plan invents EARS clauses the build plan does not contain
- **Where:** `test-plan.md` Intent Traceability vs `build-plan.md` T-042/T-043 EARS
- **Quote:** "#2 no premature retransmit | T-042 / WHEN T1 has not expired THEN nothing SHALL be returned"
- **Failure mode:** plan-test-mismatch
- **Why it matters:** Three clauses appear only in the test plan — "no premature
  retransmit" (T-042), "lost ACKs do not duplicate" and the 10% case (T-043).
  The build plan is the locked statement of what the sprint promises; a clause
  that exists only in the verification document is a promise nobody committed
  to, and the direction of authority is backwards. Both are locked atomically,
  so the inconsistency would be frozen in.
- **Suggested response:** fix-in-plan — add the three clauses to the build plan's
  EARS lists so every test traces to a committed promise.

### C-002: T-044 could break the regression contract for structural reasons
- **Where:** `build-plan.md` T-044 / `test-plan.md` Integration Tests
- **Quote:** "the six existing `radio_it` tests **unchanged** as the regression contract"
- **Failure mode:** hidden-dep
- **Why it matters:** T-044 makes the radio receive path transmit ACKs. The
  regression tests run over `MockSdr` **loopback, which echoes everything back
  to the sender**. So every ACK the receiver emits returns to it as a received
  frame, and `send_datagram` is no longer the only thing putting frames on the
  channel. If ACK frames were surfaced as payloads, or if they displaced data
  frames in a capture, tests like
  `test_radiolink_datagram_roundtrip_over_mock` would fail — not from a defect
  but from the design change itself. That is exactly the situation where the
  contract gets quietly rebaselined instead of respected.
- **Suggested response:** fix-in-plan — state explicitly that
  `process_rx_frame`'s `PacketType::Ack` arm must consume ACKs without producing
  a payload, so ACK traffic never reaches the datagram inbox, and that any
  regression-test failure is to be treated as a real defect rather than an
  expected consequence.

### C-003: T-042's touched paths omit the callers its API change breaks
- **Where:** `build-plan.md` T-042 "Touches"
- **Quote:** "**Touches:** `crates/sdr-protocols/src/packet.rs`"
- **Failure mode:** hidden-dep
- **Why it matters:** Retaining transmitted frames and queueing ACKs changes
  `ArqTransceiver`'s surface. Four call sites exist —
  `crates/sdr-mesh/src/radio.rs:175`, `crates/sdr-mesh/src/node.rs:71` and `:86`,
  `crates/sdr-protocols/src/tunnel.rs:36`, plus the crate's own tests. T-044
  covers the two mesh files; **`tunnel.rs` is named in no task at all**, so if
  the change breaks it the work lands outside any task boundary and outside the
  commit it belongs to.
- **Suggested response:** fix-in-plan — add `crates/sdr-protocols/src/tunnel.rs`
  to T-042's touched paths, or hold the existing signatures additively so no
  caller breaks. Additive is preferable: `StreamTunnel` is the one caller that
  already handles its ACK correctly, and there is no reason to disturb it.

## Resolutions (primary agent)

| Concern | Response | Action |
|---|---|---|
| C-001 | fix-in-plan | The three clauses added to the build plan's T-042 and T-043 EARS lists, so every planned test traces to a committed promise. |
| C-002 | fix-in-plan | T-044 now states that ACKs must be consumed by the state machine and never surfaced as datagrams, and that a regression-contract failure is a defect to fix rather than a baseline to move. |
| C-003 | fix-in-plan | `tunnel.rs` added to T-042's touched paths, with the explicit preference for an **additive** API change so existing callers keep working. |

Re-screened after the changes: every EARS clause maps to a named test, every
planned test traces to a clause, and no task's touched paths omit a file its
change reaches.

## Confidence
proceed-with-caveats
