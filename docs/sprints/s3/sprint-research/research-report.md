# Sprint 3 Research Report — Mesh networking & AREDN interoperability (spike)

Research-only spike (no build this sprint): study AREDN and its Babel routing
model, and produce a concrete design for evolving `sdr.rs`'s point-to-point
"SSH over radio" tunnel into a multi-hop, AREDN-interoperable mesh with an
IP-over-radio interface. The build is a future sprint the user green-lights.

## Intents Reviewed
- [INT-0008](../../../intents/INT-0008-mesh-networking-aredn.md) — **created** (`proposed`, design-only): mesh routing + IP-over-radio + AREDN gateway interop. Captures the design this spike produces.
- [INT-0006](../../../intents/INT-0006-packet-radio-ssh-tunnel.md) — **selected** (context): the realized point-to-point packet link/ARQ/tunnel that INT-0008 builds on.
- [INT-0005](../../../intents/INT-0005-regulatory-band-compliance.md) — **selected** (constraint): encryption-on-amateur-bands rules directly constrain a legal "SSH mesh".

## 1. Sprint Goal
Deliver an evidence-backed design (not code) for a mesh-networking capability
that (a) turns `sdr.rs`'s point-to-point tunnel into a multi-hop mesh, (b)
exposes standard IP so SSH and any TCP/IP service run unmodified, and (c) can
interoperate with / gateway into an AREDN mesh. Identify the routing protocol,
the IP/framing model, addressing, the regulatory constraint, and a phased build
plan with its hardware dependency (Pluto TX).

## 2. Existing Code Survey
| File | Relevance | Notes |
|------|-----------|-------|
| crates/sdr-protocols/src/tunnel.rs | high | `StreamTunnel`: packetize/ingest a byte stream over ARQ frames. Point-to-point, 8-bit local/remote addresses, MTU 32–1024. The seam a mesh/IP layer sits above. |
| crates/sdr-protocols/src/packet.rs | high | `ArqTransceiver`: preamble/sync/CRC-32/sequence/ARQ. This is the RF **link layer** that would carry IP packets (KISS/SLIP-style), not just an SSH byte pipe. |
| crates/sdr-cli/src/main.rs (`Tunnel`) | high | Current `tunnel` command constructs the tunnel and prints "Ready for ProxyCommand" but has **no runtime loop** (backlog T-103). A mesh design supersedes/absorbs this. |
| crates/sdr-core/src/compliance.rs | high | `check_compliance(..., is_encrypted)` already encodes that encrypted payloads are prohibited on amateur bands and permitted on ISM. The mesh design must consult this. |
| crates/sdr-hardware/src/pluto.rs | med | RX is real; **TX is a stub** — on-air mesh is gated on Pluto TX (backlog T-102). |
| docs/intents/INT-0006-packet-radio-ssh-tunnel.md | high | Realized point-to-point tunnel; INT-0008 extends its addressing (8-bit → IP) and topology (P2P → multi-hop). |

