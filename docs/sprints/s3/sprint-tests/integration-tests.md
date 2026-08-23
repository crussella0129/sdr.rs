# Sprint 3 Integration Tests

- **Tested head:** `0c74137bb27f9ef7c35fc12a5c4df80e29069977`
- **Result:** pass.

## Mesh two-node loopback (INT-0008)
`crates/sdr-mesh/tests/loopback_it.rs` — two `MeshNode`s joined by a
deterministic in-memory `LoopbackLink` (KISS framing over `ArqTransceiver`
frames). No radio, no threads.

- `test_two_node_datagram_roundtrip`: node A (US 915 MHz ISM) sends an
  encrypted-mode datagram containing bytes that require KISS escaping (`0xC0`,
  `0xDB`); node B receives the **exact** datagram. Reverse-direction open-mode
  reply also delivered. PASS — proves INT-0008 #1 (framed + recovered end-to-end)
  and #3 (encrypted allowed on ISM).
- `test_node_refuses_encrypted_on_amateur`: on US 2 m amateur, an encrypted send
  returns an error and **emits no frame** (peer receives `None`); a subsequent
  open-mode send on the same band is delivered. PASS — proves INT-0008 #3 at the
  node boundary (the negative path).
