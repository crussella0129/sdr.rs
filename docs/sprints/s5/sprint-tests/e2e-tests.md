# Sprint 5 End-to-End Tests

- **Tested head:** `19bd39e5e1ed338ee7282695e8c38e4ddc8905a8`
- **Status:** possible — live hardware E2E performed by the agent under internal loopback. CI stays hardware-free (4 `#[ignore]`d tests across the workspace).

## Live hardware E2E — **internal loopback**
`crates/sdr-mesh/tests/hw_radio.rs`, run against the physical Pluto+ at
`192.168.2.1:30431` with
`cargo test -p sdr-mesh --test hw_radio -- --ignored --nocapture`.

`hw_verify_mesh_datagram_over_radio` — **PASS on the first live run.**

Safety configuration engaged before anything was transmitted: AD9361 internal
digital loopback (`loopback=1`, RF section bypassed), TX `hardwaregain` at
−89.75 dB (maximum attenuation), DDS tone generators silenced.

Observed:

```
sent 10 bytes through the Pluto+ (RF bypassed, max attenuation); recovered Some(10)
```

The datagram traversed the complete path — KISS → ARQ frame → FSK modulation →
**real Pluto TX** (cyclic buffer) → hardware loopback → **real RX** → FSK
demodulation with sample-phase search → bit-level frame sync → CRC-32 → KISS
decode — and `assert_eq!` confirmed it matched the transmitted datagram
byte-for-byte. This is the project's first datagram to cross real radio hardware.

Device state independently confirmed restored after the run: `loopback = 0`,
TX gain `−10.000000 dB`, DDS `raw = 1`. Assertions run after restoration, so a
failure cannot strand the radio (Sprint 4 critique C-003).

## Not-yet-possible (named unlockers)
- **Over-the-air datagram transit** → backlog **T-108**; requires the user's explicit go-ahead on band, power and antenna, and would be gated through the compliance DB.
- **Two-radio link and multi-hop routing** → mesh **Phase B/C** (T-107), needing `tun`/babeld and a second radio.
- **A channel with clock drift** → symbol-timing recovery (Gardner/M&M), backlog **T-110**. The digital loopback shares one clock, which is precisely why the sample-phase search suffices here and would not suffice on the air.
