//! ADS-B Mode S (1090 MHz) pulse decoder and telemetry parser.

use sdr_core::sample::Complex32;
use sdr_core::traits::Result;

/// Standard Mode S Downlink Formats (DF).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownlinkFormat {
    ShortAirToAir,
    SurveillanceAltitude,
    SurveillanceIdentity,
    AllCallReply,
    ExtendedSquitter, // ADS-B broadcast
    ExtendedSquitterMilitary,
    CommBAltitude,
    CommBIdentity,
    Other(u8),
}

/// Decoded ADS-B Aircraft State.
#[derive(Debug, Clone, PartialEq)]
pub struct AdsbMessage {
    pub df: DownlinkFormat,
    pub icao_address: u32,
    pub callsign: Option<String>,
    pub altitude_ft: Option<i32>,
    pub raw_bytes: [u8; 14],
    pub crc_valid: bool,
}

/// ADS-B Mode S Demodulator and Message Parser.
pub struct AdsbDecoder;

impl AdsbDecoder {
    /// Demodulate 112-bit Mode S frame from 2 MSPS raw IQ magnitude or baseband samples.
    /// Mode S uses Pulse Position Modulation (PPM) at 2 MSPS:
    /// Bit 1: pulse in first half-microsecond [1, 0]
    /// Bit 0: pulse in second half-microsecond [0, 1]
    pub fn demodulate_ppm(samples: &[Complex32]) -> Option<[u8; 14]> {
        if samples.len() < 240 {
            // Need 16 samples preamble (8 us) + 224 samples payload (112 us) = 240 samples
            return None;
        }

        // Preamble check at 2 MSPS (half-microsecond per sample):
        // Pulses at samples 0, 2, 7, 9 (0us, 1.0us, 3.5us, 4.5us)
        let m0 = samples[0].norm();
        let m2 = samples[2].norm();
        let m7 = samples[7].norm();
        let m9 = samples[9].norm();
        let m1 = samples[1].norm();
        let m3 = samples[3].norm();

        if m0 < 0.1 || m2 < 0.1 || m7 < 0.1 || m9 < 0.1 {
            return None;
        }
        if m1 > m0 || m3 > m2 {
            return None;
        }

        // Slices 112 bits (224 samples starting after 8 us preamble = index 16)
        let payload_samples = &samples[16..16 + 224];
        let mut bytes = [0u8; 14];

        for bit_idx in 0..112 {
            let s_first = payload_samples[bit_idx * 2].norm();
            let s_second = payload_samples[bit_idx * 2 + 1].norm();
            let bit = if s_first > s_second { 1u8 } else { 0u8 };

            let byte_idx = bit_idx / 8;
            let bit_pos = 7 - (bit_idx % 8);
            bytes[byte_idx] |= bit << bit_pos;
        }

        Some(bytes)
    }

    /// Parse a 14-byte (112-bit) Mode S frame.
    pub fn parse_message(raw: &[u8; 14]) -> Result<AdsbMessage> {
        let df_val = raw[0] >> 3;
        let df = match df_val {
            0 => DownlinkFormat::ShortAirToAir,
            4 => DownlinkFormat::SurveillanceAltitude,
            5 => DownlinkFormat::SurveillanceIdentity,
            11 => DownlinkFormat::AllCallReply,
            17 => DownlinkFormat::ExtendedSquitter,
            18 => DownlinkFormat::ExtendedSquitterMilitary,
            20 => DownlinkFormat::CommBAltitude,
            21 => DownlinkFormat::CommBIdentity,
            other => DownlinkFormat::Other(other),
        };

        let crc = modes_checksum(raw);
        let crc_valid = crc == 0;

        // 24-bit ICAO aircraft address
        let icao = ((raw[1] as u32) << 16) | ((raw[2] as u32) << 8) | (raw[3] as u32);

        let mut callsign = None;
        let mut altitude_ft = None;

        if df == DownlinkFormat::ExtendedSquitter {
            let type_code = raw[4] >> 3;
            // Aircraft Identification (Type Code 1..4)
            if (1..=4).contains(&type_code) {
                callsign = Some(decode_callsign(&raw[4..11]));
            }
            // Airborne Position (Type Code 9..18)
            if (9..=18).contains(&type_code) {
                let raw_alt = (((raw[5] as u16) & 0x01) << 11)
                    | ((raw[6] as u16) << 3)
                    | ((raw[7] as u16) >> 5);
                altitude_ft = Some(decode_altitude(raw_alt));
            }
        }

        Ok(AdsbMessage {
            df,
            icao_address: icao,
            callsign,
            altitude_ft,
            raw_bytes: *raw,
            crc_valid,
        })
    }
}

/// Mode S standard 24-bit CRC polynomial division.
pub fn modes_checksum(msg: &[u8; 14]) -> u32 {
    const GENERATOR: u32 = 0x1FFF409; // Mode S generator polynomial
    let mut rem = 0u32;

    for &byte in msg.iter() {
        for bit in (0..8).rev() {
            let msb = (rem & 0x800000) != 0;
            let next_bit = ((byte >> bit) & 1) != 0;
            rem = ((rem << 1) & 0xFFFFFF) | (next_bit as u32);
            if msb {
                rem ^= GENERATOR;
            }
        }
    }
    rem
}

/// Decode 8-character callsign from 6-bit packed ASCII characters.
fn decode_callsign(bytes: &[u8]) -> String {
    const CHARSET: &[u8] = b"?ABCDEFGHIJKLMNOPQRSTUVWXYZ????? ???????????????0123456789??????";
    let mut chars = String::with_capacity(8);

    let b = bytes;
    let c1 = b[1] >> 2;
    let c2 = ((b[1] & 0x03) << 4) | (b[2] >> 4);
    let c3 = ((b[2] & 0x0F) << 2) | (b[3] >> 6);
    let c4 = b[3] & 0x3F;
    let c5 = b[4] >> 2;
    let c6 = ((b[4] & 0x03) << 4) | (b[5] >> 4);
    let c7 = ((b[5] & 0x0F) << 2) | (b[6] >> 6);
    let c8 = b[6] & 0x3F;

    for &c in &[c1, c2, c3, c4, c5, c6, c7, c8] {
        let ch = CHARSET.get(c as usize).copied().unwrap_or(b'?') as char;
        if ch != ' ' {
            chars.push(ch);
        }
    }
    chars.trim().to_string()
}

/// Decode 12-bit Gillham/standard altitude code.
fn decode_altitude(raw: u16) -> i32 {
    let q_bit = (raw & 0x10) != 0;
    if q_bit {
        // 25-ft increments: remove Q-bit (bit 4)
        let n = ((raw >> 5) << 4) | (raw & 0x0F);
        (n as i32 * 25) - 1000
    } else {
        // 100-ft increments (Gillham code)
        (raw as i32 * 100) - 1000
    }
}
