//! # sdr-dsp
//!
//! High-performance Digital Signal Processing (DSP) algorithms for `sdr.rs`:
//! FIR/IIR filter design, window functions, NCO oscillators, polyphase resamplers,
//! Hilbert transforms, Costas carrier loops, and symbol clock recovery.

pub mod clock_recovery;
pub mod costas;
pub mod fir;
pub mod hilbert;
pub mod nco;
pub mod resample;
pub mod window;

pub use clock_recovery::GardnerClockRecovery;
pub use costas::{CostasLoop, ModulationOrder};
pub use fir::{design_bandpass, design_highpass, design_lowpass, FirFilter};
pub use hilbert::HilbertTransform;
pub use nco::Nco;
pub use resample::PolyphaseResampler;
pub use window::{bessel_i0, generate_window, WindowType};

#[cfg(test)]
mod tests {
    use super::*;
    use sdr_core::sample::Complex32;
    use std::f32::consts::PI;

    #[test]
    fn test_window_functions() {
        let n = 33;
        let w_rect = generate_window(WindowType::Rectangular, n);
        assert_eq!(w_rect.len(), n);
        assert!((w_rect[0] - 1.0).abs() < 1e-6);

        let w_hann = generate_window(WindowType::Hann, n);
        assert_eq!(w_hann.len(), n);
        assert!(w_hann[0].abs() < 1e-6);
        assert!((w_hann[n / 2] - 1.0).abs() < 1e-6);

        let w_bh = generate_window(WindowType::BlackmanHarris, n);
        assert_eq!(w_bh.len(), n);
        // Symmetrical
        for i in 0..n / 2 {
            assert!((w_bh[i] - w_bh[n - 1 - i]).abs() < 1e-5);
        }

        let w_kaiser = generate_window(WindowType::Kaiser(500), n);
        assert_eq!(w_kaiser.len(), n);
        assert!((w_kaiser[n / 2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_fir_lowpass_attenuation() {
        let sample_rate = 48000.0;
        let cutoff = 4000.0;
        let num_taps = 65;
        let taps = design_lowpass(sample_rate, cutoff, num_taps, WindowType::BlackmanHarris);
        let mut filter = FirFilter::new(taps);

        // Generate passband tone at 1000 Hz and stopband tone at 16000 Hz
        let n = 1024;
        let mut passband_in = vec![Complex32::default(); n];
        let mut stopband_in = vec![Complex32::default(); n];

        for i in 0..n {
            let t = i as f32 / sample_rate as f32;
            let val_pass = (2.0 * PI * 1000.0 * t).cos();
            let val_stop = (2.0 * PI * 16000.0 * t).cos();
            passband_in[i] = Complex32::new(val_pass, 0.0);
            stopband_in[i] = Complex32::new(val_stop, 0.0);
        }

        let mut passband_out = vec![Complex32::default(); n];
        let mut stopband_out = vec![Complex32::default(); n];

        filter.filter_block(&passband_in, &mut passband_out);
        filter.reset();
        filter.filter_block(&stopband_in, &mut stopband_out);

        // Measure RMS in steady state (skip initial filter delay)
        let pass_rms: f32 = (passband_out[100..].iter().map(|s| s.norm_sqr()).sum::<f32>()
            / (n - 100) as f32)
            .sqrt();
        let stop_rms: f32 = (stopband_out[100..].iter().map(|s| s.norm_sqr()).sum::<f32>()
            / (n - 100) as f32)
            .sqrt();

        let attenuation_db = 20.0 * (pass_rms / stop_rms.max(1e-12)).log10();
        // Sinc + Blackman-Harris should easily exceed 40 dB attenuation
        assert!(
            attenuation_db > 40.0,
            "Expected >40 dB attenuation, got {:.2} dB",
            attenuation_db
        );
    }

    #[test]
    fn test_nco_frequency_shift_and_phase_continuity() {
        let sample_rate = 100000.0;
        let mut nco = Nco::new(sample_rate, 10000.0);

        // Input 10 kHz tone
        let n = 200;
        let mut input = vec![Complex32::default(); n];
        for i in 0..n {
            let phase = 2.0 * PI * 10000.0 * (i as f32 / sample_rate as f32);
            input[i] = Complex32::new(phase.cos(), phase.sin());
        }

        // Process in two chunks of 100 samples to verify phase continuity across boundaries
        let mut chunk1_out = vec![Complex32::default(); 100];
        let mut chunk2_out = vec![Complex32::default(); 100];

        nco.mix_block(&input[0..100], &mut chunk1_out);
        nco.mix_block(&input[100..200], &mut chunk2_out);

        // Downmixing 10 kHz by 10 kHz should result in DC (phasor = 1.0 + 0.0j)
        for s in chunk1_out.iter().chain(chunk2_out.iter()) {
            assert!(
                (s.re - 1.0).abs() < 1e-3,
                "Real component diverged: {}",
                s.re
            );
            assert!(s.im.abs() < 1e-3, "Imag component diverged: {}", s.im);
        }
    }

    #[test]
    fn test_polyphase_resampler_ratio() {
        let mut resampler = PolyphaseResampler::new(48000.0, 44100.0);
        let input = vec![Complex32::new(1.0, 0.0); 4800];
        let mut output = Vec::new();
        resampler.resample(&input, &mut output);

        let expected_ratio = 44100.0 / 48000.0;
        let actual_ratio = output.len() as f64 / input.len() as f64;
        assert!(
            (actual_ratio - expected_ratio).abs() < 0.01,
            "Ratio error: expected {}, got {}",
            expected_ratio,
            actual_ratio
        );
    }

    #[test]
    fn test_hilbert_transform() {
        let mut hilbert = HilbertTransform::new(65);
        let n = 512;
        let mut analytic = Vec::with_capacity(n);

        // Input 6 kHz cosine tone at 48 kHz (well within passband of 65-tap Hilbert filter)
        for i in 0..n {
            let t = i as f32 / 48000.0;
            let val = (2.0 * PI * 6000.0 * t).cos();
            analytic.push(hilbert.transform_sample(val));
        }

        // In steady state (past group delay 32), magnitude should be ~1.0
        for s in &analytic[100..400] {
            let mag = s.norm();
            assert!(
                (mag - 1.0).abs() < 0.02,
                "Analytic magnitude not ~1.0: {}",
                mag
            );
        }
    }

    #[test]
    fn test_costas_loop_phase_lock() {
        let mut costas = CostasLoop::new(0.04, ModulationOrder::Bpsk);
        let initial_phase_offset = 0.35f32;

        let n = 500;
        let mut locked_samples = Vec::with_capacity(n);

        for _ in 0..n {
            // BPSK symbol = +1.0 with static phase offset
            let sample = Complex32::new(
                initial_phase_offset.cos(),
                initial_phase_offset.sin(),
            );
            let out = costas.process_sample(sample);
            locked_samples.push(out);
        }

        // By sample 400, phase error should be near zero (Q component ~= 0, I ~= 1.0)
        let last = locked_samples[450];
        assert!(
            (last.re - 1.0).abs() < 0.05,
            "Costas did not lock real component: {}",
            last.re
        );
        assert!(
            last.im.abs() < 0.05,
            "Costas did not null imag component: {}",
            last.im
        );
    }

    #[test]
    fn test_gardner_clock_recovery() {
        let mut gardner = GardnerClockRecovery::new(4.0, 0.01, 0.001);
        let n_symbols = 100;
        let sps = 4;
        let mut input = Vec::with_capacity(n_symbols * sps);

        for i in 0..n_symbols {
            let bit = if i % 2 == 0 { 1.0 } else { -1.0 };
            for _ in 0..sps {
                input.push(Complex32::new(bit, 0.0));
            }
        }

        let mut output = Vec::new();
        gardner.process_samples(&input, &mut output);
        assert!(!output.is_empty());
        assert!(output.len() >= n_symbols - 5);
    }
}