## 3. External Sources
- [AREDN firmware (aredn/aredn)](https://github.com/aredn/aredn) — OpenWrt-based firmware turning commodity 802.11 hardware (Ubiquiti/MikroTik/GL.iNet/TP-Link) into a self-configuring amateur IP mesh; DtDLink VLANs carry standard IP so SSH/services run unmodified. **802.11 WiFi, not SDR.**
- [AREDN Babel routing guide](http://docs.arednmesh.org/en/latest/arednHow-toGuides/babel.html) — AREDN is migrating OLSR→Babel: loop-free, reactive (sends changes when needed, low overhead), **link-type-aware (wired/wireless/tunneled)**, layer-3; outperformed OLSR on stability/overhead. Tunnels moving to WireGuard.
- [Babel routing protocol — RFC 8966](https://datatracker.ietf.org/doc/html/rfc8966) — IETF standard distance-vector protocol with feasibility conditions (loop-freedom), ETX/RTT metrics, IPv6 link-local control traffic; designed to be robust on lossy/wireless links. The interop target if `sdr.rs` is to route with AREDN.
- [babeld reference implementation (jech/babeld)](https://github.com/jech/babeld) — small C daemon; can run over arbitrary interfaces. A gateway option: present the SDR radio link as a Linux `tun` interface that `babeld` routes over, rather than reimplementing Babel.
- [WireGuard](https://www.wireguard.com/) — AREDN's chosen tunnel transport (encrypted point-to-point over IP). Relevant to how an `sdr.rs` gateway would carry mesh traffic across an IP backhaul, and to the encryption/legality boundary.

## 4. Design — sdr.rs mesh networking (the spike's deliverable)

### 4.1 The honest framing: adopt & interoperate, don't "run AREDN"
AREDN is WiFi-hardware firmware; `sdr.rs` cannot run it. The value is its
proven **model**: a layer-3 IP mesh with Babel routing where services run
unmodified. `sdr.rs` should (a) **adopt** that model over its SDR packet link
and (b) **interoperate** by speaking Babel so an SDR link can extend an AREDN
mesh where WiFi cannot reach (long-haul HF/VHF/UHF, low bandwidth).

### 4.2 Layering (target architecture)
```
  SSH / any IP app
        │  (unmodified)
  ┌─────▼───────────────┐   L3  IP (v4 mesh subnet + IPv6 link-local)
  │  TUN virtual iface  │        addresses replace 8-bit station ids
  └─────▲───────────────┘
        │  IP packets
  ┌─────▼───────────────┐   Babel routing (RFC 8966) — multi-hop, loop-free,
  │  Mesh routing layer │        link-type/metric aware; AREDN-interoperable
  └─────▲───────────────┘
        │  framed IP datagrams (KISS/SLIP-style)
  ┌─────▼───────────────┐   L2  existing packet.rs: preamble/CRC-32/seq/ARQ
  │  RF link layer      │        (INT-0006) — now carries IP, not an SSH pipe
  └─────▲───────────────┘
        │  IQ bursts (GFSK/FSK)
   SdrDriver TX/RX  ── needs Pluto TX (T-102) for on-air
```

### 4.3 Key design decisions
1. **IP interface via TUN.** Expose a `tun` device so the OS routes IP over the
   radio; SSH/scp/any service then "just work." *Constraint:* TUN needs OS
   support and elevated privileges (Linux `tun`, Windows Wintun, macOS `utun`)
   — a real cross-platform cost. *Fallback for SSH-only, no privileges:* keep
   the `ProxyCommand` byte-pipe (T-103) as a simpler mode; TUN is the full mode.
2. **Routing = Babel (RFC 8966).** Two viable strategies:
   - **(a) Gateway/bridge (recommended first):** present the radio link as a
     `tun` interface and let the system's `babeld` route over it. Lowest effort,
     immediate real AREDN interop, no protocol reimplementation.
   - **(b) Native Rust Babel subset:** implement enough of Babel for embedded/
     no-daemon deployments. Larger effort; do only if a zero-dependency,
     cross-platform node is required.
3. **Addressing:** replace INT-0006's 8-bit station ids with IP — an AREDN-style
   private IPv4 mesh subnet plus IPv6 link-local for Babel control, so nodes are
   routable and AREDN-compatible.
4. **Link layer reuse:** `packet.rs` ARQ frames become the carrier for IP
   datagrams (KISS/SLIP-style encapsulation), not a bespoke SSH stream.

### 4.4 The regulatory constraint → two operating modes (ties to INT-0005)
Encryption (SSH/TLS/WireGuard) is **prohibited on amateur bands** but
**permitted on license-free ISM bands** (US 902–928 MHz, 2.4 GHz, etc.). AREDN
runs on amateur bands and therefore does **not** encrypt its RF. Rather than
pick one, the mesh should support **both modes**, chosen by band and gated by
the existing compliance DB:

- **Encrypted mode (ISM):** full encrypted mesh — SSH/WireGuard payloads ride
  the link. Allowed only where `RegulatoryDatabase::check_compliance(jur, freq,
  power, is_encrypted=true)` returns `Compliant` (ISM bands).
- **Open mode (amateur):** unencrypted mesh, AREDN-style, for the amateur
  allocations (more spectrum, higher power, long-haul). Carries non-encrypted
  services, telemetry, and authenticated-but-cleartext sessions; SSH in this
  mode is refused (or downgraded to a documented cleartext transport).

The mesh layer **must call `check_compliance(..., is_encrypted)` before every
encrypted transmission** and select the mode (or refuse) automatically from the
tuned frequency and jurisdiction — a direct reuse of Sprint 1's compliance
engine. This dual-mode gate is the single most important correctness constraint
of the design.

## 5. Risks, Unknowns, Dependencies
- **Dependency — Pluto TX (T-102).** No real over-the-air mesh without transmit; RX is real but TX is a stub. The mesh **software** (framing, TUN, routing/interop) can be built and loopback/mock-tested first; on-air is a later phase.
- **Risk — TUN privileges/portability.** `tun`/Wintun/`utun` need drivers and elevation; conflicts with INT-0006's "no kernel privileges" alternative. Mitigation: offer the ProxyCommand mode (no TUN) and the TUN mode as a documented, privileged option.
- **Risk — Babel over a half-duplex, low-bitrate, lossy link.** Timer/metric tuning (hello/update intervals, ETX vs RTT) matters; Babel is designed for this but needs configuration. Mitigation: start with the `babeld` gateway (proven) over a `tun`, measure, then decide on a native subset.
- **Constraint — encryption legality (§4.4).** Non-negotiable; drives band choice and a compliance gate.
- **Unknown — throughput.** Packet-radio bitrates (kbps) vs. AREDN WiFi (Mbps): the SDR link is a low-bandwidth *extension* of a mesh, suitable for text/telemetry/SSH, not bulk. Design for that role.

## 6. Recommended Approach (for the future build sprint)
Phased, so the CI-verifiable software lands before the TX-gated on-air work:
1. **Phase A (software, no TX):** IP framing over `packet.rs` (KISS/SLIP encap);
   a `tun` interface mode; a compliance gate on encrypted TX; loopback/mock
   tests (two in-process nodes exchanging IP). Add AREDN + Babel (RFC 8966) to
   the README references.
2. **Phase B (gateway interop):** present the radio `tun` to system `babeld`;
   demonstrate multi-hop routing across an SDR link between two IP hosts; design
   the AREDN gateway (subnet advertisement, DtDLink-equivalent).
3. **Phase C (on-air, needs T-102 Pluto TX):** real two-radio link; then
   multi-hop over the air. Combine with the hardware follow-on.
- **Alternative considered — native Rust Babel:** deferred; use `babeld` over
  `tun` first for real AREDN interop at far lower risk.
- **Rationale:** reuses the realized link layer (INT-0006) and compliance DB
  (INT-0005), reaches real AREDN interoperability via the standard protocol, and
  cleanly separates CI-verifiable software from TX-gated hardware.

## 7. Artifacts
- `docs/sprints/s3/sprint-research/research-report.md` — this report (the spike deliverable).
- `docs/intents/INT-0008-mesh-networking-aredn.md` — created (design intent).
- Design summary published as an Artifact for review (architecture layering + phased plan).

## Budget Override
None required — 5 external sources, within budget. The 16-repo README catalog
(Sprint 0) and Cloudlog (Sprint 2) remain established context; this spike's
sources are scoped to the AREDN/Babel/mesh delta the goal introduces.
