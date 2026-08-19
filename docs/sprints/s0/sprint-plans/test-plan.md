Finalized - DO NOT EDIT

# Sprint 0 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | 1. Composable streaming with lock-free ring buffers | T-001 / WHEN RingBuffer is instantiated ... THEN writing/reading SHALL succeed | `test_ring_buffer_fifo_order`, `test_ring_buffer_wraparound` |
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | 2. FIR filter attenuation > 40 dB | T-002 / WHEN a low-pass FIR filter is designed ... THEN attenuate stopband >= 40 dB | `test_fir_lowpass_attenuation`, `test_window_functions` |
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | 3. Rational polyphase resampling | T-002 / WHEN PolyphaseResampler resamples ... THEN output length SHALL equal floor(len * R2 / R1) | `test_polyphase_resampler_ratio` |
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | 4. NCO frequency mixing and phase continuity | T-002 / WHEN Nco mixes input tone ... THEN output spectrum SHALL be shifted by df | `test_nco_frequency_shift_and_phase_continuity` |
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | 5. In-band stream tag propagation | T-001 / WHEN StreamTag is attached ... THEN downstream blocks SHALL observe tag | `test_stream_tag_propagation` |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | 1. SdrDriver trait configuration & streaming | T-003 / WHEN MockSdr is configured ... THEN rx_stream SHALL yield continuous IQ | `test_sdr_driver_trait_mock_streaming` |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | 2. PlutoSDR IIO client networking | T-003 / WHEN PlutoSdr connects ... THEN configure LO frequency and sample rate | `test_pluto_iio_endpoint_url_parsing` |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | 3. SigMF and WAV dataset storage | T-003 / WHEN SigMfWriter saves samples ... THEN reconstructed dataset SHALL match | `test_sigmf_roundtrip_metadata_and_samples`, `test_wav_reader_writer` |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | 4. Graceful error handling & teardown | T-003 / Graceful error handling for unreachable endpoints | `test_driver_error_handling` |
| [INT-0003](../../../intents/INT-0003-modulation-demodulation.md) | 1. WFM demodulation | T-004 / WHEN FM modulated IQ is processed ... THEN reconstruct audio tone | `test_wfm_demod_synthetic_tone` |
| [INT-0003](../../../intents/INT-0003-modulation-demodulation.md) | 2. NFM squelch and discriminator | T-004 / Configurable hysteresis squelch | `test_nfm_demod_with_squelch` |
| [INT-0003](../../../intents/INT-0003-modulation-demodulation.md) | 3. AM and SSB demodulation | T-004 / WHEN AM modulated IQ is passed ... THEN envelope detector outputs audio | `test_am_envelope_demod`, `test_ssb_phasing_demod` |
| [INT-0003](../../../intents/INT-0003-modulation-demodulation.md) | 4. FSK/GFSK demodulation | T-004 / WHEN binary FSK symbols are processed ... THEN recover bitstream | `test_fsk_demod_bitstream_recovery` |
| [INT-0003](../../../intents/INT-0003-modulation-demodulation.md) | 5. PSK Costas loop & clock recovery | T-004 / Costas loop and symbol synchronization | `test_costas_loop_phase_lock`, `test_gardner_clock_recovery` |
| [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md) | 1. LoRa CSS de-chirping & FEC decoding | T-005 / WHEN synthetic LoRa frame is provided ... THEN extract payload & CRC | `test_lora_css_demod_and_fec` |
| [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md) | 2. ADS-B Mode S pulse decoding & CRC-24 | T-005 / WHEN ADS-B Mode S message is received ... THEN validate CRC-24 | `test_adsb_mode_s_crc24_and_decode` |
| [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md) | 3. FFT spectrum analyzer & CFAR detector | T-005 / Spectrum analyzer computes FFT power spectrum | `test_spectrum_analyzer_fft_and_cfar` |
| [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md) | 4. Hamlib Rigctl TCP command protocol | T-005 / WHEN RigctlServer receives 'f\n' ... THEN respond with frequency | `test_rigctl_tcp_command_handling` |

