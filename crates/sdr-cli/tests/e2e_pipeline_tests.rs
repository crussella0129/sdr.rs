//! End-to-End Pipeline Integration Tests for sdr.rs.

use sdr_core::compliance::{ComplianceResult, Jurisdiction, RegulatoryDatabase};
use sdr_core::sample::Complex32;
use sdr_demod::modulator::GfskModulator;
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
use sdr_protocols::tunnel::StreamTunnel;
use sdr_spectrum::cfar::CaCfarDetector;
use sdr_spectrum::fft::SpectrumAnalyzer;
use std::f32::consts::TAU;

#[test]
fn test_mock_to_wfm_audio_pipeline() {
    let sample_rate = 240000.0f64;
    let center_freq = 100.1e6;

    let mut sdr = MockSdr::new(sample_rate, center_freq);
    sdr.start_rx().unwrap();

    let chunk_size = 4800;
    let mut raw_iq = vec![Complex32::default(); chunk_size];
    sdr.read_samples(&mut raw_iq).unwrap();
    sdr.stop_rx().unwrap();

    let mut nco = Nco::new(sample_rate, 10000.0);
    let mut shifted_iq = vec![Complex32::default(); chunk_size];
    nco.mix_block(&raw_iq, &mut shifted_iq);

    let taps = design_lowpass(sample_rate, 80000.0, 45, WindowType::BlackmanHarris);
    let mut filter = FirFilter::new(taps);
    let mut filtered_iq = vec![Complex32::default(); chunk_size];
    filter.filter_block(&shifted_iq, &mut filtered_iq);

    let mut demod = WfmDemod::new(sample_rate as f32, 75000.0, DeEmphasis::Eu50us);
    let mut audio = Vec::new();
    demod.demod_block(&filtered_iq, &mut audio);

    assert_eq!(audio.len(), chunk_size);
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
    let lora = LoraDecoder::new(SpreadingFactor::SF8);
    let chirp = lora.synthesize_symbol(100);
    let decoded = lora.demodulate_symbol(&chirp).unwrap();
    assert_eq!(decoded, 100);

    let adsb_hex: [u8; 14] = [
        0x8D, 0x48, 0x40, 0xD6, 0x20, 0x2C, 0xC3, 0x71, 0xC3, 0x2C, 0xE0, 0x57, 0x60, 0x98,
    ];
    let adsb_msg = AdsbDecoder::parse_message(&adsb_hex).unwrap();
    assert_eq!(adsb_msg.df, DownlinkFormat::ExtendedSquitter);
    assert_eq!(adsb_msg.icao_address, 0x4840D6);
    assert!(adsb_msg.crc_valid);
}

#[test]
fn test_e2e_ssh_over_radio_tunnel_and_compliance() {
    // 1. Regulatory Compliance Verification for US 915 MHz ISM (SSH / Encrypted Tunnel)
    let check = RegulatoryDatabase::check_compliance(
        Jurisdiction::US,
        915_000_000,
        20.0, // 20 dBm (100 mW EIRP)
        true, // Encrypted SSH
    );
    assert!(
        matches!(check, ComplianceResult::Compliant { .. }),
        "US 915 MHz ISM should legally permit encrypted SSH tunnels"
    );

    // 2. Transceiver Initialization (Station 0x01 = SSH Client, Station 0x02 = SSH Server Node)
    let mut client_tunnel = StreamTunnel::new(0x01, 0x02, 128);
    let mut server_tunnel = StreamTunnel::new(0x02, 0x01, 128);

    // 3. SSH Client Greeting & Key Exchange Request Simulation
    let ssh_client_payload = b"SSH-2.0-OpenSSH_9.6 radio-client-node\r\n\
        kexinit:curve25519-sha256,chacha20-poly1305@openssh.com";

    // Client packetizes stream
    let client_rf_frames = client_tunnel.packetize(ssh_client_payload);
    assert!(!client_rf_frames.is_empty());

    // 4. Modulate to baseband IQ bursts via GFSK
    let sample_rate = 96000.0f32;
    let deviation = 4800.0f32;
    let sps = 8;
    let mut modulator = GfskModulator::new(sample_rate, deviation, sps, 0.5);

    let mut total_iq_samples = 0;
    for frame in &client_rf_frames {
        let mut burst_iq = Vec::new();
        modulator.modulate_bytes(frame, &mut burst_iq);
        total_iq_samples += burst_iq.len();

        // Pass RF frame to server station
        let ack_frame = server_tunnel.ingest_frame(frame).unwrap();
        assert!(
            ack_frame.is_some(),
            "Server must acknowledge received frame"
        );
    }
    assert!(total_iq_samples > 0);

    // 5. Server extracts complete reconstructed byte stream
    let server_received = server_tunnel.drain_received_bytes();
    assert_eq!(server_received.as_slice(), ssh_client_payload);

    // 6. SSH Server Response & Authenticated Shell Prompt Simulation
    let ssh_server_response = b"SSH-2.0-OpenSSH_9.6 radio-server-node\r\n\
        Welcome to Ubuntu 24.04 LTS (GNU/Linux 6.8.0-31-generic x86_64)\r\n\
        sdr-node:~$ ";

    let server_rf_frames = server_tunnel.packetize(ssh_server_response);
    for frame in &server_rf_frames {
        let ack_frame = client_tunnel.ingest_frame(frame).unwrap();
        assert!(
            ack_frame.is_some(),
            "Client must acknowledge received frame"
        );
    }

    let client_received = client_tunnel.drain_received_bytes();
    assert_eq!(client_received.as_slice(), ssh_server_response);
}
