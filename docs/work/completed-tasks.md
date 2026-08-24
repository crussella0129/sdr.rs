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

## T-024 (sprint 4)
- **Description:** PlutoSdr transmit path — resolves `cf-ad9361-dds-core-lpc` from the device context, adds `set_tx_frequency` / `set_tx_gain` (attenuation-aware, range −89.75…0 dB enforced), `apply_tx_settings`, and implements `SdrDriver::has_tx`/`start_tx`/`write_samples`/`stop_tx` over `OPEN`/`WRITEBUF`/`CLOSE` with lazy-open bookkeeping mirroring the RX path. `write_samples` returns `Ok(0)` unless `start_tx` was called, and the driver defaults to maximum attenuation so it cannot be constructed into a loud state.
- **Intent:** [INT-0002](../intents/INT-0002-hardware-drivers-pluto.md)
- **Completed:** 2026-08-23T03:53:18Z
- **Files modified:** crates/sdr-hardware/src/pluto.rs
- **Commit:** `aaced66a8d41bf475dc1bdf4a410c2e663c5a947`

## T-025 (sprint 4)
- **Description:** Loopback safety controls encoding the no-emission contract in the type system — `LoopbackMode { Disabled, InternalDigital }` deliberately cannot represent the FPGA RX→TX mode (`loopback=2`); `set_loopback`, and `enter_loopback_test_mode` / `exit_loopback_test_mode` which save and restore the prior loopback mode and TX gain. Deviation from the plan's wording, for safety: attenuation is set to maximum **before** engaging loopback (quietest-first) rather than after, so the transmitter is already attenuated regardless of what follows.
- **Intent:** [INT-0002](../intents/INT-0002-hardware-drivers-pluto.md)
- **Completed:** 2026-08-23T03:56:00Z
- **Files modified:** crates/sdr-hardware/src/pluto.rs, crates/sdr-hardware/src/lib.rs
- **Commit:** `6c0b6108716e2b1135c24f8d592be7c534c3c431`

## T-026 (sprint 4)
- **Description:** TX verification — mock-iiod `WRITEBUF`/`DEBUG` regression test plus the live zero-emission loopback test. Three real defects were found and fixed by testing against the physical radio, none of which unit tests could have caught:
  1. **`WRITEBUF` is a two-phase exchange** (the risk critique C-001 predicted): iiod acks the header with a status line *before* accepting the payload, then reports bytes written. The original single-status implementation returned 0 bytes and desynchronized the connection. Client and mock server both corrected.
  2. **DDS tone generators are enabled by default** on `cf-ad9361-dds-core-lpc` and would be transmitted instead of the caller's samples — a latent bug that would have shipped. `start_tx` now disables them (`set_dds_enabled`), and loopback test mode saves/restores their state.
  3. **A one-shot TX buffer drains before it can be observed.** Added cyclic-buffer support (`OPEN … CYCLIC`, `IiodClient::open_with`, `PlutoSdr::set_tx_cyclic`), which is also how a real transmitter sustains a waveform.
- **Live evidence (internal loopback):** with `loopback=1` (RF section bypassed), TX at −89.75 dB (max attenuation) and DDS silenced, all 4096 written samples were accepted by the real daemon and read back on RX as **4096/4096 non-zero with peak |amp| = 0.7071** — exactly √(0.5²+0.5²) for the transmitted `(0.5, −0.5)` pattern, confirming both the S16 TX and S12 RX scaling. Device state (`loopback`, TX gain, DDS) verified restored afterward.
- **Intent:** [INT-0002](../intents/INT-0002-hardware-drivers-pluto.md)
- **Completed:** 2026-08-23T04:05:31Z
- **Files modified:** crates/sdr-hardware/tests/pluto_iiod.rs, crates/sdr-hardware/tests/hw_pluto.rs, crates/sdr-hardware/src/iiod.rs, crates/sdr-hardware/src/pluto.rs
- **Commit:** `7673b041836c1e42844bc0c0f99a76524d84610b`

