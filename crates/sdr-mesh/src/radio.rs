//! A radio-backed [`MeshInterface`]: mesh datagrams carried as modulated IQ
//! through any [`SdrDriver`].
//!
//! The transmit path is
//! `datagram → KISS → ARQ frame → FSK modulate → driver.write_samples`, and the
//! receive path reverses it:
//! `driver.read_samples → FSK demodulate → bit sync → deframe → KISS → datagram`.
//!
//! # Symbol-timing recovery
//!
//! The receiver uses [`FskTimingDemod`], which runs a Gardner timing-recovery
//! loop over the frequency-discriminator output. Because the loop tracks the
//! symbol clock rather than counting samples, a single demodulation pass
//! handles both an arbitrary start phase (a receiver reading a cyclic transmit
//! buffer begins at an arbitrary point) and a transmitter/receiver clock
//! offset. Whole frames survive a clock offset of about **±0.1%** — every bit
//! must be right for CRC-32 to pass, so the frame-level figure is tighter than
//! the bare demodulator's short-burst tolerance. That is still one to two
//! orders of magnitude beyond the ±10–50 ppm of a real crystal oscillator.
//!
//! Frame alignment and validation still come from the sync-word search plus
//! CRC-32, which also absorb the symbol of start-up lag the loop may introduce.

use std::collections::VecDeque;

use sdr_core::sample::Complex32;
use sdr_core::traits::Result;
use sdr_demod::modulator::FskModulator;
use sdr_demod::FskTimingDemod;
use sdr_hardware::driver::SdrDriver;
use sdr_protocols::packet::{ArqTransceiver, PacketFramer, SYNC_WORD};

use crate::framesync::{bits_to_bytes, find_sync};
use crate::kiss::{encode, KissDecoder};
use crate::node::MeshInterface;

/// Address meaning "any station" — accepted by every receiver.
pub const BROADCAST_ADDR: u8 = 0xFF;

/// Flush bytes appended after each frame.
///
/// A timing-recovery loop consumes a symbol or so settling at the start of a
/// burst, which shifts its output stream; without trailing symbols to push the
/// last data symbols through, the frame's final byte (its CRC) is lost. The
/// alternating `0xAA` pattern also keeps the loop supplied with transitions
/// while it flushes. Trailing bytes are harmless to the receiver: the frame
/// header is length-prefixed, so the decoder ignores anything past the CRC.
const TRAILER: [u8; 2] = [0xAA, 0xAA];

/// Frame header bytes between the sync word and the payload:
/// src, dst, 2-byte sequence, type, 2-byte payload length.
const FRAME_HEADER_LEN: usize = 7;
/// Trailing CRC-32 bytes.
const FRAME_CRC_LEN: usize = 4;

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
            rx_chunk: 65_536,
        }
    }

    /// Number of samples read per `recv_datagram` call.
    ///
    /// A frame occupies `frame_bytes * 8 * samples_per_symbol` samples, so this
    /// is also the ceiling on frame size: a frame larger than one capture cannot
    /// be recovered. It should comfortably exceed twice the largest frame — both
    /// to clear that ceiling and so a complete frame is captured from a
    /// cyclically repeating buffer.
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

    /// Recover **every** framed payload present in a capture.
    ///
    /// A single demodulation pass: the timing-recovery loop locks to the symbol
    /// clock, so no candidate-phase search is needed. The resulting bit stream
    /// is then scanned repeatedly — a capture can hold several back-to-back
    /// frames, and a byte stream produces exactly that, so stopping at the first
    /// would silently drop the rest.
    fn extract_payloads(&mut self, iq: &[Complex32]) -> Vec<Vec<u8>> {
        let mut demod = FskTimingDemod::with_defaults(self.params.samples_per_symbol as f32);
        let mut bits = Vec::new();
        demod.demod_bits(iq, &mut bits);

        let mut payloads = Vec::new();
        let mut at = 0usize;
        while let Some(idx) = find_sync(&bits, at) {
            let frame = bits_to_bytes(&bits[idx..]);
            match PacketFramer::decode(&frame) {
                Ok(pkt) => {
                    // Bytes consumed from the sync word: sync + header + payload + CRC.
                    let consumed =
                        SYNC_WORD.len() + FRAME_HEADER_LEN + pkt.payload.len() + FRAME_CRC_LEN;
                    if pkt.dst_addr == self.arq.local_addr || pkt.dst_addr == BROADCAST_ADDR {
                        log::debug!("RadioLink: recovered frame seq {}", pkt.seq_num);
                        payloads.push(pkt.payload);
                    }
                    at = idx + consumed * 8;
                }
                // Not a valid frame here (noise, a truncated tail, or the sync
                // pattern appearing inside data) — resume searching just past it.
                Err(_) => at = idx + 1,
            }
        }
        payloads
    }
}

impl<D: SdrDriver> MeshInterface for RadioLink<D> {
    fn send_datagram(&mut self, datagram: &[u8]) -> Result<()> {
        let framed = encode(datagram);
        let (_seq, mut frame) = self.arq.create_data_frame(&framed);
        // Flush symbols so the receiver's timing loop can push the frame's last
        // symbols out; see TRAILER.
        frame.extend_from_slice(&TRAILER);
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

        for payload in self.extract_payloads(&buf) {
            for datagram in self.decoder.push(&payload) {
                self.inbox.push_back(datagram);
            }
        }
        Ok(self.inbox.pop_front())
    }
}
