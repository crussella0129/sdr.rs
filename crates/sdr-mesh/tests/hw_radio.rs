//! Live hardware test: a mesh datagram carried through the **real** PlutoSDR.
//!
//! `#[ignore]`d so ordinary CI (which has no radio) skips it. Run against a
//! physical Pluto+ at `192.168.2.1` with:
//!
//! ```text
//! cargo test -p sdr-mesh --test hw_radio -- --ignored --nocapture
//! ```
//!
//! # Live testing uses internal loopback
//!
//! The AD9361's internal digital loopback bypasses the entire RF section, the
//! transmitter is held at maximum attenuation, and the DDS tone generators are
//! silenced — so the datagram travels TX DMA → RX DMA inside the device.

use sdr_hardware::driver::SdrDriver;
use sdr_hardware::pluto::PlutoSdr;
use sdr_mesh::node::MeshInterface;
use sdr_mesh::{RadioLink, RadioParams, BROADCAST_ADDR};

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
         (RF bypassed, max attenuation); recovered {:?}",
        datagram.len(),
        received.as_ref().map(|d| d.len())
    );

    let received = received.expect("a datagram should be recovered from the radio loopback");
    assert_eq!(
        received, datagram,
        "the datagram must survive the full radio path byte-for-byte"
    );
}
