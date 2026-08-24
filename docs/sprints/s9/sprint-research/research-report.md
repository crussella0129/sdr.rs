# Sprint 9 Research Report

## Intents Reviewed
- [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) — **revised**; relevance: acceptance criterion 2 (ARQ retransmission under loss) is **not met** and the intent was marked `realized` on it in Sprint 7; current state: re-opened to `active`.
- [INT-0011](../../../intents/INT-0011-mesh-messaging-callsign.md) — **selected**; relevance: T-113 is its stated prerequisite; the scope finding below moves it out of this sprint; current state: `proposed`.
- [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) — **selected**; relevance: `RadioLink` is the receive path being changed; current state: `active`.

## 1. Sprint Goal

Wire genuine ARQ into the radio receive path (**T-113**) so a dropped frame is
retransmitted rather than silently truncating a stream. Research began as a
survey of where to plug the existing ARQ layer in, and found instead that the
retransmission half of that layer does not exist. The goal therefore becomes:
**build stop-and-wait ARQ properly, verify it against a lossy channel at the
30% loss INT-0006 criterion 2 names, wire it into `RadioLink`, and correct the
Book record that claimed this was already done.**

## 2. Existing Code Survey

| File | Relevance | Notes |
|------|-----------|-------|
| `crates/sdr-protocols/src/packet.rs` | **critical** | `ArqTransceiver` has `next_tx_seq`, `expected_rx_seq`, `max_retries`. It implements sequence numbering, duplicate suppression and ACK *generation*. It contains **no retransmission logic, no unacked-frame buffer, and no timer**. |
| `packet.rs:157` | **critical** | `pub max_retries: usize` — assigned `5` in `new()` and **never read anywhere in the workspace**. Verified by grep across all crates. |
| `packet.rs:216` | **critical** | `PacketType::Ack => Ok((None, None))` — a received ACK is discarded. Nothing is cleared, because no unacked buffer exists. |
| `crates/sdr-protocols/src/lib.rs:99` | **critical** | `test_arq_retransmission_lossy_channel`. Its two branches — `if !simulate_drop` and `else` — are **character-for-character identical**. No frame is ever dropped, no retransmission occurs, the ACK is discarded (`_ack`), and no loss is simulated despite the name. It proves sequence numbering over a lossless path. |
| `crates/sdr-mesh/src/radio.rs:152` | **critical** | `RadioLink::extract_payloads` calls `PacketFramer::decode` **directly**, bypassing `process_rx_frame`. On the real radio path there is therefore no sequence checking, no duplicate suppression and no ACK generation at all. |
| `radio.rs:157` | high | `self.arq` is used only for `local_addr` in an address comparison, and for `create_data_frame` on transmit. The transceiver is otherwise inert on this path. |
| `crates/sdr-mesh/src/node.rs:71` | high | `let (payload, _ack) = self.arq.process_rx_frame(&frame)?;` — the loopback path *does* run sequence checking, but **discards the ACK it generates**. It is never transmitted. |
| `crates/sdr-protocols/src/tunnel.rs:36` | medium | The only caller that keeps the ACK (`ack_opt`). `StreamTunnel` is not on the radio path. |
| `crates/sdr-mesh/src/stream.rs` | high | `StreamBridge` sits above `MeshInterface`; reliability must be solved beneath it, not here. |
| `crates/sdr-mesh/src/radio.rs:175` | high | `send_datagram` assigns a sequence number then transmits once. No copy is retained, so retransmission is impossible even in principle. |
| `docs/intents/INT-0006-...md` | **critical** | Criterion 2: "ARQ layer automatically acknowledges received frames (ACK) and **retransmits dropped packets under simulated RF packet loss up to 30%**." The second half is unimplemented and untested. |
| `docs/work/tasks.md` | high | T-113 as written ("wire sequence checking, duplicate suppression and retransmission into RadioLink's receive path") understates the work: retransmission must first be built. |
| `docs/roadmap.md` | high | Names INT-0011 ← T-113 as a load-bearing edge. The scope finding below affects Phase 1's shape. |

### The finding, stated plainly

**There is no retransmission anywhere in this codebase.** The "R" in ARQ is
absent. What exists is sequence numbering, duplicate suppression, and ACK frames
that are generated and then discarded by every caller on the radio path.

**This contradicts what I recorded in Sprint 7.** Closing that sprint I wrote
that "criterion 2's ARQ layer is real and tested under 30% simulated loss" and
marked INT-0006 `realized` partly on that basis. That was wrong. I took the name
of `test_arq_retransmission_lossy_channel` at face value without reading its
body; the test simulates no loss and performs no retransmission. The error is
mine, it is exactly the class of over-claim the Sprint 2 audit existed to
remove, and INT-0006 is re-opened below to correct it.

## 3. External Sources