## Unit Tests
### T-001 unit tests
- **Intent:** [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md)
- `test_ring_buffer_fifo_order`: write sequence of Complex32 samples, read back in exact order.
- `test_ring_buffer_wraparound`: write past ring buffer capacity and verify wraparound indexing.
- `test_stream_tag_propagation`: attach metadata tag with key `center_freq` and verify tag retention at offset.

### T-002 unit tests
- **Intent:** [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md)
- `test_fir_lowpass_attenuation`: generate 1 kHz passband and 20 kHz stopband tone at 48 kHz sample rate; verify stopband attenuation > 40 dB.
- `test_window_functions`: compute Hamming, Hann, Blackman-Harris, and Kaiser windows; verify symmetry and boundary values.
- `test_nco_frequency_shift_and_phase_continuity`: shift 10 kHz tone by -10 kHz; verify baseband DC peak with zero phase jump across chunks.
- `test_polyphase_resampler_ratio`: resample 48 kHz to 44.1 kHz; verify sample length ratio matches theoretical fractional step.
- `test_hilbert_transform`: input real cosine; verify imaginary output is 90-degree shifted sine.
- `test_costas_loop_phase_lock`: input BPSK signal with 0.2 rad phase offset; verify phase convergence within 200 iterations.
- `test_gardner_clock_recovery`: input oversampled PAM-2 signal; verify timing error converges to zero.

### T-003 unit tests
- **Intent:** [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md)
- `test_sdr_driver_trait_mock_streaming`: configure MockSdr; verify sample generation rate and non-blocking read behavior.
- `test_pluto_iio_endpoint_url_parsing`: test URI parsing for `ip:192.168.1.10`, `usb:1.2.3`, and `local:`.
- `test_sigmf_roundtrip_metadata_and_samples`: write SigMF archive (cf32_le format) and read back; verify JSON metadata fields and sample values.
- `test_wav_reader_writer`: write 16-bit stereo IQ WAV file and read back; verify RIFF header and sample data.

### T-004 unit tests
- **Intent:** [INT-0003](../../../intents/INT-0003-modulation-demodulation.md)
- `test_wfm_demod_synthetic_tone`: modulate 1 kHz audio onto 200 kHz WFM carrier; demodulate and compute cross-correlation.
- `test_nfm_demod_with_squelch`: test discriminator output with high SNR and verify squelch mute on pure noise input.
- `test_am_envelope_demod`: modulate 400 Hz tone on AM carrier; verify envelope demodulation output.
- `test_ssb_phasing_demod`: generate upper sideband tone; verify suppression of lower sideband image.
- `test_fsk_demod_bitstream_recovery`: modulate bit sequence `0b10110010` via 2-FSK; verify exact bit sequence extraction.

### T-005 unit tests
- **Intent:** [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md)
- `test_lora_css_demod_and_fec`: synthesize LoRa CSS chirp frame with known payload; verify de-chirp FFT peak recovery and Hamming FEC decode.
- `test_adsb_mode_s_crc24_and_decode`: provide standard Mode S DF17 test vector; verify CRC-24 zero remainder and decoded ICAO address `0x4840D6`.
- `test_aprs_afsk_decode`: synthesize 1200 baud Bell 202 tones for an APRS packet; verify AX.25 frame extraction.
- `test_spectrum_analyzer_fft_and_cfar`: compute 1024-point FFT on synthetic single-tone signal; verify CFAR peak detection at exact frequency bin.
- `test_rigctl_tcp_command_handling`: send `\dump_state` and `f` commands to RigctlServer protocol parser; verify compliant response format.

## Integration Tests
### Pipeline Integration
- **Intents:** [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md), [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md), [INT-0003](../../../intents/INT-0003-modulation-demodulation.md)
- `test_mock_to_wfm_audio_pipeline`: MockSdr source -> NCO frequency shifter -> PolyphaseResampler -> WfmDemod -> Audio buffer sink; verify end-to-end sample flow and audio recovery.
- `test_mock_to_spectrum_analyzer_pipeline`: MockSdr source -> SpectrumAnalyzer -> CFAR peak reporter; verify real-time spectral peak tracking.

## End-to-End Tests
- **Status:** possible
- `test_sdr_cli_end_to_end`: execute `sdr-cli` binary against synthetic SigMF captures and verify CLI exit status, decoded data summary, and exported audio.
