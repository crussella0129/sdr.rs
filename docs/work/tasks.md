# Agent Tasks (Persistent Backlog)




- [ ] T-101 (backlog) [intent: INT-0002]: Multi-vendor hardware backend (RTL-SDR/HackRF/Airspy) via SoapySDR or seify behind SdrDriver, verified with attached devices — touches: crates/sdr-hardware/**
- [ ] T-103 (backlog) [intent: INT-0006, INT-0008]: SSH-over-radio tunnel runtime — absorbed as the no-TUN mesh mode (stdin/stdout + TCP proxy loop) — touches: crates/sdr-cli/src/main.rs, crates/sdr-mesh/**
- [ ] T-104 (backlog) [intent: INT-0003]: WFM stereo pilot PLL + RDS decode (documented in Sprint 0 research but not implemented) — touches: crates/sdr-demod/src/wfm.rs
- [ ] T-105 (backlog) [intent: INT-0001]: Workspace clippy-warning cleanup (loop-index → iterators, io::Error::other, div_ceil, from_str→FromStr) across all crates — touches: crates/**
- [ ] T-106 (backlog) [intent: INT-0001]: Run `cargo fmt --all` to normalize pre-existing formatting drift across the workspace — touches: crates/**
- [ ] T-109 (backlog) [intent: INT-0002]: Close the TX buffer on `Drop` for `PlutoSdr` so a cyclic transmit cannot outlive a dropped driver without explicit teardown (test-critique C-002) — touches: crates/sdr-hardware/src/pluto.rs
- [ ] T-112 (backlog) [intent: INT-0001]: Widen GardnerClockRecovery's drift tolerance beyond ~±0.1% at frame length, and investigate its asymmetry (a fast receiver clock is the tighter direction) — loop-gain tuning was measured not to help, so this is structural to the implementation — touches: crates/sdr-dsp/src/clock_recovery.rs
- [ ] T-111 (backlog) [intent: INT-0003]: PskDemod has no round-trip coverage — add a modulator/demodulator regression so GardnerClockRecovery's two consumers (PSK on raw IQ, FSK post-discriminator) are both protected (plan critique C-003) — touches: crates/sdr-demod/src/psk.rs
- [ ] T-110 (backlog) [intent: INT-0008]: Integrate symbol-timing recovery (sdr_dsp::clock_recovery Gardner/M&M) to replace RadioLink's sample-phase search — required for a real over-the-air link where transmitter and receiver clocks drift — touches: crates/sdr-mesh/src/radio.rs, crates/sdr-dsp/src/clock_recovery.rs
- [ ] T-107 (backlog) [intent: INT-0008]: Mesh Phase B — real tun/Wintun/utun device + babeld/AREDN Babel gateway interop + AREDN IPv4-subnet/IPv6-link-local addressing — touches: crates/sdr-mesh/**
- [ ] T-108 (backlog) [intent: INT-0002, INT-0008]: Over-the-air transmit verification (band/power/antenna) — requires the user's explicit go-ahead; gate through the compliance DB — touches: crates/sdr-hardware/tests/hw_pluto.rs
