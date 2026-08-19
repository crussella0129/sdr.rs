//! # sdr-protocols
//!
//! Wireless protocol decoders for `sdr.rs`: LoRa CSS, ADS-B Mode S (1090 MHz), and APRS / AX.25.

pub mod adsb;
pub mod aprs;
pub mod lora;

pub use adsb::{modes_checksum, AdsbDecoder, AdsbMessage, DownlinkFormat};
pub use aprs::{AprsDecoder, AprsPacket};
pub use lora::{calculate_lora_crc, LoraDecoder, LoraPacket, SpreadingFactor};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lora_css_demod_and_fec() {
        let decoder = LoraDecoder::new(SpreadingFactor::SF7);
        let test_symbol_val = 42usize;

        // Synthesize chirp for symbol 42
        let chirp = decoder.synthesize_symbol(test_symbol_val);
        assert_eq!(chirp.len(), 128);

        // Demodulate chirp and verify exact peak index
        let recovered_symbol = decoder.demodulate_symbol(&chirp).unwrap();
        assert_eq!(recovered_symbol, test_symbol_val);

        // Test multi-symbol decode
        let symbols = vec![10, 25, 42, 60];
        let packet = decoder.decode_symbols_to_payload(&symbols).unwrap();
        assert!(!packet.payload.is_empty());
    }

    #[test]
    fn test_adsb_mode_s_crc24_and_decode() {
        // Standard ADS-B Mode S DF17 Test Vector (Aircraft Identification: KLM1023, ICAO: 0x4840D6)
        // Hex: 8D4840D6202CC371C32CE0576098
        let raw_hex: [u8; 14] = [
            0x8D, 0x48, 0x40, 0xD6, 0x20, 0x2C, 0xC3, 0x71, 0xC3, 0x2C, 0xE0, 0x57, 0x60, 0x98,
        ];

        let msg = AdsbDecoder::parse_message(&raw_hex).unwrap();
        assert_eq!(msg.df, DownlinkFormat::ExtendedSquitter);
        assert_eq!(msg.icao_address, 0x4840D6);
        assert!(msg.crc_valid, "Mode S CRC24 checksum failed");
        assert_eq!(msg.callsign.as_deref(), Some("KLM1023"));
    }

    #[test]
    fn test_aprs_afsk_decode() {
        // Synthetic AX.25 UI Frame: Dest="APRS  ", Source="N0CALL", Payload="!4903.50N/07201.75W-Test"
        let mut frame = Vec::new();
        // Dest: APRS (shifted by 1)
        for &b in b"APRS  " {
            frame.push(b << 1);
        }
        frame.push(0x00 << 1); // SSID 0

        // Source: N0CALL (shifted by 1)
        for &b in b"N0CALL" {
            frame.push(b << 1);
        }
        frame.push((0x00 << 1) | 0x01); // SSID 0 + end of address bit

        // Control & PID
        frame.push(0x03);
        frame.push(0xF0);

        // Info payload
        frame.extend_from_slice(b"!4903.50N/07201.75W-Test");

        // Append 2-byte dummy FCS
        frame.push(0x00);
        frame.push(0x00);

        let parsed = AprsDecoder::parse_ax25_frame(&frame).unwrap();
        assert_eq!(parsed.dest_call, "APRS");
        assert_eq!(parsed.source_call, "N0CALL");
        assert_eq!(parsed.payload, "!4903.50N/07201.75W-Test");
    }
}
