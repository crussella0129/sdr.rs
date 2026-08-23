# Completed Tasks Log (Append-Only)

## T-001 (sprint 0)
- **Description:** Cargo workspace setup and sdr-core streaming architecture
- **Intent:** [INT-0001](../intents/INT-0001-core-dsp-pipeline.md)
- **Completed:** 2026-08-19T06:31:00Z
- **Files modified:** Cargo.toml, Cargo.lock, crates/sdr-core/Cargo.toml, crates/sdr-core/src/lib.rs, crates/sdr-core/src/sample.rs, crates/sdr-core/src/buffer.rs, crates/sdr-core/src/tag.rs, crates/sdr-core/src/traits.rs
- **Commit:** `39e8e0139992aa70bd02520585c76e25f1b35b3a`

## T-002 (sprint 0)
- **Description:** sdr-dsp filtering, SIMD convolution, NCO, resamplers, and synchronization
- **Intent:** [INT-0001](../intents/INT-0001-core-dsp-pipeline.md)
- **Completed:** 2026-08-19T06:33:00Z
- **Files modified:** crates/sdr-dsp/Cargo.toml, crates/sdr-dsp/src/lib.rs, crates/sdr-dsp/src/fir.rs, crates/sdr-dsp/src/window.rs, crates/sdr-dsp/src/nco.rs, crates/sdr-dsp/src/resample.rs, crates/sdr-dsp/src/hilbert.rs, crates/sdr-dsp/src/costas.rs, crates/sdr-dsp/src/clock_recovery.rs
- **Commit:** `232bd70b116264a71ff18c6c0b1d95375ffce94b`

## T-003 (sprint 0)
- **Description:** sdr-hardware abstraction, PlutoSDR IIO client, SigMF, and mock drivers
- **Intent:** [INT-0002](../intents/INT-0002-hardware-drivers-pluto.md)
- **Completed:** 2026-08-19T06:34:00Z
- **Files modified:** crates/sdr-hardware/Cargo.toml, crates/sdr-hardware/src/lib.rs, crates/sdr-hardware/src/driver.rs, crates/sdr-hardware/src/pluto.rs, crates/sdr-hardware/src/sigmf.rs, crates/sdr-hardware/src/wav.rs, crates/sdr-hardware/src/mock.rs
- **Commit:** `76637780be5e8feecbbcb66bbdbd0a1b66df8735`

## T-004 (sprint 0)
- **Description:** sdr-demod analog (WFM/NFM/AM/SSB/CW) and digital (OOK/FSK/PSK) pipelines
- **Intent:** [INT-0003](../intents/INT-0003-modulation-demodulation.md)
- **Completed:** 2026-08-19T06:36:00Z
- **Files modified:** crates/sdr-demod/Cargo.toml, crates/sdr-demod/src/lib.rs, crates/sdr-demod/src/wfm.rs, crates/sdr-demod/src/nfm.rs, crates/sdr-demod/src/am.rs, crates/sdr-demod/src/ssb.rs, crates/sdr-demod/src/cw.rs, crates/sdr-demod/src/fsk.rs, crates/sdr-demod/src/ook.rs, crates/sdr-demod/src/psk.rs
- **Commit:** `8cb4dd2861c8c5c7d0d00f6848be28cb52b1da79`

## T-005 (sprint 0)
- **Description:** sdr-protocols (LoRa, ADS-B, APRS) and sdr-spectrum (FFT, CFAR, Rigctl)
- **Intent:** [INT-0004](../intents/INT-0004-protocol-decoders-spectrum.md)
- **Completed:** 2026-08-19T06:38:00Z
- **Files modified:** crates/sdr-protocols/Cargo.toml, crates/sdr-protocols/src/lib.rs, crates/sdr-protocols/src/lora.rs, crates/sdr-protocols/src/adsb.rs, crates/sdr-protocols/src/aprs.rs, crates/sdr-spectrum/Cargo.toml, crates/sdr-spectrum/src/lib.rs, crates/sdr-spectrum/src/fft.rs, crates/sdr-spectrum/src/cfar.rs, crates/sdr-spectrum/src/rigctl.rs
- **Commit:** `67c2eae8e367fc9bda465fec7db2f267a6d893eb`

