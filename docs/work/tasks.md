# Agent Tasks (Persistent Backlog)

- [ ] T-101 (backlog) [intent: INT-0002]: Multi-vendor hardware backend (RTL-SDR/HackRF/Airspy) via SoapySDR or seify behind SdrDriver, verified with attached devices — touches: crates/sdr-hardware/**
- [ ] T-102 (backlog) [intent: INT-0002]: PlutoSDR TX path over iiod (WRITEBUF to cf-ad9361-dds-core-lpc) — touches: crates/sdr-hardware/src/iiod.rs, crates/sdr-hardware/src/pluto.rs
- [ ] T-103 (backlog) [intent: INT-0006]: SSH-over-radio tunnel runtime (stdin/stdout + TCP proxy loop wiring StreamTunnel to `ssh -o ProxyCommand`) — touches: crates/sdr-cli/src/main.rs, crates/sdr-protocols/src/tunnel.rs
- [ ] T-104 (backlog) [intent: INT-0003]: WFM stereo pilot PLL + RDS decode (documented in Sprint 0 research but not implemented) — touches: crates/sdr-demod/src/wfm.rs
- [ ] T-105 (backlog) [intent: INT-0001]: Workspace clippy-warning cleanup (loop-index → iterators, io::Error::other, div_ceil, from_str→FromStr) across sdr-core/sdr-dsp/sdr-demod/sdr-protocols/sdr-spectrum/sdr-hardware — touches: crates/**
- [ ] T-106 (backlog) [intent: INT-0001]: Run `cargo fmt --all` to normalize pre-existing formatting drift across the workspace — touches: crates/**
