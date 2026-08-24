//! `RadioLink` over `MockSdr` loopback: a mesh datagram carried end-to-end as
//! modulated IQ. No radio required.

use std::collections::VecDeque;

use sdr_core::sample::Complex32;
use sdr_core::traits::{Result, SdrError};
use sdr_demod::modulator::FskModulator;
use sdr_hardware::driver::{GainMode, SdrDriver};
use sdr_hardware::mock::MockSdr;
use sdr_mesh::framesync::{bits_to_bytes, find_sync};
use sdr_mesh::kiss::encode;
use sdr_mesh::node::MeshInterface;
use sdr_mesh::{RadioLink, RadioParams, BROADCAST_ADDR};
use sdr_protocols::packet::{PacketFramer, PacketType, DEFAULT_T1_MS};

fn params() -> RadioParams {
    RadioParams {
        sample_rate: 1.0e6,
        deviation_hz: 100.0e3,
        samples_per_symbol: 10,
    }
}

#[derive(Debug, Clone, Copy)]
enum WriteOutcome {
    Full,
    Zero,
    Short(usize),
    Error,
}

/// Deterministic driver for transactional TX tests. Every attempted write is
/// captured even when the scripted result is zero, short, or an error.
struct ScriptedDriver {
    outcomes: VecDeque<WriteOutcome>,
    writes: Vec<Vec<Complex32>>,
    rx: VecDeque<Complex32>,
    active: bool,
}

impl ScriptedDriver {
    fn new(outcomes: impl IntoIterator<Item = WriteOutcome>) -> Self {
        Self {
            outcomes: outcomes.into_iter().collect(),
            writes: Vec::new(),
            rx: VecDeque::new(),
            active: true,
        }
    }

    fn inject_rx(&mut self, samples: &[Complex32]) {
        self.rx.extend(samples.iter().copied());
    }
}

impl SdrDriver for ScriptedDriver {
    fn name(&self) -> &str {
        "scripted radio test driver"
    }

    fn set_frequency(&mut self, _channel: usize, _freq_hz: f64) -> Result<()> {
        Ok(())
    }

    fn set_sample_rate(&mut self, _channel: usize, _rate_hz: f64) -> Result<()> {
        Ok(())
    }

    fn set_bandwidth(&mut self, _channel: usize, _bw_hz: f64) -> Result<()> {
        Ok(())
    }

    fn set_gain(&mut self, _channel: usize, _gain_db: f64) -> Result<()> {
        Ok(())
    }

    fn set_gain_mode(&mut self, _channel: usize, _mode: GainMode) -> Result<()> {
        Ok(())
    }

    fn start_rx(&mut self) -> Result<()> {
        self.active = true;
        Ok(())
    }

    fn stop_rx(&mut self) -> Result<()> {
        self.active = false;
        Ok(())
    }

    fn read_samples(&mut self, buffer: &mut [Complex32]) -> Result<usize> {
        if !self.active {
            return Ok(0);
        }
        let count = buffer.len().min(self.rx.len());
        for slot in &mut buffer[..count] {
            *slot = self
                .rx
                .pop_front()
                .expect("count is bounded by queue length");
        }
        Ok(count)
    }

    fn start_tx(&mut self) -> Result<()> {
        Ok(())
    }

    fn stop_tx(&mut self) -> Result<()> {
        Ok(())
    }

    fn write_samples(&mut self, buffer: &[Complex32]) -> Result<usize> {
        self.writes.push(buffer.to_vec());
        match self.outcomes.pop_front().unwrap_or(WriteOutcome::Full) {
            WriteOutcome::Full => Ok(buffer.len()),
            WriteOutcome::Zero => Ok(0),
            WriteOutcome::Short(count) => Ok(count.min(buffer.len().saturating_sub(1))),
            WriteOutcome::Error => Err(SdrError::Hardware("scripted TX failure".to_string())),
        }
    }

