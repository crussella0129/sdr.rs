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

pub mod framesync;
pub mod kiss;
pub mod node;
pub mod policy;
pub mod radio;
pub mod stream;

pub use framesync::sync_to_frame;
pub use kiss::{KissDecoder, FEND};
pub use node::{LoopbackLink, MeshInterface, MeshNode};
pub use policy::{Decision, MeshPolicy, TxMode};
pub use radio::{RadioLink, RadioParams, BROADCAST_ADDR};
pub use stream::{StreamBridge, DEFAULT_MTU};
