//! End-to-end tests for CLI device selection (`record --driver ...`) and the
//! `devices` subcommand. These run the real CLI binary; no radio is required.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sdr-cli"))
}

fn temp_path(ext: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("sdrcli_test_{}_{nanos}.{ext}", std::process::id()))
}

#[test]
fn test_cli_record_driver_mock() {
    let out = temp_path("wav");
    let status = bin()
        .args([
            "record",
            "--driver",
            "mock",
            "--samples",
            "64",
            "--rate",
            "1000000",
            "--freq",
            "915000000",
            "--output",
            out.to_str().unwrap(),
        ])
        .status()
        .expect("run sdr-cli record");
    assert!(status.success(), "record --driver mock should succeed");
    assert!(out.exists(), "recording file should be written");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn test_cli_record_driver_pluto_unreachable() {
    let out = temp_path("wav");
    // TEST endpoint that refuses immediately, so the driver surfaces a clean
    // error instead of connecting to any real radio.
    let output = bin()
        .args([
            "record",
            "--driver",
            "ip:127.0.0.1:1",
            "--samples",
            "64",
            "--output",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("run sdr-cli record");
    assert!(
        !output.status.success(),
        "record against an unreachable Pluto should fail, not succeed"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked"),
        "should fail gracefully, not panic: {stderr}"
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn test_cli_devices_lists() {
    let output = bin().arg("devices").output().expect("run sdr-cli devices");
    assert!(output.status.success(), "devices should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Mock SDR"),
        "devices output should list the mock device: {stdout}"
    );
}
