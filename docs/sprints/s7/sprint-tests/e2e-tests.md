# Sprint 7 — End-to-End Test Results

- **Tested head:** `91124b01d080d00ca54fc1b3990487806a79ce76`
- **Date:** 2026-08-23

Two E2E routes ran: a real OpenSSH client in the automated suite, and live
hardware performed by the agent.

## 1. Real OpenSSH client over the radio bridge (INT-0006 criterion 4)

`crates/sdr-cli/tests/ssh_tunnel_it.rs` — **2 passed** (1.07 s):
`test_ssh_proxycommand_exchanges_version` and
`test_ssh_client_banner_traverses_bridge`.

A genuine `ssh` process (OpenSSH_10.3, the system binary) connects through
`sdr-cli tunnel` and completes the **SSH version exchange** across the link.
From its `-v` trace:

```text
debug1: Local version string SSH-2.0-OpenSSH_10.3
debug1: Remote protocol version 2.0, remote software version OpenSSH_10.3
```

Those bytes made a full round trip through KISS framing, CRC-32, FSK modulation,
demodulation and symbol-timing recovery.

**What this proves:** a real SSH client's protocol bytes traverse the radio
bridge — the capability INT-0006 is named for.

**What it does not prove:** a working SSH session. There is no `sshd` on this
machine, so the "remote" version string the client receives is **its own**,
echoed by the loopback. Key exchange, authentication and a shell are unverified
(backlog T-114). The test's own doc comment states this, so a future reader
cannot mistake it for a session test.

### Both transports covered

Criterion 4 names the `ProxyCommand` contract explicitly, so it is tested
explicitly:

| Test | Transport | Trace marker |
|---|---|---|
| `test_ssh_proxycommand_exchanges_version` | `ssh -o ProxyCommand="sdr-cli tunnel --stdio …"` — ssh spawns the tunnel and speaks over its stdin/stdout, no socket in the path | `Executing proxy command: exec "…\sdr-cli.exe" tunnel --stdio --driver mock` |
| `test_ssh_client_banner_traverses_bridge` | `--listen <port>` TCP proxy socket, `ssh -p <port> user@127.0.0.1` | `Remote protocol version 2.0` |

During the build phase the ProxyCommand form appeared not to work on Windows
and the TCP mode was written as a substitute. Re-tested once the runtime was
complete, ProxyCommand works: the earlier failure was the not-yet-working
tunnel, not Windows shell semantics. Rather than amend the criterion to match
the test that happened to pass, the test the criterion names was added. Both
are retained — the criterion requires ProxyCommand, and TCP is the mode a user
running a persistent bridge would reach for.

The client is expected to stall after the version exchange: negotiating key
exchange with its own echo, it can never finish. The test therefore runs it
under a deadline with a kill guard rather than `Command::output()`, which would
block forever.

Skips cleanly (with a printed notice) where no `ssh` binary exists.

## 2. Live hardware — byte stream over the Pluto+ (INT-0008 criterion 1)

`crates/sdr-mesh/tests/hw_radio.rs` — **2 passed**, agent-run against the
physical Pluto+ at `192.168.2.1`:

```bash
cargo test -p sdr-mesh --test hw_radio -- --ignored --nocapture --test-threads=1
```

```text
test hw_verify_mesh_datagram_over_radio ... sent 10 bytes through the Pluto+
  (internal loopback, max attenuation); recovered Some(10)  ok
test hw_verify_stream_over_radio ... sent 87 bytes as 2 datagrams;
  recovered 87 bytes  ok
```

`hw_verify_stream_over_radio` satisfies *WHEN a byte stream is carried over the
Pluto+ THEN it SHALL be recovered byte-for-byte*: 87 bytes spanning 2 datagrams,
asserted equal to the source.

**Configuration** (the standing arrangement for live testing): AD9361 internal
digital loopback, transmitter held at maximum attenuation (−89.75 dB), DDS tone
generators silenced, device state saved and restored before any assertion runs
so a failure cannot strand the radio.

### Limits measured this sprint

- **A cyclic transmit buffer holds one frame and repeats it.** Chunks must
  therefore ping-pong (write one, read it back, then the next) rather than being
  written as a burst, and the repeats must be discarded rather than appended to
  the stream. Both are properties of driving a single radio's cyclic DMA buffer,
  not defects in `StreamBridge`; a two-radio link would stream continuously.
  Recorded as backlog **T-115** rather than worked around silently.
- **One radio allows one open buffer.** Running the two hardware tests in
  parallel fails with iiod `OPEN failed: errno 16` (EBUSY). `--test-threads=1`
  is required and is now documented in the test module.

### Defect found by this test, fixed in T-038

The first run **hung**. `StreamBridge::pump` drained with an unbounded
`while let Some(..)`, which never returns against a transmitter repeating a
frame from a cyclic buffer. This was a live hang, not a test artifact — a real
deployment against a continuously-transmitting peer would have hit it. Fixed by
bounding the drain at `PUMP_MAX_DATAGRAMS`.

## Not yet possible — named unlockers

| Not verified | Unlocked by |
|---|---|
| A complete SSH session (key exchange, auth, shell) | An SSH **server**; `sshd` is not installed here — backlog **T-114** |
| Reliability under packet loss | ARQ sequence checking and retransmission on receive; CRC-32 currently drops a corrupt frame with no retransmit, so a lossy channel would silently truncate a stream — backlog **T-113** |
| Continuous (non-ping-pong) streaming | Timed non-cyclic transmission or a second radio — backlog **T-115** |
| Two separate radios; any on-air operation | A second device, plus **T-108**, which requires the user's explicit go-ahead |
