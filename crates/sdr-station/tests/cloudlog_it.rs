//! Integration tests for the Cloudlog client against a local mock HTTP server.
//! No radio and no external network are required.

use std::thread;

use sdr_station::{CloudlogClient, CloudlogError, Contact, RadioState};
use tiny_http::{Response, Server};

/// A captured inbound HTTP request.
struct Captured {
    method: String,
    url: String,
    body: String,
}

/// Start a mock server on an ephemeral port that answers `n` requests with
/// `status`, capturing each. Returns the bound port and a join handle yielding
/// the captured requests in order.
fn spawn_server(status: u16, n: usize) -> (u16, thread::JoinHandle<Vec<Captured>>) {
    let server = Server::http("127.0.0.1:0").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();
    let handle = thread::spawn(move || {
        let mut caps = Vec::new();
        for _ in 0..n {
            match server.recv() {
                Ok(mut req) => {
                    let method = req.method().as_str().to_string();
                    let url = req.url().to_string();
                    let mut body = String::new();
                    let _ = req.as_reader().read_to_string(&mut body);
                    let resp =
                        Response::from_string("{\"status\":\"ok\"}").with_status_code(status);
                    let _ = req.respond(resp);
                    caps.push(Captured { method, url, body });
                }
                Err(_) => break,
            }
        }
        caps
    });
    (port, handle)
}

fn sample_state() -> RadioState {
    RadioState {
        radio: "sdr.rs Pluto+".to_string(),
        frequency_hz: 14_074_000,
        mode: "USB".to_string(),
        power_w: Some(5.0),
        timestamp: "2026/08/22 00:15:30".to_string(),
    }
}

#[test]
fn test_push_radio_ok() {
    let (port, handle) = spawn_server(200, 1);
    let client = CloudlogClient::new(format!("http://127.0.0.1:{port}"), "KEY123");

    let res = client.push_radio(&sample_state());
    assert!(res.is_ok(), "expected Ok, got {res:?}");

    let caps = handle.join().unwrap();
    assert_eq!(caps.len(), 1);
    assert_eq!(caps[0].method, "POST");
    assert_eq!(caps[0].url, "/index.php/api/radio");
    assert!(
        caps[0].body.contains("\"key\":\"KEY123\""),
        "{}",
        caps[0].body
    );
    assert!(caps[0].body.contains("\"radio\":\"sdr.rs Pluto+\""));
    assert!(caps[0].body.contains("\"frequency\":14074000"));
    assert!(caps[0].body.contains("\"mode\":\"USB\""));
    assert!(caps[0]
        .body
        .contains("\"timestamp\":\"2026/08/22 00:15:30\""));
}

#[test]
fn test_push_radio_401() {
    let (port, handle) = spawn_server(401, 1);
    let client = CloudlogClient::new(format!("http://127.0.0.1:{port}"), "BADKEY");

    let res = client.push_radio(&sample_state());
    assert!(matches!(res, Err(CloudlogError::Auth)), "got {res:?}");

    let _ = handle.join().unwrap();
}

#[test]
fn test_log_qso_adif() {
    let (port, handle) = spawn_server(200, 1);
    let client =
        CloudlogClient::new(format!("http://127.0.0.1:{port}"), "KEY123").with_station_profile("3");

    let mut contact = Contact::new("W1AW", "20260822", "001530", "20m", "SSB");
    contact.rst_sent = Some("59".to_string());
    contact.rst_rcvd = Some("57".to_string());

    let res = client.log_qso(&contact);
    assert!(res.is_ok(), "expected Ok, got {res:?}");

    let caps = handle.join().unwrap();
    assert_eq!(caps.len(), 1);
    assert_eq!(caps[0].url, "/index.php/api/qso");
    assert!(
        caps[0].body.contains("\"type\":\"adif\""),
        "{}",
        caps[0].body
    );
    assert!(caps[0].body.contains("\"station_profile_id\":\"3\""));
    // The ADIF payload is embedded in the JSON "string" field.
    assert!(caps[0].body.contains("W1AW"));
    assert!(caps[0].body.contains("<EOR>"));
}

#[test]
fn test_cloudlog_radio_then_qso() {
    let (port, handle) = spawn_server(200, 2);
    let client =
        CloudlogClient::new(format!("http://127.0.0.1:{port}"), "KEY123").with_station_profile("1");

    assert!(client.push_radio(&sample_state()).is_ok());
    let contact = Contact::new("K2ABC", "20260822", "010000", "40m", "FT8");
    assert!(client.log_qso(&contact).is_ok());

    let caps = handle.join().unwrap();
    assert_eq!(caps.len(), 2);
    assert_eq!(caps[0].url, "/index.php/api/radio");
    assert_eq!(caps[1].url, "/index.php/api/qso");
}

#[test]
fn test_readme_lists_cloudlog() {
    let readme = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../README.md"));
    assert!(
        readme.contains("magicbug/Cloudlog"),
        "README reference catalog is missing the Cloudlog entry"
    );
}