    fn has_tx(&self) -> bool {
        true
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn teardown(&mut self) -> Result<()> {
        self.active = false;
        Ok(())
    }
}

fn scripted_reliable_link(
    outcomes: impl IntoIterator<Item = WriteOutcome>,
) -> RadioLink<ScriptedDriver> {
    let mut link = RadioLink::new(ScriptedDriver::new(outcomes), 1, 2, params());
    link.set_reliable(true).unwrap();
    link
}

fn peer_datagram_iq(sequence: u16, datagram: &[u8]) -> Vec<Complex32> {
    let payload = encode(datagram);
    let mut frame = PacketFramer::encode(2, 1, sequence, PacketType::Data, &payload);
    frame.extend_from_slice(&[0xAA, 0xAA]);
    let mut modulator = FskModulator::new(
        params().sample_rate,
        params().deviation_hz,
        params().samples_per_symbol,
    );
    let mut iq = Vec::new();
    modulator.modulate_bytes(&frame, &mut iq);
    iq
}

fn decode_attempt(iq: &[Complex32]) -> sdr_protocols::DecodedPacket {
    let mut demod = sdr_demod::FskTimingDemod::with_defaults(params().samples_per_symbol as f32);
    let mut bits = Vec::new();
    demod.demod_bits(iq, &mut bits);
    let sync = find_sync(&bits, 0).expect("captured write should contain a sync word");
    PacketFramer::decode(&bits_to_bytes(&bits[sync..]))
        .expect("captured write should decode as a packet")
}

/// A `MockSdr` in loopback mode with TX and RX running.
fn loopback_link() -> RadioLink<MockSdr> {
    let mut mock = MockSdr::new(params().sample_rate as f64, 915.0e6);
    mock.enable_loopback();
    mock.start_tx().unwrap();
    mock.start_rx().unwrap();
    // Broadcast destination so a self-loopback frame is not filtered out.
    RadioLink::new(mock, 1, BROADCAST_ADDR, params())
}

#[test]
fn test_radiolink_datagram_roundtrip_over_mock() {
    let mut link = loopback_link();

    // Bytes that exercise KISS escaping (0xC0/0xDB) and a plausible IP header.
    let datagram = vec![0x45, 0x00, 0x00, 0x28, 0xC0, 0xDB, 0xAA, 0x55, 0x00, 0xFF];

    link.send_datagram(&datagram).expect("send over mock radio");

    let received = link
        .recv_datagram()
        .expect("recv over mock radio")
        .expect("a datagram should be recovered from the looped-back IQ");
    assert_eq!(
        received, datagram,
        "the datagram must survive modulate -> loopback -> demodulate byte-for-byte"
    );
}

#[test]
fn test_radiolink_recovers_from_sample_offset() {
    // The mock loopback is sample-exact, so without an injected offset the
    // receiver would always succeed at phase 0 and the sample-phase search
    // would never be exercised. Drive the pipeline manually with an offset.
    let p = params();
    let mut mock = MockSdr::new(p.sample_rate as f64, 915.0e6);
    mock.enable_loopback();
    mock.start_tx().unwrap();
    mock.start_rx().unwrap();
    let mut link = RadioLink::new(mock, 1, BROADCAST_ADDR, p);

    let datagram = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02, 0x03];
    link.send_datagram(&datagram).expect("send");

    // Pull the modulated IQ out of the mock and re-inject it shifted, so the
    // receiver starts mid-symbol.
    let mut raw = vec![Complex32::default(); 200_000];
    let n = link.driver_mut().read_samples(&mut raw).expect("drain IQ");
    raw.truncate(n);
    assert!(n > 0, "mock loopback should return the transmitted IQ");

    let offset = p.samples_per_symbol / 2 + 1; // deliberately mid-symbol
    let shifted = &raw[offset..];
    link.driver_mut().write_samples(shifted).expect("re-inject");

    let received = link
        .recv_datagram()
        .expect("recv")
        .expect("the phase search must recover a mid-symbol stream");
    assert_eq!(received, datagram);
}

