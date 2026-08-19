//! Narrowband FM (NFM) communications demodulator with hysteresis squelch.

use sdr_core::sample::Complex32;
use sdr_core::traits::{Block, Result};
use std::f32::consts::PI;

/// Narrowband FM demodulator.
#[derive(Debug, Clone)]
pub struct NfmDemod {
    sample_rate: f32,
    deviation: f32,
    last_sample: Complex32,
    squelch_threshold_db: f32,
    squelch_open: bool,
    power_ema: f32,
}

impl NfmDemod {
    /// Create a new NFM demodulator (e.g. sample_rate = 48000.0, deviation = 5000.0).
    pub fn new(sample_rate: f32, deviation: f32, squelch_threshold_db: f32) -> Self {
        Self {
            sample_rate,
            deviation,
            last_sample: Complex32::new(1.0, 0.0),
            squelch_threshold_db,
            squelch_open: false,
            power_ema: 1e-6,
        }
    }

    /// Process a single complex sample.
    #[inline]
    pub fn demod_sample(&mut self, sample: Complex32) -> f32 {
        let pwr = sample.norm_sqr();
        self.power_ema += 0.01 * (pwr - self.power_ema);
        let power_db = 10.0 * self.power_ema.max(1e-12).log10();

        // Hysteresis squelch (2 dB margin)
        if !self.squelch_open && power_db > self.squelch_threshold_db + 1.0 {
            self.squelch_open = true;
        } else if self.squelch_open && power_db < self.squelch_threshold_db - 1.0 {
            self.squelch_open = false;
        }

        let prod = sample * self.last_sample.conj();
        let phase_diff = prod.im.atan2(prod.re);
        self.last_sample = sample;

        if !self.squelch_open {
            return 0.0;
        }

        let audio = (phase_diff * self.sample_rate) / (2.0 * PI * self.deviation);
        audio.clamp(-1.0, 1.0)
    }

    /// Is squelch currently open (signal above threshold)?
    pub fn is_squelch_open(&self) -> bool {
        self.squelch_open
    }

    pub fn set_squelch_threshold(&mut self, threshold_db: f32) {
        self.squelch_threshold_db = threshold_db;
    }

    pub fn reset(&mut self) {
        self.last_sample = Complex32::new(1.0, 0.0);
        self.power_ema = 1e-6;
        self.squelch_open = false;
    }
}

impl Block<Complex32, f32> for NfmDemod {
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
