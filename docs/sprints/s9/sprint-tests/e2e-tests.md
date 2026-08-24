# Sprint 9 — End-to-End Test Results

- **Tested head:** `454e821d2c91cd05de40ed9f6867efd845322290`
- **Date:** 2026-08-24

## Status: end-to-end in simulation; not possible on hardware, for a named reason

The 30% loss test is the end-to-end proof for INT-0006 criterion 2. It is not a
unit test of a state machine in isolation — it drives real
`PacketFramer::encode`/`decode`, real CRC-32 validation, real sequence numbers
and real retransmission through a channel that actually drops frames, and
asserts every payload arrives exactly once and in order.

```text
test arq_reliability::test_arq_delivers_all_payloads_at_30pct_loss ... ok
test arq_reliability::test_arq_delivers_all_payloads_at_10pct_loss ... ok
test arq_reliability::test_arq_ack_loss_does_not_duplicate_payload ... ok
test arq_reliability::test_arq_lossy_channel_is_deterministic ... ok
```

`radio_it`'s three ARQ tests extend that to the full radio path — KISS framing,
FSK modulation, demodulation with symbol-timing recovery — over `MockSdr`
loopback.

## What is *not* verified, and why

**ARQ has not been verified on real hardware, and nothing in this sprint should
be read as implying it has.**

One radio in internal loopback hears its own transmission. An ACK exchange with
oneself is degenerate: the station acknowledges its own frames, so the exchange
proves nothing about two stations agreeing. This is the same self-negotiation
that stalls the real OpenSSH client in `ssh_tunnel_it` — it completes the
version exchange with its own echo and can never finish key exchange.

**The unlocker is a second radio.** There is one Pluto+ here.

| Not verified | Unlocked by |
|---|---|
| ARQ between two stations on real hardware | **A second radio.** One device cannot acknowledge itself meaningfully. |
| Stop-and-wait over the Pluto's cyclic transmit buffer | **T-115.** A buffer that repeats one frame indefinitely cannot express "sent once, now listening" — it looks like endless retransmission and there is no controlled turnaround. |
| A measured T1 | Half-duplex turnaround latency measured on a real link. `DEFAULT_T1_MS = 500` is a **reasoned default, not a measurement**; AX.25's 3000 ms targets far slower channels. The constant's own documentation says so. |
| Throughput under ARQ | Stop-and-wait halves effective throughput by construction; no measurement was made and none is claimed. |
| Sliding-window ARQ | Out of scope by choice — INT-0006's intent text mentions it, and stop-and-wait is what a half-duplex channel supports naturally. Recorded, not silently dropped. |

## Hardware

Not exercised this sprint. The two `#[ignore]`d Pluto+ tests were not re-run:
`RadioLink` defaults to fire-and-forget, which is the mode they use, so no code
path they cover changed. Sprint 7's live result stands — 87 bytes as 2
datagrams, recovered byte-for-byte over the internal loopback.

**Nothing went on air.** That remains gated on **T-108** and explicit
authorization.