/// Resample by `ratio` via linear interpolation; `ratio > 1` simulates a
/// receiver clock running fast relative to the transmitter.
fn resample(iq: &[Complex32], ratio: f32) -> Vec<Complex32> {
    let mut out = Vec::new();
    let mut pos = 0.0f32;
    while (pos as usize) + 1 < iq.len() {
        let i = pos as usize;
        let frac = pos - i as f32;
        let (a, b) = (iq[i], iq[i + 1]);
        out.push(Complex32::new(
            a.re + (b.re - a.re) * frac,
            a.im + (b.im - a.im) * frac,
        ));
        pos += ratio;
    }
    out
}

#[test]
fn test_radiolink_recovers_under_clock_drift() {
    // The case the removed sample-phase search could not handle at all: the
    // receiver's clock differs from the transmitter's, so no single sample
    // phase stays correct across the frame.
    //
    // ±0.1% is the measured whole-frame limit — tighter than the bare
    // demodulator's short-burst tolerance, because every bit must survive for
    // CRC-32 to pass. Real crystals are ±10–50 ppm, so this leaves ample margin.
    let p = params();
    let datagram = vec![0x45, 0x00, 0x11, 0x22, 0xC0, 0xDB, 0x99];

    for ratio in [1.001f32, 0.999] {
        let mut mock = MockSdr::new(p.sample_rate as f64, 915.0e6);
        mock.enable_loopback();
        mock.start_tx().unwrap();
        mock.start_rx().unwrap();
        let mut link = RadioLink::new(mock, 1, BROADCAST_ADDR, p);

        link.send_datagram(&datagram).expect("send");

        // Drain the modulated IQ, resample it to introduce the clock offset,
        // and re-inject it as what the receiver actually sees.
        let mut raw = vec![Complex32::default(); 200_000];
        let n = link.driver_mut().read_samples(&mut raw).expect("drain IQ");
        raw.truncate(n);
        assert!(n > 0, "mock loopback should return the transmitted IQ");

        let drifted = resample(&raw, ratio);
        link.driver_mut()
            .write_samples(&drifted)
            .expect("re-inject");

        let received = link
            .recv_datagram()
            .expect("recv")
            .unwrap_or_else(|| panic!("clock ratio {ratio}: datagram should still be recovered"));
        assert_eq!(
            received, datagram,
            "clock ratio {ratio}: datagram must survive byte-for-byte"
        );
    }
}

#[test]
fn test_radiolink_recovers_burst_of_datagrams() {
    // A byte stream produces back-to-back frames in one capture. Before
    // multi-frame extraction this recovered 1 of 4 — the rest were discarded
    // with the capture.
    let mut link = loopback_link();

    let sent: Vec<Vec<u8>> = (0u8..4).map(|i| vec![i; 6]).collect();
    for d in &sent {
        link.send_datagram(d).expect("send");
    }

    let mut got = Vec::new();
    for _ in 0..8 {
        match link.recv_datagram().expect("recv") {
            Some(d) => got.push(d),
            None => break,
        }
    }

    assert_eq!(
        got, sent,
        "every datagram in the capture must be recovered, in order"
    );
}

#[test]
fn test_radiolink_oversized_frame_not_truncated() {
    // A frame larger than one capture cannot be recovered. It must report
    // nothing rather than hand back a silently truncated payload.
    let p = params();
    let mut mock = MockSdr::new(p.sample_rate as f64, 915.0e6);
    mock.enable_loopback();
    mock.start_tx().unwrap();
    mock.start_rx().unwrap();
    let mut link = RadioLink::new(mock, 1, BROADCAST_ADDR, p);
    // Deliberately tiny capture so even a small frame cannot fit.
    link.set_rx_chunk(512);

    let datagram = vec![0x42u8; 256];
    link.send_datagram(&datagram).expect("send");

    let received = link.recv_datagram().expect("recv must not error");
    assert!(
        received.is_none(),
        "an oversized frame must yield None, not a truncated payload: {received:?}"
    );
}

