//! End-to-End Pipeline Integration Tests for sdr.rs.

use sdr_core::sample::Complex32;
use sdr_demod::{DeEmphasis, FskDemod, WfmDemod};
use sdr_dsp::fir::{design_lowpass, FirFilter};
use sdr_dsp::nco::Nco;
use sdr_dsp::resample::PolyphaseResampler;
use sdr_dsp::window::WindowType;
use sdr_hardware::driver::SdrDriver;
use sdr_hardware::mock::{MockSdr, MockSignal};
use sdr_hardware::sigmf::{SigMfReader, SigMfWriter};
use sdr_protocols::adsb::{AdsbDecoder, DownlinkFormat};
use sdr_protocols::lora::{LoraDecoder, SpreadingFactor};
use sdr_spectrum::cfar::CaCfarDetector;
use sdr_spectrum::fft::SpectrumAnalyzer;
use std::f32::consts::TAU;

#[test]
fn test_mock_to_wfm_audio_pipeline() {
    let sample_rate = 240000.0f64;
    let center_freq = 100.1e6;

    // 1. Hardware Source: Mock SDR tuned to 100.1 MHz
    let mut sdr = MockSdr::new(sample_rate, center_freq);
    sdr.start_rx().unwrap();

    let chunk_size = 4800;
    let mut raw_iq = vec![Complex32::default(); chunk_size];
    sdr.read_samples(&mut raw_iq).unwrap();
    sdr.stop_rx().unwrap();

    // 2. Frequency Shifter: Shift center by -10 kHz
    let mut nco = Nco::new(sample_rate, 10000.0);
    let mut shifted_iq = vec![Complex32::default(); chunk_size];
    nco.mix_block(&raw_iq, &mut shifted_iq);

    // 3. Channel Filter: Lowpass 80 kHz
    let taps = design_lowpass(sample_rate, 80000.0, 45, WindowType::BlackmanHarris);
    let mut filter = FirFilter::new(taps);
    let mut filtered_iq = vec![Complex32::default(); chunk_size];
    filter.filter_block(&shifted_iq, &mut filtered_iq);

    // 4. Demodulator: WFM Demod
    let mut demod = WfmDemod::new(sample_rate as f32, 75000.0, DeEmphasis::Eu50us);
    let mut audio = Vec::new();
    demod.demod_block(&filtered_iq, &mut audio);

    assert_eq!(audio.len(), chunk_size);
    // Audio samples should be in [-1.0, 1.0] range
    for &sample in &audio {
        assert!(sample >= -1.0 && sample <= 1.0);
    }
}

#[test]
fn test_mock_to_spectrum_analyzer_pipeline() {
    let sample_rate = 2000000.0f64;
    let center_freq = 433.92e6;
    let mut sdr = MockSdr::new(sample_rate, center_freq);
    sdr.set_signal(MockSignal::Tone {
        offset_hz: 500000.0,
        amplitude: 0.9,
    });
    sdr.start_rx().unwrap();

    let fft_size = 1024;
    let mut buffer = vec![Complex32::default(); fft_size];
    sdr.read_samples(&mut buffer).unwrap();
    sdr.stop_rx().unwrap();

    let mut analyzer = SpectrumAnalyzer::new(fft_size, WindowType::BlackmanHarris, 1.0);
    let spectrum = analyzer.process(&buffer);
    let (peak_bin, peak_pwr) = SpectrumAnalyzer::find_peak_in(spectrum);

    // Peak at +500 kHz (+fs/4) -> bin 768
    assert_eq!(peak_bin, 768);
    assert!(peak_pwr > -10.0);

    let cfar = CaCfarDetector::new(4, 16, 12.0);
    let detections = cfar.detect(spectrum);
    assert!(!detections.is_empty());
    assert!(detections.iter().any(|d| d.0 == 768));
}

#[test]
fn test_sigmf_resample_fsk_pipeline() {
    let temp_dir = std::env::temp_dir();
    let capture_path = temp_dir.join("test_fsk_pipeline");

    let sample_rate = 96000.0f64;
    let baud_rate = 9600.0f64;
    let deviation = 4800.0f32;
    let sps = (sample_rate / baud_rate) as usize; // 10 sps

    let test_bits = vec![1, 0, 1, 1, 0, 1, 0, 0, 1, 1];
    let mut iq = Vec::new();
    let mut phase = 0.0f32;

    for &bit in &test_bits {
        let freq = if bit == 1 { deviation } else { -deviation };
        let phase_inc = (freq / sample_rate as f32) * TAU;
        for _ in 0..sps {
            let (s_sin, s_cos) = phase.sin_cos();
            iq.push(Complex32::new(s_cos, s_sin));
            phase = (phase + phase_inc).rem_euclid(TAU);
        }
    }

    // 1. Save to SigMF archive
    let mut writer = SigMfWriter::create(&capture_path, sample_rate, 433.92e6).unwrap();
    writer.write_samples(&iq).unwrap();
    writer.close().unwrap();

    // 2. Load from SigMF
    let mut reader = SigMfReader::open(&capture_path).unwrap();
    let mut loaded_iq = vec![Complex32::default(); iq.len()];
    let n = reader.read_samples(&mut loaded_iq).unwrap();
    assert_eq!(n, iq.len());

    // 3. Demodulate FSK from loaded SigMF dataset
    let mut fsk_demod = FskDemod::new(sample_rate as f32, deviation, sps);
    let mut recovered_bits = Vec::new();
    fsk_demod.demod_bits(&loaded_iq, &mut recovered_bits);
    assert_eq!(recovered_bits, test_bits);

    // 4. Test Resampling pipeline (96 kHz to 48 kHz)
    let mut resampler = PolyphaseResampler::new(96000.0, 48000.0);
    let mut resampled_iq = Vec::new();
    resampler.resample(&loaded_iq, &mut resampled_iq);
    assert_eq!(resampled_iq.len(), loaded_iq.len() / 2);

    // Cleanup
    let _ = std::fs::remove_file(capture_path.with_extension("sigmf-meta"));
    let _ = std::fs::remove_file(capture_path.with_extension("sigmf-data"));
}

#[test]
fn test_multidecoder_verification() {
    // Verify LoRa decoder
    let lora = LoraDecoder::new(SpreadingFactor::SF8);
    let chirp = lora.synthesize_symbol(100);
    let decoded = lora.demodulate_symbol(&chirp).unwrap();
    assert_eq!(decoded, 100);

    // Verify ADS-B Mode S decoder
    let adsb_hex: [u8; 14] = [
        0x8D, 0x48, 0x40, 0xD6, 0x20, 0x2C, 0xC3, 0x71, 0xC3, 0x2C, 0xE0, 0x57, 0x60, 0x98,
    ];
    let adsb_msg = AdsbDecoder::parse_message(&adsb_hex).unwrap();
    assert_eq!(adsb_msg.df, DownlinkFormat::ExtendedSquitter);
    assert_eq!(adsb_msg.icao_address, 0x4840D6);
    assert!(adsb_msg.crc_valid);
}
