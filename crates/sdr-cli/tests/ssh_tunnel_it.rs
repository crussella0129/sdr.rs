//! A real OpenSSH client exchanging protocol bytes over the radio link.
//!
//! # What this proves, and what it does not
//!
//! **Proves:** a genuine `ssh` process connects through `sdr-cli tunnel`, and
//! its SSH protocol version string makes a full round trip through KISS
//! framing, CRC-32, FSK modulation, demodulation and symbol-timing recovery —
//! completing the SSH version exchange over the radio path.
//!
//! **Does not prove:** a working SSH session. There is no SSH *server* here, so
//! the "remote" version string the client receives is its own, echoed by the
//! loopback. Key exchange, authentication and a shell are out of scope and
//! require a server (backlog).

use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// Kills the tunnel process when the test ends, however it ends.
struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn ssh_available() -> bool {
    Command::new("ssh")
        .arg("-V")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[test]
fn test_ssh_client_banner_traverses_bridge() {
    if !ssh_available() {
        eprintln!("skipping: no `ssh` binary available on this machine");
        return;
    }

    let port = free_port();
    let tunnel = Command::new(env!("CARGO_BIN_EXE_sdr-cli"))
        .args(["tunnel", "--listen", &port.to_string(), "--driver", "mock"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn sdr-cli tunnel");
    let _guard = ChildGuard(tunnel);

    // Give the listener a moment to bind before connecting.
    std::thread::sleep(Duration::from_millis(750));

    // The client is expected to hang after the version exchange: with the
    // loopback echoing its own key-exchange packets back, it is negotiating
    // with itself and can never finish. So run it against a deadline, watching
    // its trace for the marker, and kill it once seen. `output()` would block
    // forever here.
    let trace_path = std::env::temp_dir().join(format!("sdr_ssh_trace_{port}.log"));
    let trace_file = std::fs::File::create(&trace_path).expect("create trace file");
    let ssh = Command::new("ssh")
        .args([
            "-v",
            "-p",
            &port.to_string(),
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "ConnectTimeout=15",
            "-o",
            "BatchMode=yes",
            "user@127.0.0.1",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(trace_file))
        .spawn()
        .expect("spawn ssh client");
    let _ssh_guard = ChildGuard(ssh);

    let deadline = std::time::Instant::now() + Duration::from_secs(45);
    let mut trace = String::new();
    while std::time::Instant::now() < deadline {
        trace = std::fs::read_to_string(&trace_path).unwrap_or_default();
        if trace.contains("Remote protocol version") {
            break;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    let _ = std::fs::remove_file(&trace_path);

    assert!(
        trace.contains("Local version string"),
        "ssh should have sent its version string; trace:\n{trace}"
    );
    assert!(
        trace.contains("Remote protocol version"),
        "ssh should have received a protocol version back across the radio link \
         — the SSH version exchange completing over the bridge. trace:\n{trace}"
    );
}
