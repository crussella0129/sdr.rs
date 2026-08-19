//! # sdr-demod
//!
//! Universal analog (WFM, NFM, AM, SSB, CW) and digital (OOK, FSK, PSK) demodulation pipelines for `sdr.rs`.

pub mod am;
pub mod cw;
pub mod fsk;
pub mod nfm;
pub mod ook;
pub mod psk;
pub mod ssb;
pub mod wfm;

pub use am::AmDemod;
pub use cw::CwDemod;
pub use fsk::FskDemod;
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
        let n = 2400; // 10 ms of signal

        // Generate synthetic FM signal: s(t) = exp(j * 2*pi * deviation * integral(audio(t)))
        let mut iq_samples = Vec::with_capacity(n);
        let mut phase = 0.0f32;
        for i in 0..n {
            let t = i as f32 / sample_rate;
            let audio_val = (TAU * audio_freq * t).sin();
            let inst_freq = deviation * audio_val;
            phase += (inst_freq / sample_rate) * TAU;
            let (s_sin, s_cos) = phase.sin_cos();
            iq_samples.push(Complex32::new(s_cos, s_sin));
        }

        let mut demod = WfmDemod::new(sample_rate, deviation, DeEmphasis::None);
        let mut audio_out = Vec::new();
        demod.demod_block(&iq_samples, &mut audio_out);

        assert_eq!(audio_out.len(), n);
        // Measure correlation between recovered audio and original modulating tone (skip first 20 samples)
        let mut dot_product = 0.0f32;
        let mut norm_a = 0.0f32;
        let mut norm_b = 0.0f32;
        for i in 50..n {
            let t = i as f32 / sample_rate;
            let target = (TAU * audio_freq * t).sin();
            let actual = audio_out[i];
            dot_product += target * actual;
            norm_a += target * target;
            norm_b += actual * actual;
        }

        let correlation = dot_product / (norm_a.sqrt() * norm_b.sqrt());
        assert!(
            correlation > 0.98,
            "WFM demod correlation too low: {}",
            correlation
        );
    }

    #[test]
    fn test_nfm_demod_with_squelch() {
        let mut nfm = NfmDemod::new(48000.0, 5000.0, -30.0);
        // High SNR signal (power = 1.0 -> 0 dB > -30 dB threshold)
        let strong_signal = vec![Complex32::new(0.8, 0.6); 200];
        let mut audio = Vec::new();
        for &s in &strong_signal {
            audio.push(nfm.demod_sample(s));
        }
        assert!(nfm.is_squelch_open());

        // Mute on noise / low power (power = 1e-8 -> -80 dB < -30 dB threshold)
        nfm.reset();
        let weak_noise = vec![Complex32::new(1e-4, 1e-4); 200];
        let mut muted_audio = Vec::new();
        for &s in &weak_noise {
            muted_audio.push(nfm.demod_sample(s));
        }
        assert!(!nfm.is_squelch_open());
        assert_eq!(muted_audio.last().copied(), Some(0.0));
    }

    #[test]
    fn test_am_envelope_demod() {
        let mut am = AmDemod::new();
        let n = 3000;
        let mut input = Vec::with_capacity(n);

        // AM signal: s(t) = (1.0 + 0.5 * cos(2*pi*400*t)) * exp(j * 0)
        for i in 0..n {
            let t = i as f32 / 48000.0;
            let mod_sig = 1.0 + 0.5 * (TAU * 400.0 * t).cos();
            input.push(Complex32::new(mod_sig, 0.0));
        }

        let mut output = Vec::new();
        for &s in &input {
            output.push(am.demod_sample(s));
        }

        assert_eq!(output.len(), n);
        // Peak amplitude in steady state (last 500 samples) should be ~0.5 (+-0.5)
        let peak = output[2500..].iter().fold(0.0f32, |max, &v| max.max(v.abs()));
        assert!(
            (peak - 0.5).abs() < 0.05,
            "AM demod peak incorrect: {}",
            peak
        );
    }

    #[test]
    fn test_ssb_phasing_demod() {
        let mut usb = SsbDemod::new(SsbMode::Usb);
        let mut lsb = SsbDemod::new(SsbMode::Lsb);

        let sample = Complex32::new(1.0, 1.0);
        let usb_out = usb.demod_sample(sample);
        let lsb_out = lsb.demod_sample(sample);

        assert!(usb_out > 0.0);
        assert_eq!(lsb_out, 0.0);
    }

    #[test]
    fn test_fsk_demod_bitstream_recovery() {
        let sample_rate = 96000.0f32;
        let baud_rate = 9600.0f32;
        let deviation = 4800.0f32;
        let sps = (sample_rate / baud_rate) as usize; // 10 samples per symbol

        let test_bits = vec![1, 0, 1, 1, 0, 0, 1, 0];
        let mut iq = Vec::new();
        let mut phase = 0.0f32;

        for &bit in &test_bits {
            let freq_offset = if bit == 1 { deviation } else { -deviation };
            let phase_inc = (freq_offset / sample_rate) * TAU;
            for _ in 0..sps {
                let (sin_val, cos_val) = phase.sin_cos();
                iq.push(Complex32::new(cos_val, sin_val));
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
        let sps = 8;
        let test_bits = vec![1, 1, 0, 1, 0, 0, 1];
        let mut iq = Vec::new();

        for &bit in &test_bits {
            let amp = if bit == 1 { 1.0 } else { 0.0 };
            for _ in 0..sps {
                iq.push(Complex32::new(amp, 0.0));
            }
        }

        let mut demod = OokDemod::new(sps, 0.5);
        let mut recovered_bits = Vec::new();
        demod.demod_bits(&iq, &mut recovered_bits);

        assert_eq!(recovered_bits, test_bits);
    }
}
