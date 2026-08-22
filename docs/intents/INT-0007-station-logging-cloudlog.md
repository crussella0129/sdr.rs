# INT-0007 — Station Logging and Cloudlog Integration

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0007
- **State:** realized
- **Work evidence:** [T-011 build plan](../sprints/s2/sprint-plans/build-plan.md#t-011-add-cloudlog-to-the-readme-reference-catalog), [T-012 build plan](../sprints/s2/sprint-plans/build-plan.md#t-012-cloudlog-station-logging-client)
- **Completion evidence:** [T-011 completion](../work/completed-tasks.md#t-011-sprint-2), [T-012 completion](../work/completed-tasks.md#t-012-sprint-2)
- **Code evidence:** [sdr-station](../../crates/sdr-station/src/lib.rs)
- **Test evidence:** [Sprint 2 test report](../sprints/s2/sprint-tests/test-report.md)
- **Documentation evidence:** [README.md](../../README.md)
- **Review evidence:** [Sprint 2 research report](../sprints/s2/sprint-research/research-report.md)

## Intent
Give `sdr.rs` a station-logging integration so an operator can push live radio
state and log contacts to an external logbook, targeting Cloudlog
(`magicbug/Cloudlog`) and API-compatible forks (Wavelog) first.

The subsystem provides:
1. A Cloudlog HTTP client that posts live CAT state (frequency, mode, power,
   radio name, timestamp) to `POST /index.php/api/radio`, driven from the
   existing `RigState` / rigctl model so tuning `sdr.rs` updates the logbook's
   active radio in real time.
2. Contact logging: assemble ADIF records and submit them to
   `POST /index.php/api/qso` (`{key, station_profile_id, type:"adif",
   string:<ADIF>}`), including an ADIF export path so decoded/manually-entered
   contacts can be uploaded or watched from a file.
3. API-key configuration and clear auth/error handling (401 on missing/invalid
   key), with the Cloudlog base URL and key supplied by the operator.

Non-goals for this intent: a full general-purpose logbook UI, QSL-card image
management, or non-Cloudlog logging backends (LoTW/eQSL/Club Log) — those are
possible later intents.

## Acceptance criteria
1. A `CloudlogClient` (or equivalent) posts a well-formed `/api/radio` update
   from `RigState` and handles success, 401, and network-error cases without
   panicking; verified against a mock HTTP server in CI (no radio required).
2. An ADIF record builder produces spec-valid ADIF for a contact and the client
   submits it to `/api/qso`; round-tripped/asserted against a mock server.
3. Base URL and API key are configurable and never hard-coded; the key is not
   logged.
4. README reference catalog includes Cloudlog.

## Rationale
Logging is the missing link between `sdr.rs`'s decode/receive capabilities and a
real operator workflow. Cloudlog is a widely used self-hosted logger with a
simple, documented JSON API, and `sdr.rs` already models rig state — so the
integration is small, high-value, and fully testable without hardware, making it
a good CI-verifiable deliverable while real-radio work proceeds separately.

## Alternatives
- Direct database writes to Cloudlog's MySQL: Rejected — bypasses the supported
  API, couples to schema internals, and breaks on upgrades.
- WSJT-X-style UDP broadcast only: Rejected as the sole mechanism — Cloudlog
  ingests via its HTTP API; UDP is a possible later input path, not the target.
- ADIF-file export only (no live push): Insufficient — loses the real-time CAT
  `/api/radio` benefit the rigctl model already enables.

## Consequences
- Introduces an HTTP client dependency (e.g. `reqwest`/`ureq`) in the crate that
  owns logging; keep it out of the pure DSP crates.
- Field names and endpoint paths track Cloudlog/Wavelog versions; confirm
  against the operator's instance before relying on them in production.

## Transition history
- 2026-08-21: created as `proposed` (Sprint 2 review).
- 2026-08-21: moved to `planned` for Sprint 2 execution under T-011 and T-012.
- 2026-08-21: transitioned to `active` upon starting Build Phase (T-011).
- 2026-08-22: transitioned to `realized` in Sprint 2 under T-011 and T-012 — all four acceptance criteria proven by unit + integration tests against a mock HTTP server (see [Sprint 2 test report](../sprints/s2/sprint-tests/test-report.md)).
