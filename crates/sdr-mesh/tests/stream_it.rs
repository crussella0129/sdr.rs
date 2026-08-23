//! `StreamBridge` over the real radio path (`RadioLink` + `MockSdr` loopback):
//! a byte stream spanning several MTUs, carried as modulated IQ. No radio.

use sdr_hardware::driver::SdrDriver;
use sdr_hardware::mock::MockSdr;
use sdr_mesh::{RadioLink, RadioParams, StreamBridge, BROADCAST_ADDR};

fn params() -> RadioParams {
    RadioParams {
        sample_rate: 1.0e6,
        deviation_hz: 100.0e3,
        samples_per_symbol: 10,
    }
}

fn bridge(mtu: usize) -> StreamBridge<RadioLink<MockSdr>> {
    let p = params();
    let mut mock = MockSdr::new(p.sample_rate as f64, 915.0e6);
    mock.enable_loopback();
    mock.start_tx().unwrap();
    mock.start_rx().unwrap();
    StreamBridge::with_mtu(RadioLink::new(mock, 1, BROADCAST_ADDR, p), mtu)
}

#[test]
fn test_stream_bridge_roundtrip_multi_chunk() {
    let mtu = 64;
    let mut b = bridge(mtu);

    // Several MTUs' worth, with a deterministic non-repeating pattern so a
    // reordering or a dropped chunk would be visible.
    let data: Vec<u8> = (0..300u32)
        .map(|i| (i.wrapping_mul(31) & 0xFF) as u8)
        .collect();
    assert!(data.len() > mtu * 4, "must span several datagrams");

    b.write(&data).expect("write stream");

    let got = b.read(4096).expect("read stream");
    assert_eq!(
        got, data,
        "a multi-chunk byte stream must be reassembled byte-for-byte, in order"
    );
}

#[test]
fn test_stream_bridge_carries_ssh_like_banner() {
    // The shape of traffic the bridge exists to carry.
    let mut b = bridge(64);
    let banner = b"SSH-2.0-OpenSSH_for_Windows_9.5\r\n".to_vec();

    b.write(&banner).expect("write");
    let got = b.read(1024).expect("read");
    assert_eq!(got, banner);
}
