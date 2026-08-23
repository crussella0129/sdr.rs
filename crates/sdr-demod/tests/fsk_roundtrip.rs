//! FSK modulator ↔ demodulator round-trip regression.
//!
//! The mesh radio link (INT-0008) carries every datagram through this pair, so
//! its bit-exactness is pinned here. The offset test documents *why* a receiver
//! must align to a symbol boundary rather than reading from an arbitrary point.

use sdr_core::sample::Complex32;
use sdr_demod::modulator::FskModulator;
use sdr_demod::FskDemod;

const SAMPLE_RATE: f32 = 1.0e6;
const DEVIATION: f32 = 100.0e3;
const SPS: usize = 10;

/// Expand bytes to bits, MSB first — the order `modulate_bytes` uses.
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
    let mut m = FskModulator::new(SAMPLE_RATE, DEVIATION, SPS);
    let mut iq = Vec::new();
    m.modulate_bytes(payload, &mut iq);
    iq
}

fn demodulate(iq: &[Complex32]) -> Vec<u8> {
    let mut d = FskDemod::new(SAMPLE_RATE, DEVIATION, SPS);
    let mut bits = Vec::new();
    d.demod_bits(iq, &mut bits);
    bits
}

#[test]
fn test_fsk_roundtrip_bit_exact() {
    // Includes the packet preamble and sync word, plus varied byte values.
    let payload = [0xAA, 0xAA, 0xD3, 0x91, 0x01, 0x02, 0xFF, 0x00, 0x5A];
    let expected = bits_of(&payload);

    let iq = modulate(&payload);
    assert_eq!(iq.len(), payload.len() * 8 * SPS, "one symbol per bit");

    let bits = demodulate(&iq);
    assert_eq!(bits.len(), expected.len(), "one bit per symbol period");
    assert_eq!(
        bits, expected,
        "sample-aligned FSK round trip must be bit-exact"
    );
}

#[test]
fn test_fsk_roundtrip_offset_degrades() {
    // Starting mid-symbol corrupts the bitstream: this is the reason the mesh
    // receiver searches sample phases instead of assuming alignment.
    let payload = [0xAA, 0xAA, 0xD3, 0x91, 0x01, 0x02, 0xFF, 0x00, 0x5A];
    let expected = bits_of(&payload);
    let iq = modulate(&payload);

    let bits = demodulate(&iq[SPS * 7 / 10..]);
    let n = bits.len().min(expected.len());
    let errors = (0..n).filter(|&i| bits[i] != expected[i]).count();
    assert!(
        errors > 0,
        "a large sample offset should corrupt the bitstream, got {errors} errors"
    );
}
