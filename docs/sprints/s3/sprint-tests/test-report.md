# Sprint 3 Test Report

- **Tested head:** `0c74137bb27f9ef7c35fc12a5c4df80e29069977`
- **Runner:** `cargo test --workspace` — **75 passed, 0 failed**. `cargo clippy --workspace --all-targets` — **0 errors**.
- **Critique verdict:** proceed-with-caveats (see `critique.md`).

## sdr-mesh suite results
| Suite | Kind | Result |
|-------|------|--------|
| sdr-mesh lib (kiss + policy) | unit | 7 passed |
| sdr-mesh loopback_it | integration | 2 passed |
| sdr-mesh readme_it | doc-check | 1 passed |

Pre-existing suites (sdr-core/dsp/demod/protocols/spectrum/hardware/station/cli)
remain green and unchanged; the 2 PlutoSDR hardware tests stay `#[ignore]`d in CI.

## Intent verification (INT-0008)
| Acceptance criterion | Verdict |
|----------------------|---------|
| #1 IP datagrams framed over the link + recovered (framing half) | **Verified** — KISS roundtrip/stuffing/reassembly + two-node loopback. |
| #1 real `tun` carries host IP | **Carried forward → Phase B** (privileged; `MeshInterface` seam ready). |
| #2 multi-hop routing via `babeld` / AREDN interop | **Carried forward → Phase B.** |
| #3 compliance gate before encrypted TX (encrypted-ISM / open-amateur / refuse) | **Verified** — gate matrix + node-level refusal (no frame emitted). |
| #4 AREDN-compatible IP subnet addressing | **Carried forward → Phase B.** |

## Conclusion
Mesh Phase A is fully green: IP-over-radio framing and the dual-mode compliance
gate (the user's encrypted-ISM / open-amateur design) are implemented and
verified end-to-end in a deterministic loopback. INT-0008 is **materially
advanced but stays `active`** — its `tun`/`babeld`/addressing criteria (Phase B)
and on-air criteria (Phase C, gated on Pluto TX) are explicitly carried forward.
No re-architecture failure; proceed to Loop.
