//! Symbol timing synchronization loops (Gardner and Mueller & Müller).

use sdr_core::sample::Complex32;
use sdr_core::traits::{Block, Result};

/// Gardner symbol timing recovery for PAM/QAM/PSK signals with 2+ samples per symbol.
#[derive(Debug, Clone)]
pub struct GardnerClockRecovery {
    samples_per_symbol: f32,
    omega: f32,
    omega_rel_limit: f32,
    gain_omega: f32,
    gain_mu: f32,
    mu: f32,
    last_sample: Complex32,
    mid_sample: Complex32,
    history: [Complex32; 4],
}

impl GardnerClockRecovery {
    /// Create a new Gardner clock recovery loop.
    pub fn new(samples_per_symbol: f32, gain_mu: f32, gain_omega: f32) -> Self {
        Self {
            samples_per_symbol,
            omega: samples_per_symbol,
            omega_rel_limit: 0.1,
            gain_omega,
            gain_mu,
            mu: 0.0,
            last_sample: Complex32::default(),
            mid_sample: Complex32::default(),
            history: [Complex32::default(); 4],
        }
    }

    /// Process input sample stream and output synchronized symbol samples.
    pub fn process_samples(&mut self, input: &[Complex32], output: &mut Vec<Complex32>) {
        for &s in input {
            // Shift history
            self.history[0] = self.history[1];
            self.history[1] = self.history[2];
            self.history[2] = self.history[3];
            self.history[3] = s;

            while self.mu < 1.0 {
                // Interpolate sample at current mu using 4-point cubic Hermite
                let interpolated = cubic_interpolate(self.history, self.mu);

                // Gardner Timing Error Detector: e = (I_mid * (I_cur - I_last)) + (Q_mid * (Q_cur - Q_last))
                let err_i = self.mid_sample.re * (interpolated.re - self.last_sample.re);
                let err_q = self.mid_sample.im * (interpolated.im - self.last_sample.im);
                let timing_error = (err_i + err_q).clamp(-1.0, 1.0);

                // Update loop filter
                self.omega += self.gain_omega * timing_error;
                let min_omega = self.samples_per_symbol * (1.0 - self.omega_rel_limit);
                let max_omega = self.samples_per_symbol * (1.0 + self.omega_rel_limit);
                self.omega = self.omega.clamp(min_omega, max_omega);

                self.last_sample = self.mid_sample;
                self.mid_sample = interpolated;

                output.push(interpolated);
                self.mu += self.omega + self.gain_mu * timing_error;
            }

            self.mu -= 1.0;
        }
    }

    pub fn reset(&mut self) {
        self.mu = 0.0;
        self.omega = self.samples_per_symbol;
        self.history.fill(Complex32::default());
        self.last_sample = Complex32::default();
        self.mid_sample = Complex32::default();
    }
}

impl Block<Complex32, Complex32> for GardnerClockRecovery {
    fn process(
        &mut self,
        input: &[Complex32],
        output: &mut Vec<Complex32>,
    ) -> Result<(usize, usize)> {
        output.clear();
        output.reserve(input.len() / 2 + 8);
        self.process_samples(input, output);
        Ok((input.len(), output.len()))
    }

    fn reset(&mut self) {
        self.reset();
    }
}

/// 4-point cubic Hermite interpolation.
#[inline(always)]
fn cubic_interpolate(h: [Complex32; 4], mu: f32) -> Complex32 {
    let a0 = -0.5 * h[0] + 1.5 * h[1] - 1.5 * h[2] + 0.5 * h[3];
    let a1 = h[0] - 2.5 * h[1] + 2.0 * h[2] - 0.5 * h[3];
    let a2 = -0.5 * h[0] + 0.5 * h[2];
    let a3 = h[1];

    let mu2 = mu * mu;
    let mu3 = mu2 * mu;

    a0 * mu3 + a1 * mu2 + a2 * mu + a3
}
