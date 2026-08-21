//! Continuous-Phase GFSK and 2-FSK Packet Modulators.

use sdr_core::sample::Complex32;
use std::f32::consts::TAU;

/// Continuous-Phase 2-FSK Modulator.
pub struct FskModulator {
    sample_rate: f32,
    deviation_hz: f32,
    samples_per_symbol: usize,
    phase: f32,
}

impl FskModulator {
    pub fn new(sample_rate: f32, deviation_hz: f32, samples_per_symbol: usize) -> Self {
        Self {
            sample_rate,
            deviation_hz,
            samples_per_symbol,
            phase: 0.0,
        }
    }

    /// Modulate an array of bits (0 or 1) into complex baseband IQ samples.
    pub fn modulate_bits(&mut self, bits: &[u8], output: &mut Vec<Complex32>) {
        output.reserve(bits.len() * self.samples_per_symbol);

        for &bit in bits {
            let freq = if bit == 1 {
                self.deviation_hz
            } else {
                -self.deviation_hz
            };
            let phase_inc = (freq / self.sample_rate) * TAU;

            for _ in 0..self.samples_per_symbol {
                let (sin_p, cos_p) = self.phase.sin_cos();
                output.push(Complex32::new(cos_p, sin_p));
                self.phase = (self.phase + phase_inc).rem_euclid(TAU);
            }
        }
    }

    /// Modulate raw bytes (MSB first) into complex baseband IQ samples.
    pub fn modulate_bytes(&mut self, bytes: &[u8], output: &mut Vec<Complex32>) {
        let mut bits = Vec::with_capacity(bytes.len() * 8);
        for &byte in bytes {
            for bit_pos in (0..8).rev() {
                bits.push((byte >> bit_pos) & 1);
            }
        }
        self.modulate_bits(&bits, output);
    }
}

/// Gaussian Frequency Shift Keying (GFSK) Modulator with smooth pulse shaping.
pub struct GfskModulator {
    sample_rate: f32,
    deviation_hz: f32,
    sps: usize,
    gaussian_taps: Vec<f32>,
    history: Vec<f32>,
    phase: f32,
}

impl GfskModulator {
    /// Create a new GFSK modulator with bandwidth-time product (BT, typically 0.5).
    pub fn new(sample_rate: f32, deviation_hz: f32, samples_per_symbol: usize, bt: f32) -> Self {
        let sps = samples_per_symbol;
        let span = 4; // Filter span in symbols
        let num_taps = span * sps + 1;
        let mut taps = Vec::with_capacity(num_taps);

        // Design Gaussian filter taps
        // alpha = sqrt(2 * ln(2)) / (BT)
        let alpha = (2.0f32 * 2.0f32.ln()).sqrt() / bt;
        let mut sum = 0.0f32;

        for i in 0..num_taps {
            let t = (i as f32 - (num_taps - 1) as f32 / 2.0) / (sps as f32);
            let g = (-2.0 * std::f32::consts::PI * std::f32::consts::PI * t * t / (alpha * alpha)).exp();
            taps.push(g);
            sum += g;
        }

        // Normalize
        for tap in &mut taps {
            *tap /= sum;
        }

        Self {
            sample_rate,
            deviation_hz,
            sps,
            gaussian_taps: taps,
            history: vec![0.0; span * sps + 1],
            phase: 0.0,
        }
    }

    /// Modulate bits into continuous-phase Gaussian-filtered baseband IQ samples.
    pub fn modulate_bits(&mut self, bits: &[u8], output: &mut Vec<Complex32>) {
        for &bit in bits {
            let sym_val = if bit == 1 { 1.0f32 } else { -1.0f32 };

            for sub_idx in 0..self.sps {
                // Shift history
                self.history.rotate_right(1);
                self.history[0] = if sub_idx == 0 { sym_val } else { 0.0 };

                // Compute instantaneous frequency deviation via Gaussian convolution
                let mut freq_dev = 0.0f32;
                for (tap, &hist) in self.gaussian_taps.iter().zip(self.history.iter()) {
                    freq_dev += tap * hist;
                }

                let phase_inc = (freq_dev * self.deviation_hz / self.sample_rate) * TAU;
                let (sin_p, cos_p) = self.phase.sin_cos();
                output.push(Complex32::new(cos_p, sin_p));
                self.phase = (self.phase + phase_inc).rem_euclid(TAU);
            }
        }
    }

    /// Modulate byte slice into GFSK IQ samples.
    pub fn modulate_bytes(&mut self, bytes: &[u8], output: &mut Vec<Complex32>) {
        let mut bits = Vec::with_capacity(bytes.len() * 8);
        for &byte in bytes {
            for bit_pos in (0..8).rev() {
                bits.push((byte >> bit_pos) & 1);
            }
        }
        self.modulate_bits(&bits, output);
    }
}
