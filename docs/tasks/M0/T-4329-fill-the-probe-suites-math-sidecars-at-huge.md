---
id: T-4329
title: "Fill the probe suites' math sidecars at huge gamma shapes now that Temme's expansion gives them values"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The probe suites' one-call GAMMA.INV, GAMMAINV, CHISQ.INV, CHISQ.INV.RT and CHIINV cells at shapes above 1E8 carry
50-digit math values in their math sidecars, instead of the "no single value" skip they were written with.

## Context
- Follow-up of T-3863, which gave tools/src/rustsheet_tools/accuracy/defs/gamma.py Temme's uniform asymptotic
  expansion above `TEMME_SHAPE_MIN` (1E7). Before it, the tails raised NoValueError above a shape of 1E8, so
  `rustsheet-accuracy math --suite probes-numeric` wrote those cells as skipped (`make eval-math-check` still passes:
  a skip is allowed, so nothing forces the refill).
- Cases with such skips include eval/suites/probes-numeric/cases/fit-audit-t3070-band-upper.math.json,
  t3409-r4-gamma-inv-search-sweep*.math.json and the other t3070/t3350/t3396 huge-shape probes; probes may hold some
  too (`rg -l "no single value" eval/suites/*/cases/*.math.json`).
- Cells on the closed-form branch fall in decision 0188 R3 case 3 (statistics.md "GAMMA.INV at a huge shape,
  closed-form branch"): the decision text says their 50-digit math is committed with T-3364 before the engine
  changes, so coordinate with T-3364 on whether the R3 eval-cell marks go here or there (decision 0289 marked the
  Gate A batch's points only).

## Acceptance criteria
- [x] `rustsheet-accuracy math --suite probes-numeric` (and `--suite probes` if it has such cells) rewritten; the
      huge-shape cells have values and `make eval-math-check` passes.
- [x] The Gate B / R1 counts that change because cells gained math are stated in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-3364).

## Outcome
- Sidecars refilled (`uv run rustsheet-accuracy math --suite probes-numeric --function GAMMA.INV --function GAMMAINV
  --function CHISQ.INV --function CHISQ.INV.RT --function CHIINV`): 8 probes-numeric sidecars, 550 cells that were
  skipped as "no single value" now carry 50-digit values (fit-audit-t3070-band-lower 42, -band-upper 52,
  -closed-form-cut 59, -shape-min 39; t1049-chisqinv 1; t3350-r4-chisq-inv-sweep 40;
  t3409-r4-gamma-inv-search-sweep 265, -wide 52).
- `--suite probes` with the same five functions (38 min run) rewrote nothing: no in-scope cell there is skipped as
  "no single value". Its 15 remaining skips for these functions are "not converged" at shapes of 1E7 and below
  (t3350-chisq-inv-huge-df 7, t3505-gamma-inv-budget-band 8), under `TEMME_SHAPE_MIN`, so out of this task's
  scope (shapes above 1E8). The 4 probes-numeric skips left (t1543-inverse-cdf-tiny-p A49:A52,
  `GAMMA.INV(1E-300..1E-20,0.01,1)`) are tiny shapes whose quantile is below the double range: "no single value"
  is right there.
- `make eval-math-check`: current.
- Counts, measured by `make eval` on this branch against the same tree with the merge base's sidecars: decision 0188
  R1(c) probes-numeric 1009 -> 1008 counted, Phase 1b evidence 3720 -> 3721 (one cell gains a math value within
  1e-9 of both Excel and the engine); R3 passing 250 -> 250; visibly-off total 9208 unchanged; M9 exit row and
  eval baselines unchanged. Gate B: n/a (calipers, corpus-bench and poi not fetched).
- Decision 0188 R3 case 3 marks for the closed-form branch cells stay with T-3364: its acceptance criteria list and
  mark the cells it moves to the math, and this task changes no engine value.
