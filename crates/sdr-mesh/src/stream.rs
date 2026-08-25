//! Byte stream ⇄ datagrams.
//!
//! A [`MeshInterface`] moves whole datagrams; SSH and other TCP services want a
//! continuous byte stream. [`StreamBridge`] sits between them: outbound bytes
//! are split into MTU-sized datagrams, and inbound datagrams are reassembled
//! into an ordered byte stream.
//!
//! It is pure and I/O-free — it never touches stdin, stdout or a socket — so it
//! is testable without a radio and works over `MockSdr` and `PlutoSdr` alike.
//! Framing, modulation and CRC stay the interface's responsibility.
//!
//! # Reliability
//!
//! Ordering is preserved because datagrams arrive in the order sent, but there
//! is **no retransmission**: the receive path validates CRC-32 and drops a
//! corrupt frame, so on a lossy channel a stream would silently lose bytes.
//! That is invisible over a lossless loopback and is a real gap for a real link.

use std::collections::VecDeque;

use sdr_core::traits::Result;

use crate::node::MeshInterface;

/// Default bytes of stream payload per datagram.
///
/// Sized so a whole frame — KISS escaping worst case (every byte doubled), the
/// packet header, CRC and flush trailer — fits comfortably inside one capture at
/// the default `rx_chunk`.
pub const DEFAULT_MTU: usize = 128;

/// Maximum datagrams drained per [`StreamBridge::pump`] call.
///
/// Bounded deliberately. A transmitter using a **cyclic** buffer repeats its
/// frame for as long as the buffer is open, so an unbounded drain would never
/// return — it would keep recovering the same frame forever.
pub const PUMP_MAX_DATAGRAMS: usize = 64;

/// Carries a byte stream over a datagram [`MeshInterface`].
pub struct StreamBridge<I: MeshInterface> {
    iface: I,
    mtu: usize,
    inbound: VecDeque<u8>,
}

impl<I: MeshInterface> StreamBridge<I> {
    /// Bridge with [`DEFAULT_MTU`].
    pub fn new(iface: I) -> Self {
        Self::with_mtu(iface, DEFAULT_MTU)
    }

    /// Bridge with an explicit MTU (bytes of stream payload per datagram).
    pub fn with_mtu(iface: I, mtu: usize) -> Self {
        Self {
            iface,
            mtu: mtu.max(1),
            inbound: VecDeque::new(),
        }
    }

    /// Bytes of stream payload carried per datagram.
    pub fn mtu(&self) -> usize {
        self.mtu
    }

    /// Borrow the underlying interface (to configure the driver beneath it).
    pub fn iface_mut(&mut self) -> &mut I {
        &mut self.iface
    }

    /// Split `data` into MTU-sized datagrams and send them in order.
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        for chunk in data.chunks(self.mtu) {
            self.iface.send_datagram(chunk)?;
        }
        Ok(())
    }

    /// Pull waiting datagrams into the inbound stream buffer.
    ///
    /// Returns the number of bytes added. Does not block: when nothing has
    /// arrived it returns `Ok(0)`. Drains at most [`PUMP_MAX_DATAGRAMS`] per
    /// call so a cyclically repeating transmitter cannot trap the caller in an
    /// endless drain.
    pub fn pump(&mut self) -> Result<usize> {
        let mut added = 0;
        for _ in 0..PUMP_MAX_DATAGRAMS {
            match self.iface.recv_datagram()? {
                Some(datagram) => {
                    added += datagram.len();
                    self.inbound.extend(datagram);
                }
                None => break,
            }
        }
        Ok(added)
    }

    /// Read up to `max` bytes of the reassembled stream, pumping first.
    ///
    /// Yields an empty vector rather than blocking when nothing is available.
    pub fn read(&mut self, max: usize) -> Result<Vec<u8>> {
        self.pump()?;
        let n = max.min(self.inbound.len());
        Ok(self.inbound.drain(..n).collect())
    }

    /// Bytes currently buffered and ready to read.
    pub fn available(&self) -> usize {
        self.inbound.len()
    }

    /// Drop every buffered inbound byte without reading it.
    ///
    /// Needed when the transmitter repeats a frame from a cyclic buffer: the
    /// same datagram is recovered over and over, and those repeats must be
    /// discarded rather than appended to the stream. Discards data, so it
    /// belongs to link setup and test harnesses, not to a running session.
    pub fn clear_inbound(&mut self) {
        self.inbound.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Records what was sent and replays a queue of datagrams — no radio.
    #[derive(Default)]
    struct FakeIface {
        sent: Vec<Vec<u8>>,
        to_receive: VecDeque<Vec<u8>>,
    }

    impl MeshInterface for FakeIface {
        fn send_datagram(&mut self, datagram: &[u8]) -> Result<()> {
            self.sent.push(datagram.to_vec());
            Ok(())
        }
        fn recv_datagram(&mut self) -> Result<Option<Vec<u8>>> {
            Ok(self.to_receive.pop_front())
        }
    }

    #[test]
    fn test_stream_bridge_chunks_to_mtu() {
        let mut bridge = StreamBridge::with_mtu(FakeIface::default(), 16);
        let data: Vec<u8> = (0..40u8).collect();
        bridge.write(&data).unwrap();

        let sent = &bridge.iface_mut().sent;
        assert_eq!(sent.len(), 3, "40 bytes at MTU 16 is 16 + 16 + 8");
        assert!(
            sent.iter().all(|d| d.len() <= 16),
            "no datagram may exceed the MTU"
        );
        // Concatenating the datagrams reproduces the stream, in order.
        let flat: Vec<u8> = sent.iter().flatten().copied().collect();
        assert_eq!(flat, data);
    }

    #[test]
    fn test_stream_bridge_binary_safe() {
        // Every byte value, including KISS-significant 0xC0/0xDB and NUL.
        let data: Vec<u8> = (0..=255u8).collect();
        let mut iface = FakeIface::default();
        // Feed the same bytes back in MTU-sized pieces.
        for chunk in data.chunks(32) {
            iface.to_receive.push_back(chunk.to_vec());
        }
        let mut bridge = StreamBridge::with_mtu(iface, 32);

        let got = bridge.read(1024).unwrap();
        assert_eq!(got, data, "all 256 byte values must survive unchanged");
    }

    #[test]
    fn test_stream_bridge_empty_read() {
        let mut bridge = StreamBridge::new(FakeIface::default());
        assert_eq!(bridge.available(), 0);
        let got = bridge.read(64).unwrap();
        assert!(got.is_empty(), "an empty read yields no bytes and no error");
    }

    #[test]
    fn test_stream_bridge_read_respects_max() {
        let mut iface = FakeIface::default();
        iface.to_receive.push_back(vec![1, 2, 3, 4, 5, 6]);
        let mut bridge = StreamBridge::new(iface);

        assert_eq!(bridge.read(4).unwrap(), vec![1, 2, 3, 4]);
        assert_eq!(bridge.available(), 2, "the remainder stays buffered");
        assert_eq!(bridge.read(64).unwrap(), vec![5, 6]);
    }
}
