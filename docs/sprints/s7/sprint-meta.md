# Sprint 7 Meta

- **Sprint number:** 7
- **Book schema version:** 2
- **Start timestamp:** 2026-08-23T20:20:14Z
- **End timestamp:** 2026-08-23T21:07:29Z
- **Model:** claude-opus-5
- **Exit status:** success
- **Token count:** (filled at Loop Phase if observable)
- **Summary:** Make "SSH over radio" runnable: multi-frame capture recovery, a StreamBridge chunking byte streams to MTU-sized datagrams, and a working `sdr-cli tunnel` (stdio ProxyCommand + TCP) verified with a real OpenSSH client and on the Pluto+.
- **Intents:** [INT-0006](../../intents/INT-0006-packet-radio-ssh-tunnel.md) (re-opened to active; criterion 4 restated and now built), [INT-0008](../../intents/INT-0008-mesh-networking-aredn.md) (active; multi-frame recovery + stream consumer)
- **Completion evidence:** INT-0006 realized: a real OpenSSH client completes the SSH version exchange over the radio link via stdio/ProxyCommand/TCP, and an 87-byte stream was recovered byte-for-byte on the physical Pluto+ under internal loopback; INT-0008 stays active (tun half of criterion 1 untouched). Workspace green (32 suites), clippy 0 errors, critique proceed-with-caveats.