#[test]
fn test_radiolink_noise_returns_none() {
    let mut link = loopback_link();

    // Silence contains no sync word: expect a clean miss, not an error or panic.
    let silence = vec![Complex32::new(0.0, 0.0); 4_000];
    link.driver_mut().write_samples(&silence).expect("inject");

    let received = link.recv_datagram().expect("recv must not error on noise");
    assert!(
        received.is_none(),
        "no datagram should be invented from silence, got {received:?}"
    );
}

// --- ARQ on the radio path (T-044/T-126) ----------------------------------
//
// Reliable tests use a concrete peer. Broadcast remains available to the six
// fire-and-forget MockSdr tests above, but cannot identify whose ACK may mutate
// a point-to-point stop-and-wait session.

#[test]
fn test_radiolink_acks_received_data() {
    let mut link = scripted_reliable_link([WriteOutcome::Full]);
    let iq = peer_datagram_iq(1, b"needs acknowledging");
    link.driver_mut().inject_rx(&iq);

    // Receiving queues an ACK but does not transmit as a side effect.
    let got = link.recv_datagram().unwrap();
    assert_eq!(got.as_deref(), Some(&b"needs acknowledging"[..]));
    assert!(link.driver().writes.is_empty());

    assert_eq!(link.service(0).unwrap(), 1);
    assert_eq!(link.driver().writes.len(), 1);
}

#[test]
fn test_radiolink_suppresses_duplicate_frames() {
    let mut link = scripted_reliable_link([WriteOutcome::Full, WriteOutcome::Full]);
    let iq = peer_datagram_iq(1, b"once only");

    link.driver_mut().inject_rx(&iq);
    assert_eq!(
        link.recv_datagram().unwrap().as_deref(),
        Some(&b"once only"[..])
    );

    link.driver_mut().inject_rx(&iq);
    assert!(
        link.recv_datagram().unwrap().is_none(),
        "the exact duplicate must not be delivered twice"
    );

    assert_eq!(
        link.service(0).unwrap(),
        2,
        "both the initial delivery and exact duplicate must be ACKed"
    );
}

#[test]
fn test_radiolink_retransmits_unacked_frame() {
    let mut link = scripted_reliable_link([WriteOutcome::Full, WriteOutcome::Full]);
    link.send_datagram(b"unacknowledged").unwrap();
    assert_eq!(link.unacked_len(), 1, "the frame must be retained");

    assert_eq!(
        link.service(DEFAULT_T1_MS - 1).unwrap(),
        0,
        "nothing may be retransmitted before T1 expires"
    );
    assert_eq!(link.service(DEFAULT_T1_MS).unwrap(), 1);
    assert_eq!(link.driver().writes.len(), 2);
    assert_eq!(link.driver().writes[0], link.driver().writes[1]);
}

#[test]
fn test_radiolink_stop_and_wait_blocks_second_driver_write() {
    let mut link = scripted_reliable_link([WriteOutcome::Full]);
    link.send_datagram(b"first").unwrap();
    assert_eq!(link.driver().writes.len(), 1);

    let error = link.send_datagram(b"second").unwrap_err();
    assert!(error.to_string().contains("busy"));
    assert_eq!(
        link.driver().writes.len(),
        1,
        "a busy second send must not reach the driver"
    );
    assert_eq!(link.unacked_len(), 1);
}

#[test]
fn test_radiolink_initial_write_failures_do_not_commit() {
    for failure in [
        WriteOutcome::Zero,
        WriteOutcome::Short(1),
        WriteOutcome::Error,
    ] {
        let mut link = scripted_reliable_link([failure, WriteOutcome::Full]);

        assert!(
            link.send_datagram(b"same payload").is_err(),
            "{failure:?} must be reported as an error"
        );
        assert_eq!(link.unacked_len(), 0, "{failure:?} committed ARQ state");
        assert_eq!(link.driver().writes.len(), 1);

        link.send_datagram(b"same payload")
            .unwrap_or_else(|error| panic!("full write after {failure:?} failed: {error}"));
        assert_eq!(link.unacked_len(), 1);
        assert_eq!(link.driver().writes.len(), 2);
        assert_eq!(
            decode_attempt(&link.driver().writes[1]).seq_num,
            1,
            "{failure:?} consumed the peer's expected sequence"
        );
    }
}

