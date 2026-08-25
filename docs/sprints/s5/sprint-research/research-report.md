# Sprint 5 Research Report — Mesh over a real radio (internal loopback)

Carry an IP datagram end-to-end through the actual hardware: mesh datagram →
KISS → ARQ frame → FSK modulator → **real PlutoSDR TX** → AD9361 internal
digital loopback → RX → demodulator → framing → datagram recovered, verified
under **internal loopback** per the user's standing choice.

## Intents Reviewed
- [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) — **selected**; primary. Phase A gave a `MeshInterface` seam with an in-memory `LoopbackLink`. This sprint provides the first *radio-backed* implementation, advancing criterion 1 (datagrams framed over the link and recovered) from in-process to real hardware.
- [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) — **selected** (context): supplies the ARQ/framing layer and the modulator. Its intent body claimed "TX Driver Integration … PlutoSDR AD9361 TX path", realized in Sprint 1 against `MockSdr` only; this sprint makes that true on hardware. No criterion of INT-0006 changes.
- [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) — **selected** (context): supplies the now-working TX path and loopback safety controls (Sprint 4). Not advanced here.

## 1. Sprint Goal
Implement a radio-backed `MeshInterface` (`RadioLink`) that transmits mesh
datagrams as modulated IQ through any `SdrDriver` and recovers them from
received IQ, then prove it end-to-end: in CI over `MockSdr` loopback, and live
over the PlutoSDR's internal digital loopback. Deliver honest evidence of the
first true "IP over radio" transit.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| crates/sdr-mesh/src/node.rs | high | `MeshInterface` trait (`send_datagram`/`recv_datagram`) is the exact seam to implement; `LoopbackLink` shows the KISS + `ArqTransceiver` composition to mirror. |
| crates/sdr-mesh/src/kiss.rs | high | `encode` + streaming `KissDecoder` — reused unchanged for datagram boundaries. |
| crates/sdr-protocols/src/packet.rs | high | `PacketFramer::encode/decode` (PREAMBLE `0xAA`×4, SYNC `0xD3 0x91 0xD3 0x91`, CRC-32) and `ArqTransceiver`. `decode` already **searches for the sync word**, which is the byte-level alignment mechanism. |
| crates/sdr-demod/src/modulator.rs | high | `FskModulator` (±deviation, continuous phase) and `GfskModulator`. `modulate_bytes` is MSB-first. |
| crates/sdr-demod/src/fsk.rs | high | `FskDemod` — quadrature discriminator, averages instantaneous frequency over `samples_per_symbol`, slices at 0. **No symbol-timing recovery**: it simply counts samples, so the stream must start near a symbol boundary. |
| crates/sdr-hardware/src/pluto.rs | high | Sprint 4 TX path + `enter/exit_loopback_test_mode`, `set_tx_cyclic`, `set_dds_enabled`. The live vehicle. |
| crates/sdr-hardware/src/mock.rs | high | `MockSignal::Loopback` + `tx_queue` returns written samples on read — the CI vehicle, requiring no radio. |
| crates/sdr-dsp/src/clock_recovery.rs | medium | Gardner / Mueller & Müller symbol-timing recovery exists but is unintegrated; the principled long-term answer to alignment (see §5). |

## 3. External Sources
- **Live hardware + in-repo measurement (primary authority).** No new external
  sources were needed: the questions this sprint turns on are properties of
  *this* codebase and *this* radio, both measured directly (§4). The AD9361
  loopback semantics were established in the Sprint 4 report.

## 4. Measured findings

### 4.1 The modulator ↔ demodulator pair round-trips exactly (measured)
A temporary probe modulated `AA AA D3 91 01 02 FF 00 5A` and demodulated it
(1 MSPS, 100 kHz deviation, 10 samples/symbol):

```
ALIGNED: modulated 720 samples -> 72 bits (expected 72), bit errors 0/72
```

The `FskModulator`/`FskDemod` pair is **bit-exact when sample-aligned**. This
was the foundational unknown and it is resolved favourably: no DSP rework is
needed for this sprint.

### 4.2 Sample-phase alignment is the real constraint (measured)
Feeding the same IQ starting at various sample offsets:

| start offset (of 10 samples/symbol) | bit errors |
|---|---|
| 0 | 0 / 72 |
| 3 | 0 / 71 |
| 5 | 1 / 71 |
| 7 | **37 / 71** (garbage) |

Small offsets are tolerated by the per-symbol averaging; a large offset destroys
the bitstream. Since a receiver reading from a **cyclic** TX buffer starts at an
arbitrary point, alignment cannot be assumed — it must be searched or recovered.

### 4.3 Digital loopback is a near-ideal channel (established Sprint 4)
`loopback=1` bypasses the entire RF section: no noise, no fading, no carrier
frequency offset, no phase rotation. Normalized amplitude round-trips 1:1
(0.5 in → 0.5 out, peak |amp| 0.7071 measured). Consequently the only channel
impairment this sprint must handle is **sample-phase / bit alignment** — not
SNR, not CFO. That makes an end-to-end datagram demonstration genuinely
achievable now.

### 4.4 Alignment strategy
Two mechanisms compose, and both already exist in the codebase:
1. **Sample-phase search** — demodulate the received IQ at each of the `sps`
   possible start offsets (§4.2 shows at least one will be well-aligned).
2. **Sync-word search** — `PacketFramer::decode` already scans for
   `D3 91 D3 91`, giving byte alignment and rejecting a wrong phase via CRC-32.

A candidate offset is *correct* precisely when the sync word is found and CRC-32
validates — a self-checking criterion, not a guess.

## 5. Risks, Unknowns, Dependencies
- **Risk — bit-level (sub-byte) misalignment.** The sample-phase search yields a
  bit stream whose *bit* offset within a byte may still be wrong. *Mitigation:*
  search the sync word at bit granularity over the bit stream (slide a 32-bit
  window), then byte-align from that index; CRC-32 confirms.
- **Risk — cyclic buffer boundary.** A cyclically repeating frame means the
  receiver may capture a wrapped fragment. *Mitigation:* transmit the frame
  preceded by preamble and capture ≥ 2× the frame length so at least one
  complete frame is present in the window.
- **Deferred — proper symbol-timing recovery.** `clock_recovery.rs` (Gardner /
  M&M) is the principled replacement for the offset search and would be required
  for a real over-the-air channel with clock drift. Out of scope here (the
  digital loopback has no clock offset); recorded as carry-forward.
- **Constraint — loopback testing stands.** All hardware verification uses
  `loopback=1` + maximum attenuation + DDS disabled, per the user's standing
  choice. Over-the-air remains backlog T-108 pending explicit go-ahead.
- **Dependency — none new.** Composes existing crates; no new dependencies.

## 6. Recommended Approach
1. **`RadioLink` in `sdr-mesh`** implementing `MeshInterface` over any
   `SdrDriver` (so `MockSdr` serves CI and `PlutoSdr` serves hardware):
   - `send_datagram`: KISS-encode → `ArqTransceiver::create_data_frame` →
     `FskModulator::modulate_bytes` → `driver.write_samples`.
   - `recv_datagram`: `driver.read_samples` → for each candidate sample phase,
     `FskDemod` → bits → bit-level sync-word search → bytes →
     `PacketFramer::decode` (CRC-32 validates the phase) → `ArqTransceiver` →
     `KissDecoder` → datagram.
2. **Bit-sync helper** (`sdr-mesh`): pack bits→bytes with a bit-offset sync-word
   search. Small, pure, unit-testable.
3. **CI verification** over `MockSdr` loopback: a datagram survives the full
   modulate → sample → demodulate → deframe path with no radio.
4. **Live verification** over the Pluto internal loopback, using the Sprint 4
   safety helpers; assertions after state restoration.
5. **Out of scope:** clock recovery integration, GFSK (use plain FSK, which is
   measured bit-exact), over-the-air, multi-hop routing, `tun`/babeld.

## 7. Artifacts
- `docs/sprints/s5/sprint-research/research-report.md` — this report.
- Measurements in §4.1/§4.2 produced by a temporary probe
  (`crates/sdr-demod/tests/scratch_roundtrip.rs`), run and then removed; the
  numbers are reproduced above as the durable record.
