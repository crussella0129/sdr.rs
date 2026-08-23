# Sprint 6 Research Report — Symbol-timing recovery for the mesh radio link (T-110)

Replace `RadioLink`'s brute-force sample-phase search with real symbol-timing
recovery, so the mesh receiver can lock to a transmitter whose clock differs
from its own. This is the standing prerequisite for a two-radio link, recorded
as backlog T-110 when Sprint 5 shipped the phase search.

## Intents Reviewed
- [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) — **selected**; primary. Sprint 5 met criterion 1 over a real radio, but only under a loopback that shares one clock between transmitter and receiver. Timing recovery removes that caveat and is required before criterion 2 (multi-hop across separate nodes) can be attempted honestly.
- [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) — **selected** (context): owns `sdr-dsp`, where `GardnerClockRecovery` lives. Its acceptance criterion 4 covers synchronization loops; this sprint gives that component its first real verification.
- [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) — **selected** (context): owns `FskDemod`, the component gaining a timing-recovered variant. No criterion changes.

## 1. Sprint Goal
Give the FSK receive path proper symbol-timing recovery and wire it into
`RadioLink` in place of the sample-phase search, so datagram recovery survives a
transmitter/receiver clock offset. Verify in CI across a characterized drift
range and confirm on the physical Pluto+ under internal loopback.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| crates/sdr-dsp/src/clock_recovery.rs | high | `GardnerClockRecovery`: Gardner TED + 4-point cubic Hermite interpolator, `omega` clamped to ±10% of nominal. Written in Sprint 0; **never verified for correctness** (see §4.1). |
| crates/sdr-dsp/src/lib.rs (`test_gardner_clock_recovery`) | high | The only existing test asserts `!output.is_empty()` and a loose symbol count — it never checks the recovered symbol *values*. A broken TED would pass it. |
| crates/sdr-demod/src/fsk.rs | high | `FskDemod` computes the discriminator internally, then averages over a fixed `samples_per_symbol` count — the fixed counting is exactly what timing recovery replaces. The discriminator itself is reusable. |
| crates/sdr-mesh/src/radio.rs | high | `extract_payload` loops over `0..sps` candidate phases. This is the code being replaced; its doc comment already names `sdr_dsp::clock_recovery` as the intended successor. |
| crates/sdr-demod/src/psk.rs | medium | The one existing consumer of `GardnerClockRecovery` — feeds it raw IQ, which is correct for PSK (a linear modulation) but not for FSK (see §4.2). |
| crates/sdr-demod/tests/fsk_roundtrip.rs | medium | Sprint 5 regression pinning the modulator↔demodulator pair; the new path must not regress it. |
| crates/sdr-mesh/tests/{radio_it,hw_radio}.rs | high | Existing CI and live coverage that must keep passing through the swap. |

## 3. External Sources
- **In-repo measurement (primary authority).** The questions are properties of
  this codebase's Gardner implementation against this project's FSK signal, and
  were measured directly (§4). No external sources were required.

## 4. Measured findings

### 4.1 Gardner was never actually verified
`test_gardner_clock_recovery` asserts only that output is non-empty and roughly
the right length. It never compares recovered symbols to the transmitted ones,
so the loop could be producing nonsense and still pass. Any claim built on this
component needs its own evidence — which §4.3 now supplies.

### 4.2 Gardner must be fed the **discriminator output**, not raw FSK IQ
Gardner's timing-error detector is `mid·(cur − last)` on the sample values,
which assumes a linear modulation whose symbol transitions pass through zero.
FSK is constant-envelope: every IQ sample has magnitude 1 and only the rotation
rate carries information, so the TED has nothing to lock onto. The correct
pipeline converts first:

```
IQ → frequency discriminator (arg(s·conj(prev))) → real-valued PAM
   → Gardner timing recovery → slice at 0 → bits
```

The existing PSK consumer feeds raw IQ, which is right for PSK and would have
been the wrong pattern to copy for FSK.

### 4.3 Post-discriminator, Gardner recovers FSK exactly — including under drift
Modulating `AA D3 91 4B 1E 77 00 FF 5A C3` (1 MSPS, 100 kHz deviation, 10 sps),
then discriminating, running Gardner, and slicing. "Drift" resamples the stream
so the receiver's effective clock differs from the transmitter's, on top of a
7-sample start offset:

