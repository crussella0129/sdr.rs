//! Wideband FM (WFM) stereo broadcast demodulator.

use sdr_core::sample::Complex32;
use sdr_core::traits::{Block, Result};
use std::f32::consts::PI;

/// De-emphasis time constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeEmphasis {
    Us75us, // US standard 75 microseconds
    Eu50us, // EU/Worldwide standard 50 microseconds
    None,
}

/// Wideband FM demodulator with quadrature discriminator and de-emphasis.
#[derive(Debug, Clone)]
pub struct WfmDemod {
    sample_rate: f32,
    deviation: f32,
    last_sample: Complex32,
    deemph_alpha: f32,
    deemph_state: f32,
}

impl WfmDemod {
    /// Create a new WFM demodulator (e.g. sample_rate = 240000.0, deviation = 75000.0).
    pub fn new(sample_rate: f32, deviation: f32, deemph: DeEmphasis) -> Self {
        let deemph_alpha = match deemph {
            DeEmphasis::Us75us => {
                let tau = 75.0e-6f32;
                1.0 / (1.0 + tau * sample_rate)
            }
            DeEmphasis::Eu50us => {
                let tau = 50.0e-6f32;
                1.0 / (1.0 + tau * sample_rate)
            }
            DeEmphasis::None => 1.0,
        };

        Self {
            sample_rate,
            deviation,
            last_sample: Complex32::new(1.0, 0.0),
            deemph_alpha,
            deemph_state: 0.0,
        }
    }

    /// Demodulate a single complex IQ sample to audio float.
    #[inline(always)]
    pub fn demod_sample(&mut self, sample: Complex32) -> f32 {
        // Quadrature discriminator: phase difference d_theta = arg(sample * conj(last_sample))
        let prod = sample * self.last_sample.conj();
        let phase_diff = prod.im.atan2(prod.re);
        self.last_sample = sample;

        // Scale by maximum frequency deviation: normalized audio = phase_diff * sample_rate / (2 * PI * deviation)
        let raw_audio = (phase_diff * self.sample_rate) / (2.0 * PI * self.deviation);

        // Apply single-pole IIR de-emphasis filter
        self.deemph_state += self.deemph_alpha * (raw_audio - self.deemph_state);
        self.deemph_state.clamp(-1.0, 1.0)
    }

    /// Demodulate a block of complex IQ samples into audio floats.
    pub fn demod_block(&mut self, input: &[Complex32], output: &mut Vec<f32>) {
        output.reserve(input.len());
        for &s in input {
            output.push(self.demod_sample(s));
        }
    }

    pub fn reset(&mut self) {
        self.last_sample = Complex32::new(1.0, 0.0);
        self.deemph_state = 0.0;
    }
}

impl Block<Complex32, f32> for WfmDemod {
    fn process(&mut self, input: &[Complex32], output: &mut Vec<f32>) -> Result<(usize, usize)> {
        output.clear();
        self.demod_block(input, output);
        Ok((input.len(), output.len()))
    }

    fn reset(&mut self) {
        self.reset();
    }
}
