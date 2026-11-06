---
id: T-4632
title: "Nightly CI fetches the corpora and enforces the round-trip gates of Calipers, POI and corpus-bench"
milestone: M15
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-4629]
---

## Goal
CI enforces the corpus round-trip baselines on main every day, so a loss of corpus content is caught even though the
PR gate checks only the fixtures.

## Context
- After T-4629, `roundtrip-poi` and `roundtrip-corpus-bench` are gated wherever their caches are fetched, but CI
  fetches no corpus. `roundtrip-calipers` is ungated.
- `.github/workflows/nightly.yml` runs `make eval` as a report only.
- On a developer machine, Calipers' pv26_002 and pv26_2_002 take about 55 s of the 60 s per-case timeout.
- Decision 0323.

## Acceptance criteria
- [x] The nightly job fetches calipers, poi and corpus-bench. A failed fetch fails the job.
- [x] It runs `rustsheet-eval roundtrip --check` on the three corpus round-trip suites with a 300 s per-case
      timeout, so a regression turns it red.
- [x] Decision 0323 records the protected-path change.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fetching corpora in the PR gate.
- Making Calipers' heavy cases faster.

## Outcome
- `.github/workflows/nightly.yml` gains two steps after `make eval`: `rustsheet-eval fetch` for the three suites, then
  `rustsheet-eval roundtrip --check --timeout 300` over `roundtrip-calipers`, `roundtrip-poi` and
  `roundtrip-corpus-bench`.
- Locally on main (e725f06), the same command passes:

  | suite | cases | kept |
  |---|---|---|
  | roundtrip-calipers | 541, none timed out | 99.986% |
  | roundtrip-poi | 350 | 99.997% |
  | roundtrip-corpus-bench | 912 | 99.99999% |

  Calipers' slowest case took 54.7 s.
