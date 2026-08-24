# INT-0008 — Mesh Networking & AREDN-Interoperable IP-over-Radio

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0008
- **State:** active
- **Work evidence:** [Sprint 10 build plan — T-126/T-128](../sprints/s10/sprint-plans/build-plan.md), [Sprint 3 build plan — T-019..T-022](../sprints/s3/sprint-plans/build-plan.md), [Sprint 5 build plan — T-027..T-030](../sprints/s5/sprint-plans/build-plan.md), [Sprint 6 build plan — T-031..T-034](../sprints/s6/sprint-plans/build-plan.md), [Sprint 7 build plan — T-035, T-038](../sprints/s7/sprint-plans/build-plan.md)
- **Completion evidence:** [T-035 completion](../work/completed-tasks.md#t-035-sprint-7), [T-038 completion](../work/completed-tasks.md#t-038-sprint-7)
- **Code evidence:** [sdr-mesh](../../crates/sdr-mesh/src/lib.rs)
- **Test evidence:** [Sprint 3 test report](../sprints/s3/sprint-tests/test-report.md), [Sprint 5 test report](../sprints/s5/sprint-tests/test-report.md), [Sprint 6 test report](../sprints/s6/sprint-tests/test-report.md), [Sprint 7 test report](../sprints/s7/sprint-tests/test-report.md), [Sprint 9 test report](../sprints/s9/sprint-tests/test-report.md)
- **Documentation evidence:** none
- **Review evidence:** [Sprint 10 research report](../sprints/s10/sprint-research/research-report.md) — reliability, streaming, and policy-bypass audit; [Sprint 3 research report](../sprints/s3/sprint-research/research-report.md)

## Intent
Evolve `sdr.rs`'s realized point-to-point "SSH over radio" tunnel ([INT-0006](INT-0006-packet-radio-ssh-tunnel.md))
into a **multi-hop mesh** that exposes standard IP (so SSH and any TCP/IP
service run unmodified) and can **interoperate with / gateway into an AREDN
mesh** by speaking the Babel routing protocol.

The capability comprises:
1. **IP-over-radio:** an OS `tun` interface (Linux `tun` / Windows Wintun /
   macOS `utun`) carrying IP over the existing `packet.rs` ARQ link layer
   (KISS/SLIP-style encapsulation), replacing INT-0006's 8-bit station ids with
   routable IP addresses.
2. **Mesh routing (Babel, RFC 8966):** multi-hop, loop-free, low-overhead
   routing over the SDR link — first via a **gateway** that presents the radio
   `tun` to the system `babeld` (real AREDN interop, minimal reimplementation),
   later optionally a native Rust Babel subset.
3. **Dual operating mode gated by compliance ([INT-0005](INT-0005-regulatory-band-compliance.md)):**
   an **encrypted mode** on ISM bands (SSH/WireGuard permitted) and an **open
   (unencrypted) mode** on amateur bands (AREDN-style), selected automatically
   from the tuned frequency/jurisdiction.
4. **AREDN gateway:** an SDR long-haul, low-bandwidth link that extends an AREDN
   IP mesh where 802.11 cannot reach.

Non-goals: running AREDN firmware itself (it is 802.11/OpenWrt, not SDR);
high-bandwidth/bulk transport (the SDR link is a low-bitrate mesh extension).

## Acceptance criteria
1. IP datagrams are framed over the `packet.rs` link layer and recovered without corruption between two nodes (loopback/mock), and a `tun` interface mode carries real IP on at least one supported OS.
2. Multi-hop routing works: with Babel (via `babeld` over the radio `tun`), a packet routes across an intermediate node between two IP hosts; the node interoperates with an AREDN Babel neighbor.
3. Before any encrypted transmission, the mesh consults `RegulatoryDatabase::check_compliance(..., is_encrypted=true)` and either proceeds (ISM/encrypted mode) or falls back to open mode / refuses (amateur), with a clear operator-visible decision. **(partial — `MeshNode` enforces this, but `sdr-cli tunnel` constructs `RadioLink` directly, warns on refusal, and proceeds.)**
4. Addressing uses routable IP (AREDN-compatible private IPv4 mesh subnet + IPv6 link-local for Babel control).

Verification note: criteria involving real over-the-air transmission depend on
Pluto TX (backlog T-102) and must not be claimed met on simulation alone;
software framing/routing/compliance can be proven in CI/loopback first.

## Rationale
AREDN proves the model: a layer-3 IP amateur mesh where services run
unmodified, now standardizing on Babel because it is loop-free, low-overhead,
and link-type-aware — almost a specification for a slow, lossy packet-radio
link. Reusing that model (and the standard Babel protocol) gives `sdr.rs` real
multi-hop meshing and genuine AREDN interoperability, while the dual-mode
compliance gate makes an encrypted SSH mesh legal where permitted and keeps
amateur-band operation lawful.

## Alternatives
- **Native Rust Babel implementation from the start:** rejected as the first
  step — larger effort and reimplements a standard; use `babeld` over a `tun`
  first for proven interop, add a native subset only if a zero-daemon embedded
  node is required.
- **Keep the bespoke point-to-point `ProxyCommand` byte-pipe only (T-103):**
  insufficient for meshing and non-SSH services; retained as a simple no-TUN
  fallback mode, not the target.
- **Bridge at layer 2 (Ethernet over radio):** rejected — higher overhead on a
  low-bitrate link; AREDN itself routes at layer 3.
- **Amateur-only, always-unencrypted (pure AREDN clone):** rejected — the
  dual-mode design also serves encrypted ISM operation the user wants.

## Consequences
- `tun`/Wintun/`utun` require OS drivers and elevated privileges — a
  cross-platform cost; the no-TUN `ProxyCommand` mode must remain available.
- Real over-the-air mesh depends on Pluto TX (T-102); software layers are built
  and CI/loopback-verified first.
- Babel timers/metrics need tuning for a half-duplex, low-bitrate, lossy link.
- Introduces a networking dependency surface (tun crate; optional `babeld`); it
  must stay out of the pure-DSP crates.

## Transition history
- 2026-08-22: created as `proposed` (Sprint 3 research spike). Design only; no build this sprint. Build is a future sprint the user green-lights, phased so CI-verifiable software (framing, TUN, routing/interop, compliance gate) precedes TX-gated on-air work.
- 2026-08-22: moved to `planned` for **Phase A** under T-019..T-022 (KISS IP framing, dual-mode compliance gate, MeshInterface seam + two-node loopback, README references). The real `tun` device, `babeld`/AREDN interop, and on-air work remain out of scope (Phase B/C).
- 2026-08-22: transitioned to `active` upon starting Build Phase (T-019).
- 2026-08-23: **Phase A complete (remains `active`).** New `sdr-mesh` crate delivers KISS IP-over-radio framing (T-019), the dual-mode encrypted-ISM / open-amateur compliance gate reusing INT-0005 (T-020), a `MeshInterface` seam + two-node loopback (T-021), and AREDN/Babel README references (T-022) — all CI-verified (75 workspace tests, 0 clippy errors). Acceptance criteria 1 (framing/recovery half) and 3 are met; the real `tun` device + `babeld`/AREDN interop + subnet addressing (criteria 1 tun-half, 2, 4) are carried forward to **Phase B** (backlog T-107) and on-air to **Phase C** (gated on Pluto TX, T-102). See [Sprint 3 test report](../sprints/s3/sprint-tests/test-report.md).
- 2026-08-23: **first datagram over real radio hardware in Sprint 5 (remains `active`).** A radio-backed `RadioLink` `MeshInterface` (T-029) carries datagrams as FSK-modulated IQ through any `SdrDriver`, using a bit-level frame synchronizer (T-028) and a sample-phase search whose correctness is confirmed by CRC-32. Verified live on the physical Pluto+ under internal digital loopback — RF section bypassed, maximum attenuation, DDS silenced — where a 10-byte datagram was recovered **byte-for-byte** on the first run, with device state confirmed restored. Criterion 1 is therefore met over a **real radio** (not just in-process), while the `tun` half of criterion 1 and criteria 2 and 4 remain **Phase B** (T-107), over-the-air remains gated on explicit go-ahead (T-108), and the sample-phase search must be replaced by symbol-timing recovery before a two-radio link (T-110) — so the intent stays `active`. See [Sprint 5 test report](../sprints/s5/sprint-tests/test-report.md).
- 2026-08-23: **shared-clock dependency removed in Sprint 6 (remains `active`).** `RadioLink` now recovers datagrams in a single pass through `FskTimingDemod` (frequency discriminator → Gardner timing recovery → slice); the candidate sample-phase loop is deleted. Criterion 1 no longer requires transmitter and receiver to share a clock: whole frames survive a simulated clock offset of ~±0.1%, and the live Pluto+ test still passes. Two corrections were made rather than papered over — frames now carry a 2-byte flush trailer (the timing loop's start-up lag was truncating the CRC byte), and the Sprint 6 research figure of ±0.5% drift was found not to hold at frame length, with a loop-gain sweep showing the limit is structural (backlog T-112). Drift remains verified **in simulation only** — the internal loopback shares a clock, so the hardware test cannot evidence it. `tun` (criterion 1 half), criteria 2 and 4 remain Phase B (T-107); over-the-air remains T-108. See [Sprint 6 test report](../sprints/s6/sprint-tests/test-report.md).
- 2026-08-23: **byte streams over the radio in Sprint 7 (remains `active`).** `RadioLink` now extracts *every* frame from a capture instead of decoding the first and discarding the rest (T-035) — the measured failure was 1 of 4 datagrams recovered from a burst; it is now 4 of 4 — and a multi-chunk byte stream was carried over the physical Pluto+ under the standing internal-loopback configuration, recovered byte-for-byte (T-038). The framing/recovery half of criterion 1 is therefore verified for streams, not just single datagrams.

  The intent **stays `active`, and deliberately so**: criterion 1 also requires "a `tun` interface mode carries real IP on at least one supported OS", which this sprint does not touch at all. The test critique raised this as C-001 precisely because marking the criterion realized on its framing half would repeat the over-claim Sprint 2's audit found in INT-0002, INT-0004 and INT-0006. The `tun` half plus criteria 2 and 4 remain **Phase B** (T-107); over-the-air remains **T-108**, gated on the user's explicit go-ahead.

  Two limits were measured on hardware and recorded rather than worked around: a cyclic transmit buffer holds one frame and repeats it, so streaming over a single radio is ping-pong rather than continuous (T-115); and one radio allows one open buffer, so parallel hardware tests fail with iiod EBUSY (T-116). See [Sprint 7 test report](../sprints/s7/sprint-tests/test-report.md).
- 2026-08-24: **ARQ on the receive path in Sprint 9 (remains `active`).** `RadioLink` now routes decoded frames through the ARQ state machine — address filtering, duplicate suppression, ACK consumption and generation — behind an opt-in `set_reliable`, with `service(now_ms)` as the single place acknowledgements and retransmissions reach the air. The loopback link's discarded ACK is fixed. Criterion 1's framing/recovery half is strengthened: a duplicate frame no longer delivers its payload twice.

  Still `active` for the same reason as before: criterion 1 also requires a `tun` interface carrying real IP, which this sprint does not touch (Phase B, **T-107**), and criteria 2 and 4 remain unaddressed. Receiving deliberately never transmits as a side effect, which is what let the six carried-over `radio_it` regression tests pass unchanged over a loopback that echoes every ACK.
- 2026-08-24: **Sprint 10 audit correction (remains `active`).** The review found that the CLI tunnel bypasses the mesh policy refusal path, `RadioLink` treats zero/short driver writes as successful, split-read bursts lose decoder state, and reordered reliable frames can be discarded and acknowledged. Existing loopback tests still prove their bounded paths; they do not evidence general reliable IP transport.
