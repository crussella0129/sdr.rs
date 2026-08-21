//! Bidirectional Stream Tunneling Bridge for "SSH over Radio".

use crate::packet::ArqTransceiver;
use sdr_core::traits::Result;

/// Stream Tunnel for piping bidirectional byte streams (such as SSH ProxyCommand stdin/stdout)
/// over packet radio links with ARQ retransmissions.
pub struct StreamTunnel {
    pub transceiver: ArqTransceiver,
    pub mtu: usize,
    rx_buffer: Vec<u8>,
}

impl StreamTunnel {
    pub fn new(local_addr: u8, remote_addr: u8, mtu: usize) -> Self {
        Self {
            transceiver: ArqTransceiver::new(local_addr, remote_addr),
            mtu: mtu.clamp(32, 1024),
            rx_buffer: Vec::new(),
        }
    }

    /// Slice an incoming raw data byte stream into framed RF packet bursts.
    pub fn packetize(&mut self, stream_data: &[u8]) -> Vec<Vec<u8>> {
        let mut frames = Vec::new();
        for chunk in stream_data.chunks(self.mtu) {
            let (_seq, frame) = self.transceiver.create_data_frame(chunk);
            frames.push(frame);
        }
        frames
    }

    /// Ingest received RF frame bytes, verify CRC-32, manage ARQ state, and append valid payload to stream.
    /// Returns optional ACK frame bytes to transmit back to peer.
    pub fn ingest_frame(&mut self, frame_bytes: &[u8]) -> Result<Option<Vec<u8>>> {
        let (payload_opt, ack_opt) = self.transceiver.process_rx_frame(frame_bytes)?;
        if let Some(payload) = payload_opt {
            self.rx_buffer.extend(payload);
        }
        Ok(ack_opt)
    }

    /// Drain accumulated stream bytes ready for consumption by terminal or SSH process.
    pub fn drain_received_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.rx_buffer)
    }

    /// Check if received data is waiting in stream buffer.
    pub fn has_available_data(&self) -> bool {
        !self.rx_buffer.is_empty()
    }
}
