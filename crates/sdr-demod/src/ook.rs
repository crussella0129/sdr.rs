//! On-Off Keying (OOK / ASK) Demodulator with adaptive slicing.

use sdr_core::sample::Complex32;
use sdr_core::traits::{Block, Result};

/// OOK / ASK Demodulator.
#[derive(Debug, Clone)]
pub struct OokDemod {
    samples_per_symbol: usize,
    threshold: f32,
    adaptive: bool,
    noise_floor_ema: f32,
    peak_ema: f32,
    sample_counter: usize,
    accumulated_energy: f32,
}

impl OokDemod {
    /// Create a new OOK demodulator.
    pub fn new(samples_per_symbol: usize, threshold: f32) -> Self {
        Self {
            samples_per_symbol: samples_per_symbol.max(1),
            threshold,
            adaptive: false,
            noise_floor_ema: 0.01,
            peak_ema: 0.5,
            sample_counter: 0,
            accumulated_energy: 0.0,
        }
    }

    /// Enable adaptive threshold based on exponential moving average of noise floor and peaks.
    pub fn with_adaptive_threshold(mut self) -> Self {
        self.adaptive = true;
        self
    }

    pub fn demod_bits(&mut self, input: &[Complex32], output_bits: &mut Vec<u8>) {
        for &s in input {
            let energy = s.norm_sqr();
            self.accumulated_energy += energy;
            self.sample_counter += 1;

            if self.adaptive {
                if energy > self.peak_ema {
                    self.peak_ema += 0.05 * (energy - self.peak_ema);
                } else {
                    self.noise_floor_ema += 0.01 * (energy - self.noise_floor_ema);
                }
                self.threshold = (self.peak_ema + self.noise_floor_ema) * 0.5;
            }

            if self.sample_counter >= self.samples_per_symbol {
                let mean_energy = self.accumulated_energy / self.sample_counter as f32;
                let bit = if mean_energy >= self.threshold {
                    1u8
                } else {
                    0u8
                };
                output_bits.push(bit);

                self.sample_counter = 0;
                self.accumulated_energy = 0.0;
            }
        }
    }

    pub fn reset(&mut self) {
        self.sample_counter = 0;
        self.accumulated_energy = 0.0;
    }
}

impl Block<Complex32, u8> for OokDemod {
    fn process(&mut self, input: &[Complex32], output: &mut Vec<u8>) -> Result<(usize, usize)> {
        output.clear();
        output.reserve(input.len() / self.samples_per_symbol + 2);
        self.demod_bits(input, output);
        Ok((input.len(), output.len()))
    }

    fn reset(&mut self) {
        self.reset();
    }
}
