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
- **Commit:** PENDING
