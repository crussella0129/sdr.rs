# Sprint 3 Unit Tests

- **Tested head:** `0c74137bb27f9ef7c35fc12a5c4df80e29069977`
- **Runner:** `cargo test --workspace` (75 passed, 0 failed).
- **Scope:** new `sdr-mesh` crate (Mesh Phase A). Pre-existing suites unchanged and green.

## T-019 — KISS framing (INT-0008)
- `test_kiss_roundtrip`: `encode` → `KissDecoder::push` returns the exact datagram. PASS.
- `test_kiss_byte_stuffing`: payload containing `FEND`/`FESC`/`TFEND`/`TFESC` is escaped (only the two delimiters remain bare) and recovered exactly. PASS.
- `test_kiss_stream_reassembly`: two concatenated frames + a trailing partial, fed across a split boundary, yield exactly the two complete datagrams; the partial completes later with no duplication. PASS.

## T-020 — Dual-mode compliance gate (INT-0008, reuses INT-0005)
- `test_gate_encrypted_ism_allowed`: US 915 MHz ISM + `want_encrypted` → `Allow(Encrypted)`. PASS.
- `test_gate_encrypted_refused`: US 2 m amateur (145 MHz) + `want_encrypted` → `Refuse` citing encryption prohibition. PASS.
- `test_gate_open_allowed`: US 2 m amateur, unencrypted → `Allow(Open)`. PASS.
- `test_gate_uncataloged_refused`: 50 MHz (no cataloged band) → `Refuse`. PASS.

## T-022 — README references (INT-0008)
- `test_readme_lists_aredn`: README contains `aredn/aredn` and a Babel / RFC 8966 reference. PASS.
