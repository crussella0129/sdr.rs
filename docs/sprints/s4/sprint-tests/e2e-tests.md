# Sprint 4 End-to-End Tests

- **Tested head:** `ab285092657e049991e0cd70f379aaec04d3de35`
- **Status:** possible — live hardware E2E performed by the agent under internal
  loopback. CI remains hardware-free (3 `#[ignore]`d tests).

## Live hardware E2E — **internal loopback**
Run against the physical Pluto+ at `192.168.2.1:30431` with
`cargo test -p sdr-hardware --test hw_pluto -- --ignored --nocapture`.

| Test | Result |
|------|--------|
| `hw_verify_pluto_connect` | PASS — iiod VERSION handshake + context enumeration. |
| `hw_verify_pluto_rx` (regression) | PASS — 4096/4096 non-zero, peak 0.8772 at 95.83 MHz. |
| `hw_verify_pluto_tx_loopback` | PASS — see below. |

### `hw_verify_pluto_tx_loopback`
Safety configuration engaged **before** any sample is written:
`loopback=1` (AD9361-internal digital — the entire RF section is bypassed),
TX `hardwaregain` = **−89.75 dB** (maximum attenuation), and the DDS tone
generators silenced so the readback can only contain transmitted data.

Observed:

```
wrote 4096 samples at -89.75 dB (max attenuation, RF bypassed);
read back 4096 samples, 4096 non-zero, peak |amp| = 0.7071
```

The transmitted pattern was `(0.5, −0.5)` / `(−0.5, 0.5)`, whose magnitude is
√(0.5² + 0.5²) = **0.7071** — the measured peak matches exactly, confirming the
samples made a real round trip through the transceiver and that the S16 TX and
S12/16 RX scalings are both correct. The daemon accepted all 4096 samples.

Device state was independently confirmed restored after the run:
`loopback = 0`, TX gain back to `−10.000000 dB`, DDS `raw = 1`.

Assertions run *after* restoration, so a failure cannot strand the radio in
loopback or full attenuation (critique C-003).

## Not-yet-possible (named unlockers)
- **Over-the-air transmit** → requires the user's separate explicit go-ahead on
  band, power and antenna; would be gated through the compliance DB
  (INT-0005/INT-0008). Deliberately excluded from this sprint; tracked as
  backlog **T-108**.
- **Two-radio on-air link / mesh multi-hop** → mesh **Phase C** (INT-0008),
  which this sprint unblocks but does not deliver.
