---
id: T-3997
title: "M5 exit owner for probes/t3945-atan-near-midpoint S!C8 and S!C10: they pass once T-3932 reads 0.190581435720431 as Excel does"
milestone: M5
lane: engine
status: in-progress
priority: P0
size: S
tier: sonnet
depends_on: [T-4077]
---

## Goal
The two M5 exit cells of probes/t3945-atan-near-midpoint have an owner of their milestone (decision 0096), and they
pass once T-3932 lands; if T-3932 does not fix them, this task finds and fixes the M5 part that is left.
T-3932 kept the nearest double for digit strings of 1,000,000 or more, so S!C8 and S!C10 still fail; T-4077 now
owns the 15-digit read, and this task confirms the cells after it.

## Context
- Gap triage 2026-11-03 (`make eval` on origin/main b6ab50e5): `make exit-owners MILESTONE=M5` lists two cells, and
  `scripts/phase1_exit.py` reports `owners ... 2 milestone exit cells with no owner of their milestone (M5 2)`:
  - probes/t3945-atan-near-midpoint.case.json S!C8 `=0.190581435720431*2^56-13732839000000000`: Excel 726308276,
    ours 726308274 (`NumericMismatch`). The literal reads one ulp above the nearest double in Excel; `*2^56` and the
    subtraction expose that ulp as 2 in the result.
  - probes/t3945-atan-near-midpoint.case.json S!C10 `=IMLOG2(COMPLEX(A204,A205))`: Excel
    "-6.40685300762984E-16+1.98953694302209i", ours "-6.40685300762984E-16+1.9895369430221i" (`TextMismatch`): the same
    ulp reaching IMLOG2 through COMPLEX's text (engineering.md "IMLOG10 and IMLOG2 scale IMLN").
- T-3932 (M2, engine) owns the cause: formula literals and VALUE read decimals with Excel's x87 multiply by 10^-n
  (numbers.md "Decimals reach a double through an x87 multiply by 10^-n"). Its Context lists both cells (S!C7:C10).
  This task records the split: T-3932 changes the literal reader, this one confirms the M5 cells and owns them for
  the M5 exit. Both are in eval/baseline/probes.txt (`F probes/t3945-atan-near-midpoint.case.json S!C8`, `S!C10`).
- Cells:
  - probes/t3945-atan-near-midpoint.case.json (2): S!C8, S!C10

## Acceptance criteria
- [ ] After T-3932 is merged, `make eval-show CASE=probes/t3945-atan-near-midpoint.case.json` shows S!C8 and S!C10
      passing, and their two known-failure lines leave eval/baseline/probes.txt with no new lines in any suite
      (`make eval-baseline`).
- [ ] If either still fails, the Outcome gives Excel's and the engine's value and the cause, and the fix is made
      here when it is size S and needs no numeric fitting; otherwise the task is handed up (work-loop step 5b).
- [ ] `make exit-owners MILESTONE=M5` no longer lists either cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The literal and VALUE reader itself (T-3932).
- The ATAN and ATAN2 cells of the same case (S!B203, S!B204) and the other C cells T-3932 lists.
