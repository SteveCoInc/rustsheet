---
id: T-4726
title: "CI eval job gates poi and corpus-bench: cached pinned fetch, failing fetch, --require-fetched; local make eval targets fetch missing suites (SPEC-0035)"
milestone: M2
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-3556, T-4725]
---

## Goal
poi and corpus-bench (cells and their `roundtrip-*` suites) gate every PR and merge group in CI's required `eval`
job, from a cached pinned fetch that fails the job when it fails; local `make eval`, `eval-check`, `eval-gates` and
`eval-baseline` fetch missing suites first.

## Context
- SPEC-0035 D1, D2, D3 (docs/specs/SPEC-0035-fetched-suite-gate.md) decide the design and its numbers: about 16 s
  of fetch on a cache miss and about 40 s of checks on the runner, against a budget of at most 90 s added (job under
  7 min).
- Cache key `fetched-suites-v1-` + `hashFiles` of the two suite.toml files, no `restore-keys`; fetch step on a miss
  with 3 retries 30 s apart through `scripts/eval_guard.sh` (as nightly.yml's, decision 0323).
- Makefile variables (SPEC-0035 D3's table): `EVAL_FETCH` (default `1`; ci.yml sets it empty for the `eval` job)
  and `EVAL_GATE_FLAGS` (default empty; ci.yml sets `--require-fetched`, T-4725).

## Acceptance criteria
- [ ] A decision record (`make new-decision`) covers the `.github/` and `Makefile` change and amends decision 0067's
      "`make check` and CI do not run" poi.
- [ ] ci.yml's eval job restores/fetches the two caches as SPEC-0035 D2 says, and runs `make eval-gates` with
      `EVAL_GATE_FLAGS=--require-fetched`; the merge group runs the same job.
- [ ] The Makefile follows SPEC-0035 D3's table: with `EVAL_FETCH` set, `eval-check`, `eval-gates` and
      `eval-baseline` first run `-$(EVAL_GUARDED) fetch --missing --gates`, and `eval` passes `--fetch-missing` to `run`
      and `roundtrip`; with it empty, no target fetches. `make eval-baseline` on a clean checkout fetches poi and
      corpus-bench and not calipers (Outcome shows it).
- [ ] CI's "Full report (informational)" step fetches nothing and its added time is recorded (SPEC-0035 D3).
- [ ] The PR's CI log shows `[poi]` and `[corpus-bench]` summary lines and no `skipping poi` / `skipping
      corpus-bench`; the Outcome records the eval job's time (cache miss and cache hit) against SPEC-0035 D1's budget.
      Over 7 min: do not land; go back to the spec.
- [ ] Hand test in a scratch branch: a removed poi baseline line fails the CI eval job (Outcome links the run).
- [ ] docs/process/quality-gates.md says poi and corpus-bench gate in CI.
- [ ] `make check` green; reviewers approve.

## Out of scope
- calipers (T-4728). The evidence floor and visible-parity lock (SPEC-0035 D5: unchanged).
