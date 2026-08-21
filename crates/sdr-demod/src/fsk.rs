//! Frequency Shift Keying (FSK/GFSK) digital demodulator.

use sdr_core::sample::Complex32;
use sdr_core::traits::{Block, Result};
use std::f32::consts::PI;

/// 2-FSK Digital Demodulator and Bit Slicer.
#[derive(Debug, Clone)]
pub struct FskDemod {
    sample_rate: f32,
    _deviation: f32,
    samples_per_symbol: usize,
    last_sample: Complex32,
    sample_counter: usize,
    accumulated_freq: f32,
}

impl FskDemod {
    /// Create a new 2-FSK demodulator.
    pub fn new(sample_rate: f32, deviation: f32, samples_per_symbol: usize) -> Self {
        Self {
            sample_rate,
            _deviation: deviation,
            samples_per_symbol: samples_per_symbol.max(1),
            last_sample: Complex32::new(1.0, 0.0),
            sample_counter: 0,
            accumulated_freq: 0.0,
        }
    }

    /// Process a stream of IQ samples and return demodulated binary bits (0 or 1).
    pub fn demod_bits(&mut self, input: &[Complex32], output_bits: &mut Vec<u8>) {
        for &s in input {
            let prod = s * self.last_sample.conj();
            let phase_diff = prod.im.atan2(prod.re);
            self.last_sample = s;

            let instantaneous_freq = (phase_diff * self.sample_rate) / (2.0 * PI);
            self.accumulated_freq += instantaneous_freq;
            self.sample_counter += 1;

            if self.sample_counter >= self.samples_per_symbol {
                let mean_freq = self.accumulated_freq / self.sample_counter as f32;
                // Slicer: positive frequency offset -> bit 1, negative -> bit 0
                let bit = if mean_freq >= 0.0 { 1u8 } else { 0u8 };
                output_bits.push(bit);

                self.sample_counter = 0;
                self.accumulated_freq = 0.0;
            }
        }
    }

    pub fn reset(&mut self) {
        self.last_sample = Complex32::new(1.0, 0.0);
        self.sample_counter = 0;
        self.accumulated_freq = 0.0;
    }
}

impl Block<Complex32, u8> for FskDemod {
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
