# Sprint 0 Unit Tests Execution

## sdr-core
- `test_sample_conversions`: verified Complex32, Complex64, ComplexI16, ComplexI8, ComplexU8 normalization and scaling. PASS.
- `test_ring_buffer_fifo_order`: verified bounded lock-free circular buffer FIFO ordering and capacity. PASS.
- `test_ring_buffer_wraparound`: verified multiple circular wrap-around cycles without data loss. PASS.
- `test_stream_tag_propagation`: verified in-band metadata tag queuing, range querying, and pruning. PASS.

## sdr-dsp
- `test_window_functions`: verified Rectangular, Hann, Hamming, Blackman-Harris, and Kaiser window generators. PASS.
- `test_fir_lowpass_attenuation`: verified windowed-sinc FIR filter lowpass attenuation > 40 dB in stopband. PASS.
- `test_nco_frequency_shift_and_phase_continuity`: verified NCO mixing and phase continuity across chunk boundaries. PASS.
- `test_polyphase_resampler_ratio`: verified rational polyphase resampler rate conversion length accuracy. PASS.
- `test_hilbert_transform`: verified analytic complex signal generation and 90-degree phase shift. PASS.
- `test_costas_loop_phase_lock`: verified 2nd-order Costas loop carrier phase convergence. PASS.
- `test_gardner_clock_recovery`: verified symbol timing synchronizer with Hermite cubic interpolation. PASS.

## sdr-hardware
- `test_sdr_driver_trait_mock_streaming`: verified SdrDriver configuration and sample generation. PASS.
- `test_pluto_iio_endpoint_url_parsing`: verified IP, USB, and local IIO URI endpoint parsing. PASS.
- `test_sigmf_roundtrip_metadata_and_samples`: verified SigMF v1.0.0 JSON metadata and cf32 binary dataset storage. PASS.
- `test_wav_reader_writer`: verified RIFF 16-bit stereo IQ WAV reader and writer. PASS.

## sdr-demod
- `test_wfm_demod_synthetic_tone`: verified Wideband FM demodulator audio recovery with correlation > 0.98. PASS.
- `test_nfm_demod_with_squelch`: verified Narrowband FM demodulator with hysteresis squelch. PASS.
- `test_am_envelope_demod`: verified AM envelope demodulation with single-pole DC blocker. PASS.
- `test_ssb_phasing_demod`: verified Single Sideband (USB/LSB) phasing baseband separation. PASS.
- `test_fsk_demod_bitstream_recovery`: verified 2-FSK frequency discriminator bit recovery. PASS.
- `test_ook_demod_bits`: verified OOK/ASK energy threshold demodulation. PASS.

## sdr-protocols
- `test_lora_css_demod_and_fec`: verified LoRa chirp synthesis, conjugate de-chirp FFT symbol demodulation, and FEC. PASS.
- `test_adsb_mode_s_crc24_and_decode`: verified Mode S DF17 message decoding, CRC-24 check, and callsign extraction. PASS.
- `test_aprs_afsk_decode`: verified 1200 baud Bell 202 AFSK AX.25 UI frame parsing and callsign decoding. PASS.

## sdr-spectrum
- `test_spectrum_analyzer_fft_and_cfar`: verified 1024-point FFT power spectrum calculation and CA-CFAR detection. PASS.
- `test_rigctl_tcp_command_handling`: verified Hamlib Rigctl protocol command handling for frequency, mode, and dump_state. PASS.

Total: 26 unit tests passed, 0 failed.
