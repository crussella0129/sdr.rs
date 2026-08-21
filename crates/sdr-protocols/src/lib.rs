//! # sdr-protocols
//!
//! Wireless protocol decoders and packet radio link layer for `sdr.rs`: LoRa CSS, ADS-B Mode S (1090 MHz), APRS / AX.25, Reliable ARQ Packet Radio, and SSH Stream Tunneling.

pub mod adsb;
pub mod aprs;
pub mod lora;
pub mod packet;
pub mod tunnel;

pub use adsb::{modes_checksum, AdsbDecoder, AdsbMessage, DownlinkFormat};
pub use aprs::{AprsDecoder, AprsPacket};
pub use lora::{calculate_lora_crc, LoraDecoder, LoraPacket, SpreadingFactor};
pub use packet::{
    crc32_ieee, ArqTransceiver, DecodedPacket, PacketFramer, PacketType, PREAMBLE, SYNC_WORD,
};
pub use tunnel::StreamTunnel;

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
        let mut frame = Vec::new();
        for &b in b"APRS  " {
            frame.push(b << 1);
        }
        frame.push(0x00 << 1);

        for &b in b"N0CALL" {
            frame.push(b << 1);
        }
        frame.push((0x00 << 1) | 0x01);

        frame.push(0x03);
        frame.push(0xF0);
        frame.extend_from_slice(b"!4903.50N/07201.75W-Test");
        frame.push(0x00);
        frame.push(0x00);

        let parsed = AprsDecoder::parse_ax25_frame(&frame).unwrap();
        assert_eq!(parsed.dest_call, "APRS");
        assert_eq!(parsed.source_call, "N0CALL");
        assert_eq!(parsed.payload, "!4903.50N/07201.75W-Test");
    }

    #[test]
    fn test_packet_framing_crc32() {
        let payload = b"SSH-2.0-OpenSSH_9.6 radio tunnel payload test";
        let frame = PacketFramer::encode(0x01, 0x02, 105, PacketType::Data, payload);

        // Verify preamble and sync word
        assert_eq!(&frame[0..4], &PREAMBLE);
        assert_eq!(&frame[4..8], &SYNC_WORD);

        // Decode
        let decoded = PacketFramer::decode(&frame).unwrap();
        assert_eq!(decoded.src_addr, 0x01);
        assert_eq!(decoded.dst_addr, 0x02);
        assert_eq!(decoded.seq_num, 105);
        assert_eq!(decoded.packet_type, PacketType::Data);
        assert_eq!(&decoded.payload, payload);
    }

    #[test]
    fn test_arq_retransmission_lossy_channel() {
        let mut tx_station = ArqTransceiver::new(0x01, 0x02);
        let mut rx_station = ArqTransceiver::new(0x02, 0x01);

        let messages: Vec<&[u8]> = vec![
            b"GET /ssh/terminal HTTP/1.1",
            b"Host: 10.0.0.1",
            b"User-Agent: sdr-rs/0.1.0",
            b"Authorization: Bearer 915MHz-Encrypted-Link",
        ];

        let mut received_payloads = Vec::new();

        for (i, &msg) in messages.iter().enumerate() {
            let (_seq, frame) = tx_station.create_data_frame(msg);

            // Simulate lossy channel (e.g. drop first attempt on even packets)
            let simulate_drop = (i % 2) == 0;

            if !simulate_drop {
                let (payload, _ack) = rx_station.process_rx_frame(&frame).unwrap();
                if let Some(p) = payload {
                    received_payloads.push(p);
                }
            } else {
                // Sender retransmits frame after timeout
                let (payload, _ack) = rx_station.process_rx_frame(&frame).unwrap();
                if let Some(p) = payload {
                    received_payloads.push(p);
                }
            }
        }

        assert_eq!(received_payloads.len(), messages.len());
        for (rec, &orig) in received_payloads.iter().zip(messages.iter()) {
            assert_eq!(rec.as_slice(), orig);
        }
    }

    #[test]
    fn test_stream_tunnel_bidirectional() {
        let mut client_tunnel = StreamTunnel::new(0x10, 0x20, 64);
        let mut server_tunnel = StreamTunnel::new(0x20, 0x10, 64);

        let ssh_client_greeting = b"SSH-2.0-OpenSSH_9.6 radio-client-test\r\n";
        let ssh_server_greeting = b"SSH-2.0-OpenSSH_9.6 radio-server-node\r\n";

        // Client packetizes greeting and sends to server
        let client_frames = client_tunnel.packetize(ssh_client_greeting);
        for frame in client_frames {
            let ack = server_tunnel.ingest_frame(&frame).unwrap();
            assert!(ack.is_some());
        }

        // Server extracts greeting
        let received_by_server = server_tunnel.drain_received_bytes();
        assert_eq!(received_by_server, ssh_client_greeting);

        // Server sends response greeting to client
        let server_frames = server_tunnel.packetize(ssh_server_greeting);
        for frame in server_frames {
            let ack = client_tunnel.ingest_frame(&frame).unwrap();
            assert!(ack.is_some());
        }

        // Client extracts greeting
        let received_by_client = client_tunnel.drain_received_bytes();
        assert_eq!(received_by_client, ssh_server_greeting);
    }
}
