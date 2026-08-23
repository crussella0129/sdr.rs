# Plan Critique — Sprint 7

## Concerns

### C-001: "SSH over radio" is the headline, and the evidence is weaker than the phrase
- **Where:** `build-plan.md` T-038 / `INT-0006` criterion 4
- **Quote:** "a real OpenSSH **client** launched with the bridge as its `ProxyCommand` exchanges protocol version strings across the link"
- **Failure mode:** intent-drift
- **Why it matters:** With no local `sshd`, the "peer" the client talks to is itself — the loopback echoes its banner back, which happens to be a valid `SSH-2.0-…` string. That proves bidirectional byte transit with a real client at one end; it is emphatically **not** a working SSH session. The gap between "SSH over radio works" and what is demonstrated is exactly the kind of over-claim this intent already accumulated once.
- **Suggested response:** fix-in-plan (done) — criterion 4 was rewritten **before** this plan locked to say what is demonstrable, with both limits stated in the criterion itself rather than buried in a report. The test plan repeats "what this shows / what it does not". The intent stays `active`, not `realized`, at Loop.

### C-002: No ARQ retransmission means a lossy channel silently truncates streams
- **Where:** `build-plan.md` T-036 / `crates/sdr-mesh/src/radio.rs`
- **Failure mode:** missing-risk
- **Why it matters:** SSH runs over TCP precisely because it needs reliable ordered bytes. `RadioLink` validates CRC-32 but never calls `ArqTransceiver::process_rx_frame`, so there is no sequence checking, duplicate suppression, or retransmit. Every test here runs over a lossless loopback, so the gap is invisible in all of them — a stream would simply lose bytes on a real link, with no error raised.
- **Suggested response:** defer-with-rationale, recorded as backlog — building reliability is a sprint of its own (retransmit timers, windowing, a lossy-channel model) and would swamp the goal of making the bridge exist. It must be named in the test report and the intent rather than left for a future reader to discover, and no claim of "reliable" may be made.

### C-003: The stdio pump risks deadlock or a busy-wait
- **Where:** `build-plan.md` T-037
- **Quote:** "A reader thread feeds a channel so the main loop can poll stdin and the radio"
- **Failure mode:** flake-risk
- **Why it matters:** Blocking reads on stdin and blocking radio captures in one loop is the classic bidirectional-pipe deadlock; the naive fix (tight polling) burns a core and can starve either direction. A hung `tunnel` process would also hang the CI tests that spawn it.
- **Suggested response:** fix-in-plan — the reader thread keeps stdin off the main loop, the main loop polls with a bounded sleep rather than spinning, and every spawned-process test carries a timeout and kills the child on drop, as the existing rigctl tests do.

### C-004: Raising `rx_chunk` costs latency on every receive
- **Where:** `build-plan.md` T-035
- **Failure mode:** hidden-dep
- **Why it matters:** A larger capture means each `recv_datagram` waits for more samples before returning anything, so a bigger MTU ceiling is bought with per-datagram latency — which a stream feels more than a single datagram does.
- **Suggested response:** defer-with-rationale — correctness first: today payloads over ~180 bytes fail outright, which is worse than slow. Throughput and latency are explicitly out of scope this sprint and named as backlog; the MTU is configurable so the trade-off can be tuned once there is something to measure.

## Confidence
proceed-with-caveats
