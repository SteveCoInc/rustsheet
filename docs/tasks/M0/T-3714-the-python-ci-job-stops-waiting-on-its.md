---
id: T-3714
title: "The python CI job stops waiting on its own release rustsheet-eval build"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The python CI job's `make py-check` no longer spends about 2.5 minutes with its pytest workers
blocked on a `cargo build --release -p rustsheet_eval` that the tests trigger on demand.

## Context
Follow-up of T-3700 (decision 0233). In run 41294866508 (PR #2733), pytest with `-n auto` passed
46% of the 1,248 tests within 10 s, then stalled from 09:51:59 to 09:54:24 while
`rustsheet_tools.snapshot` built the release binary; the rest took about 30 s. `make py-check`
took 199 s, of which the tests themselves were about 40 s. The eval job builds the same binary
on another runner at the same time. Options to measure:
- start the build as its own step before pytest, overlapping ruff and mypy (a small win);
- make the tests that need the binary use a cheaper build;
- share the eval job's binary as an artifact, which serializes the jobs (likely worse);
- check which tests need a real binary at all and give the rest a stub.

## Acceptance criteria
- [x] The python job's `make py-check` step is measured before and after in real PR runs, and
      the Outcome records which option won and why.
- [x] Every test that runs today still runs; `make check` green; reviewers approve.

## Out of scope
- The eval job's own release build.

## Outcome
`make py-check` builds a dev `rustsheet-eval` and points the tools/ tests at it through
`RUSTSHEET_EVAL_BIN`, unless a release binary already exists or the caller set the variable
(decision 0280). No test changed; all 1,642 still run.

Options measured (local, 4 cores): only 17 tests need the binary at all, so a stub buys
nothing over a cheap build. The fat-LTO release workspace rebuild took 187 s, a release profile
without LTO 234 s (no gain), a cold dev build 57 s; the 139 tests of the four affected files take
the same time against either binary (10 s vs 9 s). Building first, before pytest, overlaps
only ruff and mypy (about 6 s), and sharing the eval job's binary would serialize the jobs.
So the cheaper build won.

CI, `make py-check` step of the python job:
- before: 205 s (run 41864419240, PR #3189), 212 s (41901029685, #3212), 205 s (41905776348,
  #3217), 217 s (41895252244, #3215);
- after: 72 s (run 41905116166, this PR; the dev build takes about 33 s of it, pytest 36 s).
  `make py-bindings` also went from 31-33 s to 26 s, since `maturin develop` reuses the dev
  artifacts. The python job took 2 min 12 s instead of about 4.5 min.

Eval delta: none (no engine change).
