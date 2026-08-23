//! Live hardware test: a mesh datagram carried through the **real** PlutoSDR.
//!
//! `#[ignore]`d so ordinary CI (which has no radio) skips it. Run against a
//! physical Pluto+ at `192.168.2.1` with:
//!
//! ```text
//! cargo test -p sdr-mesh --test hw_radio -- --ignored --nocapture --test-threads=1
//! ```
//!
//! `--test-threads=1` is required, not cosmetic: there is one physical radio and
//! it allows one open buffer at a time, so tests running in parallel collide and
//! iiod rejects the second with `OPEN failed: errno 16` (EBUSY).
//!
//! # Live testing uses internal loopback
//!
//! The AD9361's internal digital loopback bypasses the entire RF section, the
//! transmitter is held at maximum attenuation, and the DDS tone generators are
//! silenced — so the datagram travels TX DMA → RX DMA inside the device.

use sdr_hardware::driver::SdrDriver;
use sdr_hardware::pluto::PlutoSdr;
use sdr_mesh::node::MeshInterface;
use sdr_mesh::{RadioLink, RadioParams, StreamBridge, BROADCAST_ADDR};

const PLUTO_URI: &str = "ip:192.168.2.1";
const SAMPLE_RATE: f64 = 3.0e6;

#[test]
#[ignore = "requires a physical PlutoSDR at 192.168.2.1"]
fn hw_verify_mesh_datagram_over_radio() {
    let params = RadioParams {
        sample_rate: SAMPLE_RATE as f32,
        deviation_hz: 150.0e3,
        samples_per_symbol: 20,
    };

    let mut pluto = PlutoSdr::new(PLUTO_URI).unwrap();
    pluto.connect().expect("connect to Pluto iiod endpoint");

    // Loopback configuration, engaged before anything is transmitted.
    pluto
        .enter_loopback_test_mode()
        .expect("enter internal-loopback test mode");

    pluto.set_sample_rate(0, SAMPLE_RATE).unwrap();
    pluto.set_bandwidth(0, 2.0e6).unwrap();
    pluto.set_tx_frequency(915.0e6).unwrap();
    pluto.set_frequency(0, 915.0e6).unwrap();
    pluto.set_gain(0, 30.0).unwrap();
    // Repeat the frame so the receiver can capture a complete copy.
    pluto.set_tx_cyclic(true);

    let mut link = RadioLink::new(pluto, 1, BROADCAST_ADDR, params);
    // Comfortably more than twice the modulated frame length.
    link.set_rx_chunk(32_768);

    let datagram = vec![0x45, 0x00, 0x00, 0x28, 0xC0, 0xDB, 0xAA, 0x55, 0x00, 0xFF];

    let tx_result = link
        .driver_mut()
        .start_tx()
        .and_then(|()| link.send_datagram(&datagram));

    let rx_result = link
        .driver_mut()
        .start_rx()
        .and_then(|()| link.recv_datagram());

    // --- restore device state before asserting, so a failure cannot strand the radio ---
    let _ = link.driver_mut().stop_tx();
    let _ = link.driver_mut().stop_rx();
    let restore = link.driver_mut().exit_loopback_test_mode();
    let _ = link.driver_mut().teardown();

    // --- now it is safe to fail ---
    tx_result.expect("sending the datagram over the real radio must succeed");
    restore.expect("device state must be restored");

    let received = rx_result.expect("receiving must not error");
    println!(
        "hw_verify_mesh_datagram_over_radio: sent {} bytes through the Pluto+ \
         (internal loopback, max attenuation); recovered {:?}",
        datagram.len(),
        received.as_ref().map(|d| d.len())
    );

    let received = received.expect("a datagram should be recovered from the radio loopback");
    assert_eq!(
        received, datagram,
        "the datagram must survive the full radio path byte-for-byte"
    );
}