#[test]
fn test_radiolink_ack_write_failures_remain_queued() {
    for failure in [
        WriteOutcome::Zero,
        WriteOutcome::Short(1),
        WriteOutcome::Error,
    ] {
        let mut link = scripted_reliable_link([failure, WriteOutcome::Full]);
        let iq = peer_datagram_iq(1, b"ack me");
        link.driver_mut().inject_rx(&iq);
        assert_eq!(
            link.recv_datagram().unwrap().as_deref(),
            Some(&b"ack me"[..])
        );

        assert!(link.service(0).is_err(), "{failure:?} ACK write must fail");
        assert_eq!(link.driver().writes.len(), 1);

        assert_eq!(
            link.service(0)
                .unwrap_or_else(|error| panic!("queued ACK after {failure:?} failed: {error}")),
            1
        );
        assert_eq!(link.driver().writes.len(), 2);
        assert_eq!(
            link.driver().writes[0],
            link.driver().writes[1],
            "{failure:?} removed or changed the pending ACK"
        );
    }
}

#[test]
fn test_radiolink_retry_write_failures_remain_due() {
    for failure in [
        WriteOutcome::Zero,
        WriteOutcome::Short(1),
        WriteOutcome::Error,
    ] {
        let mut link = scripted_reliable_link([WriteOutcome::Full, failure, WriteOutcome::Full]);
        link.send_datagram(b"retry me").unwrap();

        assert!(
            link.service(DEFAULT_T1_MS).is_err(),
            "{failure:?} retry write must fail"
        );
        assert_eq!(link.driver().writes.len(), 2);

        assert_eq!(
            link.service(DEFAULT_T1_MS)
                .unwrap_or_else(|error| panic!("due retry after {failure:?} failed: {error}")),
            1,
            "a failed retry must remain due at the same time"
        );
        assert_eq!(link.driver().writes.len(), 3);
        assert_eq!(
            link.driver().writes[1],
            link.driver().writes[2],
            "{failure:?} changed the retained retry"
        );
        assert_eq!(link.unacked_len(), 1);
    }
}

#[test]
fn test_radiolink_reliable_mode_rejects_broadcast_peer() {
    let driver = ScriptedDriver::new([]);
    let mut link = RadioLink::new(driver, 1, BROADCAST_ADDR, params());

    let error = link.set_reliable(true).unwrap_err();
    assert!(error.to_string().contains("concrete peer"));
    assert!(link.driver().writes.is_empty());
}

#[test]
fn test_radiolink_cannot_disable_reliability_with_pending_state() {
    let mut outstanding = scripted_reliable_link([WriteOutcome::Full]);
    outstanding.send_datagram(b"still awaiting an ACK").unwrap();

    let error = outstanding.set_reliable(false).unwrap_err();
    assert!(error.to_string().contains("pending"));
    assert_eq!(outstanding.unacked_len(), 1);
    let second = outstanding
        .send_datagram(b"must remain tracked")
        .unwrap_err();
    assert!(second.to_string().contains("busy"));
    assert_eq!(outstanding.driver().writes.len(), 1);

    let mut acknowledgement = scripted_reliable_link([WriteOutcome::Full]);
    let iq = peer_datagram_iq(1, b"ACK must not be stranded");
    acknowledgement.driver_mut().inject_rx(&iq);
    assert!(acknowledgement.recv_datagram().unwrap().is_some());

    let error = acknowledgement.set_reliable(false).unwrap_err();
    assert!(error.to_string().contains("pending"));
    assert_eq!(acknowledgement.service(0).unwrap(), 1);
}
