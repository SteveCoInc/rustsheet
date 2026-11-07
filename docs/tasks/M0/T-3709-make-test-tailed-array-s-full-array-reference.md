---
id: T-3709
title: "Make test_tailed_array's full-array reference evaluations cheaper"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`test::test_tailed_array` (988 of the 2,184 test-seconds of `make test` on 2026-11-01) costs a
fraction of that, and still checks every tailed formula against the ordinary full-array reading.

## Context
Follow-up of T-3699 (decision 0232). Each test evaluates its formulas twice through `both_ways`
(crates/rustsheet-engine/src/test/test_tailed_array.rs), with the tailed reading on and with
`TAILED_OFF`, which materializes 1,048,576-row arrays in a debug build. Single tests run up to
149 s locally (`index_picks_from_a_tailed_argument`, `keyed_xlookup_and_xmatch_over_a_whole_column_key`
130 s). Options to measure: share one full-reading model per test instead of one per formula,
evaluate the reference formulas once and cache them across tests, or build the engine at
`opt-level = 1` for tests. Decision 0122 measured opt-level 1 as a net loss (compile +127 s,
tests -110 s) when the whole suite was 585 CPU-s; it is about 2,000 now.

## Acceptance criteria
- [x] Every formula `test_tailed_array` checks today is still compared tailed against full.
- [x] Before/after test-seconds for the module, and for `make test` wall time locally and in CI,
      in the Outcome (a decision record if the profile, Makefile or CI change).
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the tailed reading itself (SPEC-0020).

## Outcome
- `Cargo.toml` builds `rustsheet_engine` at `opt-level = 1` with `debug = "line-tables-only"` in
  the `test` profile (decision 0276). The test module is unchanged, so every formula is still
  compared tailed against the full reading.
- Local, 4 threads, 2026-11-04: `test_tailed_array` 679 -> 317 test-s (186 -> 85 s wall; the
  slowest test, `index_picks_from_a_tailed_argument`, 100 -> 50 s). `make test` 512 -> 377 s
  wall (compile 114 -> 151 s, engine unit tests 359 -> 192 s).
- A test-side cache was not taken: gdb samples show the full reading spread over the generic
  evaluator, and each test already evaluates all its formulas in one model per reading.
- CI shard times: see the PR (measured on its first green run).
- Eval delta: none (test build profile only).
