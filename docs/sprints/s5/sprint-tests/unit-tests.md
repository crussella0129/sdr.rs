# Sprint 5 Unit Tests

- **Tested head:** `19bd39e5e1ed338ee7282695e8c38e4ddc8905a8`
- **Runner:** `cargo test --workspace` — 95 passed, 0 failed, 4 ignored (hardware).
- **Scope:** FSK round-trip regression and the mesh bit-level frame synchronizer.

## T-027 — FSK modulator ↔ demodulator (INT-0006)
- `test_fsk_roundtrip_bit_exact`: a payload containing the packet preamble and sync word (`AA AA D3 91 01 02 FF 00 5A`) modulated at 1 MSPS / 100 kHz deviation / 10 sps and demodulated sample-aligned recovers **all 72 bits exactly**, and the IQ length is exactly one symbol per bit. PASS.
- `test_fsk_roundtrip_offset_degrades`: demodulating the same IQ from a mid-symbol offset yields a non-zero bit-error count — the documented reason the mesh receiver searches sample phases. PASS.

## T-028 — Bit-level frame synchronizer (INT-0008)
- `test_framesync_packs_msb_first`: `bits_to_bytes`/`bytes_to_bits` round-trip MSB-first, matching the modulator's bit order. PASS.
- `test_framesync_finds_sync_at_bit_offsets`: for **every** bit offset 0..8, a stream carrying the sync word yields bytes beginning at `D3 91 D3 91` with the payload intact behind it. PASS.
- `test_framesync_absent_returns_none`: an all-zero stream and an over-short stream both yield `None`. PASS.
