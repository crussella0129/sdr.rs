# Test Critique — Sprint 7

## Concerns

### C-001: INT-0008 criterion 1 is only half covered
- **Where:** `test-plan.md` traceability map / `INT-0008` Acceptance criteria #1
- **Quote:** "IP datagrams are framed over the `packet.rs` link layer and recovered without corruption between two nodes (loopback/mock), **and a `tun` interface mode carries real IP on at least one supported OS**."
- **Failure mode:** intent-coverage
- **Why it matters:** The plan maps criterion 1 to `test_radiolink_recovers_burst_of_datagrams`, `test_radiolink_oversized_frame_not_truncated` and `hw_verify_stream_over_radio`. Every one of those exercises the *framing/recovery* half. Nothing in this sprint creates a `tun` interface or carries real IP — that is Phase B (backlog T-107). If the loop phase reads "criterion 1 verified" off this report, INT-0008 gets marked realized on half a criterion, which is the exact failure Sprint 2's audit caught in INT-0002/0004/0006.
- **Suggested response:** tighten-assertion — the report must state criterion 1 as **partially** verified, name the unmet half, and the loop phase must leave INT-0008 `active`.

### C-002: the "graceful skip" EARS clause is never executed
- **Where:** `test-plan.md` traceability map, T-038
- **Quote:** "#4 graceful skip | T-038 / WHEN ssh unavailable THEN skip cleanly | test_ssh_client_banner_traverses_bridge"
- **Failure mode:** negative-path
- **Why it matters:** `ssh` is present on this machine, so `ssh_available()` returns true and the skip branch never runs in any recorded result. The clause is claimed as verified by a test that provably did not take that path. The branch is three lines and low-risk, but "verified" is the wrong word for code that never executed.
- **Suggested response:** defer-with-rationale — record it as *not executed* rather than verified. Inverting the helper to force the branch would test the test, not the product; the real check is a machine without `ssh`, which CI on a bare container would provide.

### C-003: a fixed sleep gates the TCP tunnel test
- **Where:** `crates/sdr-cli/tests/ssh_tunnel_it.rs::test_ssh_client_banner_traverses_bridge`
- **Quote:** "`std::thread::sleep(Duration::from_millis(750));` // Give the listener a moment to bind before connecting."
- **Failure mode:** flake-risk
- **Why it matters:** 750 ms is a guess about how long `sdr-cli` takes to bind. On a loaded or cold-cache machine the bind can lose that race; `ssh` then gets connection-refused and the test fails for a reason unrelated to the radio path. The ProxyCommand test has no such race (ssh spawns the child itself), which makes the asymmetry avoidable rather than inherent.
- **Suggested response:** tighten-assertion — poll for the port accepting connections instead of sleeping a fixed interval.

### C-004: nothing enforces the serial requirement on the hardware tests
- **Where:** `e2e-tests.md` §2 / `crates/sdr-mesh/tests/hw_radio.rs`
- **Quote:** "Running the two hardware tests in parallel fails with iiod `OPEN failed: errno 16` (EBUSY). `--test-threads=1` is required"
- **Failure mode:** flake-risk
- **Why it matters:** The requirement lives in a doc comment. Anyone running the documented `--ignored` command without `--test-threads=1` gets a failure that looks like a radio or driver defect rather than test contention — which is exactly how it presented the first time. A doc comment is not a guard.
- **Suggested response:** defer-with-rationale — a process-wide mutex around device access would fix it properly and belongs with the driver, not this sprint's scope. Record as backlog; the failure mode is now documented in the one place a runner will look.

### C-005: stream ordering is proven only on a lossless channel
- **Where:** `integration-tests.md` T-036 / `INT-0006` criterion 4
- **Quote:** "WHEN a stream longer than the MTU is sent THEN it SHALL be reassembled in order"
- **Failure mode:** weak-assertion
- **Why it matters:** `MockSdr` loopback and the Pluto internal loopback are both lossless, so "reassembled in order byte-for-byte" is demonstrated where reordering and loss cannot occur. The receive path has no ARQ, so on a lossy channel a dropped frame truncates the stream silently. The passing test is true but weaker than its wording suggests to a casual reader.
- **Suggested response:** defer-with-rationale — already backlog T-113; the report must say the guarantee is demonstrated on a lossless channel, not asserted in general.

## Resolutions (primary agent)

Re-run after the evidence changed. Each concern is answered below; the verdict
is unchanged.

| Concern | Response | Outcome |
|---|---|---|
| C-001 INT-0008 #1 half covered | tighten-assertion | **Accepted.** `test-report.md` records criterion 1 as *partially* verified and names the unmet `tun` half. INT-0008 stays `active` in the loop phase; only the framing/recovery half gains Test evidence. |
| C-002 skip branch never executed | defer-with-rationale | **Accepted.** The report lists the clause as *not executed*, not verified. Forcing the branch would test the harness rather than the product. |
| C-003 fixed sleep before connect | tighten-assertion | **Fixed.** `run_tcp_tunnel` now prints `Listening on TCP port {port}` on stderr when it binds, and the test waits for that line. Runtime fell from ~20 s to ~0.35 s and passed 3/3 consecutive runs. Two probes were tried and rejected on evidence: a connect-probe consumes the single accept `ssh` needs, and a bind-probe never fails on Windows, which permits rebinding a listening port. Both are recorded in the helper's doc comment. The readiness line also fixes a real usability gap — `--listen` previously gave the operator no confirmation it was ready. |
| C-004 serial requirement unenforced | defer-with-rationale | **Accepted, backlog T-116.** A process-wide guard belongs in the driver, not this sprint. The requirement is documented where a runner will meet it. |
| C-005 lossless-channel guarantee | defer-with-rationale | **Accepted.** The report states the ordering guarantee is demonstrated on a lossless channel, not in general. Already backlog T-113. |

Post-resolution verification at head `2d77c1b`+: `cargo test --workspace` green
(32 suites, 0 failed), `cargo clippy --workspace --all-targets` 0 errors.

## Confidence
proceed-with-caveats
