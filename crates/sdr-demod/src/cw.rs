//! Continuous Wave (CW / Morse Code) Demodulator with Beat Frequency Oscillator (BFO).

use sdr_core::sample::Complex32;
use sdr_core::traits::{Block, Result};
use std::f32::consts::TAU;

/// CW demodulator mixing baseband carrier with an audio BFO tone (e.g. 700 Hz).
#[derive(Debug, Clone)]
pub struct CwDemod {
    bfo_phase: f32,
    bfo_inc: f32,
}

impl CwDemod {
    /// Create a new CW demodulator with target audio sidetone frequency (e.g. 700 Hz at 48000 Hz sample rate).
    pub fn new(sample_rate: f32, bfo_freq_hz: f32) -> Self {
        Self {
            bfo_phase: 0.0,
            bfo_inc: (bfo_freq_hz / sample_rate) * TAU,
        }
    }

    /// Demodulate a single complex sample to audible sidetone.
    #[inline]
    pub fn demod_sample(&mut self, sample: Complex32) -> f32 {
        let (sin_bfo, cos_bfo) = self.bfo_phase.sin_cos();
        self.bfo_phase = (self.bfo_phase + self.bfo_inc).rem_euclid(TAU);

        // Mix complex sample with BFO: Real(sample * e^(j*bfo))
        sample.re * cos_bfo - sample.im * sin_bfo
    }

    pub fn reset(&mut self) {
        self.bfo_phase = 0.0;
    }
}

impl Block<Complex32, f32> for CwDemod {
    fn process(&mut self, input: &[Complex32], output: &mut Vec<f32>) -> Result<(usize, usize)> {
        output.clear();
        output.reserve(input.len());
        for &s in input {
            output.push(self.demod_sample(s));
        }
        Ok((input.len(), output.len()))
    }

    fn reset(&mut self) {
        self.reset();
    }
}
