# INT-0005 — Jurisdictional RF Regulatory Band Advisor and Transmission Compliance

<!-- sprint-loop-intent-v2 -->
- **Intent ID:** INT-0005
- **State:** active
- **Review evidence:** [Sprint 10 research report](../sprints/s10/sprint-research/research-report.md) — API/enforcement audit found ignored bandwidth/duty-cycle constraints and unsafe invalid-input fallbacks.
- **Work evidence:** [Sprint 10 build plan — T-128](../sprints/s10/sprint-plans/build-plan.md), [T-007 build plan](../sprints/s1/sprint-plans/build-plan.md#t-007-jurisdictional-regulatory-compliance-database-and-transmission-advisor), [T-010 build plan](../sprints/s1/sprint-plans/build-plan.md#t-010-stream-tunnel-proxy-bridge-for-ssh-and-sdr-cli-bandstunnel-commands)
- **Completion evidence:** [T-007 completion](../work/completed-tasks.md#t-007-sprint-1), [T-010 completion](../work/completed-tasks.md#t-010-sprint-1)
- **Code evidence:** [compliance.rs](../../crates/sdr-core/src/compliance.rs), [sdr-cli](../../crates/sdr-cli/src/main.rs)
- **Test evidence:** [Sprint 1 test report](../sprints/s1/sprint-tests/test-report.md)
- **Documentation evidence:** [README.md](../../README.md)

## Intent
Provide a built-in regulatory database and compliance advisor for RF transmission. The engine guides users to legally permissible frequency bands based on their geographic jurisdiction (e.g. United States / FCC Part 15 & Part 97, European Union / ETSI & CEPT ERC Rec 70-03, United Kingdom / Ofcom, Australia / ACMA, and ITU Regions 1, 2, 3).

The engine catalogs transmission constraints including maximum effective radiated power (ERP/EIRP in mW and dBm), duty cycle limits (e.g. 1%, 10%), channel bandwidth, frequency hopping (FHSS) requirements, and critically: whether encrypted/obscured payloads (such as SSH, TLS, or AES encrypted packets) are legally permitted (allowed in license-exempt ISM / SRD bands, but strictly prohibited in amateur radio bands like FCC Part 97.113).

Non-goals for this intent: Automated power attenuation enforcement on non-compliant external amplifiers.

## Acceptance criteria
1. Database accurately represents major license-exempt ISM/SRD bands (902–928 MHz in US, 863–870 MHz in EU, 433.05–434.79 MHz in EU, 2.400–2.4835 GHz worldwide, 5.725–5.875 GHz worldwide) and Amateur bands (2m 144–148 MHz, 70cm 420–450 MHz / 430–440 MHz). **(partial — entries exist, but Sprint 10 did not revalidate their conditional/current legal accuracy.)**
2. Compliance query API verifies whether a proposed transmission (frequency, bandwidth, power, encryption status) is legally permitted in the selected jurisdiction and returns explicit advisory warnings when encryption is attempted on amateur bands. **(not met — the API accepts no occupied-bandwidth or duty-cycle input, ignores `max_bandwidth_hz`, and non-finite power can evade comparison.)**
3. Interactive CLI query (`sdr-cli bands`) outputs recommended bands filtered by jurisdiction and use-case (e.g. license-free encrypted data transmission for SSH/networking). **(partial — normal queries work, but an invalid jurisdiction silently falls back to US rules.)**

## Rationale
Transmitting over the air is subject to strict telecommunications law. Users seeking to transmit digital data or establish encrypted tunnels (like SSH over radio) often do not know which frequency bands permit encrypted payloads without an amateur radio license, or what duty cycle / power constraints apply in their country. Providing built-in jurisdictional guidance ensures safety, legal compliance, and good RF citizenship.

## Alternatives
- Relying on user external knowledge: Rejected because accidental illegal transmission (e.g. encrypted data on 2m ham radio) risks regulatory fines and RF interference.
- Cloud-based regulatory API: Rejected to maintain offline capability for field and remote SDR deployments.

## Consequences
- Requires maintaining an offline, structured regulatory database reflecting current ITU / FCC / ETSI radio standards.
- Regulatory guidelines serve as software advisory and educational reference.

## Transition history
- 2026-08-21: created as `proposed`.
- 2026-08-21: moved to `planned` for Sprint 1 execution under T-007 and T-010.
- 2026-08-21: transitioned to `realized` in Sprint 1 under T-007 and T-010.
- 2026-08-24: **re-opened to `active` after Sprint 10's audit.** The catalog/advisor is useful, but it cannot yet verify the bandwidth and duty-cycle constraints named by this intent, accepts unsafe non-finite/unknown inputs, and can be bypassed by the real tunnel path. Current-law accuracy remains a separate verification obligation; no legal-data update is claimed by this audit.
