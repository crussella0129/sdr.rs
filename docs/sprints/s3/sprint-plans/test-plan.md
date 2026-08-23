Finalized - DO NOT EDIT

# Sprint 3 Test Plan

## Intent Traceability
| Intent | Acceptance criterion | Build task / EARS clause | Verification |
|--------|----------------------|--------------------------|--------------|
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 datagram framed + recovered | T-019 / WHEN KISS-encoded THEN decode returns exact bytes | test_kiss_roundtrip, test_kiss_byte_stuffing |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 stream reassembly | T-019 / WHEN partial/multiple frames THEN each datagram once | test_kiss_stream_reassembly |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #3 encrypted allowed on ISM | T-020 / WHEN want_encrypted + Compliant THEN Allow(Encrypted) | test_gate_encrypted_ism_allowed |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #3 encrypted refused | T-020 / WHEN want_encrypted + NonCompliant THEN Refuse | test_gate_encrypted_refused, test_gate_uncataloged_refused |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #3 open allowed | T-020 / WHEN !want_encrypted + Compliant THEN Allow(Open) | test_gate_open_allowed |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #1 end-to-end transit | T-021 / WHEN A sends THEN B receives identical datagram | test_two_node_datagram_roundtrip |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | #3 gate enforced at node | T-021 / WHEN encrypted on prohibited freq THEN refuse, no frame | test_node_refuses_encrypted_on_amateur |
| [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) | rationale (references) | T-022 / WHEN README renders THEN AREDN + RFC 8966 present | test_readme_lists_aredn |

## Unit Tests

### T-019 unit tests
- **Intent:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- `test_kiss_roundtrip`: encode(datagram) → decode → identical bytes.
- `test_kiss_byte_stuffing`: payload containing `0xC0` and `0xDB` is escaped and recovered exactly.
- `test_kiss_stream_reassembly`: a buffer with two concatenated frames and a trailing partial frame yields exactly two datagrams; the partial is retained.

### T-020 unit tests
- **Intent:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- `test_gate_encrypted_ism_allowed`: encrypted on a cataloged ISM band (encryption permitted) → `Allow(Encrypted)`.
- `test_gate_encrypted_refused`: encrypted on a cataloged band where encryption is prohibited → `Refuse` with reasons.
- `test_gate_open_allowed`: unencrypted on a cataloged compliant band → `Allow(Open)`.
- `test_gate_uncataloged_refused`: a frequency in no cataloged band → `Refuse`.

### T-021 unit tests
- **Intent:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- (covered by the integration tests below; `MeshNode`/`LoopbackLink` are exercised end-to-end.)

### T-022 unit tests
- **Intent:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- `test_readme_lists_aredn`: README content contains `aredn/aredn` and an RFC 8966 / Babel reference.

## Integration Tests
### Mesh loopback (INT-0008)
- **Intents:** [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md)
- `test_two_node_datagram_roundtrip`: node A → `LoopbackLink` → node B delivers the identical IP datagram (KISS over `ArqTransceiver`).
- `test_node_refuses_encrypted_on_amateur`: `MeshNode::send(datagram, want_encrypted=true)` on an amateur/encryption-prohibited frequency returns a refusal and emits no frame.

## End-to-End Tests
- **Status:** possible (in-process) for Phase A — the two-node loopback is the end-to-end datagram path with no radio/privileges.
- **Not-yet-possible (named unlockers):**
  - Real `tun` interface carrying host IP, and `babeld`/AREDN multi-hop interop → **Phase B** (INT-0008 criterion 2; needs OS privileges + an AREDN neighbor).
  - On-air two-radio / multi-hop → **Phase C**, gated on PlutoSDR TX (backlog T-102).
