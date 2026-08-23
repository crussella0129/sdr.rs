# Agent Tasks (Persistent Backlog)

- [ ] T-028 (sprint 5) [intent: INT-0008]: Bit-level frame synchronizer (sync-word search at bit granularity) — touches: crates/sdr-mesh/src/framesync.rs, crates/sdr-mesh/src/lib.rs
- [ ] T-029 (sprint 5) [intent: INT-0008]: RadioLink — radio-backed MeshInterface over SdrDriver + CI tests over MockSdr — touches: crates/sdr-mesh/src/radio.rs, crates/sdr-mesh/src/lib.rs, crates/sdr-mesh/Cargo.toml
- [ ] T-030 (sprint 5) [intent: INT-0008]: Live datagram over the Pluto internal loopback (zero emission) — touches: hardware test file


- [ ] T-101 (backlog) [intent: INT-0002]: Multi-vendor hardware backend (RTL-SDR/HackRF/Airspy) via SoapySDR or seify behind SdrDriver, verified with attached devices — touches: crates/sdr-hardware/**
- [ ] T-103 (backlog) [intent: INT-0006, INT-0008]: SSH-over-radio tunnel runtime — absorbed as the no-TUN mesh mode (stdin/stdout + TCP proxy loop) — touches: crates/sdr-cli/src/main.rs, crates/sdr-mesh/**
- [ ] T-104 (backlog) [intent: INT-0003]: WFM stereo pilot PLL + RDS decode (documented in Sprint 0 research but not implemented) — touches: crates/sdr-demod/src/wfm.rs
- [ ] T-105 (backlog) [intent: INT-0001]: Workspace clippy-warning cleanup (loop-index → iterators, io::Error::other, div_ceil, from_str→FromStr) across all crates — touches: crates/**
- [ ] T-106 (backlog) [intent: INT-0001]: Run `cargo fmt --all` to normalize pre-existing formatting drift across the workspace — touches: crates/**
- [ ] T-109 (backlog) [intent: INT-0002]: Close the TX buffer on `Drop` for `PlutoSdr` so a cyclic transmit cannot outlive a dropped driver without explicit teardown (test-critique C-002) — touches: crates/sdr-hardware/src/pluto.rs
- [ ] T-107 (backlog) [intent: INT-0008]: Mesh Phase B — real tun/Wintun/utun device + babeld/AREDN Babel gateway interop + AREDN IPv4-subnet/IPv6-link-local addressing — touches: crates/sdr-mesh/**
- [ ] T-108 (backlog) [intent: INT-0002, INT-0008]: Over-the-air transmit verification (band/power/antenna) — requires the user's explicit go-ahead; gate through the compliance DB — touches: crates/sdr-hardware/tests/hw_pluto.rs
