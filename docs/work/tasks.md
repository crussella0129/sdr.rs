# Agent Tasks (Persistent Backlog)

- [ ] T-043 (sprint 9) [intent: INT-0006]: Seeded lossy-channel model + reliability tests at 10%/30% loss; delete the misleading test_arq_retransmission_lossy_channel — touches: crates/sdr-protocols/src/lib.rs
- [ ] T-044 (sprint 9) [intent: INT-0008, INT-0006]: Run ARQ on the mesh receive paths — RadioLink through process_rx_frame, transmit ACKs, service(now_ms); fix node.rs discarded _ack — touches: crates/sdr-mesh/src/radio.rs, crates/sdr-mesh/src/node.rs, crates/sdr-mesh/tests/radio_it.rs




- [ ] T-116 (backlog) [intent: INT-0002]: Serialize physical device access — hardware tests share one radio and one open buffer, so a parallel run fails with iiod errno 16 (EBUSY); a process-wide guard in the driver would replace the doc-comment --test-threads=1 requirement (test critique C-004) — touches: crates/sdr-hardware/src/pluto.rs
- [ ] T-115 (backlog) [intent: INT-0006, INT-0008]: Continuous streaming over a single radio — a cyclic TX buffer holds one frame and repeats it, forcing ping-pong and capping throughput; needs either timed non-cyclic transmission or a second radio — touches: crates/sdr-hardware/src/pluto.rs, crates/sdr-mesh/src/radio.rs
- [ ] T-113 (backlog) [intent: INT-0006]: ARQ reliability for streams — wire sequence checking, duplicate suppression and retransmission into RadioLink's receive path so a dropped frame does not silently truncate a stream on a lossy channel (plan critique C-002) — touches: crates/sdr-mesh/src/radio.rs, crates/sdr-protocols/src/packet.rs
- [ ] T-114 (backlog) [intent: INT-0006]: Verify a complete SSH session over the bridge — requires an SSH server; sshd is not installed on this machine — touches: crates/sdr-cli/tests/ssh_tunnel_it.rs
- [ ] T-101 (backlog) [intent: INT-0002]: Multi-vendor hardware backend (RTL-SDR/HackRF/Airspy) via SoapySDR or seify behind SdrDriver, verified with attached devices — touches: crates/sdr-hardware/**
- [ ] T-103 (backlog) [intent: INT-0006, INT-0008]: SSH-over-radio tunnel runtime — absorbed as the no-TUN mesh mode (stdin/stdout + TCP proxy loop) — touches: crates/sdr-cli/src/main.rs, crates/sdr-mesh/**
- [ ] T-104 (backlog) [intent: INT-0003]: WFM stereo pilot PLL + RDS decode (documented in Sprint 0 research but not implemented) — touches: crates/sdr-demod/src/wfm.rs
- [ ] T-105 (backlog) [intent: INT-0001]: Workspace clippy-warning cleanup (loop-index → iterators, io::Error::other, div_ceil, from_str→FromStr) across all crates — touches: crates/**
- [ ] T-106 (backlog) [intent: INT-0001]: Run `cargo fmt --all` to normalize pre-existing formatting drift across the workspace — touches: crates/**
- [ ] T-109 (backlog) [intent: INT-0002]: Close the TX buffer on `Drop` for `PlutoSdr` so a cyclic transmit cannot outlive a dropped driver without explicit teardown (test-critique C-002) — touches: crates/sdr-hardware/src/pluto.rs
- [ ] T-112 (backlog) [intent: INT-0001]: Widen GardnerClockRecovery's drift tolerance beyond ~±0.1% at frame length, and investigate its asymmetry (a fast receiver clock is the tighter direction) — loop-gain tuning was measured not to help, so this is structural to the implementation — touches: crates/sdr-dsp/src/clock_recovery.rs
- [ ] T-111 (backlog) [intent: INT-0003]: PskDemod has no round-trip coverage — add a modulator/demodulator regression so GardnerClockRecovery's two consumers (PSK on raw IQ, FSK post-discriminator) are both protected (plan critique C-003) — touches: crates/sdr-demod/src/psk.rs
- [ ] T-107 (backlog) [intent: INT-0008]: Mesh Phase B — real tun/Wintun/utun device + babeld/AREDN Babel gateway interop + AREDN IPv4-subnet/IPv6-link-local addressing — touches: crates/sdr-mesh/**
- [ ] T-108 (backlog) [intent: INT-0002, INT-0008]: Over-the-air transmit verification (band/power/antenna) — requires the user's explicit go-ahead; gate through the compliance DB — touches: crates/sdr-hardware/tests/hw_pluto.rs
