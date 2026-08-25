# Sprint 4 Test Report

- **Tested head:** `ab285092657e049991e0cd70f379aaec04d3de35`
- **Runner:** `cargo test --workspace` — **87 passed, 0 failed, 3 ignored** (hardware). `cargo clippy --workspace --all-targets` — **0 errors**.
- **Live hardware:** 3/3 pass against the physical Pluto+, **internal loopback**.
- **Critique verdict:** proceed-with-caveats (see `critique.md`).

## Suite results
| Suite | Kind | Result |
|-------|------|--------|
| sdr-hardware lib (iiod + pluto) | unit | 25 passed |
| sdr-hardware pluto_iiod | integration | 2 passed |
| sdr-hardware hw_pluto | live hardware (agent) | 3 passed (ignored in CI) |
| all other workspace crates | unit/integration/e2e | unchanged, green |

## Defects found by testing against real hardware
Three defects survived unit and mock testing and were caught only on the radio —
recorded because they are the substance of this sprint's verification:

1. **`WRITEBUF` is a two-phase exchange.** iiod acknowledges the header with a
   status line *before* accepting the payload. The initial single-status client
   returned 0 bytes written and desynchronized the connection. Predicted as risk
   C-001 in the plan critique; found and fixed on hardware. Client and mock both
   corrected, so CI now guards it.
2. **DDS tone generators are enabled by default** on `cf-ad9361-dds-core-lpc`
   and would have been transmitted instead of the caller's samples — a latent
   bug that would otherwise have shipped in the TX path. `start_tx` now disables
   them.
3. **A one-shot TX buffer drains before it can be observed.** Cyclic buffer
   support (`OPEN … CYCLIC`) was added — also the correct mechanism for a real
   transmitter to sustain a waveform.

## Intent verification (INT-0002)
| Acceptance criterion | Verdict |
|----------------------|---------|
| #1 continuous asynchronous RX/**TX** buffer streaming | **Verified** — TX transport implemented and exercised on hardware; 4096/4096 samples accepted by the real daemon. |
| #2 real Pluto I/O — transmit half, under loopback | **Verified (loopback)** — samples returned on RX at peak \|amp\| 0.7071, matching the transmitted pattern exactly. |
| #2 real Pluto I/O — over the air | **Not verified, by design** — the RF section is bypassed in the chosen verification mode. Tracked as backlog T-108, requires explicit go-ahead. |
| #6 safe device control / state restoration | **Verified** — loopback, TX gain and DDS state saved and confirmed restored; FPGA RX→TX loopback mode not constructible. |
| #3 multi-vendor (RTL/HackRF/Airspy) | **Carried forward** — backlog T-101. |

## Conclusion
The PlutoSDR transmit path is implemented and proven on real hardware under
internal loopback, and three hardware-only defects were caught and fixed in the
process.
**INT-0002 remains `active`**: its transmit criteria are met under internal
loopback, while over-the-air transmit and multi-vendor support are explicitly
carried forward. Mesh Phase C (INT-0008) is now unblocked. No re-architecture
failure; proceed to Loop.
