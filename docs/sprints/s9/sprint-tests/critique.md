# Test Critique — Sprint 9

## Concerns

### C-001: nothing outside tests enables the reliability this sprint built
- **Where:** `crates/sdr-cli/src/main.rs:577` / `crates/sdr-mesh/src/radio.rs`
- **Quote:** "`let link = RadioLink::new(sdr, local_addr, peer, params);`" — no `set_reliable`, no `service`
- **Failure mode:** intent-coverage
- **Why it matters:** T-113 was raised so "a dropped frame does not silently
  truncate a stream". The mechanism now exists and is proven — but `set_reliable`
  and `service` are called **only from `radio_it.rs`**. `sdr-cli tunnel`, and
  therefore every `StreamBridge` and SSH session over the radio, still runs
  fire-and-forget with no retransmission. The problem T-113 names is not yet
  solved for the path that motivated it. A reader seeing "ARQ delivered at 30%
  loss" in the report could easily conclude the tunnel is now reliable. It is
  not.
- **Suggested response:** defer-with-rationale, **stated prominently**. Enabling
  it in the tunnel means threading a time source into the stdio and TCP pump
  loops and deciding how retransmission interacts with a bidirectional byte
  pump — real design work, and `main.rs` is in no task's touched paths in the
  locked plan. Doing it ad hoc at the end of a sprint is how unreviewed
  behaviour lands. Backlog it and say plainly in the report that the stream path
  is not yet protected.

### C-002: the channel model drops whole frames and nothing else
- **Where:** `crates/sdr-protocols/src/lib.rs`, `mod arq_reliability`
- **Quote:** "`if what == Drop::Data && rng.drops(loss_pct) { continue; // data frame lost }`"
- **Failure mode:** stub-leak
- **Why it matters:** A real RF channel corrupts *bits*; frames then fail CRC and
  are discarded. Whole-frame drop is a fair abstraction of what survives CRC, but
  the model does not reproduce reordering, channel-induced duplication, or a
  frame that decodes with a corrupt payload and a coincidentally valid CRC. The
  30% figure therefore measures resilience to *erasure*, which is narrower than
  "30% packet loss" might suggest to a reader.
- **Suggested response:** defer-with-rationale — erasure is the right first
  model and matches what a CRC-checked link actually sees, but the report should
  say what is modelled rather than letting "30% loss" imply more.

### C-003: each reliability test runs a single fixed seed
- **Where:** `test_arq_delivers_all_payloads_at_30pct_loss` and siblings
- **Quote:** "`run(24, 30, Drop::Data, 0xC0FFEE)`"
- **Failure mode:** weak-assertion
- **Why it matters:** Determinism was chosen deliberately and is right — a
  reliability test that fails one run in twenty is a flake generator. But one
  seed exercises one drop pattern. A defect that only appears when, say, three
  consecutive retransmissions are lost could pass indefinitely.
- **Suggested response:** defer-with-rationale — the seeded single-pattern test
  is the correct default. Sweeping several fixed seeds would broaden coverage
  while staying deterministic and is worth a backlog entry, but is not a defect
  in what was delivered.

### C-004: the give-up path is untested under loss
- **Where:** `mod arq_reliability`, helper `run`
- **Quote:** "`a.max_retries = 100; // generous: we are testing delivery, not giving up`"
- **Failure mode:** negative-path
- **Why it matters:** `test_arq_gives_up_after_max_retries` covers the bound in
  isolation with no channel, and the reliability tests deliberately raise the
  bound so it never triggers. The interaction — a channel bad enough that a
  frame legitimately exhausts its retries, and the caller must observe a
  permanent failure — is exercised nowhere. That is the path a real degraded
  link takes.
- **Suggested response:** defer-with-rationale — both halves are covered
  separately and the composition is a genuine gap, not a wrong claim. Backlog
  alongside C-003.

## Resolutions (primary agent)

| Concern | Response | Outcome |
|---|---|---|
| C-001 | defer, stated prominently | **Accepted as the sprint's headline caveat.** `test-report.md` states that the tunnel and stream paths are **not** protected, and the intent records it. New backlog **T-117**. Not fixed here: `main.rs` is outside every locked task's touched paths, and threading time into the pump loops is design work that belongs in a plan, not an end-of-sprint addition. |
| C-002 | defer-with-rationale | **Accepted.** The report now says the model is **erasure** — whole frames dropped post-CRC — and names what it does not model. |
| C-003 | defer-with-rationale | **Accepted.** Determinism is the right default; a multi-seed sweep is backlog **T-118**. |
| C-004 | defer-with-rationale | **Accepted.** Both halves covered separately; the composition is backlog **T-118** alongside C-003. |

No concern required a code change. C-001 changes what the sprint may claim,
which is the more consequential correction.

Post-resolution verification at head `454e821`: `cargo test --workspace` green
(**33 suites**, 0 failed), `cargo clippy --workspace --all-targets` 0 errors.

## Confidence
proceed-with-caveats
