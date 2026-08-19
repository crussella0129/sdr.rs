//! APRS / AX.25 Packet Radio Demodulator and Telemetry Parser.

use sdr_core::traits::{Result, SdrError};

/// Decoded APRS packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AprsPacket {
    pub source_call: String,
    pub dest_call: String,
    pub path: Vec<String>,
    pub payload: String,
    pub crc_valid: bool,
}

/// AX.25 / APRS Packet Parser.
pub struct AprsDecoder;

impl AprsDecoder {
    /// Parse raw AX.25 HDLC framed bytes.
    pub fn parse_ax25_frame(frame: &[u8]) -> Result<AprsPacket> {
        if frame.len() < 16 {
            return Err(SdrError::Protocol(format!(
                "AX.25 frame too short: {} bytes (min 16)",
                frame.len()
            )));
        }

        let crc_valid = check_fcs(frame);

        // Destination Call (bytes 0..7) and Source Call (bytes 7..14)
        let dest_call = decode_ax25_callsign(&frame[0..7]);
        let source_call = decode_ax25_callsign(&frame[7..14]);

        let mut idx = 14;
        let mut path = Vec::new();

        // Check for digipeaters (each is 7 bytes)
        while idx + 7 <= frame.len() && (frame[idx - 1] & 0x01) == 0 {
            path.push(decode_ax25_callsign(&frame[idx..idx + 7]));
            idx += 7;
        }

        // Skip Control byte and PID byte (typically 0x03, 0xF0 for UI frames)
        if idx + 2 <= frame.len() {
            idx += 2;
        }

        // Payload is remaining bytes minus 2-byte FCS (CRC)
        let payload_len = frame.len().saturating_sub(idx + 2);
        let payload_bytes = &frame[idx..idx + payload_len];
        let payload = String::from_utf8_lossy(payload_bytes).to_string();

        Ok(AprsPacket {
            source_call,
            dest_call,
            path,
            payload,
            crc_valid,
        })
    }
}

/// Decode AX.25 shifted ASCII callsign (6 characters + 1 SSID byte).
fn decode_ax25_callsign(bytes: &[u8]) -> String {
    let mut call = String::with_capacity(7);
    for &b in &bytes[0..6] {
        let ch = (b >> 1) as char;
        if ch != ' ' {
            call.push(ch);
        }
    }
    let ssid = (bytes[6] >> 1) & 0x0F;
    if ssid > 0 {
        format!("{}-{}", call, ssid)
    } else {
        call
    }
}

/// Compute and verify standard CCITT CRC-16 / FCS for AX.25.
fn check_fcs(data: &[u8]) -> bool {
    let mut crc = 0xFFFFu16;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0x8408;
            } else {
                crc >>= 1;
            }
        }
    }
    // Expected residual FCS constant: 0xF0B8
    crc == 0xF0B8
}
