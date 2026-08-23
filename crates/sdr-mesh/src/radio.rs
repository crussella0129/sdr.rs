//! A radio-backed [`MeshInterface`]: mesh datagrams carried as modulated IQ
//! through any [`SdrDriver`].
//!
//! The transmit path is
//! `datagram → KISS → ARQ frame → FSK modulate → driver.write_samples`, and the
//! receive path reverses it:
//! `driver.read_samples → FSK demodulate → bit sync → deframe → KISS → datagram`.
//!
//! # Sample-phase search
//!
//! [`FskDemod`] has no symbol-timing recovery: it counts samples, so a stream
//! that begins mid-symbol demodulates to garbage. A receiver reading from a
//! cyclic transmit buffer starts at an arbitrary point, so `recv_datagram`
//! demodulates at each candidate sample phase and keeps the one whose frame
//! passes CRC-32. That check makes the search *self-verifying* rather than a
//! guess — but it works because a digital loopback shares one clock between
//! transmitter and receiver. A real over-the-air link between two radios has
//! clock drift and needs proper symbol-timing recovery
//! (`sdr_dsp::clock_recovery`), which is not wired up here.

use std::collections::VecDeque;

use sdr_core::sample::Complex32;
use sdr_core::traits::Result;
use sdr_demod::modulator::FskModulator;
use sdr_demod::FskDemod;
use sdr_hardware::driver::SdrDriver;
use sdr_protocols::packet::{ArqTransceiver, PacketFramer};

use crate::framesync::sync_to_frame;
use crate::kiss::{encode, KissDecoder};
use crate::node::MeshInterface;

/// Address meaning "any station" — accepted by every receiver.
pub const BROADCAST_ADDR: u8 = 0xFF;

/// Modulation parameters for the radio link. Both ends must agree.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadioParams {
    /// Baseband sample rate in Hz.
    pub sample_rate: f32,
    /// FSK frequency deviation in Hz.
    pub deviation_hz: f32,
    /// Samples per transmitted symbol (one symbol per bit).
    pub samples_per_symbol: usize,
}

impl Default for RadioParams {
    fn default() -> Self {
        Self {
            sample_rate: 1.0e6,
            deviation_hz: 100.0e3,
            samples_per_symbol: 10,
        }
    }
}

/// Carries mesh datagrams over a radio via FSK.
pub struct RadioLink<D: SdrDriver> {
    driver: D,
    arq: ArqTransceiver,
    decoder: KissDecoder,
    params: RadioParams,
    inbox: VecDeque<Vec<u8>>,
    /// Samples requested per `recv_datagram` call.
    rx_chunk: usize,
}

impl<D: SdrDriver> RadioLink<D> {
    /// Create a link. Use [`BROADCAST_ADDR`] as `remote_addr` when transmitter
    /// and receiver are the same station (loopback), so frames are not filtered
    /// out by destination.
    pub fn new(driver: D, local_addr: u8, remote_addr: u8, params: RadioParams) -> Self {
        Self {
            driver,
            arq: ArqTransceiver::new(local_addr, remote_addr),
            decoder: KissDecoder::new(),
            params,
            inbox: VecDeque::new(),
            rx_chunk: 16_384,
        }
    }

    /// Number of samples read per `recv_datagram` call. Should comfortably
    /// exceed twice the modulated frame length so a complete frame is captured
    /// from a cyclically repeating buffer.
    pub fn set_rx_chunk(&mut self, samples: usize) {
        self.rx_chunk = samples.max(1);
    }

    /// Borrow the underlying driver (to configure or start it).
    pub fn driver_mut(&mut self) -> &mut D {
        &mut self.driver
    }

    /// Borrow the underlying driver.
    pub fn driver(&self) -> &D {
        &self.driver
    }

    /// Modulate a framed packet into baseband IQ.
    fn modulate(&self, frame: &[u8]) -> Vec<Complex32> {
        let mut m = FskModulator::new(
            self.params.sample_rate,
            self.params.deviation_hz,
            self.params.samples_per_symbol,
        );
        let mut iq = Vec::with_capacity(frame.len() * 8 * self.params.samples_per_symbol);
        m.modulate_bytes(frame, &mut iq);
        iq
    }

    /// Try to recover framed payloads from received IQ, searching sample phases.
    ///
    /// Returns the payload bytes of the first frame that passes CRC-32.
    fn extract_payload(&mut self, iq: &[Complex32]) -> Option<Vec<u8>> {
        let sps = self.params.samples_per_symbol;
        for phase in 0..sps {
            if phase >= iq.len() {
                break;
            }
            let mut demod = FskDemod::new(self.params.sample_rate, self.params.deviation_hz, sps);
            let mut bits = Vec::new();
            demod.demod_bits(&iq[phase..], &mut bits);

            let Some(frame) = sync_to_frame(&bits) else {
                continue;
            };
            // CRC-32 inside `decode` is what confirms this phase is the right one.
            if let Ok(pkt) = PacketFramer::decode(&frame) {
                if pkt.dst_addr == self.arq.local_addr || pkt.dst_addr == BROADCAST_ADDR {
                    log::debug!(
                        "RadioLink: recovered frame seq {} at sample phase {phase}",
                        pkt.seq_num
                    );
                    return Some(pkt.payload);
                }
            }
        }
        None
    }
}

impl<D: SdrDriver> MeshInterface for RadioLink<D> {
    fn send_datagram(&mut self, datagram: &[u8]) -> Result<()> {
        let framed = encode(datagram);
        let (_seq, frame) = self.arq.create_data_frame(&framed);
        let iq = self.modulate(&frame);
        self.driver.write_samples(&iq)?;
        Ok(())
    }

    fn recv_datagram(&mut self) -> Result<Option<Vec<u8>>> {
        if let Some(d) = self.inbox.pop_front() {
            return Ok(Some(d));
        }
        let mut buf = vec![Complex32::default(); self.rx_chunk];
        let n = self.driver.read_samples(&mut buf)?;
        if n == 0 {
            return Ok(None);
        }
        buf.truncate(n);

        let Some(payload) = self.extract_payload(&buf) else {
            return Ok(None);
        };
        for datagram in self.decoder.push(&payload) {
            self.inbox.push_back(datagram);
        }
        Ok(self.inbox.pop_front())
    }
}
