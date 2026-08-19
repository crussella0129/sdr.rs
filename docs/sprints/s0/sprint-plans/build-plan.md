Finalized - DO NOT EDIT

# Sprint 0 Build Plan

## Intents
- [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) — state: planned; acceptance criteria covered: 1, 2, 3, 4, 5
- [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) — state: planned; acceptance criteria covered: 1, 2, 3, 4
- [INT-0003](../../../intents/INT-0003-modulation-demodulation.md) — state: planned; acceptance criteria covered: 1, 2, 3, 4, 5
- [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md) — state: planned; acceptance criteria covered: 1, 2, 3, 4

## Schema Tree
- Sprint 0: Foundation & Core SDR Suite Implementation
  - Workspace & Core Engine (INT-0001)
    - T-001: Cargo workspace setup and sdr-core streaming architecture
    - T-002: sdr-dsp filtering, SIMD convolution, NCO, resamplers, and synchronization
  - Hardware Driver Layer (INT-0002)
    - T-003: sdr-hardware abstraction, PlutoSDR IIO client, SigMF, and mock drivers
  - Demodulation Engine (INT-0003)
    - T-004: sdr-demod analog (WFM/NFM/AM/SSB/CW) and digital (OOK/FSK/PSK) pipelines
  - Protocol Decoders & Spectrum Analysis (INT-0004)
    - T-005: sdr-protocols (LoRa, ADS-B, APRS) and sdr-spectrum (FFT, CFAR, Rigctl)
  - CLI & End-to-End Verification (INT-0001, INT-0002, INT-0003, INT-0004)
    - T-006: sdr-cli tool and end-to-end integration testing

## Execution Sequence

### T-001: Cargo workspace setup and sdr-core streaming architecture
- **Intent:** [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md)
- **Touches:** Cargo.toml, crates/sdr-core/Cargo.toml, crates/sdr-core/src/lib.rs, crates/sdr-core/src/sample.rs, crates/sdr-core/src/buffer.rs, crates/sdr-core/src/tag.rs, crates/sdr-core/src/traits.rs
- **Depends on:** (none)
- **Acceptance criterion:** Core streaming traits (`Source`, `Sink`, `Block`, `Stream`) permit composable pipeline execution with lock-free bounded ring buffers; sample tags propagate with sample offsets.
- **Success criterion (EARS):**
  - **WHEN** `RingBuffer` is instantiated with capacity N, **THEN** writing M <= N samples **SHALL** succeed and reading M samples **SHALL** return them in FIFO order without loss.
  - **WHEN** `StreamTag` is attached at sample offset K, **THEN** downstream blocks **SHALL** observe the tag at the exact sample index K.
- **Notes:** Define `Complex32` and `Complex64` type aliases wrapping `num_complex::Complex32` and `Complex64`.

### T-002: sdr-dsp filtering, SIMD convolution, NCO, resamplers, and synchronization
- **Intent:** [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md)
- **Touches:** crates/sdr-dsp/Cargo.toml, crates/sdr-dsp/src/lib.rs, crates/sdr-dsp/src/fir.rs, crates/sdr-dsp/src/window.rs, crates/sdr-dsp/src/nco.rs, crates/sdr-dsp/src/resample.rs, crates/sdr-dsp/src/hilbert.rs, crates/sdr-dsp/src/costas.rs, crates/sdr-dsp/src/clock_recovery.rs
- **Depends on:** T-001
- **Acceptance criterion:** FIR filter implementation passes frequency response validation; polyphase resamplers resample IQ streams with > 60 dB stopband rejection; NCO shifts frequency with phase continuity.
- **Success criterion (EARS):**
  - **WHEN** a low-pass FIR filter is designed with cutoff fc and transition width tw, **THEN** the filter **SHALL** attenuate stopband signals by >= 40 dB.
  - **WHEN** `Nco` mixes an input tone with frequency offset df, **THEN** the output spectrum **SHALL** be shifted by exactly df.
  - **WHEN** `PolyphaseResampler` resamples from rate R1 to R2, **THEN** the output stream **SHALL** have length `floor(input_len * R2 / R1)` without phase distortion.
- **Notes:** Support Hamming, Hann, Blackman-Harris, and Kaiser windows.

### T-003: sdr-hardware abstraction, PlutoSDR IIO client, SigMF, and mock drivers
- **Intent:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- **Touches:** crates/sdr-hardware/Cargo.toml, crates/sdr-hardware/src/lib.rs, crates/sdr-hardware/src/driver.rs, crates/sdr-hardware/src/pluto.rs, crates/sdr-hardware/src/sigmf.rs, crates/sdr-hardware/src/wav.rs, crates/sdr-hardware/src/mock.rs
- **Depends on:** T-001
- **Acceptance criterion:** `SdrDriver` trait abstracts device configuration and streaming; PlutoSDR driver communicates over network IIO; SigMF reader/writer parses metadata and datasets; Mock driver enables hardware-free test execution.
- **Success criterion (EARS):**
  - **WHEN** `MockSdr` is configured with center frequency F and sample rate S, **THEN** `rx_stream()` **SHALL** yield continuous synthetic IQ samples at rate S.
  - **WHEN** `SigMfWriter` saves samples with JSON metadata and `SigMfReader` loads them, **THEN** the reconstructed dataset **SHALL** match original samples byte-for-byte.
  - **WHEN** `PlutoSdr` connects to an IIO endpoint, **THEN** it **SHALL** configure LO frequency, sample rate, and RF gain through IIO attributes.
