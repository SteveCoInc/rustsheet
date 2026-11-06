---
id: T-4606
title: "AVERAGEIFS, AVERAGEIF, MAXIFS and MINIFS with an array criterion over an error in the value range: probe positions past the first, and match"
milestone: M9
lane: functions-criteria
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel cells show what AVERAGEIFS, AVERAGEIF, MAXIFS and MINIFS give at each position of an array criterion when the
value range holds an error that only some positions select. criteria.md states the rule for all four as it does for
SUMIF and SUMIFS, and ours matches every cell.

## Context
- Gap triage 2026-11-06 (`make eval` on agent/tasks/refill-sonnet-1206, cut from origin/main). No unowned visibly-off
  cells were left, so this triage read the rules the semantics docs leave half-probed.
- criteria.md "*IF(S) lift array criteria": "for SUMIFS and SUMIF an error in the sum range is `#N/A` only at the
  criteria positions whose rows select it (`SUMIFS(E1:E3,A1:A3,{"a","b"})` with E1 = #N/A is #N/A, 5; AVERAGEIFS and
  MAXIFS probed only at position 1: `#N/A`)". The evidence is probes/criteria-tilde-blank-single-cell-edges Arr!D1:K8
  (excel-graph, T-2235). MINIFS and AVERAGEIF are not probed at all.
- Ours (release build of this branch, scratch case with A1:A3 = a, b, c and E1:E3 = #N/A, 5, 7): AVERAGEIFS, MAXIFS,
  MINIFS, AVERAGEIF and SUMIFS over `{"a","b","c"}` are all `#N/A`, 5, 7. Every position selects on its own.
- Code: `apply_ifs`, `apply_ifs_at`, `fn_averageifs` and `fn_maxifs` in
  crates/rustsheet-engine/src/functions/statistical/if_ifs.rs.

## Probe
excel-probe skill, excel-graph, tag M9. Name the case probes/t4606-ifs-array-criterion-value-error.
- Data: A1:A4 = a, b, a, c; E1:E4 = #N/A (`=NA()`), 5, 7, #DIV/0! (`=1/0`).
- For AVERAGEIFS, MAXIFS, MINIFS, AVERAGEIF and SUMIFS as the control:
  - criteria `{"a","b","c"}` and `{"b";"c"}` (a row and a column);
  - a second criteria pair that excludes the error rows (`B1:B4` = 1, 1, 0, 1 with `">0"`);
  - an all-error position (`{"c"}`).
- Show every position with INDEX(..,1,k), and with ERROR.TYPE beside it.

## Acceptance criteria
- [x] probes/t4606-ifs-array-criterion-value-error is committed with its excel-graph oracle sidecar.
- [x] Every probe cell passes, or each visibly-off cell has an owner task.
- [x] criteria.md "*IF(S) lift array criteria" states the per-position error rule for the five functions with the
      cells, and "probed only at position 1" is gone.
- [x] If the engine changes, a unit test pins one changed cell.
- [x] No new known-failure line in eval/baseline/probes.txt.
- [x] `make check` green; reviewers approve.

## Out of scope
- AVERAGEIF numeric-like criteria with a resized average_range (T-4540).
- A `NA()` or non-range first argument with an array criterion: excel-graph refuses that workbook (criteria.md).

## Outcome
Probed excel-graph (probes/t4606-ifs-array-criterion-value-error, 86 cells): AVERAGEIFS, AVERAGEIF, MAXIFS, MINIFS and
SUMIFS agree at every position of an array criterion over an error in the value range (error only where selected rows
include it). Ours already matches every cell, so no engine change. criteria.md states the rule. Note the task's second
pair `B>0` drops row 3 only, not the error rows. Eval delta: +86 passing probe cells, no new known-failure line (one generated checked-cells line in eval/baseline/probes.txt).
