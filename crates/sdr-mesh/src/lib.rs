//! # sdr-mesh
//!
//! The IP-over-radio mesh layer for `sdr.rs` — Phase A of INT-0008.
//!
//! It sits above the realized packet-radio link layer (`sdr-protocols`
//! `ArqTransceiver`) and adds KISS framing so IP datagram boundaries survive a
//! byte-oriented radio link. Subsequent Phase-A tasks add a dual-mode
//! compliance gate (`policy`) and a `MeshInterface` seam with an in-memory
//! loopback link (`node`).
//!
//! Networking dependencies are confined to this crate; the pure-DSP crates stay
//! dependency-light.

pub mod kiss;

pub use kiss::{KissDecoder, FEND};
