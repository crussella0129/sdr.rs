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

        let start =
            sync_idx.ok_or_else(|| SdrError::Protocol("Sync word not found".to_string()))?;
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

/// Default T1 retransmission timeout, in milliseconds.
///
/// Chosen locally rather than inherited: AX.25's 3000 ms default targets HF/VHF
/// packet over far slower channels. **This value is not measured** — half-duplex
/// turnaround latency on real hardware is still unknown — so treat it as a
/// reasoned default to be replaced once a real link is characterized.
pub const DEFAULT_T1_MS: u64 = 500;

/// A transmitted frame awaiting acknowledgement.
#[derive(Debug, Clone)]
struct Unacked {
    seq: u16,
    frame: Vec<u8>,
    /// Absolute time at which this frame is next due for retransmission.
    next_due_ms: u64,
    /// Retransmissions performed so far (the original send is not a retry).
    retries: usize,
}

/// Reliable ARQ (Automatic Repeat reQuest) State Machine for lossy half-duplex RF channels.
///
/// Stop-and-wait, following AX.25's shape: a T1 retransmission timeout, an N2
/// retry limit ([`max_retries`](Self::max_retries)), and linear backoff.
///
/// # Time is a parameter, never a clock read
///
/// Nothing here reads the system clock. Every time-dependent method takes an
/// explicit `now_ms`, supplied by the caller — real monotonic time in
/// production, whatever a test likes in tests. This is deliberate: retransmission
/// is *defined* by timeouts, and a state machine that reads the clock itself can
/// only be tested by sleeping, which makes timeout coverage slow, flaky, and in
/// practice skipped.
///
/// # Two ways to send
///
/// [`create_data_frame`](Self::create_data_frame) frames a payload and nothing
/// more. [`send_data`](Self::send_data) frames it *and* retains it for
/// retransmission until acknowledged — that is the one to use on a real link.
pub struct ArqTransceiver {
    pub local_addr: u8,
    pub remote_addr: u8,
    pub next_tx_seq: u16,
    pub expected_rx_seq: u16,
    /// Retransmissions attempted before a frame is abandoned (AX.25's N2).
    pub max_retries: usize,
    /// Retransmission timeout in milliseconds (AX.25's T1).
    pub t1_ms: u64,
    unacked: Vec<Unacked>,
    abandoned: Vec<u16>,
}

impl ArqTransceiver {
    pub fn new(local_addr: u8, remote_addr: u8) -> Self {
        Self {
            local_addr,
            remote_addr,
            next_tx_seq: 1,
            expected_rx_seq: 1,
            max_retries: 5,
            t1_ms: DEFAULT_T1_MS,
            unacked: Vec::new(),
            abandoned: Vec::new(),
        }
    }

    /// Frame a data packet **and retain it** until acknowledged.
    ///
    /// Unlike [`create_data_frame`](Self::create_data_frame), the frame is
    /// tracked: if no ACK arrives within T1 it will be returned by
    /// [`due_retransmissions`](Self::due_retransmissions).
    pub fn send_data(&mut self, payload: &[u8], now_ms: u64) -> (u16, Vec<u8>) {
        let (seq, frame) = self.create_data_frame(payload);
        self.unacked.push(Unacked {
            seq,
            frame: frame.clone(),
            next_due_ms: now_ms.saturating_add(self.t1_ms),
            retries: 0,
        });
        (seq, frame)
    }

    /// Frames whose T1 has expired, ready to be sent again.
    ///
    /// Each returned frame has its retry count incremented and its timer
    /// re-armed with linear backoff, so a caller that transmits them and calls
    /// again later sees the next round rather than the same frames immediately.
    /// A frame that has already been retried `max_retries` times is abandoned
    /// instead of returned; see [`take_abandoned`](Self::take_abandoned).
    pub fn due_retransmissions(&mut self, now_ms: u64) -> Vec<(u16, Vec<u8>)> {
        let mut due = Vec::new();
        let max = self.max_retries;
        let t1 = self.t1_ms;
        let mut give_up = Vec::new();

        for u in self.unacked.iter_mut() {
            if now_ms < u.next_due_ms {
                continue;
            }
            if u.retries >= max {
                give_up.push(u.seq);
                continue;
            }
            u.retries += 1;
            // Linear backoff: each successive attempt waits proportionally
            // longer, so a persistently bad channel is not hammered.
            u.next_due_ms = now_ms.saturating_add(t1.saturating_mul(u.retries as u64 + 1));
            due.push((u.seq, u.frame.clone()));
        }

        if !give_up.is_empty() {
            self.unacked.retain(|u| !give_up.contains(&u.seq));
            self.abandoned.extend(give_up);
        }
        due
    }

