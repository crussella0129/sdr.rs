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
        .args(["tunnel", "--stdio", "--driver", "mock"])
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
fn test_cli_tunnel_reports_compliance() {
    // 145 MHz is a cataloged amateur band where encrypted payloads are
    // prohibited; the command must surface that before carrying traffic.
    let out = bin()
        .args([
            "tunnel",
            "--freq",
            "145000000",
            "--jurisdiction",
            "US",
            "--driver",
            "mock",
        ])
        .output()
        .expect("run sdr-cli tunnel");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("WARNING") && stderr.to_uppercase().contains("ENCRYPT"),
        "an encryption-prohibited band must surface a compliance warning, got: {stderr}"
    );
}

#[test]
fn test_cli_tunnel_status_goes_to_stderr() {
    // In --stdio mode stdout carries the tunnelled stream, so banners must not
    // pollute it — otherwise an SSH client would read them as protocol data.
    let out = bin()
        .args(["tunnel", "--driver", "mock"])
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