## T-027 (sprint 5)
- **Description:** FSK modulator/demodulator round-trip regression test, making the Sprint 5 research measurement permanent: a payload containing the packet preamble and sync word round-trips **bit-exact** when sample-aligned, and demodulating from a large sample offset is asserted to corrupt the bitstream — documenting why the mesh receiver must search sample phases rather than assume alignment. The mesh radio path now depends on this pair, which previously had no round-trip coverage.
- **Intent:** [INT-0006](../intents/INT-0006-packet-radio-ssh-tunnel.md)
- **Completed:** 2026-08-23T06:00:00Z
- **Files modified:** crates/sdr-demod/tests/fsk_roundtrip.rs
- **Commit:** `e12f9c505ad3b1872ec1ccd93e4eb366cd4e619f`

## T-028 (sprint 5)
- **Description:** Bit-level frame synchronizer (`sdr-mesh::framesync`) — `bytes_to_bits`/`bits_to_bytes` (MSB-first, matching the modulator) and `sync_to_frame`, which searches the demodulated bit stream for the packet `SYNC_WORD` at **bit** granularity and repacks from there, so an arbitrary bit offset introduced by the demodulator is recovered. Returns `None` when no sync word is present. Pure and dependency-free; produces exactly the form `PacketFramer::decode` scans for.
- **Intent:** [INT-0008](../intents/INT-0008-mesh-networking-aredn.md)
- **Completed:** 2026-08-23T06:02:00Z
- **Files modified:** crates/sdr-mesh/src/framesync.rs, crates/sdr-mesh/src/lib.rs
- **Commit:** `faaf7f57ee969ba857257105a95692919e370d1f`

## T-029 (sprint 5)
- **Description:** `RadioLink` — a radio-backed `MeshInterface` generic over `SdrDriver`, so `MockSdr` serves CI and `PlutoSdr` serves hardware. TX: datagram → KISS → `ArqTransceiver` frame → `FskModulator` → `write_samples`. RX: `read_samples` → for each candidate sample phase, `FskDemod` → `framesync` → `PacketFramer::decode`, where **CRC-32 confirms the correct phase**, making the search self-verifying rather than a guess. Composes existing parts only; no new protocol logic and no new external dependencies. CI tests run over `MockSdr` loopback, including a deliberately mid-symbol stream so the phase search is genuinely exercised (the mock loopback is sample-exact and would otherwise always succeed at phase 0), plus a silence case asserting `Ok(None)` rather than an invented datagram.
- **Intent:** [INT-0008](../intents/INT-0008-mesh-networking-aredn.md)
- **Completed:** 2026-08-23T06:06:00Z
- **Files modified:** crates/sdr-mesh/src/radio.rs, crates/sdr-mesh/src/lib.rs, crates/sdr-mesh/Cargo.toml, crates/sdr-mesh/tests/radio_it.rs
- **Commit:** `6c330ec01a1fc6fc97c8dcd241e84033bfd9baf5`

## T-030 (sprint 5)
- **Description:** Live hardware verification — a mesh datagram carried through the **real PlutoSDR** with internal loopback. Under internal digital loopback (RF section bypassed), maximum attenuation and DDS silenced, a 10-byte datagram traversed the full path: KISS → ARQ frame → FSK modulation → real Pluto TX (cyclic buffer) → hardware loopback → real RX → FSK demodulation with sample-phase search → bit-level frame sync → CRC-32 → KISS decode, and was recovered **byte-for-byte**. Passed on the first live run. Device state (`loopback`, TX gain, DDS) independently confirmed restored afterward; assertions run after restoration so a failure cannot strand the radio.
- **Intent:** [INT-0008](../intents/INT-0008-mesh-networking-aredn.md)
- **Completed:** 2026-08-23T06:12:00Z
- **Files modified:** crates/sdr-mesh/tests/hw_radio.rs
- **Commit:** `12facf52dd9be7fa54db3de6c05bd57ad6a3e089`

## T-031 (sprint 6)
- **Description:** Real correctness coverage for `GardnerClockRecovery`, which had existed since Sprint 0 with none — its only test asserted the output was non-empty and roughly the right length, so a broken timing-error detector would have passed. Now asserts the recovered symbol *values* against a **non-periodic** pseudo-random sequence (an alternating pattern was rejected: with period 2 a wrong lag still aligns, hiding mismatches), plus a new `test_gardner_tracks_clock_drift` proving recovery survives a ±0.2% receiver clock offset — the condition a fixed sample phase cannot handle.
- **Intent:** [INT-0001](../intents/INT-0001-core-dsp-pipeline.md)
- **Completed:** 2026-08-23T14:20:00Z
- **Files modified:** crates/sdr-dsp/src/lib.rs
- **Commit:** `985defbdfab46ee14b1d5d063e66fdff70523ceb`

