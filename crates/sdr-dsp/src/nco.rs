//! Numerically Controlled Oscillator (NCO) and CORDIC Frequency Mixer.

use sdr_core::sample::Complex32;
use sdr_core::traits::{Block, Result};
use std::f64::consts::TAU;

/// A high-precision Numerically Controlled Oscillator (NCO / DDS).
#[derive(Debug, Clone)]
pub struct Nco {
    phase: f64,
    phase_inc: f64,
    freq_hz: f64,
    sample_rate: f64,
}

impl Nco {
    /// Create a new NCO tuned to `freq_hz` at given `sample_rate`.
    pub fn new(sample_rate: f64, freq_hz: f64) -> Self {
        let mut nco = Self {
            phase: 0.0,
            phase_inc: 0.0,
            freq_hz,
            sample_rate,
        };
        nco.update_phase_inc();
        nco
    }

    fn update_phase_inc(&mut self) {
        if self.sample_rate > 0.0 {
            self.phase_inc = (self.freq_hz / self.sample_rate) * TAU;
        } else {
            self.phase_inc = 0.0;
        }
    }

    /// Retune the NCO frequency while preserving current phase.
    pub fn set_frequency(&mut self, freq_hz: f64) {
        self.freq_hz = freq_hz;
        self.update_phase_inc();
    }

    /// Set the sample rate.
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        self.update_phase_inc();
    }

    /// Set absolute phase in radians `[0, 2*PI)`.
    pub fn set_phase(&mut self, phase: f64) {
        self.phase = phase.rem_euclid(TAU);
    }

    /// Current instantaneous phase in radians.
    pub fn phase(&self) -> f64 {
        self.phase
    }

    /// Advance oscillator by one sample and return current complex phasor $e^{j \theta}$.
    #[inline(always)]
    pub fn step(&mut self) -> Complex32 {
        let (sin_val, cos_val) = self.phase.sin_cos();
        let phasor = Complex32::new(cos_val as f32, sin_val as f32);
        self.phase += self.phase_inc;
        if self.phase >= TAU || self.phase < 0.0 {
            self.phase = self.phase.rem_euclid(TAU);
        }
        phasor
    }

    /// Mix (frequency-shift) a single input sample by multiplying with conjugate phasor $e^{-j \theta}$ (downconversion) or $e^{j \theta}$ (upconversion).
    #[inline(always)]
    pub fn mix_sample_down(&mut self, sample: Complex32) -> Complex32 {
        let (sin_val, cos_val) = self.phase.sin_cos();
        // Conjugate: cos(theta) - j * sin(theta)
        let phasor_conj = Complex32::new(cos_val as f32, -sin_val as f32);
        self.phase += self.phase_inc;
        if self.phase >= TAU || self.phase < 0.0 {
            self.phase = self.phase.rem_euclid(TAU);
        }
        sample * phasor_conj
    }

    /// Mix a block of samples (downconversion) in-place or into destination slice.
    pub fn mix_block(&mut self, input: &[Complex32], output: &mut [Complex32]) {
        assert_eq!(input.len(), output.len());
        for (s_in, s_out) in input.iter().zip(output.iter_mut()) {
            *s_out = self.mix_sample_down(*s_in);
        }
    }

    /// Reset phase to zero.
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }
}

impl Block<Complex32, Complex32> for Nco {
    fn process(
        &mut self,
        input: &[Complex32],
        output: &mut Vec<Complex32>,
    ) -> Result<(usize, usize)> {
        output.clear();
        output.reserve(input.len());
        for &s in input {
            output.push(self.mix_sample_down(s));
        }
        Ok((input.len(), output.len()))
    }

    fn reset(&mut self) {
        self.reset();
    }
}
