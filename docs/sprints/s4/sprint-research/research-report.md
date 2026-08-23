# Sprint 4 Research Report — PlutoSDR transmit path over iiod (T-102)

Implement the real TX path for the PlutoSDR driver, completing the transmit half
of the hardware layer and unblocking mesh Phase C. Per the user's explicit
decision, hardware verification uses the AD9361's **internal loopback only**.

## Intents Reviewed
- [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) — **selected**; primary. Criterion 1 requires continuous asynchronous RX/**TX** buffer streaming and criterion 2 requires real Pluto I/O; RX was proven in Sprint 2 but `write_samples` is still the trait's no-op default. This sprint delivers TX.
- [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) — **selected** (context, not advanced): mesh Phase C (on-air) is gated on this work; nothing in Phase C is built this sprint.
- [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) — **selected** (context): its intent body point 3 ("TX Driver Integration … PlutoSDR AD9361 TX path") was realized in Sprint 1 against `MockSdr` only; real Pluto TX makes that body claim true in hardware. No criterion of INT-0006 changes.

## 1. Sprint Goal
Implement `PlutoSdr` transmit over the pure-Rust iiod client: TX device/channel
configuration (TX_LO, TX sampling rate, RF bandwidth, TX attenuation), an output
buffer (`OPEN` + `WRITEBUF`), and `Complex32` → interleaved int16 conversion,
wired through the existing `SdrDriver` TX methods (`start_tx`/`write_samples`/
`stop_tx`/`has_tx`). Verify end-to-end **on the physical Pluto+ with the AD9361
internal digital loopback engaged and TX attenuation at maximum**, so real TX
samples are proven to traverse the hardware without emitting RF.

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| crates/sdr-hardware/src/iiod.rs | high | Pure-Rust iiod client (Sprint 2). Has `OPEN`/`READBUF`/`READ`/`WRITE`/`PRINT`. **Missing:** `WRITEBUF` and a `DEBUG`-direction accessor. `Direction` enum has only `Input`/`Output`. |
| crates/sdr-hardware/src/pluto.rs | high | Real RX driver. TX methods fall through to the `SdrDriver` defaults: `write_samples` → `Ok(0)`, `has_tx` → `false`. Holds `phy_dev`/`rx_dev` ids resolved from context; needs a `tx_dev`. |
| crates/sdr-hardware/src/driver.rs | high | `SdrDriver` already declares `start_tx`/`stop_tx`/`write_samples`/`has_tx` with safe defaults — the seam exists; only PlutoSDR's implementation is missing. |
| crates/sdr-hardware/src/mock.rs | medium | `MockSignal::Loopback` + `tx_queue` is the established TX-then-readback API shape; the hardware loopback test mirrors this pattern. |
| crates/sdr-hardware/tests/hw_pluto.rs | high | Existing `#[ignore]`d live-hardware tests (`hw_verify_pluto_connect`, `hw_verify_pluto_rx`). The TX loopback test joins this file and stays CI-excluded. |
| crates/sdr-hardware/tests/pluto_iiod.rs | high | Mock-iiod replay server; must be extended to answer `WRITEBUF` so the TX path has a CI regression test with no radio. |
| crates/sdr-hardware/tests/fixtures/pluto_ctx.xml | high | Real captured context. Confirms TX device + channel/format facts (below). |
| crates/sdr-demod/src/modulator.rs | medium | Produces the GFSK/FSK IQ bursts that TX will eventually carry (INT-0006). Not modified this sprint. |

## 3. External Sources
- [AD9361 Linux driver — debug attributes / loopback](https://wiki.analog.com/resources/tools-software/linux-drivers/iio-transceiver/ad9361) — `loopback` modes: **0 = disabled; 1 = AD9361-internal digital TX→RX, "the entire RF section is bypassed"; 2 = FPGA-internal RX→TX (the RF chain is active and transmits)**. Mode 1 is the zero-emission verification path; mode 2 must be avoided.
- [ADI AD9361 driver docs (developer mirror)](https://developer.analog.com/docs/linux/drivers/iio-transceiver/ad9361.html) — TX `hardwaregain` on the AD936x is **attenuation** (0 dB = full output, negative = attenuated); confirms the semantics of the range read from the live device.
- **Live hardware probe (primary authority, this machine, iiod 0.21 @ 192.168.2.1:30431)** — read-only queries establishing the TX facts in §4.

## 4. Hardware findings (probed live, read-only — nothing transmitted)

| Fact | Value | Source |
|------|-------|--------|
| TX streaming device | `cf-ad9361-dds-core-lpc` = `iio:device2` | context XML |
| TX scan channels | `voltage0..3`, **`le:S16/16>>0`** | context XML |
| RX scan format (contrast) | `le:S12/16>>0` | context XML |
| TX LO | `ad9361-phy` `OUTPUT altvoltage1` attr `frequency` (read 2 450 000 000) | live `READ` |
| TX gain/attenuation | `ad9361-phy` `OUTPUT voltage0` attr `hardwaregain`, read `-10.000000 dB` | live `READ` |
| TX gain range | `hardwaregain_available` = `[-89.750000 0.250000 0.000000]` → min **−89.75 dB**, step 0.25, max 0 dB | live `READ` |
| TX sample rate | `ad9361-phy` `OUTPUT voltage0` attr `sampling_frequency` = 3 000 000 | live `READ` |
| Loopback control | `ad9361-phy` debug attr **`loopback`**, currently `0` | live `READ` |
| iiod debug access form | **`READ <dev> DEBUG <attr>`** works (`READ iio:device0 DEBUG loopback` → `0`); the no-direction form returns `-2` (ENOENT) | live probe |

Two consequences for the implementation:
1. **TX samples are full-scale S16** (÷32768), unlike RX's 12-bit-in-16 (÷2048).
   Reusing the RX scale factor for TX would be a 16× amplitude error.
2. **The `Direction` enum needs a `Debug` variant** so the driver can drive the
   `loopback` attribute through the existing attribute read/write helpers.

## 5. Risks, Unknowns, Dependencies
- **Risk — unintended RF emission (the sprint's most important risk).**
  *Mitigations, all required together:* engage `loopback=1` (AD9361-internal,
  RF section bypassed) **before** any buffer write; set TX `hardwaregain` to
  **−89.75 dB** (maximum attenuation) for the duration of the test; restore both
  (`loopback=0`, prior gain) on teardown. Never use `loopback=2` (that mode
  actively transmits). Tests use internal loopback only; over-the-air TX is out of scope and
  requires the user's separate go-ahead.
- **Unknown — `WRITEBUF` response framing.** `READBUF`'s framing was
  established live in Sprint 2 (`<nbytes>\n<mask>\n<payload>`); `WRITEBUF` was
  deliberately **not** probed during research because writing a TX buffer is the
  action that could emit. *Mitigation:* implement per the libiio line protocol
  (`WRITEBUF <dev> <nbytes>` + raw payload → status line) and settle it
  empirically in Build **with loopback engaged and attenuation maxed**; the
  mock-iiod replay test pins whatever framing proves correct.
- **Risk — half-duplex contention.** The AD9361 RX and TX buffers are opened
  independently; a loopback test opens both. *Mitigation:* explicit
  open/close ordering and teardown that closes both buffers.
- **Dependency — none new.** Pure-Rust `std::net` only; no new crates, no C libs.
- **Constraint — CI must stay hardware-free.** Every hardware test remains
  `#[ignore]`d; the `WRITEBUF` path gets a mock-iiod regression test.

## 6. Recommended Approach
1. **Extend the iiod client:** add `Direction::Debug` (token `DEBUG`), a
   `write_buf` method implementing `WRITEBUF`, and a `complex32_to_iq_bytes`
   converter using the **S16 full-scale (32768)** factor.
2. **Implement TX on `PlutoSdr`:** resolve `tx_dev` (`cf-ad9361-dds-core-lpc`)
   from the context; add `set_tx_frequency`/`set_tx_gain` (attenuation-aware,
   validated against −89.75…0 dB); implement `start_tx`/`write_samples`/
   `stop_tx`/`has_tx` over `OPEN`/`WRITEBUF`/`CLOSE`.
3. **Safety helpers:** `set_loopback(mode)` and an explicit
   `enter_loopback_test_mode()` / `exit_loopback_test_mode()` pair that engages
   mode 1 + max attenuation and restores prior state, so the no-emission
   contract is expressed in code rather than only in a test.
4. **Verification:** (a) CI — extend the mock-iiod replay server to answer
   `WRITEBUF`, asserting command framing and S16 conversion, no radio needed;
   (b) live, agent-run — `#[ignore]`d `hw_verify_pluto_tx_loopback`: engage
   loopback + max attenuation, write a known IQ pattern, read it back on RX,
   assert non-zero correlated samples, then restore state.
5. **Out of scope:** over-the-air transmit, TX DDS tone generation, duplex
   streaming performance work, and mesh Phase C integration.

## 7. Artifacts
- `docs/sprints/s4/sprint-research/research-report.md` — this report.
- Live probe evidence recorded in §4 (read-only; nothing transmitted).
- Reviewed source: `crates/sdr-hardware/src/{iiod,pluto,driver,mock}.rs`,
  `crates/sdr-hardware/tests/{hw_pluto,pluto_iiod}.rs`, context fixture.
