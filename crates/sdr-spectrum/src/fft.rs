//! FFT Power Spectrum Analyzer for real-time waterfall and spectral analysis.

use rustfft::{num_complex::Complex, FftPlanner};
use sdr_core::sample::Complex32;
use sdr_dsp::window::{generate_window, WindowType};
use std::sync::Arc;

/// Spectrum Analyzer with windowing, FFT, and power spectral density (PSD) calculation.
pub struct SpectrumAnalyzer {
    fft_size: usize,
    window: Vec<f32>,
    fft: Arc<dyn rustfft::Fft<f32>>,
    fft_buffer: Vec<Complex<f32>>,
    power_spectrum: Vec<f32>,
    history_ema: Vec<f32>,
    alpha: f32,
}

impl SpectrumAnalyzer {
    /// Create a new spectrum analyzer with given FFT size (e.g. 512, 1024, 2048, 4096).
    pub fn new(fft_size: usize, window_type: WindowType, averaging_factor: f32) -> Self {
        assert!(fft_size.is_power_of_two());
        let window = generate_window(window_type, fft_size);
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);

        Self {
            fft_size,
            window,
            fft,
            fft_buffer: vec![Complex::new(0.0, 0.0); fft_size],
            power_spectrum: vec![-120.0; fft_size],
            history_ema: vec![-120.0; fft_size],
            alpha: averaging_factor.clamp(0.01, 1.0),
        }
    }

    /// Process a block of samples and compute centered power spectrum in dBFS `[-fs/2, +fs/2]`.
    pub fn process(&mut self, samples: &[Complex32]) -> &[f32] {
        let n = self.fft_size.min(samples.len());
        for i in 0..n {
            let w = self.window[i];
            self.fft_buffer[i] = Complex::new(samples[i].re * w, samples[i].im * w);
        }
        for i in n..self.fft_size {
            self.fft_buffer[i] = Complex::new(0.0, 0.0);
        }

        self.fft.process(&mut self.fft_buffer);

        let half = self.fft_size / 2;
        let scale = 1.0 / (self.fft_size as f32);

        // Shift zero frequency to center and compute logarithmic power in dBFS
        for i in 0..self.fft_size {
            // FFT shift: output index 0..half comes from second half of FFT, second half from first
            let src_idx = (i + half) % self.fft_size;
            let val = self.fft_buffer[src_idx];
            let mag_sqr = (val.re * val.re + val.im * val.im) * scale * scale;
            let pwr_db = 10.0 * mag_sqr.max(1e-12).log10();

            // Exponential moving average smoothing
            self.history_ema[i] += self.alpha * (pwr_db - self.history_ema[i]);
            self.power_spectrum[i] = self.history_ema[i];
        }

        &self.power_spectrum
    }

    /// Get the latest computed power spectrum in dBFS.
    pub fn power_spectrum(&self) -> &[f32] {
        &self.power_spectrum
    }

    /// Find highest spectral peak in a given spectrum slice (bin index and power in dBFS).
    pub fn find_peak_in(spectrum: &[f32]) -> (usize, f32) {
        let mut max_idx = 0;
        let mut max_pwr = -999.0f32;
        for (i, &pwr) in spectrum.iter().enumerate() {
            if pwr > max_pwr {
                max_pwr = pwr;
                max_idx = i;
            }
        }
        (max_idx, max_pwr)
    }

    /// Find highest spectral peak in latest power spectrum.
    pub fn find_peak(&self) -> (usize, f32) {
        Self::find_peak_in(&self.power_spectrum)
    }

    pub fn fft_size(&self) -> usize {
        self.fft_size
    }
}
