//! Live hardware-in-the-loop tests for the PlutoSDR iiod driver.
//!
//! These are `#[ignore]`d so ordinary CI (which has no radio) skips them. Run
//! them against a physical Pluto+ reachable at `192.168.2.1` with:
//!
//! ```text
//! cargo test -p sdr-hardware --test hw_pluto -- --ignored --nocapture
//! ```

use sdr_core::sample::Complex32;
use sdr_hardware::driver::{GainMode, SdrDriver};
use sdr_hardware::pluto::PlutoSdr;

const PLUTO_URI: &str = "ip:192.168.2.1";

#[test]
#[ignore = "requires a physical PlutoSDR at 192.168.2.1"]
fn hw_verify_pluto_connect() {
    let mut pluto = PlutoSdr::new(PLUTO_URI).unwrap();
    pluto.connect().expect("connect to Pluto iiod endpoint");
    pluto.teardown().unwrap();
}

#[test]
#[ignore = "requires a physical PlutoSDR at 192.168.2.1"]
fn hw_verify_pluto_rx() {
    let mut pluto = PlutoSdr::new(PLUTO_URI).unwrap();
    // Tune to a strong broadcast-FM carrier so a real signal is present.
    pluto.set_sample_rate(0, 3.0e6).unwrap();
    pluto.set_bandwidth(0, 2.0e6).unwrap();
    pluto.set_gain_mode(0, GainMode::SlowAttack).unwrap();
    pluto.set_frequency(0, 95.83e6).unwrap();

    pluto.start_rx().expect("start RX streaming");

    let mut buf = vec![Complex32::default(); 4096];
    let n = pluto.read_samples(&mut buf).expect("read IQ samples");
    assert!(n > 0, "expected samples, read {n}");

    let nonzero = buf[..n]
        .iter()
        .filter(|c| c.re != 0.0 || c.im != 0.0)
        .count();
    let peak = buf[..n].iter().map(|c| c.norm()).fold(0.0_f32, f32::max);
    println!("hw_verify_pluto_rx: read {n} samples, {nonzero} non-zero, peak |amp| = {peak:.4}");
    assert!(
        nonzero > n / 2,
        "expected mostly non-zero IQ from real hardware, got {nonzero}/{n}"
    );

    pluto.stop_rx().unwrap();
    pluto.teardown().unwrap();
}
