---
id: T-4726
title: "CI eval job gates poi and corpus-bench: cached pinned fetch, failing fetch, --require-fetched; local make eval targets fetch missing suites (SPEC-0035)"
milestone: M2
lane: harness
status: done
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
- [x] A decision record (`make new-decision`) covers the `.github/` and `Makefile` change and amends decision 0067's
      "`make check` and CI do not run" poi.
- [x] ci.yml's eval job restores/fetches the two caches as SPEC-0035 D2 says, and runs `make eval-gates` with
      `EVAL_GATE_FLAGS=--require-fetched`; the merge group runs the same job.
- [x] The Makefile follows SPEC-0035 D3's table: with `EVAL_FETCH` set, `eval-check`, `eval-gates` and
      `eval-baseline` first run `-$(EVAL_GUARDED) fetch --missing --gates`, and `eval` passes `--fetch-missing` to `run`
      and `roundtrip`; with it empty, no target fetches. `make eval-baseline` on a clean checkout fetches poi and
      corpus-bench and not calipers (Outcome shows it).
- [x] CI's "Full report (informational)" step fetches nothing and its added time is recorded (SPEC-0035 D3).
- [x] The PR's CI log shows `[poi]` and `[corpus-bench]` summary lines and no `skipping poi` / `skipping
      corpus-bench`; the Outcome records the eval job's time (cache miss and cache hit) against SPEC-0035 D1's budget.
      Over 7 min: do not land; go back to the spec.
- [x] Hand test in a scratch branch: a removed poi baseline line fails the CI eval job (Outcome links the run).
- [x] docs/process/quality-gates.md says poi and corpus-bench gate in CI.
- [x] `make check` green; reviewers approve.

## Out of scope
- calipers (T-4728). The evidence floor and visible-parity lock (SPEC-0035 D5: unchanged).

## Outcome
Landed as two parallel CI jobs, not one, after the one-job design measured over SPEC-0035 D1's budget. The amendment
is in SPEC-0035 D1 and decision 0344. The acceptance criteria above hold, with the gate split across the jobs.

- **Jobs:**
  - `eval (vendored suites)` is the old `eval` job and fetches nothing.
  - `eval (fetched gate suites)` restores the poi and corpus-bench caches (key `fetched-suites-v1-` plus the hash of
    their `suite.toml`, no `restore-keys`). On a miss it fetches them, three tries. It then runs
    `make eval-fetched-gates` with `EVAL_GATE_FLAGS=--require-fetched`: `check` over poi and corpus-bench, and
    `roundtrip --check` over their round-trip suites.
  - `eval (Excel-compatibility gate)` is a summary job that keeps the required name. It passes only when both jobs
    passed or both were skipped on purpose, as decision 0122's `rust (fmt, clippy, test)` does. The merge group
    runs the same workflow.
- **Makefile (SPEC-0035 D3):**
  - `EVAL_FETCH ?= 1` and `EVAL_GATE_FLAGS ?=`.
  - `EVAL_PREFETCH` (`-…fetch --missing --gates`) heads `eval-check`, `eval-gates`, `eval-baseline` and
    `eval-fetched-gates`.
  - `eval` passes `--fetch-missing` to `run` and `roundtrip`. With `EVAL_FETCH` empty, no target fetches.
  - Shown by `make -n` in both settings. A throwaway Makefile confirmed that GNU make honours the `-` after expansion.
  - With the local poi cache moved aside, `make eval-check` printed `corpus-bench is current` and
    `fetched poi (366 files)`, leaving a byte-identical cache. It left calipers alone, and the ungated pivot corpora
    skip as before. `eval-baseline` runs the same pre-fetch line.
- **The Full report step** runs in the vendored job with `EVAL_FETCH` empty. It fetches nothing, adds no time, and its
  report skips the fetched suites as before.
- **CI runs on PR #3999:**

  | run | layout | result | eval time |
  |---|---|---|---|
  | 42396840652 | one job, cache miss | green | 8 min 30 s; the fetch itself is a few seconds |
  | 42397868004 | one job, cache hit | **hand test**: `eval/baseline/poi.txt` without `F poi/FormulaEvalTestData_Copy.xlsx EverythingTests!J136`. The `eval` job **failed** with `REGRESSION poi/FormulaEvalTestData_Copy.xlsx EverythingTests!J136`; main's CI would have passed it. The line came back in the next commit. | |
  | 42398662614 | one job, cache hit | green | 8 min 19 s. Main's job that day took 5 min 36 s. The corpora's round trips add 82 s (doubled by R1, T-4514) and their cells 28 s. Over budget, so back to the spec. |
  | 42399714353 | two jobs, cache hit | green | vendored 4 min 34 s; fetched 5 min 54 s (release build 3 min 44 s, cells 9 s, round trips 1 min 25 s); slowest rust shard 5 min 30 s; PR wall about 6 min |

  The fetched job's log shows `[poi]`, `[corpus-bench]`, `[roundtrip-poi]` and `[roundtrip-corpus-bench]`, and no
  skipping line for them.
- **Docs:** docs/process/quality-gates.md and docs/architecture/eval-harness.md say poi and corpus-bench gate in CI.
- **Cost:** one more runner per PR, about 6 min of it.

