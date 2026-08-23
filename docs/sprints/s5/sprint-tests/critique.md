# Test Critique — Sprint 5

## Concerns

### C-001: The loopback shares one clock, so the phase search is not a receiver
- **Where:** `e2e-tests.md` / `crates/sdr-mesh/src/radio.rs` (`extract_payload`)
- **Quote:** "the digital loopback shares one clock, which is precisely why the sample-phase search suffices here and would not suffice on the air"
- **Failure mode:** intent-coverage
- **Why it matters:** A single fixed sample phase holds for the whole capture only because transmitter and receiver are the same device driven by the same clock. Between two radios the clocks drift, the correct phase slips mid-frame, and no per-capture phase choice recovers it. Reading this sprint as "the mesh receiver works" would overstate it.
- **Suggested response:** defer-with-rationale — the claim made is precisely "datagram recovered under internal loopback", INT-0008 stays `active`, and symbol-timing recovery is recorded as backlog **T-110** with the reason stated. The `radio.rs` module documentation carries the same caveat at the point of use, so a future reader meets it in the code and not only in the Book.

### C-002: The hardware test proves the digital path, not the RF chain
- **Where:** `e2e-tests.md` `hw_verify_mesh_datagram_over_radio`
- **Failure mode:** intent-coverage
- **Why it matters:** The property that makes the test safe — bypassing the RF section — also means the mixer, PA, antenna path, and any real channel impairment (noise, fading, interference) are unexercised. The DSP and framing are proven against real hardware timing and real DMA; propagation is not.
- **Suggested response:** defer-with-rationale — this is the verification depth the user chose and it is stated plainly in the report. Over-the-air is backlog T-108 pending explicit go-ahead.

### C-003: A single datagram size and one modulation setting were exercised on hardware
- **Where:** `e2e-tests.md` (10-byte datagram, 3 MSPS / 150 kHz deviation / 20 sps)
- **Failure mode:** weak-assertion
- **Why it matters:** Fragmentation behaviour, datagrams exceeding one frame, and other symbol rates are untested on hardware; the live evidence covers one point in that space.
- **Suggested response:** defer-with-rationale — the CI suite varies content (KISS-escaped bytes, mid-symbol offsets, silence) at a second parameter set, and the sprint's claim is existence of hardware transit, not coverage of the parameter space. Worth broadening when the link carries real traffic; not required for this claim.

## Confidence
proceed-with-caveats
