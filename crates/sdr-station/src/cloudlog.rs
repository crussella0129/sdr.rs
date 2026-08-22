//! Cloudlog / Wavelog station-logging HTTP client.
//!
//! Targets the JSON API exposed by [Cloudlog](https://github.com/magicbug/Cloudlog):
//!
//! - `POST <base>/index.php/api/radio` — live CAT state
//!   (`{key, radio, frequency, mode, power, timestamp}`).
//! - `POST <base>/index.php/api/qso` — ADIF contact upload
//!   (`{key, station_profile_id, type: "adif", string}`).
//!
//! Both require an API key; a missing/invalid key yields HTTP 401.

use crate::adif::{to_adif_record, Contact};
use serde_json::json;
use std::fmt;
use thiserror::Error;

/// Errors returned by [`CloudlogClient`] operations.
#[derive(Error, Debug)]
pub enum CloudlogError {
    /// The API rejected the key (HTTP 401).
    #[error("cloudlog authentication failed (HTTP 401): missing or invalid API key")]
    Auth,
    /// The API returned a non-success HTTP status other than 401.
    #[error("cloudlog API returned HTTP {0}")]
    Http(u16),
    /// A transport/network-level failure occurred.
    #[error("cloudlog transport error: {0}")]
    Transport(String),
}

/// Convenience result alias for station-logging operations.
pub type Result<T> = std::result::Result<T, CloudlogError>;

/// Live radio (CAT) state pushed to Cloudlog's `/api/radio` endpoint.
///
/// `timestamp` is caller-provided (Cloudlog expects `YYYY/MM/DD HH:MM:SS`),
/// keeping the client free of a datetime dependency and deterministic in tests.
#[derive(Debug, Clone, PartialEq)]
pub struct RadioState {
    /// Radio name/identifier shown in the logbook (e.g. `sdr.rs Pluto+`).
    pub radio: String,
    /// Tuned center frequency in Hz.
    pub frequency_hz: u64,
    /// Operating mode string (e.g. `USB`, `FM`, `CW`).
    pub mode: String,
    /// Optional transmit power in watts.
    pub power_w: Option<f64>,
    /// Timestamp string in Cloudlog's `YYYY/MM/DD HH:MM:SS` form.
    pub timestamp: String,
}

/// HTTP client for a Cloudlog (or API-compatible Wavelog) instance.
///
/// The API key is held privately and deliberately redacted from the [`fmt::Debug`]
/// output so it never leaks into logs.
pub struct CloudlogClient {
    base_url: String,
    api_key: String,
    station_profile_id: Option<String>,
    agent: ureq::Agent,
}

impl fmt::Debug for CloudlogClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CloudlogClient")
            .field("base_url", &self.base_url)
            .field("api_key", &"<redacted>")
            .field("station_profile_id", &self.station_profile_id)
            .finish()
    }
}

impl CloudlogClient {
    /// Create a client for `base_url` (e.g. `http://192.168.1.50/cloudlog`)
    /// authenticating with `api_key`.
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(15))
            .build();
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            station_profile_id: None,
            agent,
        }
    }

    /// Set the Cloudlog station profile id used for QSO uploads.
    pub fn with_station_profile(mut self, id: impl Into<String>) -> Self {
        self.station_profile_id = Some(id.into());
        self
    }

    fn radio_url(&self) -> String {
        format!(
            "{}/index.php/api/radio",
            self.base_url.trim_end_matches('/')
        )
    }

    fn qso_url(&self) -> String {
        format!("{}/index.php/api/qso", self.base_url.trim_end_matches('/'))
    }

    /// Push live CAT state to `/api/radio`. Returns `Ok(())` on HTTP 200.
    pub fn push_radio(&self, state: &RadioState) -> Result<()> {
        let body = json!({
            "key": self.api_key,
            "radio": state.radio,
            "frequency": state.frequency_hz,
            "mode": state.mode,
            "power": state.power_w,
            "timestamp": state.timestamp,
        });
        self.post_json(&self.radio_url(), body)
    }

    /// Upload a contact to `/api/qso` as a single ADIF record.
    pub fn log_qso(&self, contact: &Contact) -> Result<()> {
        let body = json!({
            "key": self.api_key,
            "station_profile_id": self.station_profile_id,
            "type": "adif",
            "string": to_adif_record(contact),
        });
        self.post_json(&self.qso_url(), body)
    }

    fn post_json(&self, url: &str, body: serde_json::Value) -> Result<()> {
        match self.agent.post(url).send_json(body) {
            Ok(_response) => Ok(()),
            Err(ureq::Error::Status(401, _)) => Err(CloudlogError::Auth),
            Err(ureq::Error::Status(code, _)) => Err(CloudlogError::Http(code)),
            Err(ureq::Error::Transport(t)) => Err(CloudlogError::Transport(t.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_not_logged() {
        let client = CloudlogClient::new("http://example.invalid/cloudlog", "SECRETKEY123");
        let dbg = format!("{client:?}");
        assert!(
            !dbg.contains("SECRETKEY123"),
            "api key leaked in Debug: {dbg}"
        );
        assert!(dbg.contains("<redacted>"));
    }

    #[test]
    fn test_endpoint_urls_trim_trailing_slash() {
        let client = CloudlogClient::new("http://host/cloudlog/", "k");
        assert_eq!(
            client.radio_url(),
            "http://host/cloudlog/index.php/api/radio"
        );
        assert_eq!(client.qso_url(), "http://host/cloudlog/index.php/api/qso");
    }
}