## T-032 (sprint 6)
- **Description:** `FskTimingDemod` — 2-FSK demodulation with Gardner symbol-timing recovery, added **alongside** `FskDemod` (which `PskDemod` and the Sprint 5 round-trip regression still use). Runs the frequency discriminator **first**, converting constant-envelope FSK into a real-valued PAM signal so the Gardner detector has symbol transitions to lock onto; feeding it raw FSK IQ does not work. Streaming-stateful with a `Block<Complex32, u8>` impl mirroring `FskDemod`. Tests prove bit-exact recovery when aligned, from a mid-symbol start (7 of 10 samples in — the case that corrupts roughly half the bits with the fixed-count demodulator), and across ±0.1% and ±0.5% clock offsets combined with a start offset.
- **Intent:** [INT-0006](../intents/INT-0006-packet-radio-ssh-tunnel.md)
- **Completed:** 2026-08-23T14:24:00Z
- **Files modified:** crates/sdr-demod/src/fsk.rs, crates/sdr-demod/src/lib.rs, crates/sdr-demod/tests/fsk_timing.rs
- **Commit:** `d39f8ff2f0c1ca26b4bb0431ec02db35ae7dd493`

## T-033 (sprint 6)
- **Description:** `RadioLink` now recovers datagrams in a **single** demodulation pass through `FskTimingDemod`; the `0..sps` candidate-phase loop is deleted. `sync_to_frame` + CRC-32 still provide frame alignment and validation.
- **Two findings from the switch, both caught by the regression contract rather than assumed:**
  1. **Frames need trailing flush symbols.** A timing loop consumes a symbol settling at the start of a burst, which shifts its output stream and truncated the frame's final CRC byte — every payload failed to decode, while the old fixed-count path succeeded. Fixed by appending a 2-byte `TRAILER` (`0xAA 0xAA`) after each frame: standard postamble practice, and the alternating pattern keeps the loop supplied with transitions while it flushes. Trailing bytes are harmless since the frame header is length-prefixed.
  2. **The research report's ±0.5% drift figure does not hold at frame level.** That was measured on a short (~80-bit) burst; across a full ~256-bit frame, where every bit must survive for CRC-32, the limit is about **±0.1%**, and it is slightly asymmetric (a fast receiver clock is tighter). A sweep of five loop-gain settings showed tuning does **not** widen it, so the limit is structural to this Gardner implementation — recorded as backlog T-112. The claim was corrected in `radio.rs`, `fsk.rs` and the drift test rather than asserting the optimistic number.
- **Intent:** [INT-0008](../intents/INT-0008-mesh-networking-aredn.md)
- **Completed:** 2026-08-23T14:40:00Z
- **Files modified:** crates/sdr-mesh/src/radio.rs, crates/sdr-mesh/tests/radio_it.rs, crates/sdr-demod/src/fsk.rs
- **Commit:** `b73448c5a80ce8714bcc5d5b034050a4995df4d2`

## T-034 (sprint 6)
- **Description:** Live re-verification on the physical Pluto+ with the sample-phase search removed. The existing `hw_verify_mesh_datagram_over_radio` test recovered the identical 10-byte datagram through the real radio using the single timing-recovered demodulation pass, confirming the swap did not break device integration. The known risk — a cyclic buffer's wrap discontinuity briefly unlocking the loop — did not materialise; capturing well beyond the frame length leaves a complete frame clear of the wrap. Device state (`loopback`, TX gain, DDS) independently confirmed restored afterward. Note this test cannot evidence drift tolerance: the internal loopback shares one clock, so drift is proven in CI where an offset can be injected deliberately.
- **Intent:** [INT-0008](../intents/INT-0008-mesh-networking-aredn.md)
- **Completed:** 2026-08-23T14:46:00Z
- **Files modified:** crates/sdr-mesh/tests/hw_radio.rs
- **Commit:** `45834b1fcb6bcf6b05983d1e71cecdf08fc464f1`

