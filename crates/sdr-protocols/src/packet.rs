//! Packet Radio Framing, CRC-32, and ARQ Reliable Link Layer for sdr.rs.

use sdr_core::traits::{Result, SdrError};

/// Preamble for bit clock synchronization (4 bytes = 32 transitions).
pub const PREAMBLE: [u8; 4] = [0xAA, 0xAA, 0xAA, 0xAA];

/// 32-bit Synchronization Word for byte framing.
pub const SYNC_WORD: [u8; 4] = [0xD3, 0x91, 0xD3, 0x91];

/// Packet type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketType {
    Data = 0x01,
    Ack = 0x02,
    Nack = 0x03,
    Ping = 0x04,
    Pong = 0x05,
}

impl PacketType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(PacketType::Data),
            0x02 => Some(PacketType::Ack),
            0x03 => Some(PacketType::Nack),
            0x04 => Some(PacketType::Ping),
            0x05 => Some(PacketType::Pong),
            _ => None,
        }
    }
}

/// Decoded packet header and payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedPacket {
    pub src_addr: u8,
    pub dst_addr: u8,
    pub seq_num: u16,
    pub packet_type: PacketType,
    pub payload: Vec<u8>,
}

/// Packet encoder and decoder.
pub struct PacketFramer;

impl PacketFramer {
    /// Encode a packet into a complete RF frame (Preamble + Sync Word + Header + Payload + CRC-32).
    pub fn encode(
        src_addr: u8,
        dst_addr: u8,
        seq_num: u16,
        packet_type: PacketType,
        payload: &[u8],
    ) -> Vec<u8> {
        let payload_len = payload.len() as u16;
        let mut frame = Vec::with_capacity(4 + 4 + 7 + payload.len() + 4);

        // 1. Preamble & Sync Word
        frame.extend_from_slice(&PREAMBLE);
        frame.extend_from_slice(&SYNC_WORD);

        // 2. Header (7 bytes)
        let header_start = frame.len();
        frame.push(src_addr);
        frame.push(dst_addr);
        frame.extend_from_slice(&seq_num.to_be_bytes());
        frame.push(packet_type as u8);
        frame.extend_from_slice(&payload_len.to_be_bytes());

        // 3. Payload
        frame.extend_from_slice(payload);

        // 4. CRC-32 over Header + Payload
        let crc = crc32_ieee(&frame[header_start..]);
        frame.extend_from_slice(&crc.to_be_bytes());

        frame
    }

    /// Decode a raw frame slice starting immediately at Preamble or Sync Word.
    pub fn decode(raw: &[u8]) -> Result<DecodedPacket> {
        if raw.len() < 19 {
            return Err(SdrError::Protocol(format!(
                "Frame too short: {} bytes (min 19)",
                raw.len()
            )));
        }

        // Search for Sync Word
        let mut sync_idx = None;
        for i in 0..=raw.len().saturating_sub(15) {
            if raw[i..i + 4] == SYNC_WORD {
                sync_idx = Some(i + 4);
                break;
            }
        }

        let start = sync_idx.ok_or_else(|| SdrError::Protocol("Sync word not found".to_string()))?;
        if raw.len() < start + 7 + 4 {
            return Err(SdrError::Protocol("Incomplete packet header".to_string()));
        }

        let src_addr = raw[start];
        let dst_addr = raw[start + 1];
        let seq_num = u16::from_be_bytes([raw[start + 2], raw[start + 3]]);
        let pkt_type_u8 = raw[start + 4];
        let pkt_type = PacketType::from_u8(pkt_type_u8)
            .ok_or_else(|| SdrError::Protocol(format!("Invalid packet type: {}", pkt_type_u8)))?;
        let payload_len = u16::from_be_bytes([raw[start + 5], raw[start + 6]]) as usize;

        let total_needed = start + 7 + payload_len + 4;
        if raw.len() < total_needed {
            return Err(SdrError::Protocol(format!(
                "Incomplete payload: expected {} bytes, got {}",
                total_needed,
                raw.len()
            )));
        }

        let payload_start = start + 7;
        let payload_end = payload_start + payload_len;
        let payload = raw[payload_start..payload_end].to_vec();

        let received_crc = u32::from_be_bytes([
            raw[payload_end],
            raw[payload_end + 1],
            raw[payload_end + 2],
            raw[payload_end + 3],
        ]);

        let calculated_crc = crc32_ieee(&raw[start..payload_end]);
        if received_crc != calculated_crc {
            return Err(SdrError::Protocol(format!(
                "CRC-32 mismatch: received 0x{:08X}, calculated 0x{:08X}",
                received_crc, calculated_crc
            )));
        }

        Ok(DecodedPacket {
            src_addr,
            dst_addr,
            seq_num,
            packet_type: pkt_type,
            payload,
        })
    }
}

/// Reliable ARQ (Automatic Repeat reQuest) State Machine for lossy half-duplex RF channels.
pub struct ArqTransceiver {
    pub local_addr: u8,
    pub remote_addr: u8,
    pub next_tx_seq: u16,
    pub expected_rx_seq: u16,
    pub max_retries: usize,
}

impl ArqTransceiver {
    pub fn new(local_addr: u8, remote_addr: u8) -> Self {
        Self {
            local_addr,
            remote_addr,
            next_tx_seq: 1,
            expected_rx_seq: 1,
            max_retries: 5,
        }
    }

    /// Frame a data packet for transmission.
    pub fn create_data_frame(&mut self, payload: &[u8]) -> (u16, Vec<u8>) {
        let seq = self.next_tx_seq;
        self.next_tx_seq = self.next_tx_seq.wrapping_add(1);
        let frame = PacketFramer::encode(
            self.local_addr,
            self.remote_addr,
            seq,
            PacketType::Data,
            payload,
        );
        (seq, frame)
    }

    /// Create an ACK acknowledgment frame for a received sequence number.
    pub fn create_ack_frame(&self, seq_num: u16) -> Vec<u8> {
        PacketFramer::encode(
            self.local_addr,
            self.remote_addr,
            seq_num,
            PacketType::Ack,
            &[],
        )
    }

    /// Process an incoming frame and return extracted data payload and optional ACK frame to transmit.
    pub fn process_rx_frame(&mut self, frame_bytes: &[u8]) -> Result<(Option<Vec<u8>>, Option<Vec<u8>>)> {
        let pkt = PacketFramer::decode(frame_bytes)?;

        // Ignore packets not destined for us or broadcast (0xFF)
        if pkt.dst_addr != self.local_addr && pkt.dst_addr != 0xFF {
            return Ok((None, None));
        }

        match pkt.packet_type {
            PacketType::Data => {
                let ack_frame = self.create_ack_frame(pkt.seq_num);
                if pkt.seq_num == self.expected_rx_seq {
                    self.expected_rx_seq = self.expected_rx_seq.wrapping_add(1);
                    Ok((Some(pkt.payload), Some(ack_frame)))
                } else {
                    // Duplicate packet received (due to lost ACK) - resend ACK but drop payload
                    Ok((None, Some(ack_frame)))
                }
            }
            PacketType::Ack => Ok((None, None)),
            _ => Ok((None, None)),
        }
    }
}

/// Compute standard IEEE 802.3 CRC-32 checksum.
pub fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}
