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
- **Commit:** `669a74d1954340b9dd85bed474a827e0993a98eb`
