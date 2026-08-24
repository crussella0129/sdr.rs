# INT-0011 — Mesh Messaging: Chat, File Transfer, and Callsign Identity

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0011
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** [Roadmap](../roadmap.md) — phase and dependency placement
- **Review evidence:** [Sprint 8 research report](../sprints/s8/sprint-research/research-report.md)

## Intent
Turn the radio mesh from a transport into something people actually talk over: a
messaging service with the shape users already understand from Discord — named
channels, direct messages, presence, and file transfer — carried entirely over
RF by [INT-0008](INT-0008-mesh-networking-aredn.md) and
[INT-0006](INT-0006-packet-radio-ssh-tunnel.md).

Scope:
1. **Channels and direct messages.** Text routed between nodes addressed by
   **callsign**, not just a numeric node address.
2. **Store-and-forward.** A message for an unreachable station is held and
   delivered when it returns — the normal state of an RF link, not an edge case.
3. **File transfer.** Chunked, integrity-checked, and **resumable**, because an
   intermittent link will interrupt any transfer worth making.
4. **Presence and roster.** Which stations are reachable, over how many hops,
   and at what link quality.
5. **Callsign identity.** A callsign binds to a node address and is
   cryptographically verifiable.

**Non-goals:** voice; interoperating with the Discord service itself (the
comparison is to its *interaction model*, nothing more); and inventing a new
radio protocol — this rides the existing mesh.

### The regulatory distinction this intent turns on

On amateur bands, messages that obscure meaning are prohibited, and
[INT-0005](INT-0005-regulatory-band-compliance.md) already refuses encryption
there. **Digital signatures are a different thing from encryption**: a signature
authenticates the sender without concealing the content, which is what makes
callsign verification available on amateur bands where confidentiality is not.
This intent therefore treats *authentication* and *confidentiality* as
independently switchable, and never conflates them:

| Band class | Confidentiality | Sender authentication |
|---|---|---|
| ISM | permitted (encrypted mode) | permitted |
| Amateur | refused per INT-0005 | **permitted — signatures do not obscure meaning** |

Getting this wrong in either direction is costly: conflating them either
forfeits verifiable identity on amateur bands, or ships prohibited
confidentiality there.

## Acceptance criteria
1. Text messages route between mesh nodes addressed by callsign, across at least
   one intermediate hop, with delivery verified end to end.
2. A message addressed to an unreachable station is stored and delivered on its
   return, verified by a test that partitions and rejoins the mesh.
3. A file transfer completes with integrity verification, and a transfer
   interrupted mid-flight **resumes** without retransmitting completed chunks.
4. A roster reports reachable stations with hop count and a link-quality metric
   derived from actual link statistics, not a placeholder.
5. Callsign identity is bound to a node address and verifiable by signature; a
   forged callsign is detected and rejected by a test that attempts one.
6. On an amateur band the station transmits its identification per regulation
   and confidentiality is refused per INT-0005, while signature verification
   still functions — both asserted by tests.

## Rationale
The mesh currently carries an SSH stream and raw datagrams. That proves the
transport but serves one narrow use. Messaging is the capability that makes an
off-grid mesh worth deploying: emergency coordination, field teams, and ordinary
conversation between stations all reduce to reliable text and file movement
between identified operators.

Callsigns rather than opaque addresses because that is how operators already
identify each other, because amateur regulation requires station identification
anyway, and because a verifiable callsign is the natural trust anchor for
everything else built on the mesh.

Store-and-forward and resumable transfer are listed as acceptance criteria
rather than refinements because an RF mesh is intermittent by nature; a design
that treats disconnection as exceptional will fail in normal use.

## Alternatives
- **Adopt Meshtastic's protocol wholesale.** Rejected as the primary path: it is
  tied to LoRa radio modules and a fixed packet model, whereas this mesh is
  SDR-defined and waveform-agnostic. Worth revisiting as an *interoperability*
  target in its own intent.
- **JS8Call-style weak-signal messaging.** Different problem — optimized for
  marginal HF links at very low throughput. Complementary, not a substitute.
- **Run an existing chat protocol (IRC, XMPP, Matrix) over the mesh IP layer.**
  Genuinely attractive once INT-0008 Phase B lands a `tun` device, and it
  inherits mature clients. Rejected as the *first* move because all three assume
  a server and far more bandwidth than this link has; revisit as a gateway.
- **Encrypt everywhere and simply refuse to operate on amateur bands.**
  Rejected: it abandons the dual-mode design INT-0005 exists to provide.

## Consequences
- Key management becomes a user-facing concern: callsign-to-key binding needs a
  trust model (manual pinning at minimum) and a story for compromise.
- Store-and-forward implies persistent local state, retention limits, and a
  policy for undeliverable messages.
- Resumable transfer requires stable chunk identity across sessions.
- This intent depends on **T-113** (ARQ wired into the stream path). Reliable
  messaging over a lossy channel is not achievable while the receive path drops
  frames without retransmission, so T-113 is a prerequisite, not a parallel
  nicety.

## Transition history
- 2026-08-23: created as `proposed` (Sprint 8 roadmap).
