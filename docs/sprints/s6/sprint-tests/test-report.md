# Sprint 6 Test Report

- **Tested head:** `2821e3fc480b7eaf6fde937dfc12252ff8ea8680`
- **Runner:** `cargo test --workspace` — **100 passed, 0 failed, 4 ignored** (hardware). `cargo clippy --workspace --all-targets` — **0 errors**.
- **Live hardware:** `hw_verify_mesh_datagram_over_radio` passes with the phase search removed.
- **Critique verdict:** proceed-with-caveats (see `critique.md`).

## Suite results
| Suite | Kind | Result |
|-------|------|--------|
| sdr-dsp lib (incl. Gardner) | unit | 8 passed |
| sdr-demod lib | unit | 7 passed |
| sdr-demod fsk_roundtrip (Sprint 5 regression) | unit | 2 passed, unchanged |
| sdr-demod fsk_timing | unit | 3 passed |
| sdr-mesh lib | unit | 10 passed |
| sdr-mesh radio_it | integration | 4 passed (3 carried over unchanged) |
| sdr-mesh hw_radio | live hardware (agent) | 1 passed (ignored in CI) |
| all other workspace crates | — | unchanged, green |

## What changed
`RadioLink` now recovers datagrams in a **single** demodulation pass through
`FskTimingDemod` (frequency discriminator → Gardner timing recovery → slice).
The `0..samples_per_symbol` candidate-phase loop is gone. The receiver no longer
depends on transmitter and receiver sharing a clock — the caveat Sprint 5 shipped
with and recorded as T-110.

## Two corrections this sprint made, rather than papered over

**1. Frames needed trailing flush symbols.** The first switch to timing recovery
broke two of the three carried-over Sprint 5 tests. Diagnosis showed the timing
loop consumes a symbol settling at the start of a burst, shifting its output
stream so each frame's final CRC byte was truncated. A measurement across four
different payloads showed `fixed=true, timing=false, timing+pad=true` in every
case. Fixed by appending a 2-byte `TRAILER` — standard postamble practice, and
its alternating pattern also keeps the loop supplied with transitions while it
flushes.

**2. The research report's drift figure did not survive contact with a frame.**
Research measured ±0.5% on a short (~80-bit) burst. Across a full ~256-bit frame,
where every bit must survive for CRC-32 to pass, the limit is about **±0.1%**,
and it is slightly asymmetric (a receiver clock running fast is the tighter
direction). A five-point loop-gain sweep confirmed tuning does **not** widen it,
so the limit is structural to this Gardner implementation. The claim was
corrected in `radio.rs`, `fsk.rs`, the drift test and the completion record
rather than asserting the optimistic number; widening it is backlog **T-112**.

Both were caught because the Sprint 5 tests were carried over as an explicit
regression contract instead of being rebaselined.

## Intent verification
| Intent | Criterion | Verdict |
|--------|-----------|---------|
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | #4 synchronization loops hold timing | **Verified** — `GardnerClockRecovery` has real correctness coverage for the first time (values, non-periodic sequence, plus a drift case). |
| [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) | #3 bursts demodulable | **Verified** — bit-exact aligned, mid-symbol, and under short-burst drift. Existing behaviour unchanged. |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 datagrams recovered without corruption | **Strengthened** — no longer requires a shared clock; tolerates ±0.1% clock offset in simulation, re-verified on real hardware. |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 `tun`, #2 multi-hop, #4 addressing | **Carried forward** — Phase B (T-107). |

## Honest limits
- Drift is verified **in simulation** (clean interpolation resampling: no noise,
  no jitter, no phase noise). The live test cannot show drift because the
  loopback shares one clock.
- Two separate radios exchanging datagrams remains untested — needs a second
  device and over-the-air operation (T-108, explicit go-ahead required). This
  sprint removes the shared-clock blocker; it does not clear the others.
- ±0.1% is a *simulated frame-level* figure, not a specification.

## Conclusion
The mesh receiver no longer depends on a shared clock, and the component it
relies on has real coverage for the first time. **INT-0008 remains `active`**:
criterion 1 is strengthened, while `tun`, multi-hop routing and addressing stay
in Phase B and over-the-air stays gated. No re-architecture failure; proceed to
Loop.
