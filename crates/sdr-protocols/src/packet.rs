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

/// Payload accepted from a received frame and the optional ACK to transmit.
pub type ProcessedFrame = (Option<Vec<u8>>, Option<Vec<u8>>);

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

/// A framed transmission prepared by [`ArqTransceiver`].
///
/// The bytes and sequence number are private so callers cannot commit a
/// different frame from the one the state machine prepared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedTransmission {
    seq: u16,
    frame: Vec<u8>,
}

impl PreparedTransmission {
    /// Sequence number reserved for this transmission.
    pub fn sequence(&self) -> u16 {
        self.seq
    }

    /// Complete framed packet bytes to hand to the link.
    pub fn frame(&self) -> &[u8] {
        &self.frame
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TxPhase {
    /// The initial write has not yet completed.
    Prepared,
    /// The initial write completed and the frame is awaiting its ACK.
    AwaitingAck {
        /// Absolute time at which this frame is next due for retransmission.
        next_due_ms: u64,
        /// Committed retransmissions (the original send is not a retry).
        retries: usize,
    },
}

/// The one frame occupying the stop-and-wait transmit slot.
#[derive(Debug, Clone)]
struct PendingTx {
    transmission: PreparedTransmission,
    phase: TxPhase,
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
/// # Tracked and untracked transmission
///
/// [`create_data_frame`](Self::create_data_frame) frames a payload and nothing
/// more. I/O-backed reliable links use [`prepare_data`](Self::prepare_data),
/// then commit or abort the prepared frame according to the driver result.
/// [`send_data`](Self::send_data) is a convenience for simulations where the
/// initial transmission cannot fail.
pub struct ArqTransceiver {
    pub local_addr: u8,
    pub remote_addr: u8,
    pub next_tx_seq: u16,
    pub expected_rx_seq: u16,
    /// Retransmissions attempted before a frame is abandoned (AX.25's N2).
    pub max_retries: usize,
    /// Retransmission timeout in milliseconds (AX.25's T1).
    pub t1_ms: u64,
    pending_tx: Option<PendingTx>,
    abandoned: Vec<u16>,
    /// Exact sequence most recently delivered, used to recognize one valid
    /// duplicate without mistaking an arbitrary future frame for a duplicate.
    last_delivered_seq: Option<u16>,
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
            pending_tx: None,
            abandoned: Vec::new(),
            last_delivered_seq: None,
        }
    }

    /// Prepare the sole tracked data packet without committing a transmission.
    ///
    /// Stop-and-wait permits exactly one prepared or awaiting-ACK frame. A
    /// second request returns explicit backpressure without consuming a
    /// sequence number.
    pub fn prepare_data(&mut self, payload: &[u8]) -> Result<PreparedTransmission> {
        if self.pending_tx.is_some() {
            return Err(SdrError::Protocol(
                "ARQ transmitter busy: one frame is already outstanding".to_string(),
            ));
        }

        let transmission = PreparedTransmission {
            seq: self.next_tx_seq,
            frame: PacketFramer::encode(
                self.local_addr,
                self.remote_addr,
                self.next_tx_seq,
                PacketType::Data,
                payload,
            ),
        };
        self.pending_tx = Some(PendingTx {
            transmission: transmission.clone(),
            phase: TxPhase::Prepared,
        });
        Ok(transmission)
    }

    /// Commit a fully written initial transmission and start its T1 timer.
    pub fn commit_initial(
        &mut self,
        transmission: &PreparedTransmission,
        now_ms: u64,
    ) -> Result<()> {
        let pending = self.pending_tx.as_mut().ok_or_else(|| {
            SdrError::Protocol("no prepared ARQ transmission to commit".to_string())
        })?;
        if pending.transmission != *transmission || pending.phase != TxPhase::Prepared {
            return Err(SdrError::Protocol(
                "stale or mismatched prepared ARQ transmission".to_string(),
            ));
        }

        pending.phase = TxPhase::AwaitingAck {
            next_due_ms: now_ms.saturating_add(self.t1_ms),
            retries: 0,
        };
        self.next_tx_seq = self.next_tx_seq.wrapping_add(1);
        Ok(())
    }

    /// Abort an initial transmission whose driver write did not complete.
    ///
    /// No sequence number is consumed, so a later successful preparation uses
    /// the same sequence the peer is still expecting.
    pub fn abort_initial(&mut self, transmission: &PreparedTransmission) -> Result<()> {
        let pending = self.pending_tx.as_ref().ok_or_else(|| {
            SdrError::Protocol("no prepared ARQ transmission to abort".to_string())
        })?;
        if pending.transmission != *transmission || pending.phase != TxPhase::Prepared {
            return Err(SdrError::Protocol(
                "stale or mismatched prepared ARQ transmission".to_string(),
            ));
        }
        self.pending_tx = None;
        Ok(())
    }

    /// Frame a data packet and assume its initial write completed.
    ///
    /// This convenience is for protocol simulations with no fallible driver.
    /// I/O-backed callers must use [`prepare_data`](Self::prepare_data) followed
    /// by [`commit_initial`](Self::commit_initial) or
    /// [`abort_initial`](Self::abort_initial).
    pub fn send_data(&mut self, payload: &[u8], now_ms: u64) -> Result<(u16, Vec<u8>)> {
        let transmission = self.prepare_data(payload)?;
        self.commit_initial(&transmission, now_ms)?;
        Ok((transmission.seq, transmission.frame))
    }

    /// Peek at a due retry without consuming retry budget or rearming T1.
    ///
    /// The returned transmission remains due until
    /// [`commit_retransmission`](Self::commit_retransmission) records a full
    /// driver write.
    pub fn peek_due_retransmission(&self, now_ms: u64) -> Option<PreparedTransmission> {
        let pending = self.pending_tx.as_ref()?;
        match pending.phase {
            TxPhase::AwaitingAck {
                next_due_ms,
                retries,
            } if now_ms >= next_due_ms && retries < self.max_retries => {
                Some(pending.transmission.clone())
            }
            _ => None,
        }
    }

    /// Commit a fully written retry and rearm T1 with linear backoff.
    pub fn commit_retransmission(
        &mut self,
        transmission: &PreparedTransmission,
        now_ms: u64,
    ) -> Result<()> {
        let pending = self.pending_tx.as_mut().ok_or_else(|| {
            SdrError::Protocol("no outstanding ARQ transmission to retry".to_string())
        })?;
        if pending.transmission != *transmission {
            return Err(SdrError::Protocol(
                "stale or mismatched ARQ retransmission".to_string(),
            ));
        }

        let TxPhase::AwaitingAck {
            next_due_ms,
            retries,
        } = &mut pending.phase
        else {
            return Err(SdrError::Protocol(
                "initial ARQ transmission has not been committed".to_string(),
            ));
        };
        if now_ms < *next_due_ms {
            return Err(SdrError::Protocol(
                "ARQ retransmission committed before T1 expiry".to_string(),
            ));
        }
        if *retries >= self.max_retries {
            return Err(SdrError::Protocol(
                "ARQ retransmission committed after retry exhaustion".to_string(),
            ));
        }

        *retries += 1;
        *next_due_ms = now_ms.saturating_add(
            self.t1_ms
                .saturating_mul((*retries as u64).saturating_add(1)),
        );
        Ok(())
    }

    /// Abandon an exhausted frame once its next committed deadline is due.
    pub fn abandon_if_exhausted(&mut self, now_ms: u64) -> Option<u16> {
        let should_abandon = self.pending_tx.as_ref().is_some_and(|pending| {
            matches!(
                pending.phase,
                TxPhase::AwaitingAck {
                    next_due_ms,
                    retries,
                } if now_ms >= next_due_ms && retries >= self.max_retries
            )
        });
        if !should_abandon {
            return None;
        }

        let seq = self.pending_tx.take()?.transmission.seq;
        self.abandoned.push(seq);
        Some(seq)
    }

    /// Sequence numbers abandoned after exceeding `max_retries`, draining them.
    ///
    /// A permanent delivery failure is surfaced rather than swallowed: the
    /// caller decides whether to tear down, report, or retry at a higher level.
    pub fn take_abandoned(&mut self) -> Vec<u16> {
        std::mem::take(&mut self.abandoned)
    }

    /// Number of frames occupying the stop-and-wait transmit slot.
    pub fn unacked_len(&self) -> usize {
        usize::from(self.pending_tx.is_some())
    }

    /// Frame an untracked, fire-and-forget data packet for transmission.
    ///
    /// This does not occupy the reliable stop-and-wait slot. Reliable callers
    /// must use [`prepare_data`](Self::prepare_data).
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
    pub fn process_rx_frame(&mut self, frame_bytes: &[u8]) -> Result<ProcessedFrame> {
        let pkt = PacketFramer::decode(frame_bytes)?;

        // A point-to-point ARQ session is pinned to one configured peer. Header
        // addresses are not cryptographic authentication, but frames from a
        // different station must never mutate sequence or acknowledgement state.
        if pkt.src_addr != self.remote_addr {
            return Ok((None, None));
        }

        match pkt.packet_type {
            PacketType::Data => {
                // Data may be addressed directly or broadcast by the configured
                // peer; its acknowledgement is always unicast back to that peer.
                if pkt.dst_addr != self.local_addr && pkt.dst_addr != 0xFF {
                    return Ok((None, None));
                }
                if pkt.seq_num == self.expected_rx_seq {
                    let ack_frame = self.create_ack_frame(pkt.seq_num);
                    self.last_delivered_seq = Some(pkt.seq_num);
                    self.expected_rx_seq = self.expected_rx_seq.wrapping_add(1);
                    Ok((Some(pkt.payload), Some(ack_frame)))
                } else if self.last_delivered_seq == Some(pkt.seq_num) {
                    // The exact last delivery can legitimately repeat when its
                    // ACK was lost. Re-ACK it without delivering twice.
                    Ok((None, Some(self.create_ack_frame(pkt.seq_num))))
                } else {
                    // A future/stale sequence is not a duplicate. ACKing it
                    // would let the sender discard data we never delivered.
                    Ok((None, None))
                }
            }
            PacketType::Ack => {
                // A broadcast ACK is ambiguous and must not clear point-to-point
                // state, even when it carries the expected sequence number.
                if pkt.dst_addr != self.local_addr {
                    return Ok((None, None));
                }
                let clears_pending = self.pending_tx.as_ref().is_some_and(|pending| {
                    pending.transmission.seq == pkt.seq_num
                        && matches!(pending.phase, TxPhase::AwaitingAck { .. })
                });
                if clears_pending {
                    self.pending_tx = None;
                }
                Ok((None, None))
            }
            _ => {
                if pkt.dst_addr != self.local_addr && pkt.dst_addr != 0xFF {
                    return Ok((None, None));
                }
                Ok((None, None))
            }
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
    const C: u8 = 0x03;

    /// Two transceivers addressed at each other.
    fn pair() -> (ArqTransceiver, ArqTransceiver) {
        (ArqTransceiver::new(A, B), ArqTransceiver::new(B, A))
    }

    fn data(src: u8, dst: u8, seq: u16, payload: &[u8]) -> Vec<u8> {
        PacketFramer::encode(src, dst, seq, PacketType::Data, payload)
    }

    fn ack(src: u8, dst: u8, seq: u16) -> Vec<u8> {
        PacketFramer::encode(src, dst, seq, PacketType::Ack, &[])
    }

    #[test]
    fn test_arq_retransmits_after_timeout() {
        let (mut tx, _rx) = pair();
        let (seq, _frame) = tx.send_data(b"hello", 0).unwrap();

        let due = tx
            .peek_due_retransmission(DEFAULT_T1_MS)
            .expect("an unacknowledged frame must come back due");
        assert_eq!(due.sequence(), seq);
    }

    #[test]
    fn test_arq_no_retransmission_before_timeout() {
        let (mut tx, _rx) = pair();
        tx.send_data(b"hello", 0).unwrap();

        // The negative case. Without it, an implementation that retransmits
        // unconditionally would satisfy the timeout test vacuously.
        assert!(
            tx.peek_due_retransmission(DEFAULT_T1_MS - 1).is_none(),
            "nothing may be retransmitted before T1 expires"
        );
    }

    #[test]
    fn test_arq_ack_stops_retransmission() {
        let (mut tx, mut rx) = pair();
        let (_seq, frame) = tx.send_data(b"hello", 0).unwrap();

        // The receiver produces an ACK; feed it back to the sender.
        let (payload, ack) = rx.process_rx_frame(&frame).unwrap();
        assert_eq!(payload.as_deref(), Some(&b"hello"[..]));
        let ack = ack.expect("a data frame must be acknowledged");
        tx.process_rx_frame(&ack).unwrap();

        assert_eq!(tx.unacked_len(), 0, "the ACK must clear the retained frame");
        assert!(
            tx.peek_due_retransmission(DEFAULT_T1_MS * 100).is_none(),
            "an acknowledged frame must never be retransmitted, however long we wait"
        );
    }

    #[test]
    fn test_arq_gives_up_after_max_retries() {
        let (mut tx, _rx) = pair();
        tx.max_retries = 3;
        tx.send_data(b"unreachable", 0).unwrap();

        // Advance well past each successive backoff deadline.
        let mut now = 0u64;
        let mut attempts = 0;
        for _ in 0..20 {
            now += DEFAULT_T1_MS * 10;
            if let Some(retry) = tx.peek_due_retransmission(now) {
                tx.commit_retransmission(&retry, now).unwrap();
                attempts += 1;
            }
            tx.abandon_if_exhausted(now);
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
    fn test_arq_duplicate_last_delivered_is_acked_without_redelivery() {
        let (mut tx, mut rx) = pair();
        let (_seq, frame) = tx.send_data(b"once", 0).unwrap();

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

    #[test]
    fn test_arq_rejects_second_outstanding_send_without_advancing_sequence() {
        let (mut tx, _rx) = pair();
        let (seq, _frame) = tx.send_data(b"first", 0).unwrap();
        assert_eq!(seq, 1);
        assert_eq!(tx.next_tx_seq, 2);

        let error = tx.send_data(b"second", 0).unwrap_err();
        assert!(error.to_string().contains("busy"));
        assert_eq!(tx.next_tx_seq, 2, "a refused send consumes no sequence");
        assert_eq!(tx.unacked_len(), 1, "the first frame remains outstanding");
    }

    #[test]
    fn test_arq_prepare_abort_and_commit_are_transactional() {
        let (mut tx, _rx) = pair();
        let prepared = tx.prepare_data(b"first attempt").unwrap();
        assert_eq!(prepared.sequence(), 1);
        assert_eq!(tx.next_tx_seq, 1, "preparation does not commit a sequence");

        tx.abort_initial(&prepared).unwrap();
        assert_eq!(tx.next_tx_seq, 1, "an aborted write consumes no sequence");
        assert_eq!(tx.unacked_len(), 0);

        let retry = tx.prepare_data(b"second attempt").unwrap();
        assert_eq!(retry.sequence(), 1);
        tx.commit_initial(&retry, 10).unwrap();
        assert_eq!(tx.next_tx_seq, 2);
        assert_eq!(tx.unacked_len(), 1);
    }

    #[test]
    fn test_arq_reordered_future_frame_is_not_acked_or_delivered() {
        let (_tx, mut rx) = pair();
        let second = data(A, B, 2, b"second");
        let first = data(A, B, 1, b"first");

        let (payload, response) = rx.process_rx_frame(&second).unwrap();
        assert!(payload.is_none(), "a future payload must not be delivered");
        assert!(response.is_none(), "a future payload must not be ACKed");
        assert_eq!(rx.expected_rx_seq, 1);

        let (payload, response) = rx.process_rx_frame(&first).unwrap();
        assert_eq!(payload.as_deref(), Some(&b"first"[..]));
        assert!(response.is_some());

        let (payload, response) = rx.process_rx_frame(&second).unwrap();
        assert_eq!(payload.as_deref(), Some(&b"second"[..]));
        assert!(response.is_some());
        assert_eq!(rx.expected_rx_seq, 3);
    }

    #[test]
    fn test_arq_wrong_source_data_is_ignored_without_state_change() {
        let (_tx, mut rx) = pair();
        let wrong_peer = data(C, B, 1, b"spoofed");

        let (payload, response) = rx.process_rx_frame(&wrong_peer).unwrap();
        assert!(payload.is_none());
        assert!(response.is_none());
        assert_eq!(rx.expected_rx_seq, 1);
        assert_eq!(rx.last_delivered_seq, None);

        let correct_peer = data(A, B, 1, b"expected");
        let (payload, response) = rx.process_rx_frame(&correct_peer).unwrap();
        assert_eq!(payload.as_deref(), Some(&b"expected"[..]));
        assert!(response.is_some());
    }

    #[test]
    fn test_arq_wrong_destination_data_is_ignored_without_state_change() {
        let (_tx, mut rx) = pair();
        let wrong_destination = data(A, C, 1, b"not for this station");

        let (payload, response) = rx.process_rx_frame(&wrong_destination).unwrap();
        assert!(payload.is_none());
        assert!(response.is_none());
        assert_eq!(rx.expected_rx_seq, 1);
        assert_eq!(rx.last_delivered_seq, None);
    }

    #[test]
    fn test_arq_wrong_source_ack_cannot_clear_outstanding_frame() {
        let (mut tx, _rx) = pair();
        let (seq, _frame) = tx.send_data(b"pending", 0).unwrap();

        tx.process_rx_frame(&ack(C, A, seq)).unwrap();
        assert_eq!(tx.unacked_len(), 1, "wrong-source ACK changed TX state");

        tx.process_rx_frame(&ack(B, 0xFF, seq)).unwrap();
        assert_eq!(tx.unacked_len(), 1, "broadcast ACK changed TX state");

        tx.process_rx_frame(&ack(B, A, seq)).unwrap();
        assert_eq!(tx.unacked_len(), 0, "the configured peer ACK must clear it");
    }

    #[test]
    fn test_arq_retry_state_changes_only_after_commit() {
        let (mut tx, _rx) = pair();
        tx.max_retries = 1;
        tx.send_data(b"retry", 0).unwrap();

        let first_peek = tx.peek_due_retransmission(DEFAULT_T1_MS).unwrap();
        let second_peek = tx.peek_due_retransmission(DEFAULT_T1_MS).unwrap();
        assert_eq!(
            first_peek, second_peek,
            "peeking must not mutate retry state"
        );
        let TxPhase::AwaitingAck {
            next_due_ms,
            retries,
        } = tx.pending_tx.as_ref().unwrap().phase
        else {
            panic!("initial send should be awaiting its ACK");
        };
        assert_eq!(next_due_ms, DEFAULT_T1_MS);
        assert_eq!(retries, 0);

        tx.commit_retransmission(&first_peek, DEFAULT_T1_MS)
            .unwrap();
        let TxPhase::AwaitingAck {
            next_due_ms,
            retries,
        } = tx.pending_tx.as_ref().unwrap().phase
        else {
            panic!("committed retry should still await its ACK");
        };
        assert_eq!(next_due_ms, DEFAULT_T1_MS * 3);
        assert_eq!(retries, 1);

        assert_eq!(tx.abandon_if_exhausted(DEFAULT_T1_MS * 3 - 1), None);
        assert_eq!(tx.unacked_len(), 1);
        assert_eq!(
            tx.abandon_if_exhausted(DEFAULT_T1_MS * 3),
            Some(first_peek.sequence())
        );
        assert_eq!(tx.unacked_len(), 0);
        assert_eq!(tx.take_abandoned(), vec![first_peek.sequence()]);
    }

    #[test]
    fn test_arq_sequence_wrap_preserves_duplicate_classification() {
        let (_tx, mut rx) = pair();
        rx.expected_rx_seq = u16::MAX;

        let last = data(A, B, u16::MAX, b"last");
        let wrapped = data(A, B, 0, b"wrapped");
        let future = data(A, B, 1, b"future");

        let (payload, ack) = rx.process_rx_frame(&last).unwrap();
        assert_eq!(payload.as_deref(), Some(&b"last"[..]));
        assert!(ack.is_some());
        assert_eq!(rx.expected_rx_seq, 0);

        let (payload, ack) = rx.process_rx_frame(&last).unwrap();
        assert!(payload.is_none());
        assert!(ack.is_some(), "the exact last sequence remains a duplicate");

        let (payload, ack) = rx.process_rx_frame(&future).unwrap();
        assert!(payload.is_none());
        assert!(ack.is_none(), "a post-wrap future frame is not a duplicate");

        let (payload, ack) = rx.process_rx_frame(&wrapped).unwrap();
        assert_eq!(payload.as_deref(), Some(&b"wrapped"[..]));
        assert!(ack.is_some());
        assert_eq!(rx.expected_rx_seq, 1);
    }
}
