//! Amplitude Modulation (AM) demodulator with envelope detection and DC blocker.

use sdr_core::sample::Complex32;
use sdr_core::traits::{Block, Result};

/// AM demodulator with envelope detection and DC-blocking audio filter.
#[derive(Debug, Clone)]
pub struct AmDemod {
    dc_ema: f32,
    dc_alpha: f32,
}

impl AmDemod {
    /// Create a new AM demodulator.
    pub fn new() -> Self {
        Self {
            dc_ema: 0.0,
            dc_alpha: 0.01,
        }
    }

    /// Process a single complex sample and return demodulated audio.
    #[inline]
    pub fn demod_sample(&mut self, sample: Complex32) -> f32 {
        let envelope = sample.norm();
        // Single-pole DC blocker
        self.dc_ema += self.dc_alpha * (envelope - self.dc_ema);
        let ac_audio = envelope - self.dc_ema;
        ac_audio.clamp(-1.0, 1.0)
    }

    pub fn reset(&mut self) {
        self.dc_ema = 0.0;
    }
}

impl Default for AmDemod {
    fn default() -> Self {
        Self::new()
    }
}

impl Block<Complex32, f32> for AmDemod {
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