## T-006 (sprint 0)
- **Description:** sdr-cli tool and end-to-end integration testing
- **Intent:** [INT-0001](../intents/INT-0001-core-dsp-pipeline.md), [INT-0002](../intents/INT-0002-hardware-drivers-pluto.md), [INT-0003](../intents/INT-0003-modulation-demodulation.md), [INT-0004](../intents/INT-0004-protocol-decoders-spectrum.md)
- **Completed:** 2026-08-19T06:40:00Z
- **Files modified:** crates/sdr-cli/Cargo.toml, crates/sdr-cli/src/main.rs, crates/sdr-cli/tests/e2e_pipeline_tests.rs
- **Commit:** `f0b78b1d7d65fc97b212fefc2cb57053e1644fc2`

## T-007 (sprint 1)
- **Description:** Jurisdictional Regulatory Compliance database and transmission advisor
- **Intent:** [INT-0005](../intents/INT-0005-regulatory-band-compliance.md)
- **Completed:** 2026-08-21T12:42:00Z
- **Files modified:** crates/sdr-core/src/compliance.rs, crates/sdr-core/src/lib.rs
- **Commit:** `c19b853a4db29559c5dca9735d4872fc4cf33aa5`

## T-008 (sprint 1)
- **Description:** Packet framing, CRC-32, sequence numbering, and ARQ retransmission
- **Intent:** [INT-0006](../intents/INT-0006-packet-radio-ssh-tunnel.md)
- **Completed:** 2026-08-21T12:43:00Z
- **Files modified:** crates/sdr-protocols/src/packet.rs, crates/sdr-protocols/src/lib.rs
- **Commit:** `669a74d28472da41d6365ba2066d7e0078170c0c`

## T-009 (sprint 1)
- **Description:** Continuous-phase GFSK/FSK packet modulator and SdrDriver TX streaming pipeline
- **Intent:** [INT-0006](../intents/INT-0006-packet-radio-ssh-tunnel.md)
- **Completed:** 2026-08-21T12:44:00Z
- **Files modified:** crates/sdr-demod/src/modulator.rs, crates/sdr-demod/src/lib.rs, crates/sdr-hardware/src/driver.rs, crates/sdr-hardware/src/mock.rs, crates/sdr-hardware/src/pluto.rs, crates/sdr-hardware/src/lib.rs
- **Commit:** `cd1158c894234033878b274c4e7fa0799be06173`

## T-010 (sprint 1)
- **Description:** Stream tunnel proxy bridge for SSH and sdr-cli bands/tunnel commands
- **Intent:** [INT-0005](../intents/INT-0005-regulatory-band-compliance.md), [INT-0006](../intents/INT-0006-packet-radio-ssh-tunnel.md)
- **Completed:** 2026-08-21T12:46:00Z
- **Files modified:** crates/sdr-protocols/src/tunnel.rs, crates/sdr-cli/src/main.rs, crates/sdr-cli/tests/e2e_pipeline_tests.rs
- **Commit:** `03beac6d5c5bc64a26cff26ad4744c993c00fa55`

## T-011 (sprint 2)
- **Description:** Add Cloudlog to the README reference catalog with its /api/radio and /api/qso integration note
- **Intent:** [INT-0007](../intents/INT-0007-station-logging-cloudlog.md)
- **Completed:** 2026-08-22T00:08:24Z
- **Files modified:** README.md
- **Commit:** `e5ccafcdc448a6f9798cf7014c192855f10f149e`

## T-012 (sprint 2)
- **Description:** Cloudlog station-logging client — /api/radio CAT push, /api/qso ADIF upload, ADIF record builder, API-key redaction; new sdr-station crate
- **Intent:** [INT-0007](../intents/INT-0007-station-logging-cloudlog.md)
- **Completed:** 2026-08-22T00:12:59Z
- **Files modified:** crates/sdr-station/Cargo.toml, crates/sdr-station/src/lib.rs, crates/sdr-station/src/adif.rs, crates/sdr-station/src/cloudlog.rs, crates/sdr-station/tests/cloudlog_it.rs, Cargo.toml, Cargo.lock
- **Commit:** `5a8f4ee23a28cf05862bc121cf33394e262f481d`