## T-035 (sprint 7)
- **Description:** `RadioLink::extract_payloads` now recovers **every** frame in a capture instead of stopping at the first, which is exactly what a byte stream produces (measured before: a burst of 4 datagrams yielded 1). Added `framesync::find_sync(bits, from_bit)` so the scan can resume past each decoded frame, advancing by the frame's true length (sync + header + payload + CRC) and falling back to a one-bit step when a candidate does not decode. Raised the default `rx_chunk` from 16 384 to 65 536 samples, lifting the frame-size ceiling that made payloads beyond ~180 bytes fail outright, and documented that `rx_chunk` *is* the frame-size ceiling. An oversized frame now yields `None` rather than a truncated payload.
- **Intent:** [INT-0008](../intents/INT-0008-mesh-networking-aredn.md)
- **Completed:** 2026-08-23T21:05:00Z
- **Files modified:** crates/sdr-mesh/src/radio.rs, crates/sdr-mesh/src/framesync.rs, crates/sdr-mesh/tests/radio_it.rs
- **Commit:** `2c628c8ebb8eca69b58da6787281aa5797e5951e`

## T-036 (sprint 7)
- **Description:** `StreamBridge` — carries a byte stream over any datagram `MeshInterface`, splitting outbound bytes into MTU-sized datagrams (default 128 B, sized so a worst-case KISS-escaped frame still fits one capture) and reassembling inbound datagrams into an ordered stream. Pure and I/O-free, so it is unit-testable against a fake interface and works unchanged over `MockSdr` and `PlutoSdr`. Deliberately **not** routed through `StreamTunnel`, which would frame every payload twice now that `RadioLink` owns framing. Verified over the real radio path: a 300-byte stream spanning five datagrams reassembles byte-for-byte, all 256 byte values survive (including KISS-significant `0xC0`/`0xDB`), a read with nothing available yields no bytes rather than blocking, and an SSH-style banner round-trips.
- **Intent:** [INT-0006](../intents/INT-0006-packet-radio-ssh-tunnel.md)
- **Completed:** 2026-08-23T21:10:00Z
- **Files modified:** crates/sdr-mesh/src/stream.rs, crates/sdr-mesh/src/lib.rs, crates/sdr-mesh/tests/stream_it.rs
- **Commit:** `a8e69893a4e54ece64b66fda6de40a338fd9eb85`

## T-037 (sprint 7)
- **Description:** `sdr-cli tunnel` is now a working pipe rather than a status printer. Two modes over `StreamBridge`: `--stdio` (the OpenSSH `ProxyCommand` contract) and `--listen <port>` (one TCP connection), with `--driver mock|pluto|<uri>` and a configurable `--mtu`. stdin is read on its own thread feeding a channel so a blocking read cannot stall the radio side, and the main loop polls both directions on a bounded 5 ms tick rather than spinning. The existing compliance gate is retained and still warns when the band prohibits encrypted payloads.
- **Two details that mattered:** status output was moved to **stderr**, because in `--stdio` mode stdout carries the tunnelled stream and a banner there would be read as protocol data by an SSH client; and the mock driver addresses **broadcast**, because it echoes what it transmits — a frame addressed to a distinct peer was correctly filtered out on return, which is why the first manual run produced no output.
- **Verified manually end-to-end:** `printf 'HELLO-OVER-RADIO' | sdr-cli tunnel --stdio --driver mock` returns the same bytes through modulation, framing and demodulation.
- **Intent:** [INT-0006](../intents/INT-0006-packet-radio-ssh-tunnel.md)
- **Completed:** 2026-08-23T21:25:00Z
- **Files modified:** crates/sdr-cli/src/main.rs, crates/sdr-cli/Cargo.toml, crates/sdr-cli/tests/tunnel_it.rs, Cargo.lock
- **Commit:** `92e81c70f63052ff465e888431d0294d57d8c5b5`

