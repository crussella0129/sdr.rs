//! # sdr-demod
//!
//! Analog and digital modulation and demodulation pipelines for `sdr.rs`.

pub mod am;
pub mod cw;
pub mod fsk;
pub mod modulator;
pub mod nfm;
pub mod ook;
pub mod psk;
pub mod ssb;
pub mod wfm;

pub use am::AmDemod;
pub use cw::CwDemod;
pub use fsk::{FskDemod, FskTimingDemod};
pub use modulator::{FskModulator, GfskModulator};
pub use nfm::NfmDemod;
pub use ook::OokDemod;
pub use psk::PskDemod;
pub use ssb::{SsbDemod, SsbMode};
pub use wfm::{DeEmphasis, WfmDemod};

#[cfg(test)]
mod tests {
    use super::*;
    use sdr_core::sample::Complex32;
    use std::f32::consts::TAU;

    #[test]
    fn test_wfm_demod_synthetic_tone() {
        let sample_rate = 240000.0f32;
        let audio_freq = 1000.0f32;
        let deviation = 75000.0f32;
        let num_samples = 4800;

        let mut iq = Vec::with_capacity(num_samples);
        let mut phase = 0.0f32;

        for n in 0..num_samples {
            let t = n as f32 / sample_rate;
            let audio_val = (TAU * audio_freq * t).sin();
            let inst_freq = audio_val * deviation;
            phase += (inst_freq / sample_rate) * TAU;

            let (sin_p, cos_p) = phase.sin_cos();
            iq.push(Complex32::new(cos_p, sin_p));
        }

        let mut demod = WfmDemod::new(sample_rate, deviation, DeEmphasis::Eu50us);
        let mut audio_out = Vec::new();
        demod.demod_block(&iq, &mut audio_out);

        assert_eq!(audio_out.len(), num_samples);
        let mut corr = 0.0f32;
        for n in 1000..2000 {
            let t = n as f32 / sample_rate;
            let expected = (TAU * audio_freq * t).sin();
            corr += audio_out[n] * expected;
        }
        assert!(
            corr > 100.0,
            "Demodulated audio correlation too low: {}",
            corr
        );
    }

    #[test]
    fn test_nfm_demod_with_squelch() {
        let sample_rate = 48000.0f32;
        let deviation = 5000.0f32;
        let mut demod = NfmDemod::new(sample_rate, deviation, -20.0);

        let noise_sample = Complex32::new(0.001, 0.001);
        let out_muted = demod.demod_sample(noise_sample);
        assert_eq!(out_muted, 0.0);

        let signal_sample = Complex32::new(0.5, 0.5);
        let out_unmuted = demod.demod_sample(signal_sample);
        assert!(out_unmuted.abs() <= 1.0);
    }

    #[test]
    fn test_am_envelope_demod() {
        let mut demod = AmDemod::new();
        let mut audio_out = Vec::new();
        let sample_rate = 48000.0f32;
        let audio_freq = 1000.0f32;

        for n in 0..3000 {
            let t = n as f32 / sample_rate;
            let envelope = 1.0 + 0.5 * (TAU * audio_freq * t).sin();
            let sample = Complex32::new(envelope, 0.0);
            audio_out.push(demod.demod_sample(sample));
        }

        let steady_state = &audio_out[2500..];
        let max_val = steady_state
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
        let min_val = steady_state.iter().cloned().fold(f32::INFINITY, f32::min);

        assert!((max_val - 0.5).abs() < 0.1, "AM peak error: {}", max_val);
        assert!(
            (min_val - (-0.5)).abs() < 0.1,
            "AM valley error: {}",
            min_val
        );
    }

    #[test]
    fn test_ssb_phasing_demod() {
        let mut ssb = SsbDemod::new(SsbMode::Usb);
        let s = Complex32::new(0.707, 0.707);
        let out = ssb.demod_sample(s);
        assert!(out.is_finite());
    }

    #[test]
    fn test_fsk_demod_bitstream_recovery() {
        let sample_rate = 96000.0f32;
        let deviation = 4800.0f32;
        let sps = 10;

        let test_bits = vec![1, 0, 1, 1, 0, 0, 1, 0, 1, 1];
        let mut iq = Vec::new();
        let mut phase = 0.0f32;

        for &bit in &test_bits {
            let freq = if bit == 1 { deviation } else { -deviation };
            let phase_inc = (freq / sample_rate) * TAU;
            for _ in 0..sps {
                let (s_sin, s_cos) = phase.sin_cos();
                iq.push(Complex32::new(s_cos, s_sin));
                phase = (phase + phase_inc).rem_euclid(TAU);
            }
        }

        let mut demod = FskDemod::new(sample_rate, deviation, sps);
        let mut recovered_bits = Vec::new();
        demod.demod_bits(&iq, &mut recovered_bits);

        assert_eq!(recovered_bits, test_bits);
    }

    #[test]
    fn test_ook_demod_bits() {
        let mut demod = OokDemod::new(1, 0.3);
        let samples = vec![
            Complex32::new(0.8, 0.0),
            Complex32::new(0.05, 0.0),
            Complex32::new(0.9, 0.0),
        ];
        let mut bits = Vec::new();
        demod.demod_bits(&samples, &mut bits);
        assert_eq!(bits, vec![1, 0, 1]);
    }

    #[test]
    fn test_gfsk_modulator_continuous_phase() {
        let sample_rate = 96000.0f32;
        let deviation = 4800.0f32;
        let sps = 8;
        let mut gfsk = GfskModulator::new(sample_rate, deviation, sps, 0.5);

        let test_payload = b"SSH-RF";
        let mut iq_out = Vec::new();
        gfsk.modulate_bytes(test_payload, &mut iq_out);

        assert_eq!(iq_out.len(), test_payload.len() * 8 * sps);
        // Verify constant envelope unit power
        for &s in &iq_out {
            let mag = s.norm();
            assert!(
                (mag - 1.0).abs() < 1e-4,
                "GFSK envelope is not unit: {}",
                mag
            );
        }
    }
}
