---
id: T-4329
title: "Fill the probe suites' math sidecars at huge gamma shapes now that Temme's expansion gives them values"
milestone: M0
lane: harness
status: in-progress
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
- [ ] `rustsheet-accuracy math --suite probes-numeric` (and `--suite probes` if it has such cells) rewritten; the
      huge-shape cells have values and `make eval-math-check` passes.
- [ ] The Gate B / R1 counts that change because cells gained math are stated in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-3364).

## Landing notes
- No PR yet. Branch agent/harness/T-4329-huge-shape-math, head after this commit; the session's window closed.
- Done (commit 607e71ba3): `uv run rustsheet-accuracy math --suite probes-numeric --function GAMMA.INV --function
  GAMMAINV --function CHISQ.INV --function CHISQ.INV.RT --function CHIINV` rewrote 8 sidecars. 550 "no single value"
  skips now carry values: fit-audit-t3070-band-lower 42, -band-upper 52, -closed-form-cut 59, -shape-min 39;
  t1049-chisqinv 1; t3350-r4-chisq-inv-sweep 40; t3409-r4-gamma-inv-search-sweep 265, -wide 52. 13 in-scope cells
  still report "no single value" in that run; look at why (other shapes, or not one-call) before calling it done.
- Not done: the same command with `--suite probes`. It ran for over 30 minutes and was stopped at the background-job
  limit without writing anything. Run it in the foreground or with a longer limit; the skips in eval/suites/probes
  may be for other functions, so check `rg -l "no single value" eval/suites/probes/cases/*.math.json` against the
  five functions first.
- Then: `make eval-math-check`, and state the Gate B / R1(c) count changes in the Outcome. Gate B is n/a without
  the fetched corpora (calipers, corpus-bench, poi); R1(c)'s classes come from `make eval`'s report (compare against
  main). `rustsheet-eval run --desktop-owned` needs `eval/runs/desktop-owned.tsv` from `scripts/desktop_owned.py`
  first (use `make eval`).
- Decision 0188 R3 case 3 cells (GAMMA.INV closed-form branch at huge shapes): check with T-3364 whether their R3
  eval-cell marks belong here, per the task's Context.