## T-038 (sprint 7)
- **Description:** Real OpenSSH client completes the SSH version exchange over the radio link; a multi-chunk byte stream verified over the Pluto+ under internal loopback. Bounded `StreamBridge::pump` (an unbounded drain never returns against a cyclic transmitter) and added `clear_inbound` to discard cyclic repeats.
- **Intent:** [INT-0006](../intents/INT-0006-packet-radio-ssh-tunnel.md), [INT-0008](../intents/INT-0008-mesh-networking.md)
- **Completed:** 2026-08-23T20:56:21Z
- **Files modified:** crates/sdr-cli/tests/ssh_tunnel_it.rs, crates/sdr-mesh/tests/hw_radio.rs, crates/sdr-mesh/src/stream.rs
- **Commit:** `91124b01d080d00ca54fc1b3990487806a79ce76`
- **Evidence:** `cargo test --workspace` green. Live: `cargo test -p sdr-mesh --test hw_radio -- --ignored --nocapture --test-threads=1` → 2 passed; stream test sent 87 bytes as 2 datagrams, recovered 87 byte-for-byte. ssh trace showed `Local version string SSH-2.0-OpenSSH_10.3` and `Remote protocol version 2.0`.
- **Limits recorded, not implied away:** no sshd on this machine, so a complete session (key exchange, auth, shell) is unverified and the peer banner the client sees is its own echo (T-114). A cyclic TX buffer holds one frame and repeats it, so chunks must ping-pong rather than burst — a property of one radio in loopback, not of the bridge. No ARQ retransmit on receive (T-113).

## T-039 (sprint 8)
- **Description:** Adopted the four candidate categories as intent chapters (INT-0014 satellite/space, INT-0015 distributed sensing & DF, INT-0016 propagation & beacon reporting, INT-0017 test & measurement), moved them into the Book README taxonomy, and closed a pre-existing navigation gap by adding the six missing SUMMARY links for INT-0001..INT-0006.
- **Intent:** [INT-0014](../intents/INT-0014-satellite-space-operations.md), [INT-0015](../intents/INT-0015-distributed-sensing-df.md), [INT-0016](../intents/INT-0016-propagation-beacon-reporting.md), [INT-0017](../intents/INT-0017-test-and-measurement.md)
- **Completed:** 2026-08-24T01:17:42Z
- **Files modified:** docs/intents/INT-0014-satellite-space-operations.md, docs/intents/INT-0015-distributed-sensing-df.md, docs/intents/INT-0016-propagation-beacon-reporting.md, docs/intents/INT-0017-test-and-measurement.md, docs/SUMMARY.md, docs/README.md
- **Commit:** `b5f0405e308be9dfc14202d5332483fc6e4e676f`
- **Evidence:** `check-book.sh` reports a valid v2 Book with **17** intent chapters; every chapter on disk is reachable from `SUMMARY.md` (verified by enumeration); the README's "Candidate categories" section is gone (0 occurrences).
- **Note:** All four created `proposed`, not `planned`. This task authors the chapters; it does not advance them. Marking them `planned` would assert scheduled implementation — see plan critique C-004.

## T-040 (sprint 8)
- **Description:** Published `docs/roadmap.md` — five phases plus Later/Continuous, a dependency map with one row per intent, and the five load-bearing edges called out. States plainly that phases are ordering rather than commitment, and that only intent state schedules work.
- **Intent:** [INT-0009](../intents/INT-0009-receiver-application.md), [INT-0010](../intents/INT-0010-desktop-gui-shell.md), [INT-0011](../intents/INT-0011-mesh-messaging-callsign.md), [INT-0012](../intents/INT-0012-radio-astronomy-suite.md), [INT-0013](../intents/INT-0013-ml-signal-analysis.md)
- **Completed:** 2026-08-24T01:20:14Z
- **Files modified:** docs/roadmap.md, docs/SUMMARY.md
- **Commit:** `7bb5de720146410e063a2aa2ed2812533a7b2b1a`
- **Evidence:** `test_roadmap_covers_every_intent_exactly_once`, `test_roadmap_names_blocking_dependencies` and `test_roadmap_is_reachable_from_summary` all pass; all 17 intents have exactly one dependency-map row.
- **Deviation from the locked plan, recorded not hidden:** the plan's EARS clause said every intent SHALL appear "exactly once in the **phase listing**". Implementing it exposed that the clause is not satisfiable by a sensible roadmap — the phase listing is forward-looking and deliberately omits the six already-realized chapters, and prose that mentions an intent twice (e.g. noting INT-0015/INT-0016 are cheaper than their position suggests) is useful rather than a defect. The **dependency map** is the structure that genuinely holds one canonical entry per intent, so the test asserts against that instead, plus at-least-one mention anywhere in the roadmap. This verifies what the clause is *for* — total coverage, no duplicates, no omissions — rather than its literal wording. Rewording the roadmap to satisfy the literal clause would have made it worse.
- **No dates or effort estimates** appear in the roadmap; none would be evidence-backed.

