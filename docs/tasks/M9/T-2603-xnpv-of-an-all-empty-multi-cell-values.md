---
id: T-2603
title: "XNPV of an all-empty multi-cell values range is #VALUE! (probes/xnpv-empty-values-cell C8)"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`XNPV(1,A2:A3,B11:B12)` with A2:A3 empty is `#VALUE!`, as Excel (probes/xnpv-empty-values-cell C8), while an empty cell inside a values range that also holds numbers stays `#NUM!` (probes/xirr-xnpv-whole-column-nogaps A1).

## Context
Follow-up of T-2599, which made one empty values cell a 0 cash flow. C8 is a known failure (ours `#NUM!`) with no explanation yet: the refusal of an all-empty range seems to be a different check from the whole-column one. Probe first (excel-probe): an all-empty 2-cell values range with 1-cell and 2-cell dates, a values range of an empty cell and a number (`A2:A3` with A3 = 5), and dates that are all empty. Code: `get_array_of_numbers_xpnv` in functions/financial/mod.rs. Semantics: docs/excel-semantics/financial.md "XNPV reads one empty values cell as a single 0 cash flow".

Cell added by gap triage (2026-10-31, `make eval` on origin/main, excel-graph; `scripts/exit_owners.py --all-off`
listed it as unattributed, and no open task named it):
- gen-m9/58e3253bbd3a7a82 Calc!A189 `=EXPONDIST(1E+307,EXP(XNPV(100,Data!B8:C8,Data!C1:D1)),"x")`: Excel `#VALUE!`, ours
  `#NUM!`. Data!B8:C8 (the values) is blank, FALSE: no number at all, the same family as C8. Data!C1:D1 (the dates) is
  FALSE, 123456789.123. Ours: XNPV `#NUM!`, and EXPONDIST passes it before reading "x". Either Excel's XNPV is `#VALUE!`
  here (the all-non-number values rule of C8) or EXPONDIST reads its text cumulative "x" first: add
  `XNPV(100,Data!B8:C8,Data!C1:D1)` itself and `EXPONDIST(1,1/0,"x")` to the probe to tell them apart, and hand the
  EXPONDIST order to a functions-stat task if it is that.
Cells:
- gen-m9/58e3253bbd3a7a82.case.json (1): Calc!A189

## Acceptance criteria
- [x] probes/xnpv-empty-values-cell C8 passes, and the new probe cells pass or their causes are recorded in financial.md.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- **Probe** probes/t2603-xnpv-no-number-values (24 cells, excel-graph, 2026-11-03; filled twice, the second fill adding
  C17-C24 after the first rule regressed three gen-m9 cells).
- **Finding.** A multi-cell values range whose first cell is empty, a boolean or non-numeric text is `#VALUE!`, whatever
  the later cells hold. A later non-number stays `#NUM!`. The check comes after the counts and a rate at or below 0. It
  also comes after dates whose first cell is a number when a date is a boolean, non-numeric text or past the last
  serial. "No number at all" was the task's guess; C3 (empty, 5) refutes it.
- **Code.** `Model::xnpv_first_value_refused` in functions/financial/mod.rs, applied in `fn_xnpv` after the size checks.
- **financial.md**: new entry "XNPV is #VALUE! when a values range's first cell is no number". The T-2599 entry points
  to it.
- **EXPONDIST.** gen-m9/58e3253bbd3a7a82 A189 was XNPV's: EXPONDIST passes the `#VALUE!` on (C15, C16). Nothing goes to
  functions-stat.
- **Eval delta.** gen-m9 2031 → 2029 (A189 and 1b60bf99d71424e7 A171 fixed). probes 4969 → 4968 (xnpv-empty-values-cell
  C8 fixed; the new case passes 24/24). 0 regressions.

