//! Hilbert transform for generating complex analytic signals from real signals.

use crate::window::{generate_window, WindowType};
use sdr_core::sample::Complex32;
use sdr_core::traits::{Block, Result};
use std::f64::consts::PI;

/// Hilbert Transformer producing analytic complex signal $x[n] + j \cdot \hat{x}[n]$.
#[derive(Debug, Clone)]
pub struct HilbertTransform {
    taps: Vec<f32>,
    history: Vec<f32>,
    history_idx: usize,
    delay: usize,
}

impl HilbertTransform {
    /// Create a new Hilbert transformer with given odd tap length (e.g. 65).
    pub fn new(num_taps: usize) -> Self {
        let n = if num_taps % 2 == 0 {
            num_taps + 1
        } else {
            num_taps
        };
        let m = (n - 1) / 2;
        let mut taps = vec![0.0f32; n];
        let window = generate_window(WindowType::BlackmanHarris, n);

        for i in 0..n {
            let k = i as isize - m as isize;
            if k % 2 != 0 {
                // For odd k: 2 / (pi * k)
                let val = (2.0 / (PI * k as f64)) * window[i] as f64;
                taps[i] = val as f32;
            } else {
                taps[i] = 0.0;
            }
        }

        Self {
            taps,
            history: vec![0.0f32; n],
            history_idx: 0,
            delay: m,
        }
    }

    /// Process a single real sample and return analytic complex sample.
    pub fn transform_sample(&mut self, sample: f32) -> Complex32 {
        let n = self.taps.len();
        self.history[self.history_idx] = sample;

        // Compute imaginary part via convolution with Hilbert taps
        let mut imag = 0.0f32;
        let mut h_idx = self.history_idx;
        for i in 0..n {
            imag += self.taps[i] * self.history[h_idx];
            if h_idx == 0 {
                h_idx = n - 1;
            } else {
                h_idx -= 1;
            }
        }

        // Real part is delayed by group delay (m samples)
        let real_idx = (self.history_idx + n - self.delay) % n;
        let real = self.history[real_idx];

        self.history_idx = (self.history_idx + 1) % n;
        Complex32::new(real, imag)
    }

    pub fn reset(&mut self) {
        self.history.fill(0.0);
        self.history_idx = 0;
    }
}

impl Block<f32, Complex32> for HilbertTransform {
    fn process(&mut self, input: &[f32], output: &mut Vec<Complex32>) -> Result<(usize, usize)> {
        output.clear();
        output.reserve(input.len());
        for &s in input {
            output.push(self.transform_sample(s));
        }
        Ok((input.len(), output.len()))
    }

    fn reset(&mut self) {
        self.reset();
    }
}
