# INT-0008 — Mesh Networking & AREDN-Interoperable IP-over-Radio

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0008
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** none
- **Review evidence:** [Sprint 3 research report](../sprints/s3/sprint-research/research-report.md)

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
3. Before any encrypted transmission, the mesh consults `RegulatoryDatabase::check_compliance(..., is_encrypted=true)` and either proceeds (ISM/encrypted mode) or falls back to open mode / refuses (amateur), with a clear operator-visible decision.
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
