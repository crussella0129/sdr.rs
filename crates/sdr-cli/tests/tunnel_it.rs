//! `sdr-cli tunnel` runtime: the command must actually pipe bytes, not just
//! print that it is ready. Runs the real binary over the mock loopback driver.

use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sdr-cli"))
}

#[test]
fn test_cli_tunnel_stdio_pipes_bytes() {
    let mut child = bin()
        .args(["tunnel", "--stdio", "--driver", "mock", "--eirp-dbm", "0"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn sdr-cli tunnel");

    let payload = b"SSH-2.0-sdr.rs-test\r\n";
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(payload)
        .expect("write to tunnel stdin");
    // Dropping stdin signals EOF, so the pump drains and exits.

    let mut stdout = child.stdout.take().expect("stdout");
    let mut got = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(30);
    // read_to_end returns when the child closes stdout.
    let read_result = stdout.read_to_end(&mut got);
    let _ = child.wait();
    read_result.expect("read tunnel stdout");
    assert!(
        Instant::now() < deadline,
        "the tunnel should finish promptly after EOF"
    );

    assert_eq!(
        got, payload,
        "bytes written to the tunnel must come back over the loopback link"
    );
}

#[test]
fn test_cli_tunnel_refuses_encrypted_amateur_band() {
    // 145 MHz is a cataloged amateur band where encrypted payloads are
    // prohibited; the command must surface that before carrying traffic.
    let out = bin()
        .args([
            "tunnel",
            "--freq",
            "145000000",
            "--jurisdiction",
            "US",
            "--eirp-dbm",
            "20",
            "--driver",
            "ip:127.0.0.1:1",
            "--stdio",
        ])
        .output()
        .expect("run sdr-cli tunnel");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success() && stderr.to_uppercase().contains("ENCRYPTION PROHIBITED"),
        "an encryption-prohibited band must be refused, got: {stderr}"
    );
}

#[test]
fn test_cli_tunnel_status_goes_to_stderr() {
    // In --stdio mode stdout carries the tunnelled stream, so banners must not
    // pollute it — otherwise an SSH client would read them as protocol data.
    let out = bin()
        .args(["tunnel", "--driver", "mock", "--eirp-dbm", "0"])
        .output()
        .expect("run sdr-cli tunnel");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.is_empty(),
        "status output must not go to stdout, got: {stdout}"
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("Tunnel Bridge"),
        "status belongs on stderr"
    );
}

#[test]
fn test_cli_bands_rejects_unknown_jurisdiction() {
    let out = bin()
        .args(["bands", "--jurisdiction", "Atlantis"])
        .output()
        .expect("run sdr-cli bands");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "unknown jurisdiction must fail");
    assert!(
        stderr.contains("unknown jurisdiction"),
        "parse error should explain the refusal, got: {stderr}"
    );
}

#[test]
fn test_cli_tunnel_rejects_non_finite_plan() {
    let out = bin()
        .args([
            "tunnel",
            "--stdio",
            "--freq",
            "NaN",
            "--eirp-dbm",
            "0",
            "--driver",
            "ip:127.0.0.1:1",
        ])
        .output()
        .expect("run sdr-cli tunnel");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "non-finite plan must fail");
    assert!(
        stderr.contains("Center frequency must be finite"),
        "expected finite-frequency refusal, got: {stderr}"
    );
}

#[test]
fn test_cli_tunnel_rejects_non_positive_rate_before_driver_connection() {
    let unreachable_driver = "ip:203.0.113.1:65535";
    for rate in ["0", "-1"] {
        let out = bin()
            .args(["tunnel", "--stdio"])
            .arg(format!("--rate={rate}"))
            .args(["--eirp-dbm", "0", "--driver", unreachable_driver])
            .output()
            .expect("run sdr-cli tunnel");

        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(!out.status.success(), "rate {rate} must fail");
        assert!(
            stderr.contains("sample rate") && !stderr.contains(unreachable_driver),
            "rate validation must precede driver construction, got: {stderr}"
        );
    }
}

#[test]
fn test_cli_tunnel_uses_one_normalized_rate_for_policy_modem_and_driver() {
    let out = bin()
        .args([
            "tunnel",
            "--stdio",
            "--driver",
            "mock",
            "--rate=3000000.1",
            "--eirp-dbm",
            "20",
        ])
        .output()
        .expect("run normalized-rate mock tunnel");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "normalized rate should run: {stderr}");
    assert!(
        stderr.contains("3000000.100000 Hz normalized")
            && stderr.contains("3000000.000000 Hz")
            && stderr.contains("Regulatory Status: COMPLIANT"),
        "policy and the runtime should expose the same effective rate: {stderr}"
    );
}

#[test]
fn test_cli_tunnel_accepts_negative_eirp_input() {
    let out = bin()
        .args(["tunnel", "--driver", "mock", "--eirp-dbm=-10"])
        .output()
        .expect("run sdr-cli tunnel");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "finite negative EIRP is valid; stderr: {stderr}"
    );
    assert!(
        stderr.contains("Regulatory Status: COMPLIANT"),
        "the accepted plan should be evaluated, got: {stderr}"
    );
}

#[test]
fn test_cli_tunnel_refuses_derived_bandwidth_before_driver_connection() {
    let out = bin()
        .args([
            "tunnel",
            "--stdio",
            "--freq",
            "915000000",
            "--rate",
            "4000000",
            "--jurisdiction",
            "US",
            "--eirp-dbm",
            "20",
            "--driver",
            "ip:127.0.0.1:1",
        ])
        .output()
        .expect("run sdr-cli tunnel");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "excess derived bandwidth must fail");
    assert!(
        stderr.contains("Occupied bandwidth") && stderr.contains("exceeds legal limit"),
        "expected bandwidth refusal before the unreachable driver, got: {stderr}"
    );
}

#[test]
fn test_cli_tunnel_refuses_continuous_duty_before_driver_connection() {
    let out = bin()
        .args([
            "tunnel",
            "--stdio",
            "--freq",
            "433920000",
            "--rate",
            "96000",
            "--jurisdiction",
            "EU",
            "--eirp-dbm",
            "0",
            "--driver",
            "ip:127.0.0.1:1",
        ])
        .output()
        .expect("run sdr-cli tunnel");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "continuous duty must fail on EU 433");
    assert!(
        stderr.contains("Duty cycle") && stderr.contains("exceeds legal limit"),
        "expected duty-only refusal before the unreachable driver, got: {stderr}"
    );
    assert!(
        !stderr.contains("Occupied bandwidth") && !stderr.contains("Occupied span"),
        "this scenario must isolate the duty-cycle refusal, got: {stderr}"
    );
}

#[test]
fn test_cli_tunnel_policy_refusal_precedes_driver_connection() {
    let unreachable_driver = "ip:203.0.113.1:65535";
    let out = bin()
        .args([
            "tunnel",
            "--stdio",
            "--freq",
            "915000000",
            "--jurisdiction",
            "US",
            "--eirp-dbm",
            "31",
            "--driver",
            unreachable_driver,
        ])
        .output()
        .expect("run sdr-cli tunnel");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "over-power plan must fail");
    assert!(
        stderr.contains("Power 31.0 dBm exceeds legal limit"),
        "policy should provide the terminal error, got: {stderr}"
    );
    assert!(
        !stderr.contains(unreachable_driver),
        "driver construction must not be attempted after refusal: {stderr}"
    );
}