## T-041 (sprint 8)
- **Description:** Six Book-integrity tests in `crates/sdr-cli/tests/book_it.rs`, parsing `docs/` at run time: every intent reachable from SUMMARY, every intent has non-empty acceptance criteria, adopted categories no longer listed as candidates, roadmap covers every intent exactly once, roadmap names the load-bearing blocking dependencies, roadmap reachable from SUMMARY.
- **Intent:** [INT-0001](../intents/INT-0001-core-dsp-pipeline.md)
- **Completed:** 2026-08-24T01:20:33Z
- **Files modified:** crates/sdr-cli/tests/book_it.rs
- **Commit:** `f50542528af1f5b04dc281669a78fb71cec0baf8`
- **Evidence:** 6 passed, 0 failed. **Negative capability verified rather than assumed:** removing INT-0003's SUMMARY link made `test_every_intent_is_reachable_from_summary` fail with the exact chapter named (`["INT-0003"]`), and it passed again on restore. A green test that cannot go red proves nothing, so this was checked directly.
- **Why a documentation sprint carries tests at all:** these invariants rot silently — a chapter added without a navigation link or roadmap row is invisible until someone happens to notice. The tests make that failure loud instead of claiming a docs-only exemption from verification.

## T-042 (sprint 9)
- **Description:** Built the retransmission half of ARQ, which did not previously exist. `ArqTransceiver` now retains each transmitted frame (`send_data`), clears it when the matching ACK arrives, returns frames whose T1 has expired (`due_retransmissions`) with linear backoff, enforces `max_retries` and surfaces exhausted frames as permanent failures (`take_abandoned`). Shape follows AX.25 — T1/N2/backoff.
- **Intent:** [INT-0006](../intents/INT-0006-packet-radio-ssh-tunnel.md) (criterion 2)
- **Completed:** 2026-08-24T01:46:48Z
- **Files modified:** crates/sdr-protocols/src/packet.rs
- **Commit:** `b88b96dafcb6f692c43c63bc8fb8b56bb175da7d`
- **Evidence:** 5 unit tests pass; full crate suite 11 passed, 0 failed; clippy 0 errors.
- **Negative capability verified for all five**, the standard this sprint set itself: reverting the ACK arm to discarding broke `test_arq_ack_stops_retransmission`; removing the T1 comparison broke `test_arq_no_retransmission_before_timeout`; removing frame retention broke `test_arq_retransmits_after_timeout` and `test_arq_gives_up_after_max_retries`; delivering duplicates broke `test_arq_duplicate_suppressed_but_acked`. Each restored cleanly afterwards.
- **Time is a parameter, never a clock read.** Every time-dependent method takes an explicit `now_ms`. No `SystemTime::now()` in the state machine, so timeout paths run instantly and deterministically instead of via sleeps.
- **API kept additive** (plan critique C-003): `create_data_frame` and `process_rx_frame` retain their signatures, so all four existing call sites — including `tunnel.rs`, which already handled its ACK correctly — were left untouched. `tunnel.rs` needed no change after all.
- **T1 = 500 ms is a reasoned default, not a measured one.** AX.25's 3000 ms targets far slower channels; real half-duplex turnaround latency here is still unmeasured and the constant says so.

