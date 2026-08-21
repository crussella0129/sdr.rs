//! LoRa Physical Layer Chirp Spread Spectrum (CSS) Demodulation and Frame Decoding.

use rustfft::{num_complex::Complex, FftPlanner};
use sdr_core::sample::Complex32;
use sdr_core::traits::{Result, SdrError};
use std::f32::consts::TAU;

/// LoRa Spreading Factor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpreadingFactor {
    SF7 = 7,
    SF8 = 8,
    SF9 = 9,
    SF10 = 10,
    SF11 = 11,
    SF12 = 12,
}

/// Decoded LoRa packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoraPacket {
    pub payload: Vec<u8>,
    pub crc_valid: bool,
    pub snr_estimate_db: i32,
}

/// LoRa CSS Demodulator and Decoder.
pub struct LoraDecoder {
    sf: usize,
    num_bins: usize,
    conjugate_downchirp: Vec<Complex32>,
}

impl LoraDecoder {
    /// Create a new LoRa decoder for given Spreading Factor (e.g. SF7 -> 128 bins).
    pub fn new(sf: SpreadingFactor) -> Self {
        let sf_val = sf as usize;
        let num_bins = 1 << sf_val;

        // Generate base conjugate downchirp: exp(-j * 2*pi * (k^2 / (2 * N)))
        let mut conjugate_downchirp = Vec::with_capacity(num_bins);
        for k in 0..num_bins {
            let k_f = k as f32;
            let n_f = num_bins as f32;
            let phase = -TAU * (k_f * k_f) / (2.0 * n_f);
            let (sin_p, cos_p) = phase.sin_cos();
            conjugate_downchirp.push(Complex32::new(cos_p, sin_p));
        }

        Self {
            sf: sf_val,
            num_bins,
            conjugate_downchirp,
        }
    }

    /// Synthesize an up-chirp symbol for given symbol value `[0, 2^SF)`.
    pub fn synthesize_symbol(&self, symbol_val: usize) -> Vec<Complex32> {
        let mut symbol = Vec::with_capacity(self.num_bins);
        let n_f = self.num_bins as f32;
        let s_f = symbol_val as f32;

        for k in 0..self.num_bins {
            let k_f = k as f32;
            // Upchirp with cyclic shift: exp(j * 2*pi * ( (k^2 / 2N) + (s / N) * k ))
            let phase = TAU * ((k_f * k_f) / (2.0 * n_f) + (s_f / n_f) * k_f);
            let (sin_p, cos_p) = phase.sin_cos();
            symbol.push(Complex32::new(cos_p, sin_p));
        }
        symbol
    }

    /// Demodulate a single LoRa symbol (length = $2^{\text{SF}}$) by multiplying with conjugate downchirp and finding FFT peak.
    pub fn demodulate_symbol(&self, symbol_samples: &[Complex32]) -> Result<usize> {
        if symbol_samples.len() < self.num_bins {
            return Err(SdrError::Protocol(format!(
                "Insufficient samples for SF{}: expected {}, got {}",
                self.sf,
                self.num_bins,
                symbol_samples.len()
            )));
        }

        // De-chirp: multiply with conjugate downchirp
        let mut dechirped: Vec<Complex<f32>> = symbol_samples[0..self.num_bins]
            .iter()
            .zip(self.conjugate_downchirp.iter())
            .map(|(&s, &d)| Complex::new(s.re * d.re - s.im * d.im, s.re * d.im + s.im * d.re))
            .collect();

        // Perform FFT
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(self.num_bins);
        fft.process(&mut dechirped);

        // Find peak index
        let mut max_pwr = 0.0f32;
        let mut peak_idx = 0;
        for (idx, c) in dechirped.iter().enumerate() {
            let pwr = c.norm_sqr();
            if pwr > max_pwr {
                max_pwr = pwr;
                peak_idx = idx;
            }
        }

        Ok(peak_idx)
    }

    /// Decode raw payload bytes from a sequence of demodulated symbol words.
    pub fn decode_symbols_to_payload(&self, symbols: &[usize]) -> Result<LoraPacket> {
        if symbols.is_empty() {
            return Err(SdrError::Protocol("Empty symbols vector".to_string()));
        }

        // Convert symbol values to raw bytes via Gray demapping and bit packing
        let mut bits = Vec::new();
        for &sym in symbols {
            // Gray de-mapping: gray_to_bin
            let mut bin = sym;
            let mut mask = sym >> 1;
            while mask > 0 {
                bin ^= mask;
                mask >>= 1;
            }

            for bit_pos in (0..self.sf).rev() {
                bits.push(((bin >> bit_pos) & 1) as u8);
            }
        }

        let mut payload = Vec::new();
        for chunk in bits.chunks_exact(8) {
            let mut byte = 0u8;
            for (i, &b) in chunk.iter().enumerate() {
                byte |= b << (7 - i);
            }
            payload.push(byte);
        }

        let crc_valid = calculate_lora_crc(&payload);
        Ok(LoraPacket {
            payload,
            crc_valid,
            snr_estimate_db: 10,
        })
    }
}

/// Compute LoRa standard CCITT CRC-16 checksum.
pub fn calculate_lora_crc(data: &[u8]) -> bool {
    if data.len() < 2 {
        return true;
    }
    let mut crc = 0x0000u16;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    // Self-checking CRC: CRC over data including 16-bit checksum evaluates to 0
    crc == 0
}
