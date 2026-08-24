//! `RadioLink` over `MockSdr` loopback: a mesh datagram carried end-to-end as
//! modulated IQ. No radio required.

use sdr_core::sample::Complex32;
use sdr_hardware::driver::SdrDriver;
use sdr_hardware::mock::MockSdr;
use sdr_mesh::node::MeshInterface;
use sdr_mesh::{RadioLink, RadioParams, BROADCAST_ADDR};

fn params() -> RadioParams {
    RadioParams {
        sample_rate: 1.0e6,
        deviation_hz: 100.0e3,
        samples_per_symbol: 10,
    }
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

// --- ARQ on the radio path (T-044) -----------------------------------------
//
// These use `set_reliable(true)`. The link is fire-and-forget by default, which
// is why the six tests above are unaffected: enabling ARQ changes what goes on
// the channel, so it is opt-in rather than imposed.

/// A reliable link over the same mock loopback.
fn reliable_link() -> RadioLink<MockSdr> {
    let mut link = loopback_link();
    link.set_reliable(true);
    link
}

#[test]
fn test_radiolink_acks_received_data() {
    let mut link = reliable_link();
    link.send_datagram(b"needs acknowledging").unwrap();

    // Receiving decodes the frame and queues an ACK; it must not transmit as a
    // side effect of receiving.
    let got = link.recv_datagram().unwrap();
    assert_eq!(got.as_deref(), Some(&b"needs acknowledging"[..]));

    // `service` is where the ACK actually reaches the air.
    let sent = link.service(0).unwrap();
    assert!(
        sent >= 1,
        "receiving a data frame must produce an ACK to transmit, sent {sent}"
    );
}

#[test]
fn test_radiolink_suppresses_duplicate_frames() {
    let mut link = reliable_link();

    // The same datagram transmitted twice: the mock echoes both copies back,
    // and the second carries a sequence number already seen.
    link.send_datagram(b"once only").unwrap();
    let first = link.recv_datagram().unwrap();
    assert_eq!(first.as_deref(), Some(&b"once only"[..]));

    // Re-transmit the identical frame by retransmission rather than a new send,
    // so the sequence number repeats.
    link.service(0).unwrap();
    let due = link
        .service(sdr_protocols::packet::DEFAULT_T1_MS * 4)
        .unwrap();
    assert!(due >= 1, "the unacked frame should have been retransmitted");

    let second = link.recv_datagram().unwrap();
    assert!(
        second.is_none(),
        "a duplicate sequence number must not be delivered twice — this is what \
         corrupts a byte stream; got {second:?}"
    );
}

#[test]
fn test_radiolink_retransmits_unacked_frame() {
    let mut link = reliable_link();
    link.send_datagram(b"unacknowledged").unwrap();
    assert_eq!(link.unacked_len(), 1, "the frame must be retained");

    // Before T1 nothing is due.
    assert_eq!(
        link.service(0).unwrap(),
        0,
        "nothing may be retransmitted before T1 expires"
    );

    // After T1 the frame goes out again.
    let sent = link
        .service(sdr_protocols::packet::DEFAULT_T1_MS * 2)
        .unwrap();
    assert!(
        sent >= 1,
        "an unacknowledged frame must be retransmitted once T1 expires, sent {sent}"
    );
}