## T-013 (sprint 2)
- **Description:** Real PlutoSDR driver via a pure-Rust iiod network client (no C deps) — VERSION/PRINT/READ/WRITE/OPEN/READBUF protocol, context XML device enumeration, int16→Complex32 RX. Fixes wrong iiod port (50901→30431) and default addr (192.168.1.10→192.168.2.1). Verified live against physical Pluto+ (4096/4096 non-zero IQ) plus a mock-iiod replay test.
- **Intent:** [INT-0002](../intents/INT-0002-hardware-drivers-pluto.md)
- **Completed:** 2026-08-22T00:31:13Z
- **Files modified:** crates/sdr-hardware/src/iiod.rs, crates/sdr-hardware/src/pluto.rs, crates/sdr-hardware/src/lib.rs, crates/sdr-hardware/tests/fixtures/pluto_ctx.xml, crates/sdr-hardware/tests/hw_pluto.rs, crates/sdr-hardware/tests/pluto_iiod.rs
- **Commit:** `3e962ab58efc743c0a8bf9352bb5b158e87752b2`

## T-018 (sprint 2)
- **Description:** Review-discovered fix for two deny-level clippy errors blocking the workspace linter — `never_loop` in the compliance band evaluator (first-match made explicit; behavior preserved) and `approx_constant` in the SSB demodulator (0.7071 → `std::f32::consts::FRAC_1_SQRT_2`). Non-semantic; no acceptance criteria changed.
- **Intent:** [INT-0005](../intents/INT-0005-regulatory-band-compliance.md), [INT-0003](../intents/INT-0003-modulation-demodulation.md)
- **Completed:** 2026-08-22T00:32:10Z
- **Files modified:** crates/sdr-core/src/compliance.rs, crates/sdr-demod/src/ssb.rs
- **Commit:** `346042820ce68797b4690f971931c17aed4d1a22`

## T-014 (sprint 2)
- **Description:** Device enumeration API — `list_devices()` (always includes the mock device; best-effort short-timeout probe of the default Pluto endpoint via iiod, returning a `DeviceInfo`) plus `IiodClient::connect_with_timeout`. The full multi-vendor SoapySDR/seify backend (RTL/HackRF/Airspy) is deferred to a hardware follow-on: it needs SoapySDR host C libraries absent on this machine, so it cannot be compiled or verified here — committing an unverifiable binding is avoided. The `SdrDriver` trait is the extension seam.
- **Intent:** [INT-0002](../intents/INT-0002-hardware-drivers-pluto.md)
- **Completed:** 2026-08-22T00:34:00Z
- **Files modified:** crates/sdr-hardware/src/iiod.rs, crates/sdr-hardware/src/pluto.rs, crates/sdr-hardware/src/lib.rs
- **Commit:** `9cf6d8b7f68e0261fe6a65c6ca0717d6e6463625`

## T-015 (sprint 2)
- **Description:** Real Hamlib Rigctl TCP server — the `rigctl` command now binds a tokio TCP listener and serves the existing `RigctlHandler` engine per connection (f/F/m/M/v/\dump_state), holding connections open until `q`. Replaces the prior one-shot canned-command stub. Verified by e2e tests that spawn the CLI binary and drive it over loopback.
- **Intent:** [INT-0004](../intents/INT-0004-protocol-decoders-spectrum.md)
- **Completed:** 2026-08-22T00:37:00Z
- **Files modified:** crates/sdr-cli/src/main.rs, crates/sdr-cli/tests/rigctl_server_it.rs
- **Commit:** `6b1a92241225be399db926f8100b0daaa1a4541c`

## T-016 (sprint 2)
- **Description:** CLI device selection wired up — `record --driver` now constructs the chosen driver (`mock`, `pluto`, or an explicit `ip:`/`usb:` iiod URI) instead of always using the mock; added a `devices` subcommand printing `list_devices()`. Verified live: `sdr-cli devices` enumerates the real PlutoSDR at ip:192.168.2.1 and `record --driver pluto` captured 8192 real IQ samples at 95.83 MHz; plus CI e2e tests for mock success, unreachable-Pluto graceful failure, and devices listing.
- **Intent:** [INT-0002](../intents/INT-0002-hardware-drivers-pluto.md)
- **Completed:** 2026-08-22T00:40:00Z
- **Files modified:** crates/sdr-cli/src/main.rs, crates/sdr-cli/tests/driver_cli_it.rs
- **Commit:** `05eba0b72fc9a484afd9788f3f2c65dc063ba042`

