# Sprint 8 — Unit Test Results

- **Tested head:** `3cb6098916c71dd46c051570f868595a0dd55f5e`
- **Runner:** `cargo test --workspace --lib`
- **Date:** 2026-08-23
- **CI authority:** no hosted CI on this repository; the local workspace runner
  is the canonical suite. Recorded as such rather than implying a green badge.

## No unit tests were added, and none should have been

This sprint added **no library code** — its deliverable is Book structure
(intent chapters, a roadmap, navigation) plus integration tests that guard it.
There is no unit under test.

This is stated plainly rather than manufacturing a unit suite to fill the
section. The sprint's verification lives in
[integration-tests.md](integration-tests.md).

## Existing unit suites re-verified

Every pre-existing library suite ran unchanged as part of the regression
contract. No behaviour changed, and none was expected to.

| Crate | Passed | Failed |
|---|---|---|
| sdr-core | 8 | 0 |
| sdr-dsp | 25 | 0 |
| sdr-demod | 8 | 0 |
| sdr-hardware | 25 | 0 |
| sdr-mesh | 14 | 0 |
| sdr-protocols | 6 | 0 |
| sdr-spectrum | 2 | 0 |
| sdr-station | 4 | 0 |

## Lint

`cargo clippy --workspace --all-targets` — **0 errors**. Pre-existing warnings
remain tracked as backlog T-105; this sprint added none.
