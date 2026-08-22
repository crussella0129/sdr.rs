//! End-to-end tests for the `sdr-cli rigctl` TCP server. Each test launches the
//! real CLI binary, connects a TCP client, and drives the Hamlib rigctl protocol
//! over loopback. No radio is required.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Kills the spawned server process when the test ends.
struct ServerGuard(Child);
impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Grab an unused TCP port by binding to port 0 and releasing it.
fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// Launch `sdr-cli rigctl --port <port>` and return a kill-guard and the port.
fn start_server() -> (ServerGuard, u16) {
    let port = free_port();
    let child = Command::new(env!("CARGO_BIN_EXE_sdr-cli"))
        .args(["rigctl", "--port", &port.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn sdr-cli rigctl");
    (ServerGuard(child), port)
}

/// Connect to the server, retrying until it has bound its listener.
fn connect(port: u16) -> TcpStream {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(stream) => {
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                return stream;
            }
            Err(_) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => panic!("could not connect to rigctl server on {port}: {e}"),
        }
    }
}

#[test]
fn test_rigctl_server_get_freq() {
    let (_guard, port) = start_server();
    let mut stream = connect(port);
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    stream.write_all(b"f\n").unwrap();
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    assert_eq!(line.trim(), "144000000", "default VFO frequency");
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
