---
id: T-4483
title: "rustsheet-accuracy math --function: regenerate only the sidecars of cases that call the named functions"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
`rustsheet-accuracy math --suite S --function NAME` (repeatable) recomputes and rewrites only the math sidecars of
the cases whose formulas call one of the named functions, leaving every other sidecar of the suite untouched.

## Context
- Every Gate A table task that adds a definition must regenerate the probes and probes-numeric math sidecars so
  `make eval-math-check` passes (decision 0243). `write_suite` recomputes every case: about an hour of four CPUs for
  probes-numeric, and the cells that hit the time limit ("not converged or too slow") flip between a value and a skip
  from one machine to the next, so a full rewrite drags unrelated diffs into the PR (T-4083, #3157: 11 such files).
- About 25 open Gate A table tasks (T-4090 to T-4116) each need this step.

## Acceptance criteria
- [x] `--function NAME` (repeatable, a registered definition's name) selects the cases whose case file calls
      `NAME(`; only those sidecars are written. An unknown name is an error. Without it, behaviour is unchanged.
- [x] Tests in tools/tests; `make check-fast` green; reviewers approve.

## Out of scope
- Caching unchanged cells across runs.

## Outcome
- `rustsheet-accuracy math --suite S --function NAME` (repeatable): `write_suite` keeps only the cases whose case file
  calls `NAME(` (`calls_any`, a whole-name match: `T.INV` does not select `T.INV.2T`, `S.INV` not `NORM.S.INV`), and
  rewrites only their sidecars. An unknown name is an error (the CLI's `_definition`). `--function` with `--check` is
  refused. Without `--function` nothing changes.
- Measured: `--suite probes-numeric --function BESSELJ` takes 6 s (one case, 356 cells) and reproduces the committed
  sidecar byte for byte, against about an hour of four CPUs for the whole suite.
- Use in a Gate A table task: after adding definitions, run it once per suite with every new function, then
  `make eval-math-check`.
- Tests: tools/tests/test_accuracy_eval_math.py (the filter leaves an unselected stale sidecar untouched; whole-name
  matching).
- Eval delta: none.
