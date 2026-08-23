//! Mesh interface seam, an in-memory loopback link, and a compliance-gated node.
//!
//! [`MeshInterface`] is the datagram-oriented seam a real OS `tun` device plugs
//! into during Phase B. [`LoopbackLink`] is the CI-verifiable in-memory
//! implementation: it carries KISS-framed datagrams through the realized
//! `ArqTransceiver` link layer (`sdr-protocols`). [`MeshNode`] ties an interface
//! to the [`MeshPolicy`] gate so every send is checked for legality first.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::kiss::{encode, KissDecoder};
use crate::policy::{Decision, MeshPolicy};
use sdr_core::compliance::Jurisdiction;
use sdr_core::traits::{Result, SdrError};
use sdr_protocols::packet::ArqTransceiver;

/// A datagram-oriented mesh interface. Implementations move whole IP datagrams;
/// framing/reassembly and the radio link are their responsibility.
pub trait MeshInterface {
    /// Queue a datagram for transmission.
    fn send_datagram(&mut self, datagram: &[u8]) -> Result<()>;
    /// Return the next fully received datagram, or `None` if none is ready.
    fn recv_datagram(&mut self) -> Result<Option<Vec<u8>>>;
}

type FrameQueue = Rc<RefCell<VecDeque<Vec<u8>>>>;

/// One endpoint of a deterministic, in-memory two-node link. Datagrams are
/// KISS-framed, chunked to the MTU, and carried as `ArqTransceiver` frames;
/// inbound frames are ARQ-decoded and KISS-reassembled.
pub struct LoopbackLink {
    tx: FrameQueue,
    rx: FrameQueue,
    arq: ArqTransceiver,
    decoder: KissDecoder,
    inbox: VecDeque<Vec<u8>>,
    mtu: usize,
}

impl LoopbackLink {
    /// Create a connected pair of endpoints with the given radio addresses.
    pub fn pair(addr_a: u8, addr_b: u8) -> (LoopbackLink, LoopbackLink) {
        let ab: FrameQueue = Rc::new(RefCell::new(VecDeque::new()));
        let ba: FrameQueue = Rc::new(RefCell::new(VecDeque::new()));
        let a = LoopbackLink {
            tx: Rc::clone(&ab),
            rx: Rc::clone(&ba),
            arq: ArqTransceiver::new(addr_a, addr_b),
            decoder: KissDecoder::new(),
            inbox: VecDeque::new(),
            mtu: 256,
        };
        let b = LoopbackLink {
            tx: ba,
            rx: ab,
            arq: ArqTransceiver::new(addr_b, addr_a),
            decoder: KissDecoder::new(),
            inbox: VecDeque::new(),
            mtu: 256,
        };
        (a, b)
    }

    /// Drain inbound frames, decode ARQ payloads, and KISS-reassemble datagrams.
    fn pump(&mut self) -> Result<()> {
        loop {
            let frame = self.rx.borrow_mut().pop_front();
            let Some(frame) = frame else { break };
            let (payload, _ack) = self.arq.process_rx_frame(&frame)?;
            if let Some(bytes) = payload {
                for datagram in self.decoder.push(&bytes) {
                    self.inbox.push_back(datagram);
                }
            }
        }
        Ok(())
    }
}

impl MeshInterface for LoopbackLink {
    fn send_datagram(&mut self, datagram: &[u8]) -> Result<()> {
        let framed = encode(datagram);
        for chunk in framed.chunks(self.mtu) {
            let (_seq, frame) = self.arq.create_data_frame(chunk);
            self.tx.borrow_mut().push_back(frame);
        }
        Ok(())
    }

    fn recv_datagram(&mut self) -> Result<Option<Vec<u8>>> {
        self.pump()?;
        Ok(self.inbox.pop_front())
    }
}

/// A mesh node: a [`MeshInterface`] plus the frequency/jurisdiction context that
/// the compliance gate needs. Every [`MeshNode::send`] consults [`MeshPolicy`]
/// before a frame is emitted.
pub struct MeshNode<I: MeshInterface> {
    iface: I,
    jurisdiction: Jurisdiction,
    freq_hz: u64,
    power_dbm: f32,
}

impl<I: MeshInterface> MeshNode<I> {
    /// Create a node bound to a transmit frequency, jurisdiction, and power.
    pub fn new(iface: I, jurisdiction: Jurisdiction, freq_hz: u64, power_dbm: f32) -> Self {
        Self {
            iface,
            jurisdiction,
            freq_hz,
            power_dbm,
        }
    }

    /// Send a datagram after checking transmission legality. When the gate
    /// refuses, no frame is emitted and the regulator's reasons are returned.
    pub fn send(&mut self, datagram: &[u8], want_encrypted: bool) -> Result<()> {
        match MeshPolicy::evaluate(
            self.jurisdiction,
            self.freq_hz,
            self.power_dbm,
            want_encrypted,
        ) {
            Decision::Allow { .. } => self.iface.send_datagram(datagram),
            Decision::Refuse { reasons } => {
                log::warn!("mesh TX refused by compliance gate: {}", reasons.join("; "));
                Err(SdrError::Config(format!(
                    "mesh transmission refused by compliance gate: {}",
                    reasons.join("; ")
                )))
            }
        }
    }

    /// Receive the next datagram, if any is ready.
    pub fn recv(&mut self) -> Result<Option<Vec<u8>>> {
        self.iface.recv_datagram()
    }
}
