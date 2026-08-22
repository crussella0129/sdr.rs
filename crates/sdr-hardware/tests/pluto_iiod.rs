//! CI regression test: drive the `PlutoSdr` iiod client end-to-end against an
//! in-process mock iiod server that replays the network protocol. No radio and
//! no external network are required.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use sdr_core::sample::Complex32;
use sdr_hardware::driver::SdrDriver;
use sdr_hardware::pluto::PlutoSdr;

const CTX_XML: &str = concat!(
    r#"<?xml version="1.0" encoding="utf-8"?><!DOCTYPE context><context name="network">"#,
    r#"<device id="iio:device0" name="ad9361-phy">"#,
    r#"<channel id="voltage0" type="input"><attribute name="sampling_frequency" /></channel>"#,
    r#"<channel id="altvoltage0" name="RX_LO" type="output"><attribute name="frequency" /></channel>"#,
    r#"</device>"#,
    r#"<device id="iio:device3" name="cf-ad9361-lpc">"#,
    r#"<channel id="voltage0" type="input"><scan-element index="0" /></channel>"#,
    r#"<channel id="voltage1" type="input"><scan-element index="1" /></channel>"#,
    r#"</device></context>"#,
);

/// Start a mock iiod server on an ephemeral port; returns the bound port.
fn spawn_mock_iiod() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        if let Ok((stream, _)) = listener.accept() {
            handle_connection(stream);
        }
    });
    port
}

/// Reply to the exact command sequence a `PlutoSdr` issues, mimicking iiod's
/// framing (including the trailing newline after text payloads, and none after
/// `READBUF` data).
fn handle_connection(stream: TcpStream) {
    let mut writer = stream.try_clone().unwrap();
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    loop {
        line.clear();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        let cmd = line.trim_end();
        let last_token: usize = cmd
            .rsplit(' ')
            .next()
            .and_then(|t| t.parse().ok())
            .unwrap_or(0);

        if cmd == "VERSION" {
            writer.write_all(b"0.21.v0.21\n").unwrap();
        } else if cmd.starts_with("PRINT") {
            let xml = CTX_XML.as_bytes();
            writer
                .write_all(format!("{}\n", xml.len()).as_bytes())
                .unwrap();
            writer.write_all(xml).unwrap();
            writer.write_all(b"\n").unwrap(); // trailing newline after text payload
        } else if cmd.starts_with("WRITE") {
            // Consume the attribute value bytes, then ack the byte count.
            let mut data = vec![0u8; last_token];
            reader.read_exact(&mut data).unwrap();
            writer
                .write_all(format!("{last_token}\n").as_bytes())
                .unwrap();
        } else if cmd.starts_with("OPEN") {
            writer.write_all(b"0\n").unwrap();
        } else if cmd.starts_with("READBUF") {
            writer
                .write_all(format!("{last_token}\n00000003\n").as_bytes())
                .unwrap();
            // Non-zero synthetic IQ payload (no trailing newline, matching iiod).
            let data: Vec<u8> = (0..last_token).map(|i| (i as u8) | 0x21).collect();
            writer.write_all(&data).unwrap();
        } else if cmd.starts_with("CLOSE") {
            writer.write_all(b"0\n").unwrap();
        } else {
            writer.write_all(b"-22\n").unwrap();
        }
        writer.flush().unwrap();
    }
}

#[test]
fn test_pluto_iiod_replay() {
    let port = spawn_mock_iiod();
    let mut pluto = PlutoSdr::new(&format!("ip:127.0.0.1:{port}")).unwrap();

    // start_rx performs: connect (VERSION + PRINT + context parse) and
    // apply_settings (a sequence of WRITE attribute commands).
    pluto.start_rx().expect("start_rx over mock iiod");
    assert!(pluto.is_active());

    let mut buf = vec![Complex32::default(); 8];
    let n = pluto
        .read_samples(&mut buf)
        .expect("read_samples over mock iiod");
    assert_eq!(n, 8, "expected 8 IQ samples from a 32-byte READBUF");

    let nonzero = buf.iter().filter(|c| c.re != 0.0 || c.im != 0.0).count();
    assert!(nonzero > 0, "expected non-zero IQ from the replayed buffer");

    pluto.stop_rx().unwrap();
    pluto.teardown().unwrap();
}