| condition | bit errors |
|---|---|
| aligned | 0 / 80 |
| **7-sample offset** | **0 / 80** |
| drift ±0.1% | 0 / 80 |
| **drift ±0.5%** | **0 / 79–80** |
| drift ±1% | 10–12 / 79 |
| drift ±2% | 18 / 57–80 |
| drift ±5%, ±8% | 19–27 (garbage) |

Two results matter. First, the 7-sample offset — the case that produced **37/71
bit errors** with the current fixed-count demodulator (Sprint 5 §4.2) — recovers
with **zero errors**. Second, drift is handled at all, which the sample-phase
search fundamentally cannot do: a single phase choice cannot track a clock that
is continuously slipping.

### 4.4 Drift tolerance is ~±0.5%, which is ~100× more than real radios need
Clean recovery holds to ±0.5% and breaks down by ±1%, consistent with the loop's
`omega_rel_limit` of ±10% being far wider than the loop gains can usefully track
at the default settings (`gain_mu = 0.01`, `gain_omega = 0.001`). For scale, real
crystal oscillators are specified at ±10–50 ppm (±0.001–0.005%), so ±0.5%
(±5000 ppm) leaves roughly two orders of magnitude of margin. Gain tuning is
available if a wider range is ever wanted, but is not needed for this purpose.

### 4.5 Recovered streams carry a 0–1 symbol lag
The loop's first output lands at an arbitrary point in the first symbol, so the
recovered bit stream may lead or lag by one symbol. This is harmless here: the
receiver already locates frames by searching for the sync word
(`framesync::sync_to_frame`), which absorbs a constant offset. No extra handling
is needed.

## 5. Risks, Unknowns, Dependencies
- **Risk — regressing a working path.** `RadioLink` currently passes in CI and on
  hardware. *Mitigation:* keep the existing tests unchanged as the contract, add
  drift coverage, and treat any change in their behaviour as a failure.
- **Risk — Gardner's correctness rests on this sprint's evidence alone.** It is
  unproven in the repo today (§4.1). *Mitigation:* strengthen
  `test_gardner_clock_recovery` to assert recovered *values*, not just length, so
  the component carries its own regression coverage independent of the mesh.
- **Unknown — loop gains under a real (noisy) channel.** Everything measured here
  is noiseless. The digital loopback is also noiseless, so this sprint cannot
  settle noise behaviour; that belongs with a real channel.
- **Constraint — testing stays on internal loopback**, per the standing choice;
  anything on-air remains backlog T-108 pending explicit go-ahead.
- **Dependency — none new.** `sdr-mesh` already depends on `sdr-demod`; adding
  `sdr-dsp` is a workspace-internal edge with no external crates.

## 6. Recommended Approach
1. **Strengthen `test_gardner_clock_recovery`** to assert the recovered symbol
   values match the transmitted alternating pattern, giving the component real
   coverage before anything depends on it.
2. **Add a timing-recovering FSK demodulator** in `sdr-demod` (e.g.
   `FskTimingDemod`) implementing discriminator → Gardner → slice, alongside the
   existing `FskDemod` rather than replacing it (PSK and the Sprint 5 regression
   both still use the current behaviour).
3. **Switch `RadioLink::extract_payload`** to a single demodulation pass through
   the new path, deleting the `0..sps` loop. Keep the sync-word + CRC-32 check,
   which still provides frame alignment and validation.
4. **CI verification:** the existing `radio_it` tests unchanged, plus a new drift
   test asserting datagram recovery across the characterized range, and a bit-
   level drift test in `sdr-demod`.
5. **Live verification** on the Pluto+ under internal loopback, confirming the
   swap holds on real hardware.
6. **Out of scope:** noise/BER characterization, loop-gain auto-tuning, GFSK,
   over-the-air, Phase B (`tun`/babeld), multi-vendor.

## 7. Artifacts
- `docs/sprints/s6/sprint-research/research-report.md` — this report.
- Measurements in §4.3/§4.4 produced by a temporary probe
  (`crates/sdr-demod/tests/scratch_timing.rs`), run and then removed; the tables
  above are the durable record.