## T-043 (sprint 9)
- **Description:** Deleted `test_arq_retransmission_lossy_channel` and replaced it with an `arq_reliability` module that actually drops frames: a seeded dependency-free xorshift PRNG schedules losses, two transceivers exchange payloads with explicit simulated time, and delivery is asserted exactly-once and in order at 10% and **30%** loss — the figure INT-0006 criterion 2 names and had never had.
- **Intent:** [INT-0006](../intents/INT-0006-packet-radio-ssh-tunnel.md) (criterion 2)
- **Completed:** 2026-08-24T01:48:32Z
- **Files modified:** crates/sdr-protocols/src/lib.rs
- **Commit:** `cc25843a771d2189012d72081c52591dd48211b8`
- **Evidence:** 14 crate tests pass (4 new), clippy 0 errors.
- **The loss is proven real, not assumed.** Disabling retransmission in the state machine made **all four** reliability tests fail, each naming the exact payload that was never acknowledged (e.g. "payload 2 was never acknowledged at 30% loss"). That demonstrates frames are genuinely being dropped and that delivery depends on retransmission — precisely what the deleted test could never have shown.
- **Duplicate suppression is discriminated separately.** Disabling dedup failed `test_arq_ack_loss_does_not_duplicate_payload` alone and left the other three passing, which is correct: only the ACK-dropping scenario re-delivers an already-received frame.
- **Deletion, not amendment.** The old test's name was cited as evidence for a criterion it never verified; leaving a repaired version under that name would preserve the confusion. A comment at the old site records what happened and points to the replacement.
- **Seeded, so failures reproduce.** A reliability test that fails once in twenty runs is a flake generator; `test_arq_lossy_channel_is_deterministic` pins this.

## T-044 (sprint 9)
- **Description:** Ran ARQ on the mesh receive paths. `RadioLink` routes decoded frames through `process_rx_frame` (address filtering, duplicate suppression, ACK consumption and generation), queues ACKs, and gained `service(now_ms)` — the single place ACKs and retransmissions reach the air. `node.rs`'s discarded `_ack` is now delivered to the peer.
- **Intent:** [INT-0008](../intents/INT-0008-mesh-networking-aredn.md), [INT-0006](../intents/INT-0006-packet-radio-ssh-tunnel.md)
- **Completed:** 2026-08-24T01:51:46Z
- **Files modified:** crates/sdr-mesh/src/radio.rs, crates/sdr-mesh/src/node.rs, crates/sdr-mesh/tests/radio_it.rs
- **Commit:** `20fe4e5a5922bd3661e175477b0f0eac1e820f87`
- **Evidence:** `radio_it` 9 passed (6 carried-over + 3 new); workspace 33 suites, 0 failed; clippy 0 errors.
- **Regression contract held, which plan critique C-002 warned was at risk.** The six carried-over `radio_it` tests pass **unchanged**. Two design choices made that structural rather than lucky: ARQ is **opt-in** via `set_reliable(true)` so a fire-and-forget link behaves exactly as before, and **receiving never transmits as a side effect** — ACKs are queued and only leave in `service()`. Without both, ACK traffic echoing round the mock loopback would have disturbed the tests covering that path.
- **Negative capability verified for all three new tests:** removing ACK queueing failed `test_radiolink_acks_received_data`; removing frame retention failed `test_radiolink_retransmits_unacked_frame` and `test_radiolink_suppresses_duplicate_frames`. The six contract tests stayed green throughout, confirming they genuinely do not depend on the new paths.
- **Not verified on hardware, and not claimed to be.** One radio in internal loopback hears its own transmission, so an ACK exchange is degenerate. Meaningful hardware ARQ needs **two radios**. Verification here is simulation plus mock loopback.

## T-119 (sprint 10)
- **Description:** Replaced the unsound handwritten shared-reference ring buffer with `crossbeam_queue::ArrayQueue`, preserving the bounded usable-capacity, FIFO, and partial-I/O contracts while removing raw-pointer mutation and manual unsafe trait implementations. `sdr-core` now forbids unsafe code, and concurrent wraparound, clear/reuse, partial-count, and `Send + Sync` regressions cover the public boundary.
- **Intent:** [INT-0001](../intents/INT-0001-core-dsp-pipeline.md) (criterion 1, bounded-buffer safety slice)
- **Completed:** 2026-08-24T22:39:58Z
- **Files modified:** Cargo.toml, Cargo.lock, crates/sdr-core/Cargo.toml, crates/sdr-core/src/buffer.rs, crates/sdr-core/src/lib.rs
- **Commit:** PENDING
- **Evidence:** `cargo +1.93.0 test -p sdr-core` passed all 31 tests after T-119/T-120; `cargo +1.93.0 check -p sdr-core`, `cargo +1.93.0 clippy -p sdr-core --all-targets`, exact-file rustfmt, and scoped diff checks passed. The former implementation also fails the new crate-level unsafe prohibition by construction.