/// A multi-chunk byte stream carried over the real radio.
///
/// Same standing configuration as the datagram test: AD9361 internal loopback,
/// maximum attenuation, DDS silenced, device state restored before assertions.
///
/// The transmit buffer is **cyclic**, as in the datagram test — a one-shot
/// buffer drains before the receiver can capture a complete copy. Cyclic
/// transmission constrains how a stream can be carried, and this test is shaped
/// by those constraints rather than hiding them:
///
/// 1. The buffer holds **one frame**, so a later chunk replaces the earlier one.
///    Chunks must therefore ping-pong — write one, read it back, then the next —
///    rather than being written as a burst.
/// 2. The loaded frame **repeats indefinitely**, so the same datagram is
///    recovered many times. Each chunk is taken once and the repeats are
///    discarded via `clear_inbound`.
///
/// Both are properties of driving a cyclic DMA buffer, not defects in the
/// bridge: a two-radio link would stream continuously instead.
#[test]
#[ignore = "requires a physical PlutoSDR at 192.168.2.1"]
fn hw_verify_stream_over_radio() {
    let params = RadioParams {
        sample_rate: SAMPLE_RATE as f32,
        deviation_hz: 150.0e3,
        samples_per_symbol: 20,
    };
    const MTU: usize = 64;

    let mut pluto = PlutoSdr::new(PLUTO_URI).unwrap();
    pluto.connect().expect("connect to Pluto iiod endpoint");
    pluto
        .enter_loopback_test_mode()
        .expect("enter internal-loopback test mode");

    pluto.set_sample_rate(0, SAMPLE_RATE).unwrap();
    pluto.set_bandwidth(0, 2.0e6).unwrap();
    pluto.set_tx_frequency(915.0e6).unwrap();
    pluto.set_frequency(0, 915.0e6).unwrap();
    pluto.set_gain(0, 30.0).unwrap();
    pluto.set_tx_cyclic(true);

    // Broadcast: the radio is echoing to itself, not talking to a second station.
    let link = RadioLink::new(pluto, 1, BROADCAST_ADDR, params);
    let mut bridge = StreamBridge::with_mtu(link, MTU);

    let stream: Vec<u8> =
        b"SSH-2.0-sdr.rs over radio; the quick brown fox jumps over the lazy dog 0123456789ABCDEF"
            .to_vec();
    assert!(stream.len() > MTU, "must span more than one datagram");

    let started = bridge
        .iface_mut()
        .driver_mut()
        .start_tx()
        .and_then(|()| bridge.iface_mut().driver_mut().start_rx());

    // Ping-pong: one chunk loaded at a time, read back before the next is sent.
    let result = started.and_then(|()| {
        let mut got: Vec<u8> = Vec::new();
        for chunk in stream.chunks(MTU) {
            bridge.write(chunk)?;

            // Flush captures still holding the previous chunk. Safe to discard:
            // the cyclic buffer keeps repeating the new one.
            let _ = bridge.read(usize::MAX)?;
            bridge.clear_inbound();

            let mut recovered = Vec::new();
            for _ in 0..8 {
                let r = bridge.read(chunk.len())?;
                if r.len() == chunk.len() {
                    recovered = r;
                    break;
                }
            }
            got.extend_from_slice(&recovered);

            // Drop the repeats so they do not land in the stream.
            bridge.clear_inbound();
        }
        Ok(got)
    });

    // --- restore device state before asserting ---
    let _ = bridge.iface_mut().driver_mut().stop_tx();
    let _ = bridge.iface_mut().driver_mut().stop_rx();
    let restore = bridge.iface_mut().driver_mut().exit_loopback_test_mode();
    let _ = bridge.iface_mut().driver_mut().teardown();

    // --- now it is safe to fail ---
    let got = result.expect("stream transfer over the radio must succeed");
    restore.expect("device state must be restored");

    println!(
        "hw_verify_stream_over_radio: sent {} bytes as {} datagrams; recovered {} bytes",
        stream.len(),
        stream.len().div_ceil(MTU),
        got.len()
    );
    assert_eq!(
        got, stream,
        "the byte stream must be reassembled byte-for-byte over the radio"
    );
}
