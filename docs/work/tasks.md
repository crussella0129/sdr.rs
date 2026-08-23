# Agent Tasks (Persistent Backlog)

- [ ] T-020 (sprint 3) [intent: INT-0008]: Dual-mode compliance gate (encrypted-ISM / open-amateur) reusing RegulatoryDatabase — touches: crates/sdr-mesh/src/policy.rs, crates/sdr-mesh/src/lib.rs
- [ ] T-021 (sprint 3) [intent: INT-0008]: MeshInterface seam + LoopbackLink + MeshNode two-node loopback — touches: crates/sdr-mesh/src/node.rs, crates/sdr-mesh/src/lib.rs, crates/sdr-mesh/tests/loopback_it.rs
- [ ] T-022 (sprint 3) [intent: INT-0008]: Add AREDN + Babel (RFC 8966) to README references — touches: README.md

- [ ] T-101 (backlog) [intent: INT-0002]: Multi-vendor hardware backend (RTL-SDR/HackRF/Airspy) via SoapySDR or seify behind SdrDriver, verified with attached devices — touches: crates/sdr-hardware/**
- [ ] T-102 (backlog) [intent: INT-0002]: PlutoSDR TX path over iiod (WRITEBUF to cf-ad9361-dds-core-lpc) — touches: crates/sdr-hardware/src/iiod.rs, crates/sdr-hardware/src/pluto.rs
- [ ] T-103 (backlog) [intent: INT-0006, INT-0008]: SSH-over-radio tunnel runtime — absorbed as the no-TUN mesh mode (stdin/stdout + TCP proxy loop) — touches: crates/sdr-cli/src/main.rs, crates/sdr-mesh/**
- [ ] T-104 (backlog) [intent: INT-0003]: WFM stereo pilot PLL + RDS decode (documented in Sprint 0 research but not implemented) — touches: crates/sdr-demod/src/wfm.rs
- [ ] T-105 (backlog) [intent: INT-0001]: Workspace clippy-warning cleanup (loop-index → iterators, io::Error::other, div_ceil, from_str→FromStr) across all crates — touches: crates/**
- [ ] T-106 (backlog) [intent: INT-0001]: Run `cargo fmt --all` to normalize pre-existing formatting drift across the workspace — touches: crates/**
- [ ] T-107 (backlog) [intent: INT-0008]: Mesh Phase B — real tun/Wintun/utun device + babeld/AREDN Babel gateway interop + AREDN IPv4-subnet/IPv6-link-local addressing — touches: crates/sdr-mesh/**