- **Notes:** Provide graceful error handling for unreachable network endpoints.

### T-004: sdr-demod analog and digital demodulation pipelines
- **Intent:** [INT-0003](../../../intents/INT-0003-modulation-demodulation.md)
- **Touches:** crates/sdr-demod/Cargo.toml, crates/sdr-demod/src/lib.rs, crates/sdr-demod/src/wfm.rs, crates/sdr-demod/src/nfm.rs, crates/sdr-demod/src/am.rs, crates/sdr-demod/src/ssb.rs, crates/sdr-demod/src/cw.rs, crates/sdr-demod/src/fsk.rs, crates/sdr-demod/src/ook.rs, crates/sdr-demod/src/psk.rs
- **Depends on:** T-001, T-002
- **Acceptance criterion:** Analog demodulators (WFM, NFM, AM, SSB, CW) extract audio waveforms; digital demodulators (OOK, FSK, PSK) recover binary bitstreams with carrier and symbol synchronization.
- **Success criterion (EARS):**
  - **WHEN** frequency modulated IQ data is processed by `WfmDemod`, **THEN** the demodulated audio **SHALL** reconstruct the modulating audio tone with correlation > 0.99.
  - **WHEN** binary FSK symbols are processed by `FskDemod`, **THEN** the recovered bitstream **SHALL** match the transmitted bits with zero bit errors in noiseless conditions.
  - **WHEN** amplitude modulated IQ is passed to `AmDemod`, **THEN** the envelope detector **SHALL** output baseband audio proportional to signal envelope.
- **Notes:** Include hysteresis squelch for NFM and pilot tone filter for WFM.

### T-005: sdr-protocols decoders and sdr-spectrum analysis
- **Intent:** [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md)
- **Touches:** crates/sdr-protocols/Cargo.toml, crates/sdr-protocols/src/lib.rs, crates/sdr-protocols/src/lora.rs, crates/sdr-protocols/src/adsb.rs, crates/sdr-protocols/src/aprs.rs, crates/sdr-spectrum/Cargo.toml, crates/sdr-spectrum/src/lib.rs, crates/sdr-spectrum/src/fft.rs, crates/sdr-spectrum/src/cfar.rs, crates/sdr-spectrum/src/rigctl.rs
- **Depends on:** T-001, T-002
- **Acceptance criterion:** LoRa decoder performs CSS de-chirping and FEC decoding; ADS-B decodes Mode S messages with CRC-24 check; spectrum analyzer computes FFT power spectrum with CFAR peak detection; Rigctl server handles Hamlib TCP commands.
- **Success criterion (EARS):**
  - **WHEN** a synthetic LoRa frame (SF7, CR 4/5) is provided, **THEN** `LoraDecoder` **SHALL** extract the payload bytes and pass CRC-16 check.
  - **WHEN** an ADS-B Mode S preamble and 112-bit message is provided, **THEN** `AdsbDecoder` **SHALL** validate CRC-24 parity and extract the aircraft ICAO address.
  - **WHEN** `RigctlServer` receives the TCP string `f\n`, **THEN** it **SHALL** respond with the current VFO frequency as an ASCII integer followed by a newline.
- **Notes:** Use `rustfft` for spectrum FFT calculations.

### T-006: sdr-cli tool and end-to-end integration testing
- **Intent:** [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md), [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md), [INT-0003](../../../intents/INT-0003-modulation-demodulation.md), [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md)
- **Touches:** crates/sdr-cli/Cargo.toml, crates/sdr-cli/src/main.rs, tests/e2e_pipeline_tests.rs
- **Depends on:** T-001, T-002, T-003, T-004, T-005
- **Acceptance criterion:** CLI provides subcommands for device inspection, recording, playback, demodulation, and spectrum analysis; integration tests verify end-to-end pipeline execution from mock source through DSP and demodulation to sink.
- **Success criterion (EARS):**
  - **WHEN** `sdr-cli` is invoked with `--help`, **THEN** it **SHALL** display usage instructions and available subcommands.
  - **WHEN** end-to-end integration test runs, **THEN** synthetic IQ data generated by `MockSdr` **SHALL** flow through `PolyphaseResampler` and `WfmDemod` without data loss or buffer stalls.
- **Notes:** CLI built with `clap` with colored output and clear diagnostic messages.
