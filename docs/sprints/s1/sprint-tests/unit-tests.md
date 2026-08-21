# Sprint 1 Unit Tests Execution

## sdr-core
- `test_sample_conversions`: sample normalization and conversion. PASS.
- `test_ring_buffer_fifo_order`: SPSC lock-free ring buffer FIFO ordering. PASS.
- `test_ring_buffer_wraparound`: circular wraparound integrity across multi-cycles. PASS.
- `test_stream_tag_propagation`: synchronous stream tag querying and range retention. PASS.
- `test_regulatory_compliance_ism_bands`: queried US, EU, AU, Global license-free ISM/SRD bands; verified EIRP/ERP power limits and encryption legality. PASS.
- `test_regulatory_compliance_amateur_encryption_rejection`: verified that encrypted transmissions on Amateur bands (144.2 MHz / 433 MHz) return explicit regulatory violation warnings. PASS.

## sdr-dsp
- `test_costas_loop_phase_lock`: 2nd-order Costas carrier phase synchronization. PASS.
- `test_gardner_clock_recovery`: symbol timing clock recovery with Hermite cubic interpolation. PASS.
- `test_nco_frequency_shift_and_phase_continuity`: NCO phase continuity and mixing across buffer boundaries. PASS.
- `test_window_functions`: Rectangular, Hann, Hamming, Blackman-Harris, Kaiser window coefficients. PASS.
- `test_hilbert_transform`: Hilbert filter analytic signal generation. PASS.
- `test_polyphase_resampler_ratio`: rational polyphase filterbank resampling. PASS.
- `test_fir_lowpass_attenuation`: windowed-sinc FIR filter stopband attenuation > 40 dB. PASS.

## sdr-hardware
- `test_mock_sdr_tx_loopback`: verified SdrDriver TX streaming via write_samples() and loopback reception. PASS.
- `test_pluto_iio_endpoint_url_parsing`: verified IIO URI parsing and initialization. PASS.
- `test_sdr_driver_trait_mock_streaming`: verified device discovery, configuration, and RX streaming. PASS.
- `test_wav_reader_writer`: verified RIFF 16-bit stereo IQ WAV reader and writer. PASS.
- `test_sigmf_roundtrip_metadata_and_samples`: verified SigMF v1.0.0 JSON metadata and cf32 binary dataset storage. PASS.

## sdr-demod
- `test_fsk_demod_bitstream_recovery`: FSK frequency discriminator and bit slicing. PASS.
- `test_nfm_demod_with_squelch`: NFM demodulation with squelch. PASS.
- `test_ook_demod_bits`: OOK/ASK energy threshold demodulation. PASS.
- `test_ssb_phasing_demod`: SSB phasing method baseband separation. PASS.
- `test_am_envelope_demod`: AM envelope demodulation with DC blocking. PASS.
- `test_gfsk_modulator_continuous_phase`: verified continuous-phase Gaussian frequency shift keying modulation and unit envelope magnitude. PASS.
- `test_wfm_demod_synthetic_tone`: WFM audio recovery with correlation > 0.98. PASS.

## sdr-protocols
- `test_adsb_mode_s_crc24_and_decode`: Mode S DF17 message decoding and CRC-24 check. PASS.
- `test_packet_framing_crc32`: verified preamble, sync word, headers, payload, and IEEE 802.3 CRC-32 encoding and decoding. PASS.
- `test_aprs_afsk_decode`: 1200 baud Bell 202 AFSK AX.25 UI frame parsing. PASS.
- `test_stream_tunnel_bidirectional`: verified bidirectional SSH stream tunneling and packet exchange. PASS.
- `test_arq_retransmission_lossy_channel`: verified Stop-and-Wait ARQ retransmission under simulated 25% packet drops with 100% in-order delivery. PASS.
- `test_lora_css_demod_and_fec`: LoRa chirp synthesis, FFT symbol demodulation, and FEC. PASS.

## sdr-spectrum
- `test_rigctl_tcp_command_handling`: Hamlib TCP command protocol engine. PASS.
- `test_spectrum_analyzer_fft_and_cfar`: 1024-point FFT power spectrum calculation and CA-CFAR detection. PASS.

Total: 33 unit tests passed, 0 failed.
