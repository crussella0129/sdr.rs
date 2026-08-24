//! End-to-end tests for the `sdr-cli rigctl` TCP server. Each test launches the
//! real CLI binary, connects a TCP client, and drives the Hamlib rigctl protocol
//! over loopback. No radio is required.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Kills the spawned server process when the test ends.
struct ServerGuard {
    child: Child,
    stdout_thread: Option<JoinHandle<()>>,
    stderr_thread: Option<JoinHandle<()>>,
}

impl ServerGuard {
    fn join_output_threads(&mut self) {
        if let Some(thread) = self.stdout_thread.take() {
            let _ = thread.join();
        }
        if let Some(thread) = self.stderr_thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.join_output_threads();
    }
}

fn captured_stderr(lines: &Arc<Mutex<Vec<String>>>) -> Vec<String> {
    lines
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

fn wait_for_exit(child: &mut Child, deadline: Instant) -> Option<ExitStatus> {
    loop {
        if let Some(status) = child.try_wait().expect("poll rigctl child") {
            return Some(status);
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn readiness_port(line: &str) -> Option<u16> {
    const PREFIX: &str = "Listening for Hamlib connections on TCP port ";
    line.strip_prefix(PREFIX)?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

/// Launch on port zero and return the exact port reported after the real bind.
fn start_server() -> (ServerGuard, u16) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sdr-cli"))
        .args(["rigctl", "--port", "0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn sdr-cli rigctl");

    let stdout = child.stdout.take().expect("capture rigctl readiness");
    let stderr = child.stderr.take().expect("capture rigctl startup errors");
    let (line_tx, line_rx) = mpsc::channel();
    let stdout_thread = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if line_tx.send(line).is_err() {
                break;
            }
        }
    });
    let stderr_lines = Arc::new(Mutex::new(Vec::new()));
    let captured_lines = Arc::clone(&stderr_lines);
    let stderr_thread = std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines() {
            let line = line.unwrap_or_else(|error| format!("<stderr read error: {error}>"));
            captured_lines
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(line);
        }
    });
    let mut guard = ServerGuard {
        child,
        stdout_thread: Some(stdout_thread),
        stderr_thread: Some(stderr_thread),
    };

    let deadline = Instant::now() + Duration::from_secs(10);
    let mut stdout_lines = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            let status = guard.child.try_wait().expect("poll rigctl child");
            if status.is_some() {
                guard.join_output_threads();
            }
            let stderr_lines = captured_stderr(&stderr_lines);
            panic!(
                "rigctl server did not report its bound address (status {status:?}); \
                 stdout: {stdout_lines:?}; stderr: {stderr_lines:?}"
            );
        }

        match line_rx.recv_timeout(remaining.min(Duration::from_millis(100))) {
            Ok(Ok(line)) => {
                if let Some(port) = readiness_port(&line) {
                    return (guard, port);
                }
                stdout_lines.push(line);
            }
            Ok(Err(error)) => {
                let stderr_lines = captured_stderr(&stderr_lines);
                panic!(
                    "could not read rigctl readiness: {error}; stdout: {stdout_lines:?}; \
                     stderr: {stderr_lines:?}"
                );
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Some(status) = guard.child.try_wait().expect("poll rigctl child") {
                    guard.join_output_threads();
                    let stderr_lines = captured_stderr(&stderr_lines);
                    panic!(
                        "rigctl server exited before readiness ({status}); \
                         stdout: {stdout_lines:?}; stderr: {stderr_lines:?}"
                    );
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let status =
                    wait_for_exit(&mut guard.child, Instant::now() + Duration::from_secs(1));
                if status.is_some() {
                    guard.join_output_threads();
                }
                let stderr_lines = captured_stderr(&stderr_lines);
                panic!(
                    "rigctl readiness stream closed (status {status:?}); \
                     stdout: {stdout_lines:?}; stderr: {stderr_lines:?}"
                );
            }
        }
    }
}

/// Connect after the server has reported that its listener is bound.
fn connect(port: u16) -> TcpStream {
    let stream = TcpStream::connect(("127.0.0.1", port)).unwrap_or_else(|error| {
        panic!("could not connect to ready rigctl server on {port}: {error}")
    });
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
}

fn get_frequency(port: u16) -> String {
    let mut stream = connect(port);
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    stream.write_all(b"f\n").unwrap();
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    line.trim().to_string()
}

#[test]
fn test_rigctl_server_get_freq() {
    let (_guard, port) = start_server();
    assert_eq!(get_frequency(port), "144000000", "default VFO frequency");
}

#[test]
fn test_rigctl_server_set_and_get_mode() {
    let (_guard, port) = start_server();
    let mut stream = connect(port);
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // Set mode, then read it back on the same (still-open) connection.
    stream.write_all(b"M USB 2800\n").unwrap();
    let mut ack = String::new();
    reader.read_line(&mut ack).unwrap();
    assert_eq!(ack.trim(), "RPRT 0");

    stream.write_all(b"m\n").unwrap();
    let mut mode = String::new();
    reader.read_line(&mut mode).unwrap();
    let mut width = String::new();
    reader.read_line(&mut width).unwrap();
    assert_eq!(mode.trim(), "USB");
    assert_eq!(width.trim(), "2800");
}

#[test]
fn test_rigctl_server_dump_state() {
    let (_guard, port) = start_server();
    let mut stream = connect(port);
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    stream.write_all(b"\\dump_state\n").unwrap();
    // Read lines until the terminating RPRT status.
    let mut saw_rprt = false;
    for _ in 0..32 {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap() == 0 {
            break;
        }
        if line.trim() == "RPRT 0" {
            saw_rprt = true;
            break;
        }
    }
    assert!(saw_rprt, "dump_state response should end with RPRT 0");
}

#[test]
fn test_rigctl_servers_use_unique_ephemeral_ports() {
    let mut servers: Vec<_> = (0..4).map(|_| start_server()).collect();
    let ports: Vec<_> = servers.iter().map(|(_, port)| *port).collect();
    let unique: std::collections::HashSet<_> = ports.iter().copied().collect();

    assert_eq!(
        unique.len(),
        ports.len(),
        "each child must own a unique port"
    );
    for port in &ports {
        assert_eq!(
            get_frequency(*port),
            "144000000",
            "server on {port} must answer independently"
        );
    }

    while let Some((guard, _port)) = servers.pop() {
        drop(guard);
        for (_, live_port) in &servers {
            assert_eq!(
                get_frequency(*live_port),
                "144000000",
                "reaping one child must not disturb server on {live_port}"
            );
        }
    }
}