- [AX.25 Link Access Protocol v2.2](https://www.ax25.net/AX25.2.2-Jul%2098-2.pdf) — the domain-standard answer to this exact problem. Confirms the shape to adopt: a retransmission timer (T1), a retry counter (N2), and explicit handling of half-duplex turnaround.
- [Stop-and-Wait ARQ (Univ. of Aberdeen)](https://www.erg.abdn.ac.uk/users/gorry/course/arq-pages/saw.html) — "Stop-and-wait ARQ is suitable for use with half-duplex transmission channels", and the sender "must rely upon a timer to detect the lack of a response". Confirms stop-and-wait is the right protocol for a half-duplex radio, and that the timer is the load-bearing component.
- [AX.25 protocol family manpage](https://manpages.ubuntu.com/manpages/focal/man4/ax25.4.html) — concrete parameter defaults: `AX25_T1` default **3000 ms**, `AX25_N2` retry count, and `AX25_BACKOFF` selecting exponential or linear backoff. Useful starting values rather than invented ones.

## 4. Risks, Unknowns, Dependencies

- **Risk — a wall-clock timer makes ARQ untestable.** Retransmission is defined
  by timeouts; if the implementation reads the system clock directly, tests
  either sleep (slow, flaky) or cannot exercise timeout paths at all. The clock
  must be injectable so tests advance time deterministically. This is the single
  most important design decision in the sprint.
- **Risk — the cyclic transmit buffer actively conflicts with ARQ.** Sprint 7
  measured that the Pluto's cyclic buffer repeats one frame indefinitely
  (T-115). Stop-and-wait requires *stopping* transmission and listening; a
  buffer that repeats forever cannot express "I have sent this once and am now
  waiting." Radio-level ARQ needs one-shot transmission with a controlled
  turnaround, which Sprint 5 found drains before the receiver observes it.
- **Risk — repeating the same class of error.** The defect found here is a test
  whose name asserted more than its body. Any new ARQ test must be checked by
  breaking the behaviour and watching it fail, as the Sprint 8 Book tests were.
- **Unknown — half-duplex turnaround latency on the Pluto.** The T1 timeout must
  exceed the real turnaround plus propagation, and that figure is unmeasured.
  AX.25's 3000 ms is a starting point for HF/VHF packet, almost certainly far
  too slow here, but nothing local has been measured.
- **Unknown — whether ARQ can be verified on hardware at all with one radio.**
  With a single device in internal loopback the station hears its own
  transmission, so an ACK exchange is degenerate — the same self-negotiation
  that stalls the SSH client in `ssh_tunnel_it`. **Meaningful hardware ARQ
  verification likely requires two radios**, which do not exist here.
  Simulation with a lossy channel model is the honest verification route.
- **Dependency — INT-0011 messaging slips.** T-113 is a full sprint on its own
  now that retransmission must be built rather than connected. Messaging should
  not be started in the same sprint; the roadmap's Phase 1 ordering is unchanged
  but its contents split across two sprints.
- **Dependency — T-115** (cyclic buffer) is entangled with radio-level ARQ, per
  the risk above.

## 5. Recommended Approach

**Primary: build stop-and-wait ARQ with an injectable clock, verify it in
simulation against a lossy channel, then wire it into `RadioLink`.**

1. **Complete `ArqTransceiver`.** Retain each transmitted frame with its
   sequence number and send time; clear it when a matching ACK arrives (fixing
   `PacketType::Ack => (None, None)`); expose a `poll_timeouts(now)` that
   returns frames due for retransmission; enforce `max_retries` and surface
   permanent failure to the caller rather than silently giving up. Adopt AX.25's
   shape — T1 timeout, N2 retries, backoff — with locally chosen values.
2. **Make time injectable.** A `Clock` seam (trait or an explicit `now`
   parameter) so tests advance time deterministically. No `SystemTime::now()`
   inside the state machine.
3. **Build a lossy channel model** that drops frames at a configured rate with a
   seeded RNG, so a 30% loss test is reproducible rather than flaky. This is
   what INT-0006 criterion 2 actually requires and has never had.
4. **Wire `RadioLink` to `process_rx_frame`** instead of `PacketFramer::decode`,
   and **transmit the returned ACK**. Fix `node.rs`'s discarded `_ack` likewise.
5. **Replace `test_arq_retransmission_lossy_channel`** — it is actively
   misleading and must not survive in its current form.
6. **Verify every new test's failure path** by breaking the behaviour, as in
   Sprint 8.

**Alternative considered — selective-repeat or a sliding window.** Higher
throughput, and INT-0006's intent text mentions sliding-window ARQ. Rejected for
this sprint: stop-and-wait is what a half-duplex channel supports naturally, it
is far simpler to verify, and there is no measured throughput requirement to
justify the complexity. Recorded as a later option, not a silent omission.

**Alternative considered — implement ARQ only in `RadioLink`.** Rejected: it
would leave `ArqTransceiver` a misleading half-implementation and duplicate the
state machine. The fix belongs where the type already claims to live.

**Alternative considered — hardware verification this sprint.** Rejected on the
evidence above: one radio in loopback cannot meaningfully exercise an ACK
exchange. Simulation is the honest route, and the limitation is recorded rather
than worked around.

**Rationale.** The gap is larger than T-113's wording implied, and the Book
currently claims a capability that does not exist. Correcting the record and
building the real thing — verified against actual simulated loss, with
deterministic tests — is the whole of this sprint. Messaging follows once the
transport is genuinely reliable.

## Artifacts
- Verified by direct inspection: `max_retries` never read; no
  retransmit/timeout/unacked logic anywhere in `sdr-protocols` or `sdr-mesh`;
  `test_arq_retransmission_lossy_channel`'s branches identical;
  `RadioLink::extract_payloads` bypasses `process_rx_frame`; `node.rs` discards
  its ACK.
- [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) — revised and
  re-opened to `active`.
