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
    crc32_ieee, ArqTransceiver, DecodedPacket, PacketFramer, PacketType, PreparedTransmission,
    ProcessedFrame, PREAMBLE, SYNC_WORD,
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

    // NOTE: `test_arq_retransmission_lossy_channel` was deleted in Sprint 9.
    // Its two branches were character-for-character identical, it simulated no
    // loss and performed no retransmission, yet its name was cited as evidence
    // that INT-0006 criterion 2 was met. The honest replacements live in
    // `arq_reliability` below.

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

/// Reliability of the ARQ layer over a channel that actually drops frames.
///
/// This module exists because the test it replaces did not do what its name
/// said. INT-0006 criterion 2 requires delivery "under simulated RF packet loss
/// up to 30%", and until Sprint 9 nothing in this workspace simulated any loss
/// at all.
///
/// Two properties make these tests trustworthy rather than merely green:
/// **loss is seeded**, so a failure reproduces exactly instead of appearing
/// once in twenty runs; and **time is explicit**, so retransmission timeouts
/// are exercised instantly with no sleeping.
#[cfg(test)]
mod arq_reliability {
    use crate::packet::{ArqTransceiver, DEFAULT_T1_MS};

    /// Seeded xorshift64*. A dependency-free PRNG is enough to schedule drops
    /// reproducibly, and keeps the crate's dependency surface unchanged for a
    /// test-only need.
    struct Rng(u64);

    impl Rng {
        fn new(seed: u64) -> Self {
            // Any non-zero state works; xorshift is degenerate at zero.
            Rng(seed | 1)
        }
        fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            self.0 = x;
            x.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }
        /// True with probability `percent`/100.
        fn drops(&mut self, percent: u32) -> bool {
            (self.next_u64() % 100) < percent as u64
        }
    }

    /// What the channel is allowed to drop.
    #[derive(Clone, Copy, PartialEq)]
    enum Drop {
        Data,
        Acks,
    }

    /// Run `count` payloads from A to B over a channel dropping `loss_pct`.
    ///
    /// Returns the payloads B delivered, in order. Stop-and-wait: A does not
    /// send the next payload until the current one is acknowledged, retrying on
    /// T1 expiry; the loop advances simulated time rather than sleeping.
    fn run(count: usize, loss_pct: u32, what: Drop, seed: u64) -> Vec<Vec<u8>> {
        let mut a = ArqTransceiver::new(0x01, 0x02);
        let mut b = ArqTransceiver::new(0x02, 0x01);
        a.max_retries = 100; // generous: we are testing delivery, not giving up
        let mut rng = Rng::new(seed);
        let mut delivered = Vec::new();
        let mut now = 0u64;

        for i in 0..count {
            let payload = format!("payload-{i}").into_bytes();
            let (_seq, frame) = a.send_data(&payload, now).unwrap();
            let mut on_air = vec![frame];

            // Deliver this payload before moving to the next.
            for _ in 0..500 {
                for frame in std::mem::take(&mut on_air) {
                    if what == Drop::Data && rng.drops(loss_pct) {
                        continue; // data frame lost
                    }
                    let (payload, ack) = b.process_rx_frame(&frame).unwrap();
                    if let Some(p) = payload {
                        delivered.push(p);
                    }
                    if let Some(ack) = ack {
                        if what == Drop::Acks && rng.drops(loss_pct) {
                            continue; // ACK lost — sender will retransmit
                        }
                        a.process_rx_frame(&ack).unwrap();
                    }
                }
                if a.unacked_len() == 0 {
                    break; // acknowledged; move on
                }
                now += DEFAULT_T1_MS * 2;
                if let Some(retry) = a.peek_due_retransmission(now) {
                    // Reaching the channel is a completed transmit operation;
                    // the seeded channel may then erase the frame.
                    a.commit_retransmission(&retry, now).unwrap();
                    on_air.push(retry.frame().to_vec());
                }
                a.abandon_if_exhausted(now);
            }
            assert_eq!(
                a.unacked_len(),
                0,
                "payload {i} was never acknowledged at {loss_pct}% loss"
            );
        }
        delivered
    }

    fn expected(count: usize) -> Vec<Vec<u8>> {
        (0..count)
            .map(|i| format!("payload-{i}").into_bytes())
            .collect()
    }

    #[test]
    fn test_arq_delivers_all_payloads_at_30pct_loss() {
        // The criterion-2 test: "retransmits dropped packets under simulated RF
        // packet loss up to 30%".
        let got = run(24, 30, Drop::Data, 0xC0FFEE);
        assert_eq!(
            got,
            expected(24),
            "every payload must arrive exactly once, in order, at 30% frame loss"
        );
    }

    #[test]
    fn test_arq_delivers_all_payloads_at_10pct_loss() {
        let got = run(24, 10, Drop::Data, 0xBEEF);
        assert_eq!(got, expected(24), "same guarantee at a milder loss rate");
    }

    #[test]
    fn test_arq_ack_loss_does_not_duplicate_payload() {
        // Losing ACKs is the nastier case: the data arrived, so retransmission
        // re-delivers something the receiver already has. Suppressing that
        // duplicate is what keeps a byte stream from being corrupted.
        let got = run(16, 40, Drop::Acks, 0xACC0);
        assert_eq!(
            got,
            expected(16),
            "a lost ACK must not cause the payload to be delivered twice"
        );
    }

    #[test]
    fn test_arq_lossy_channel_is_deterministic() {
        // A reliability test that cannot be reproduced is a flake generator.
        let a = run(16, 30, Drop::Data, 0x1234_5678);
        let b = run(16, 30, Drop::Data, 0x1234_5678);
        assert_eq!(a, b, "the same seed must produce the same run");
    }
}
