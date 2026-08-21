//! Rational and Polyphase Resampling for arbitrary rate transitions.

use crate::fir::design_lowpass;
use crate::window::WindowType;
use sdr_core::sample::Complex32;
use sdr_core::traits::{Block, Result};

/// Polyphase FIR filterbank resampler for rational rate conversion $L/M$.
#[derive(Debug, Clone)]
pub struct PolyphaseResampler {
    interpolation: usize,
    decimation: usize,
    subfilters: Vec<Vec<f32>>,
    taps_per_filter: usize,
    history: Vec<Complex32>,
    history_idx: usize,
    phase: usize,
}

impl PolyphaseResampler {
    /// Create a new polyphase resampler converting from `in_rate` to `out_rate`.
    pub fn new(in_rate: f64, out_rate: f64) -> Self {
        let (interp, decim) = rational_approximation(out_rate / in_rate);
        Self::with_factors(interp, decim, 64)
    }

    /// Create resampler with explicit integer interpolation and decimation factors.
    pub fn with_factors(interp: usize, decim: usize, filter_length: usize) -> Self {
        assert!(interp > 0 && decim > 0);
        let num_taps = filter_length.max(interp * 4);
        let cutoff = 0.5 / (interp.max(decim) as f64);
        let prototype_taps = design_lowpass(1.0, cutoff, num_taps, WindowType::BlackmanHarris);

        // Decompose prototype filter into L polyphase subfilters
        let taps_per_filter = (prototype_taps.len() + interp - 1) / interp;
        let mut subfilters = vec![vec![0.0f32; taps_per_filter]; interp];

        for i in 0..prototype_taps.len() {
            let phase_idx = i % interp;
            let sub_tap_idx = i / interp;
            if sub_tap_idx < taps_per_filter {
                // Scale taps by interpolation factor to preserve amplitude
                subfilters[phase_idx][sub_tap_idx] = prototype_taps[i] * interp as f32;
            }
        }

        Self {
            interpolation: interp,
            decimation: decim,
            subfilters,
            taps_per_filter,
            history: vec![Complex32::default(); taps_per_filter],
            history_idx: 0,
            phase: 0,
        }
    }

    /// Process input slice and append resampled samples to `output`.
    pub fn resample(&mut self, input: &[Complex32], output: &mut Vec<Complex32>) {
        for &sample in input {
            self.history[self.history_idx] = sample;

            while self.phase < self.interpolation {
                let subfilter = &self.subfilters[self.phase];
                let mut acc_re = 0.0f32;
                let mut acc_im = 0.0f32;

                let mut h_idx = self.history_idx;
                for &tap in subfilter.iter() {
                    let s = self.history[h_idx];
                    acc_re += tap * s.re;
                    acc_im += tap * s.im;

                    if h_idx == 0 {
                        h_idx = self.taps_per_filter - 1;
                    } else {
                        h_idx -= 1;
                    }
                }

                output.push(Complex32::new(acc_re, acc_im));
                self.phase += self.decimation;
            }

            self.phase -= self.interpolation;
            self.history_idx = (self.history_idx + 1) % self.taps_per_filter;
        }
    }

    pub fn reset(&mut self) {
        self.history.fill(Complex32::default());
        self.history_idx = 0;
        self.phase = 0;
    }
}

impl Block<Complex32, Complex32> for PolyphaseResampler {
    fn process(
        &mut self,
        input: &[Complex32],
        output: &mut Vec<Complex32>,
    ) -> Result<(usize, usize)> {
        output.clear();
        let est_out = (input.len() * self.interpolation / self.decimation) + 16;
        output.reserve(est_out);
        self.resample(input, output);
        Ok((input.len(), output.len()))
    }

    fn reset(&mut self) {
        self.reset();
    }
}

/// Compute greatest common divisor (GCD).
fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Find integer fraction (L, M) closest to target ratio.
fn rational_approximation(ratio: f64) -> (usize, usize) {
    if (ratio - 1.0).abs() < 1e-6 {
        return (1, 1);
    }
    // Search best fraction with max denominator 1024
    let mut best_l = 1;
    let mut best_m = 1;
    let mut min_diff = f64::MAX;

    for m in 1..=256 {
        let l = (ratio * m as f64).round() as usize;
        if l == 0 {
            continue;
        }
        let diff = (ratio - l as f64 / m as f64).abs();
        if diff < min_diff {
            min_diff = diff;
            let g = gcd(l, m);
            best_l = l / g;
            best_m = m / g;
            if min_diff < 1e-7 {
                break;
            }
        }
    }

    (best_l.max(1), best_m.max(1))
}
