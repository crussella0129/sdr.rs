//! `FskTimingDemod` regression: recovery without assuming symbol alignment.
//!
//! These are the cases the fixed-count [`FskDemod`] cannot handle — a
//! mid-symbol start, and a transmitter/receiver clock offset.

use sdr_core::sample::Complex32;
use sdr_demod::modulator::FskModulator;
use sdr_demod::FskTimingDemod;

const SR: f32 = 1.0e6;
const DEV: f32 = 100.0e3;
const SPS: usize = 10;

/// Payload including the packet preamble and sync word, plus varied bytes.
const PAYLOAD: [u8; 10] = [0xAA, 0xD3, 0x91, 0x4B, 0x1E, 0x77, 0x00, 0xFF, 0x5A, 0xC3];

fn bits_of(bytes: &[u8]) -> Vec<u8> {
    let mut bits = Vec::with_capacity(bytes.len() * 8);
    for &b in bytes {
        for pos in (0..8).rev() {
            bits.push((b >> pos) & 1);
        }
    }
    bits
}

fn modulate(payload: &[u8]) -> Vec<Complex32> {
    let mut m = FskModulator::new(SR, DEV, SPS);
    let mut iq = Vec::new();
    m.modulate_bytes(payload, &mut iq);
    iq
}

/// Resample by `ratio` via linear interpolation; `ratio > 1` simulates a
/// receiver clock running fast relative to the transmitter.
fn resample(iq: &[Complex32], ratio: f32) -> Vec<Complex32> {
    let mut out = Vec::new();
    let mut pos = 0.0f32;
    while (pos as usize) + 1 < iq.len() {
        let i = pos as usize;
        let frac = pos - i as f32;
        let (a, b) = (iq[i], iq[i + 1]);
        out.push(Complex32::new(
            a.re + (b.re - a.re) * frac,
            a.im + (b.im - a.im) * frac,
        ));
        pos += ratio;
    }
    out
}

fn demodulate(iq: &[Complex32]) -> Vec<u8> {
    let mut d = FskTimingDemod::with_defaults(SPS as f32);
    let mut bits = Vec::new();
    d.demod_bits(iq, &mut bits);
    bits
}

/// Fewest mismatches over a bounded start-up lag — the loop's first output can
/// land a symbol early or late while it settles.
fn best_bit_errors(got: &[u8], expected: &[u8]) -> (usize, usize) {
    let mut best = (usize::MAX, 0usize);
    for lag in 0..got.len().min(4) {
        let n = (got.len() - lag).min(expected.len());
        if n < 32 {
            break;
        }
        let errs = (0..n).filter(|&i| got[lag + i] != expected[i]).count();
        if errs < best.0 {
            best = (errs, n);
        }
    }
    best
}

#[test]
fn test_timing_demod_aligned() {
    let expected = bits_of(&PAYLOAD);
    let bits = demodulate(&modulate(&PAYLOAD));
    let (errors, compared) = best_bit_errors(&bits, &expected);
    assert_eq!(
        errors, 0,
        "aligned burst must demodulate bit-exact ({errors} errors over {compared})"
    );
}

#[test]
fn test_timing_demod_mid_symbol() {
    // 7 of 10 samples into the first symbol — the offset that produces ~half
    // the bits wrong with the fixed-count demodulator.
    let expected = bits_of(&PAYLOAD);
    let iq = modulate(&PAYLOAD);
    let bits = demodulate(&iq[7..]);
    let (errors, compared) = best_bit_errors(&bits, &expected);
    assert_eq!(
        errors, 0,
        "a mid-symbol start must still demodulate bit-exact ({errors} errors over {compared})"
    );
}

#[test]
fn test_timing_demod_clock_drift() {
    // A fixed sample phase cannot track a clock offset at all; the loop must.
    let expected = bits_of(&PAYLOAD);
    let iq = modulate(&PAYLOAD);

    for pct in [0.1f32, 0.5] {
        for ratio in [1.0 + pct / 100.0, 1.0 - pct / 100.0] {
            // Offset the start as well, so timing and rate are both wrong.
            let bits = demodulate(&resample(&iq[7..], ratio));
            let (errors, compared) = best_bit_errors(&bits, &expected);
            assert_eq!(
                errors, 0,
                "clock ratio {ratio} (±{pct}%) must demodulate bit-exact \
                 ({errors} errors over {compared})"
            );
        }
    }
}
