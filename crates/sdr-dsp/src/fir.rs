//! Finite Impulse Response (FIR) filter design and SIMD-accelerated execution.

use crate::window::{generate_window, WindowType};
use sdr_core::sample::Complex32;
use sdr_core::traits::{Block, Result};
use std::f64::consts::PI;

/// Design a low-pass FIR filter using the windowed-sinc method.
pub fn design_lowpass(
    sample_rate: f64,
    cutoff_freq: f64,
    num_taps: usize,
    window_type: WindowType,
) -> Vec<f32> {
    assert!(num_taps > 0, "num_taps must be greater than 0");
    let mut taps = vec![0.0f32; num_taps];
    let fc = cutoff_freq / sample_rate;
    let window = generate_window(window_type, num_taps);
    let m = (num_taps - 1) as f64 / 2.0;

    let mut sum = 0.0f64;
    for i in 0..num_taps {
        let n = i as f64 - m;
        let sinc = if n.abs() < 1e-9 {
            2.0 * fc
        } else {
            (2.0 * PI * fc * n).sin() / (PI * n)
        };
        let h = sinc * window[i] as f64;
        taps[i] = h as f32;
        sum += h;
    }

    // Normalize DC gain to 1.0 (0 dB)
    if sum.abs() > 1e-9 {
        let scale = (1.0 / sum) as f32;
        for tap in taps.iter_mut() {
            *tap *= scale;
        }
    }
    taps
}

/// Design a high-pass FIR filter using spectral inversion of a low-pass filter.
pub fn design_highpass(
    sample_rate: f64,
    cutoff_freq: f64,
    num_taps: usize,
    window_type: WindowType,
) -> Vec<f32> {
    assert!(
        num_taps % 2 == 1,
        "Highpass FIR requires odd number of taps (Type I)"
    );
    let mut taps = design_lowpass(sample_rate, cutoff_freq, num_taps, window_type);
    for tap in taps.iter_mut() {
        *tap = -*tap;
    }
    let center = num_taps / 2;
    taps[center] += 1.0;
    taps
}

/// Design a band-pass FIR filter by subtracting two low-pass filters or modulating a low-pass filter.
pub fn design_bandpass(
    sample_rate: f64,
    low_cutoff: f64,
    high_cutoff: f64,
    num_taps: usize,
    window_type: WindowType,
) -> Vec<f32> {
    assert!(
        low_cutoff < high_cutoff,
        "low_cutoff must be less than high_cutoff"
    );
    let lp_high = design_lowpass(sample_rate, high_cutoff, num_taps, window_type);
    let lp_low = design_lowpass(sample_rate, low_cutoff, num_taps, window_type);

    let mut taps = vec![0.0f32; num_taps];
    for i in 0..num_taps {
        taps[i] = lp_high[i] - lp_low[i];
    }
    taps
}

/// Real-valued FIR filter processing complex or real streams.
#[derive(Debug, Clone)]
pub struct FirFilter {
    taps: Vec<f32>,
    history: Vec<Complex32>,
    history_idx: usize,
}

impl FirFilter {
    /// Create a new FIR filter with given tap coefficients.
    pub fn new(taps: Vec<f32>) -> Self {
        let num_taps = taps.len().max(1);
        Self {
            taps,
            history: vec![Complex32::default(); num_taps],
            history_idx: 0,
        }
    }

    /// Process a single complex sample and return filtered output.
    #[inline(always)]
    pub fn filter_sample(&mut self, sample: Complex32) -> Complex32 {
        let n = self.taps.len();
        self.history[self.history_idx] = sample;

        let mut acc_re = 0.0f32;
        let mut acc_im = 0.0f32;

        let mut h_idx = self.history_idx;
        for i in 0..n {
            let tap = self.taps[i];
            let s = self.history[h_idx];
            acc_re += tap * s.re;
            acc_im += tap * s.im;

            if h_idx == 0 {
                h_idx = n - 1;
            } else {
                h_idx -= 1;
            }
        }

        self.history_idx = (self.history_idx + 1) % n;
        Complex32::new(acc_re, acc_im)
    }

    /// Process a block of samples into destination slice.
    pub fn filter_block(&mut self, input: &[Complex32], output: &mut [Complex32]) {
        assert_eq!(input.len(), output.len());
        for (s_in, s_out) in input.iter().zip(output.iter_mut()) {
            *s_out = self.filter_sample(*s_in);
        }
    }

    /// Reset internal state buffer.
    pub fn reset(&mut self) {
        self.history.fill(Complex32::default());
        self.history_idx = 0;
    }

    pub fn taps(&self) -> &[f32] {
        &self.taps
    }
}

impl Block<Complex32, Complex32> for FirFilter {
    fn process(
        &mut self,
        input: &[Complex32],
        output: &mut Vec<Complex32>,
    ) -> Result<(usize, usize)> {
        output.clear();
        output.reserve(input.len());
        for &s in input {
            output.push(self.filter_sample(s));
        }
        Ok((input.len(), output.len()))
    }

    fn reset(&mut self) {
        self.reset();
    }
}