## T-017 (sprint 2)
- **Description:** `LinearPipeline` now reuses preallocated input/output buffers across `step()` calls instead of heap-allocating a fresh `Vec` each iteration, keeping the hot path allocation-free (consistent with INT-0001's zero-copy consequence). Behavior-preserving; verified by multi-step correctness and buffer-reuse (stable backing pointer) tests.
- **Intent:** [INT-0001](../intents/INT-0001-core-dsp-pipeline.md)
- **Completed:** 2026-08-22T00:44:00Z
- **Files modified:** crates/sdr-core/src/traits.rs
- **Commit:** `b358fc7e0642cb91d5522ff62ef07fb76ab9813a`

## T-019 (sprint 3)
- **Description:** New `sdr-mesh` crate + KISS datagram framing (`encode` + streaming `KissDecoder`) so IP datagram boundaries survive the byte-oriented packet-radio link. Byte-stuffs FEND/FESC; reassembles split/concatenated frames. Phase A of the mesh (INT-0008).
- **Intent:** [INT-0008](../intents/INT-0008-mesh-networking-aredn.md)
- **Completed:** 2026-08-23T00:38:55Z
- **Files modified:** crates/sdr-mesh/Cargo.toml, crates/sdr-mesh/src/lib.rs, crates/sdr-mesh/src/kiss.rs, Cargo.toml, Cargo.lock
- **Commit:** `c84783fc65636e0e06cb68a2cf1638fc46906867`

## T-020 (sprint 3)
- **Description:** Dual-mode compliance gate — `MeshPolicy::evaluate(jur, freq, power, want_encrypted)` reuses `RegulatoryDatabase::check_compliance` to return `Allow(Encrypted)` on ISM, `Allow(Open)` on amateur, or `Refuse(reasons)` (never silently transmits encrypted where prohibited). Directly implements the user's encrypted-ISM / open-amateur design and reuses INT-0005.
- **Intent:** [INT-0008](../intents/INT-0008-mesh-networking-aredn.md)
- **Completed:** 2026-08-23T00:41:00Z
- **Files modified:** crates/sdr-mesh/src/policy.rs, crates/sdr-mesh/src/lib.rs
- **Commit:** `f3a5dc02ed7aa2a167f07e6ad91b051b89b788f4`

## T-021 (sprint 3)
- **Description:** `MeshInterface` seam (a real tun device plugs in for Phase B) + deterministic in-memory `LoopbackLink` carrying KISS-framed datagrams through `ArqTransceiver` frames + `MeshNode` that consults the compliance gate before emitting. Integration tests prove end-to-end datagram roundtrip (both directions, with escaped bytes) and that an encrypted send on an amateur band is refused with no frame emitted.
- **Intent:** [INT-0008](../intents/INT-0008-mesh-networking-aredn.md)
- **Completed:** 2026-08-23T00:44:00Z
- **Files modified:** crates/sdr-mesh/src/node.rs, crates/sdr-mesh/src/lib.rs, crates/sdr-mesh/tests/loopback_it.rs
- **Commit:** `e1a2acf275d4b6bf47d71875e3cfb3ca30f0e0ac`

## T-022 (sprint 3)
- **Description:** Added AREDN (`aredn/aredn`) and the Babel routing RFC (RFC 8966) to the README reference catalog with mesh-relevance notes; a content-check test asserts both are present.
- **Intent:** [INT-0008](../intents/INT-0008-mesh-networking-aredn.md)
- **Completed:** 2026-08-23T00:46:00Z
- **Files modified:** README.md, crates/sdr-mesh/tests/readme_it.rs
- **Commit:** `770e43d5b2491ff6e04fc7e89300b4df8841a815`

## T-023 (sprint 4)
- **Description:** iiod client TX transport — `Direction::Debug` (`DEBUG` token, device-level attrs with no channel name), `cmd_writebuf`/`IiodClient::write_buf` (`WRITEBUF <dev> <nbytes>` + payload), `read_debug_attr`/`write_debug_attr`, and `complex32_to_iq_bytes` using the **S16 full scale (32768)** with saturating clamp — distinct from RX's S12/16 scale (2048), which would otherwise transmit at 1/16 amplitude.
- **Intent:** [INT-0002](../intents/INT-0002-hardware-drivers-pluto.md)
- **Completed:** 2026-08-23T03:49:36Z
- **Files modified:** crates/sdr-hardware/src/iiod.rs
- **Commit:** `9794c7af833b65341e6723fe05ee9356e8960784`
