//! Costas Loop for carrier frequency and phase synchronization (BPSK, QPSK, 8PSK).

use sdr_core::sample::Complex32;
use sdr_core::traits::{Block, Result};
use std::f32::consts::PI;

/// Modulation order for Costas Loop phase error detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModulationOrder {
    Bpsk = 2,
    Qpsk = 4,
    Psk8 = 8,
}

/// A 2nd-order Costas Loop for carrier phase and frequency tracking.
#[derive(Debug, Clone)]
pub struct CostasLoop {
    order: ModulationOrder,
    alpha: f32, // Proportional loop gain
    beta: f32,  // Integral loop gain
    phase: f32,
    freq: f32,
    max_freq: f32,
    min_freq: f32,
}

impl CostasLoop {
    /// Create a new Costas loop with given loop bandwidth (normalized, e.g. 0.01 to 0.05).
    pub fn new(loop_bw: f32, order: ModulationOrder) -> Self {
        let damping = 1.0 / std::f32::consts::SQRT_2; // 0.707
        let denom = 1.0 + 2.0 * damping * loop_bw + loop_bw * loop_bw;
        let alpha = (4.0 * damping * loop_bw) / denom;
        let beta = (4.0 * loop_bw * loop_bw) / denom;

        Self {
            order,
            alpha,
            beta,
            phase: 0.0,
            freq: 0.0,
            max_freq: 0.5 * PI,
            min_freq: -0.5 * PI,
        }
    }

    /// Process a single complex sample and return phase-corrected sample.
    #[inline]
    pub fn process_sample(&mut self, sample: Complex32) -> Complex32 {
        // De-rotate input sample by current estimated phase
        let (sin_p, cos_p) = self.phase.sin_cos();
        let rotated = Complex32::new(
            sample.re * cos_p + sample.im * sin_p,
            sample.im * cos_p - sample.re * sin_p,
        );

        // Compute phase error detector (PED) based on modulation order
        let error = match self.order {
            ModulationOrder::Bpsk => {
                let sign_re = if rotated.re >= 0.0 { 1.0 } else { -1.0 };
                sign_re * rotated.im
            }
            ModulationOrder::Qpsk => {
                let sign_re = if rotated.re >= 0.0 { 1.0 } else { -1.0 };
                let sign_im = if rotated.im >= 0.0 { 1.0 } else { -1.0 };
                sign_re * rotated.im - sign_im * rotated.re
            }
            ModulationOrder::Psk8 => {
                // 8-PSK phase error via nearest 8-PSK constellation point
                let angle = rotated.im.atan2(rotated.re);
                let rounded = (angle * 4.0 / PI).round() * (PI / 4.0);
                (angle - rounded).sin()
            }
        };

        // Update 2nd-order loop filter
        self.freq += self.beta * error;
        self.freq = self.freq.clamp(self.min_freq, self.max_freq);
        self.phase += self.freq + self.alpha * error;
        self.phase = self.phase.rem_euclid(2.0 * PI);

        rotated
    }

    /// Current frequency offset estimate in rad/sample.
    pub fn frequency(&self) -> f32 {
        self.freq
    }

    /// Current phase estimate in radians.
    pub fn phase(&self) -> f32 {
        self.phase
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.freq = 0.0;
    }
}

impl Block<Complex32, Complex32> for CostasLoop {
    fn process(
        &mut self,
        input: &[Complex32],
        output: &mut Vec<Complex32>,
    ) -> Result<(usize, usize)> {
        output.clear();
        output.reserve(input.len());
        for &s in input {
            output.push(self.process_sample(s));
        }
        Ok((input.len(), output.len()))
    }

    fn reset(&mut self) {
        self.reset();
    }
}
