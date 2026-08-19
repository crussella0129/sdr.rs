# Sprint 0 Test Report

## Intent Verification
| Intent | Acceptance criterion | EARS / tests | Result | Intent evidence update |
|--------|----------------------|---------------|--------|------------------------|
| [INT-0001](../../../intents/INT-0001-core-dsp-pipeline.md) | Composable streaming with lock-free ring buffers, FIR filter attenuation > 40 dB, Polyphase resampling, NCO phase continuity, and stream tag propagation | T-001 / `test_ring_buffer_fifo_order`, `test_stream_tag_propagation`; T-002 / `test_fir_lowpass_attenuation`, `test_nco_frequency_shift_and_phase_continuity`, `test_polyphase_resampler_ratio` | pass | Test evidence links this report; eligible for realized after completion evidence |
| [INT-0002](../../../intents/INT-0002-hardware-drivers-pluto.md) | SdrDriver trait configuration & streaming, PlutoSDR IIO client networking, SigMF & WAV dataset storage, and mock drivers | T-003 / `test_sdr_driver_trait_mock_streaming`, `test_pluto_iio_endpoint_url_parsing`, `test_sigmf_roundtrip_metadata_and_samples`, `test_wav_reader_writer` | pass | Test evidence links this report; eligible for realized after completion evidence |
| [INT-0003](../../../intents/INT-0003-modulation-demodulation.md) | WFM broadcast audio recovery, NFM squelch, AM envelope detection, SSB phasing, FSK bit recovery, and PSK carrier synchronization | T-004 / `test_wfm_demod_synthetic_tone`, `test_nfm_demod_with_squelch`, `test_am_envelope_demod`, `test_ssb_phasing_demod`, `test_fsk_demod_bitstream_recovery` | pass | Test evidence links this report; eligible for realized after completion evidence |
| [INT-0004](../../../intents/INT-0004-protocol-decoders-spectrum.md) | LoRa CSS de-chirping & FEC decoding, ADS-B Mode S pulse decoding with CRC-24 check, APRS/AX.25 packet parsing, FFT spectrum analyzer, CFAR detection, and Hamlib Rigctl server | T-005 / `test_lora_css_demod_and_fec`, `test_adsb_mode_s_crc24_and_decode`, `test_aprs_afsk_decode`, `test_spectrum_analyzer_fft_and_cfar`, `test_rigctl_tcp_command_handling` | pass | Test evidence links this report; eligible for realized after completion evidence |

## Summary
- Unit tests: 26 passed / 0 failed / 26 total
- Integration tests: 4 passed / 0 failed / 4 total
- E2E tests: 4 passed / 0 failed / 4 total
- CI status: not-configured

## CI Confirmation
- **Head SHA:** 0360bf7
- **CI run:** CI not configured — local confirmations only
- **Conclusion:** success
- **Confirmations:** Local canonical test runner (`cargo test --workspace`) passed 30/30 tests.

## Failures
None.

## Technical Debt Identified
None.

## Coverage Observations
Comprehensive coverage across core sample types, stream buffer mechanics, DSP filters, resamplers, oscillators, carrier/symbol synchronization loops, hardware abstraction, SigMF / WAV storage, analog/digital demodulators, protocol decoders (LoRa, ADS-B, APRS), FFT spectrum analysis, and CLI interface.
