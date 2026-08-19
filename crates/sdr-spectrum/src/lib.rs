//! # sdr-spectrum
//!
//! Real-time FFT power spectrum analysis, CA-CFAR adaptive thresholding, and Hamlib Rigctl protocol server for `sdr.rs`.

pub mod cfar;
pub mod fft;
pub mod rigctl;

pub use cfar::CaCfarDetector;
pub use fft::SpectrumAnalyzer;
pub use rigctl::{RigMode, RigState, RigctlHandler};

#[cfg(test)]
mod tests {
    use super::*;
    use sdr_core::sample::Complex32;
    use sdr_dsp::window::WindowType;
    use std::f32::consts::TAU;

    #[test]
    fn test_spectrum_analyzer_fft_and_cfar() {
        let fft_size = 1024;
        let mut analyzer = SpectrumAnalyzer::new(fft_size, WindowType::BlackmanHarris, 1.0);

        // Generate synthetic single-tone signal at +fs/4 (bin 768 when shifted)
        let mut samples = Vec::with_capacity(fft_size);
        for i in 0..fft_size {
            let phase = 0.25 * TAU * (i as f32);
            let (sin_p, cos_p) = phase.sin_cos();
            samples.push(Complex32::new(cos_p, sin_p));
        }

        let spectrum = analyzer.process(&samples);
        assert_eq!(spectrum.len(), fft_size);

        let (peak_bin, peak_pwr) = SpectrumAnalyzer::find_peak_in(spectrum);
        assert_eq!(peak_bin, 768, "Peak bin shifted incorrectly");
        assert!(peak_pwr > -10.0, "Peak power too low: {}", peak_pwr);

        // Test CA-CFAR detection
        let cfar = CaCfarDetector::new(4, 16, 15.0);
        let detections = cfar.detect(spectrum);
        assert!(!detections.is_empty(), "CFAR failed to detect peak tone");
        assert!(
            detections.iter().any(|d| d.0 == peak_bin),
            "CFAR detections did not contain peak bin {}",
            peak_bin
        );
    }

    #[test]
    fn test_rigctl_tcp_command_handling() {
        let mut handler = RigctlHandler::new(RigState::default());

        // Test get frequency
        let freq_resp = handler.handle_command("f\n").unwrap();
        assert_eq!(freq_resp, "144000000\n");

        // Test set frequency
        let set_resp = handler.handle_command("F 433920000\n").unwrap();
        assert_eq!(set_resp, "RPRT 0\n");
        assert_eq!(handler.state().frequency_hz, 433920000);

        // Test get mode
        let mode_resp = handler.handle_command("m\n").unwrap();
        assert!(mode_resp.starts_with("FM\n"));

        // Test set mode
        let set_mode_resp = handler.handle_command("M USB 2800\n").unwrap();
        assert_eq!(set_mode_resp, "RPRT 0\n");
        assert_eq!(handler.state().mode, RigMode::USB);
        assert_eq!(handler.state().passband_width_hz, 2800);

        // Test dump state
        let dump_resp = handler.handle_command("\\dump_state\n").unwrap();
        assert!(dump_resp.contains("6000000000"));
    }
}
