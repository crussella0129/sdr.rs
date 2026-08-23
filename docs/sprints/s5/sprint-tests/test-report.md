# Sprint 5 Test Report

- **Tested head:** `19bd39e5e1ed338ee7282695e8c38e4ddc8905a8`
- **Runner:** `cargo test --workspace` — **95 passed, 0 failed, 4 ignored** (hardware). `cargo clippy --workspace --all-targets` — **0 errors**.
- **Live hardware:** `hw_verify_mesh_datagram_over_radio` passed on the first run, **internal loopback**.
- **Critique verdict:** proceed-with-caveats (see `critique.md`).

## Suite results
| Suite | Kind | Result |
|-------|------|--------|
| sdr-demod fsk_roundtrip | unit | 2 passed |
| sdr-mesh lib (kiss + policy + framesync) | unit | 10 passed |
| sdr-mesh radio_it | integration | 3 passed |
| sdr-mesh loopback_it / readme_it | integration | 3 passed |
| sdr-mesh hw_radio | live hardware (agent) | 1 passed (ignored in CI) |
| sdr-hardware hw_pluto | live hardware (agent) | 3 ignored in CI (unchanged) |
| all other workspace crates | unit/integration/e2e | unchanged, green |

## The headline result
A mesh datagram crossed **real radio hardware** for the first time. Under the
AD9361 internal digital loopback — RF section bypassed, transmitter at maximum
attenuation, DDS tones silenced — a 10-byte datagram traversed:

```
KISS → ARQ frame → FSK modulate → real Pluto TX (cyclic)
     → hardware loopback → real RX → FSK demodulate (sample-phase search)
     → bit sync → CRC-32 → KISS decode → datagram
```

and was recovered **byte-for-byte**. Device state (`loopback`, TX gain, DDS) was
independently confirmed restored afterward.

## Intent verification (INT-0008)
| Acceptance criterion | Verdict |
|----------------------|---------|
| #1 IP datagrams framed over the link layer and recovered without corruption — **over a real radio** | **Verified (internal loopback)** — CI over `MockSdr` including a mid-symbol stream, plus live hardware transit recovered byte-for-byte. |
| #1 real `tun` interface carrying host IP | **Carried forward → Phase B** (T-107). |
| #2 multi-hop routing via `babeld` / AREDN interop | **Carried forward → Phase B** (T-107). |
| #4 AREDN-compatible subnet addressing | **Carried forward → Phase B** (T-107). |
| over-the-air transit | **Not verified, by design** — backlog T-108, requires explicit go-ahead. |

INT-0006 is unchanged (`realized`); T-027 added regression coverage only.

## Honest limits
- The sample-phase search works because the loopback shares **one clock** between
  transmitter and receiver. A real two-radio link has clock drift and needs
  symbol-timing recovery (Gardner/M&M) — backlog **T-110**. This caveat is
  documented in `radio.rs` itself, not only here.
- The RF chain (mixer, PA, antenna) and real channel impairments are unexercised;
  the loopback that makes the test safe is what excludes them.
- One datagram size at one modulation setting was exercised on hardware.

## Conclusion
The first true IP-over-radio transit is implemented and proven on real hardware
under internal loopback. **INT-0008 remains `active`**: criterion 1 is met over a
real radio under loopback, while `tun`, babeld/AREDN interop and addressing stay
in Phase B and over-the-air stays gated on explicit go-ahead. No re-architecture
failure; proceed to Loop.