    /// Sequence numbers abandoned after exceeding `max_retries`, draining them.
    ///
    /// A permanent delivery failure is surfaced rather than swallowed: the
    /// caller decides whether to tear down, report, or retry at a higher level.
    pub fn take_abandoned(&mut self) -> Vec<u16> {
        std::mem::take(&mut self.abandoned)
    }

    /// Number of frames sent but not yet acknowledged.
    pub fn unacked_len(&self) -> usize {
        self.unacked.len()
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
    pub fn process_rx_frame(
        &mut self,
        frame_bytes: &[u8],
    ) -> Result<(Option<Vec<u8>>, Option<Vec<u8>>)> {
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
            PacketType::Ack => {
                // Clear the acknowledged frame so it is never retransmitted.
                // This arm previously discarded the ACK entirely, which — with
                // nothing retained to clear — made the "R" in ARQ unreachable.
                self.unacked.retain(|u| u.seq != pkt.seq_num);
                Ok((None, None))
            }
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

#[cfg(test)]
mod arq_tests {
    use super::*;

    const A: u8 = 0x01;
    const B: u8 = 0x02;

    /// Two transceivers addressed at each other.
    fn pair() -> (ArqTransceiver, ArqTransceiver) {
        (ArqTransceiver::new(A, B), ArqTransceiver::new(B, A))
    }

    #[test]
    fn test_arq_retransmits_after_timeout() {
        let (mut tx, _rx) = pair();
        let (seq, _frame) = tx.send_data(b"hello", 0);

        let due = tx.due_retransmissions(DEFAULT_T1_MS);
        assert_eq!(due.len(), 1, "an unacknowledged frame must come back due");
        assert_eq!(due[0].0, seq, "and it must be the frame that was sent");
    }

    #[test]
    fn test_arq_no_retransmission_before_timeout() {
        let (mut tx, _rx) = pair();
        tx.send_data(b"hello", 0);

        // The negative case. Without it, an implementation that retransmits
        // unconditionally would satisfy the timeout test vacuously.
        let due = tx.due_retransmissions(DEFAULT_T1_MS - 1);
        assert!(
            due.is_empty(),
            "nothing may be retransmitted before T1 expires"
        );
    }

    #[test]
    fn test_arq_ack_stops_retransmission() {
        let (mut tx, mut rx) = pair();
        let (_seq, frame) = tx.send_data(b"hello", 0);

        // The receiver produces an ACK; feed it back to the sender.
        let (payload, ack) = rx.process_rx_frame(&frame).unwrap();
        assert_eq!(payload.as_deref(), Some(&b"hello"[..]));
        let ack = ack.expect("a data frame must be acknowledged");
        tx.process_rx_frame(&ack).unwrap();

        assert_eq!(tx.unacked_len(), 0, "the ACK must clear the retained frame");
        assert!(
            tx.due_retransmissions(DEFAULT_T1_MS * 100).is_empty(),
            "an acknowledged frame must never be retransmitted, however long we wait"
        );
    }

    #[test]
    fn test_arq_gives_up_after_max_retries() {
        let (mut tx, _rx) = pair();
        tx.max_retries = 3;
        tx.send_data(b"unreachable", 0);

        // Advance well past each successive backoff deadline.
        let mut now = 0u64;
        let mut attempts = 0;
        for _ in 0..20 {
            now += DEFAULT_T1_MS * 10;
            attempts += tx.due_retransmissions(now).len();
        }

        assert_eq!(
            attempts, 3,
            "exactly max_retries retransmissions, then stop"
        );
        assert_eq!(
            tx.take_abandoned().len(),
            1,
            "the frame must be reported as a permanent failure, not silently dropped"
        );
        assert_eq!(tx.unacked_len(), 0, "and no longer tracked");
    }

    #[test]
    fn test_arq_duplicate_suppressed_but_acked() {
        let (mut tx, mut rx) = pair();
        let (_seq, frame) = tx.send_data(b"once", 0);

        let (first, ack1) = rx.process_rx_frame(&frame).unwrap();
        assert_eq!(first.as_deref(), Some(&b"once"[..]));
        assert!(ack1.is_some());

        // The same frame again — as happens when our ACK was lost and the
        // sender retransmitted.
        let (second, ack2) = rx.process_rx_frame(&frame).unwrap();
        assert!(
            second.is_none(),
            "a duplicate payload must not be delivered twice — this is what \
             corrupts a byte stream"
        );
        assert!(
            ack2.is_some(),
            "but it must still be acknowledged, or the sender retries forever"
        );
    }
}
