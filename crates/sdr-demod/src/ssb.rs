//! Single Sideband (SSB: USB and LSB) demodulator.

use sdr_core::sample::Complex32;
use sdr_core::traits::{Block, Result};

/// SSB mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsbMode {
    Usb, // Upper Sideband
    Lsb, // Lower Sideband
}

/// Single Sideband Demodulator using Phasing / Complex Baseband Separation.
#[derive(Debug, Clone)]
pub struct SsbDemod {
    mode: SsbMode,
}

impl SsbDemod {
    pub fn new(mode: SsbMode) -> Self {
        Self { mode }
    }

    /// Process a single complex sample.
    #[inline]
    pub fn demod_sample(&mut self, sample: Complex32) -> f32 {
        use std::f32::consts::FRAC_1_SQRT_2;
        match self.mode {
            SsbMode::Usb => (sample.re + sample.im) * FRAC_1_SQRT_2,
            SsbMode::Lsb => (sample.re - sample.im) * FRAC_1_SQRT_2,
        }
    }

    pub fn set_mode(&mut self, mode: SsbMode) {
        self.mode = mode;
    }
}

impl Block<Complex32, f32> for SsbDemod {
    fn process(&mut self, input: &[Complex32], output: &mut Vec<f32>) -> Result<(usize, usize)> {
        output.clear();
        output.reserve(input.len());
        for &s in input {
            output.push(self.demod_sample(s));
        }
        Ok((input.len(), output.len()))
    }
}
