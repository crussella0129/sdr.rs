//! # sdr-station
//!
//! Station-logging integrations for the `sdr.rs` suite.
//!
//! The first supported backend is [Cloudlog](https://github.com/magicbug/Cloudlog)
//! (and the API-compatible Wavelog fork). It exposes two capabilities:
//!
//! - [`CloudlogClient::push_radio`] posts live CAT state (frequency, mode,
//!   power) to Cloudlog's `/index.php/api/radio` endpoint, so tuning `sdr.rs`
//!   updates the logbook's active radio in real time.
//! - [`CloudlogClient::log_qso`] serializes a [`Contact`] as an ADIF record and
//!   posts it to `/index.php/api/qso`.
//!
//! The HTTP dependency lives in this crate only, keeping the pure-DSP crates
//! (`sdr-core`, `sdr-dsp`) dependency-light.

pub mod adif;
pub mod cloudlog;

pub use adif::{to_adif_record, Contact};
pub use cloudlog::{CloudlogClient, CloudlogError, RadioState, Result};
