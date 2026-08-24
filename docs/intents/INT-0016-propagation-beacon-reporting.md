# INT-0016 — Propagation Monitoring and Beacon Reporting

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0016
- **State:** proposed
- **Work evidence:** none
- **Completion evidence:** none
- **Code evidence:** none
- **Test evidence:** none
- **Documentation evidence:** [Roadmap](../roadmap.md) — phase and dependency placement
- **Review evidence:** [Sprint 8 research report](../sprints/s8/sprint-research/research-report.md)

## Intent
Run the station as a propagation instrument: monitor weak-signal beacons and
digital modes continuously, report what is heard, and turn the accumulated
record into a usable picture of how the bands are behaving.

Scope:
1. **Weak-signal mode decoding.** WSPR primarily, with FT8/FT4 as the natural
   companions — modes designed to be decodable far below the noise floor, which
   is what makes them propagation probes.
2. **Beacon monitoring.** Unattended watch of the international beacon network
   and other known beacons, logging signal reports over time.
3. **Reporting.** Upload spots to the established aggregators (WSPRnet,
   PSK Reporter, Reverse Beacon Network) using their published interfaces.
4. **Local propagation record.** A queryable history of what was heard, when,
   from where, at what strength — the station's own data, independent of any
   external service.
5. **Presentation.** Propagation over time and distance, so the record answers
   "what is open, and when."

**Non-goals:** transmitting beacons or CQ calls (transmit remains gated by
T-108 and [INT-0005](INT-0005-regulatory-band-compliance.md)); and propagation
*prediction* modelling, which is a different discipline from measurement.

## Acceptance criteria
1. WSPR decodes from a recorded IQ capture containing known spots, recovering
   callsign, locator and power with results matching a reference decoder.
2. Unattended monitoring runs across a multi-hour session, decoding on a
   schedule without drift or leaks, and persisting every spot locally.
3. Spots upload to at least one established aggregator through its published
   interface, with failures retried and never silently dropped.
4. The local record is queryable by band, time, station and distance, and
   remains complete and usable **with no network connection** — the station's
   own data does not depend on a third-party service.
5. Reports include the measurement conditions (frequency, mode, SNR, and the
   receive configuration), so a spot is interpretable rather than a bare
   sighting.

## Rationale
Propagation monitoring is the natural extension of the station-logging work
already realized in [INT-0007](INT-0007-station-logging-cloudlog.md): both are
about recording what the station observed and sharing it. The infrastructure —
ADIF handling, an HTTP client with credential hygiene, scheduling — largely
exists.

It is also the category with the best effort-to-value ratio for an unattended
station. A receiver left running produces genuinely useful data for both the
operator and the wider community, with no transmit involved and therefore none
of the regulatory gating that constrains most other categories.

Criterion 4 is deliberate: many tools in this space are thin clients for an
external service, and the record vanishes when the service does. The station's
own observations should be first-class local data.

## Alternatives
- **Use WSJT-X/WSPR-X and forward their output.** The pragmatic first move —
  they are the reference implementations and criterion 1 measures against them.
  Rejected as the endpoint because it requires a second application and its
  audio plumbing, forfeiting the direct-from-IQ path this suite already has.
- **Implement only the aggregator upload and skip local storage.** Rejected per
  criterion 4 — it makes the station a sensor for someone else's database with
  nothing of its own.
- **Fold this into INT-0007 station logging.** Tempting given the shared
  infrastructure, but the desired outcomes differ: INT-0007 records *contacts
  made*, this records *propagation observed*, largely unattended and without
  transmitting. Different acceptance criteria, so a separate chapter.
- **Include propagation prediction.** Rejected as a non-goal: modelling is a
  distinct discipline, and conflating measurement with prediction would blur
  what a result means.

## Consequences
- Unattended long-running operation raises requirements this suite has not yet
  had to meet: scheduled decoding without resource leaks, recovery from device
  disconnection, and log rotation.
- Weak-signal modes demand accurate frequency and time. WSPR in particular needs
  time accuracy on the order of a second or better, so NTP discipline becomes a
  documented requirement — a mild version of the constraint that dominates
  [INT-0015](INT-0015-distributed-sensing-df.md).
- Aggregator uploads mean credential handling and rate limits; the existing
  Cloudlog integration's credential hygiene (API keys never logged) is the
  precedent to follow.
- A growing local record implies a storage format and retention policy.

## Transition history
- 2026-08-23: created as `proposed` (Sprint 8, T-039) — adopted from the
  candidate list.
