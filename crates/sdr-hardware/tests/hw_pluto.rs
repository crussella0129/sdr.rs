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
use sdr_hardware::pluto::{PlutoSdr, TX_GAIN_MIN_DB};

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

/// Transmit verification with **internal loopback**.
///
/// The AD9361's internal digital loopback (mode 1) bypasses the entire RF
/// section, and the transmitter is additionally held at maximum attenuation, so
/// samples travel TX DMA → RX DMA inside the device without reaching the
/// antenna port. The DDS tone generators are silenced so anything observed on
/// RX comes from the buffer this test wrote.
///
/// Device state is restored *before* any assertion runs, so a failure cannot
/// leave the radio in loopback or fully attenuated.
#[test]
#[ignore = "requires a physical PlutoSDR at 192.168.2.1"]
fn hw_verify_pluto_tx_loopback() {
    let mut pluto = PlutoSdr::new(PLUTO_URI).unwrap();
    pluto.connect().expect("connect to Pluto iiod endpoint");

    // Engage the loopback configuration before anything is transmitted.
    pluto
        .enter_loopback_test_mode()
        .expect("enter internal-loopback test mode");

    pluto.set_sample_rate(0, 3.0e6).unwrap();
    pluto.set_bandwidth(0, 2.0e6).unwrap();
    pluto.set_tx_frequency(915.0e6).unwrap();
    pluto.set_frequency(0, 915.0e6).unwrap();
    pluto.set_gain_mode(0, GainMode::Manual).unwrap();
    pluto.set_gain(0, 30.0).unwrap();

    // A constant-amplitude pattern: distinguishable from noise and from the
    // (now disabled) DDS tones.
    let pattern: Vec<Complex32> = (0..4096)
        .map(|i| {
            if i % 2 == 0 {
                Complex32::new(0.5, -0.5)
            } else {
                Complex32::new(-0.5, 0.5)
            }
        })
        .collect();

    // A cyclic buffer repeats until closed; a one-shot buffer would drain before
    // the receiver could observe it.
    pluto.set_tx_cyclic(true);

    let tx_result = pluto
        .start_tx()
        .and_then(|()| pluto.write_samples(&pattern));

    let rx_result = pluto.start_rx().and_then(|()| {
        let mut buf = vec![Complex32::default(); 4096];
        pluto.read_samples(&mut buf).map(|n| (n, buf))
    });

    // --- restore device state before asserting (critique C-003) ---
    let _ = pluto.stop_tx();
    let _ = pluto.stop_rx();
    let restore = pluto.exit_loopback_test_mode();
    let _ = pluto.teardown();

    // --- now it is safe to fail ---
    let written = tx_result.expect("TX path must accept samples over real iiod");
    assert_eq!(
        written,
        pattern.len(),
        "the daemon should accept every sample written"
    );
    restore.expect("device state must be restored");

    let (n, buf) = rx_result.expect("RX readback under loopback");
    let nonzero = buf[..n]
        .iter()
        .filter(|c| c.re != 0.0 || c.im != 0.0)
        .count();
    let peak = buf[..n].iter().map(|c| c.norm()).fold(0.0_f32, f32::max);
    println!(
        "hw_verify_pluto_tx_loopback: wrote {} samples at {TX_GAIN_MIN_DB} dB (max attenuation, RF bypassed); \
         read back {n} samples, {nonzero} non-zero, peak |amp| = {peak:.4}",
        pattern.len()
    );
    assert!(n > 0, "expected RX samples under loopback, read {n}");
    // With the DDS silenced and the RF section bypassed, anything received came
    // from the buffer this test transmitted.
    assert!(
        nonzero > n / 2,
        "expected the transmitted pattern to return on RX, got {nonzero}/{n} non-zero"
    );
    // The pattern was written at amplitude 0.5 per component (|z| ~= 0.707).
    assert!(
        peak > 0.1,
        "loopback amplitude {peak:.4} is too low to be the transmitted pattern"
    );
}
