# Sprint 3 End-to-End Tests

- **Tested head:** `0c74137bb27f9ef7c35fc12a5c4df80e29069977`
- **Status:** possible (in-process) for Phase A.

The Phase A end-to-end datagram path is the two-node loopback in
`crates/sdr-mesh/tests/loopback_it.rs` (see integration-tests.md): an IP-style
datagram travels application → KISS framing → `ArqTransceiver` link → peer →
reassembly → application, with the compliance gate enforced at the sending node.
This runs with no radio and no privileges.

## Not-yet-possible (named unlockers)
- **Real `tun` interface** carrying host IP, and **`babeld` / AREDN multi-hop
  interop** → **Phase B** (INT-0008 criterion 2; needs OS privileges and an
  AREDN Babel neighbor). The `MeshInterface` trait is the seam.
- **On-air two-radio / multi-hop** → **Phase C**, gated on PlutoSDR TX
  (backlog T-102).
