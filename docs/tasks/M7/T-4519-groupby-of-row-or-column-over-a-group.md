---
id: T-4519
title: "GROUPBY of ROW or COLUMN over a group's values: ARRAYTOTEXT of the result is #CALC!, and RANDARRAY's group value is a scalar"
milestone: M7
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
GROUPBY's result for a function that returns an array of errors, or an array, per group reads in
ARRAYTOTEXT and INDEX as Excel's does.

## Context
- probes/t4450-eta-special-forms (T-4450, excel-graph): `GROUPBY(A1:A3,B1:B3,_xleta.ROW)` is 4x2
  with `#VALUE!` in each value cell (`ERROR.TYPE` of it is 3 in H109:H112; `INDEX(..,1,2)` is
  `#VALUE!`, J9), yet `ARRAYTOTEXT` of the whole result is `#CALC!` (F109; COLUMN F115;
  `LAMBDA(v,ROW(v))` J65), while `ARRAYTOTEXT(INDEX(..,0,1))` is "1, 3, 5, Total" (J15) and
  SHEET's `#N/A` cells print (F121). The engine prints "1, #VALUE!, ..." for F109, F115 and J65.
  A guess to test first: the group's value is ROW lifted over the values (an array of `#VALUE!`),
  which a cell shows as `#VALUE!` (an array result, as IF and ROUND, arrays.md "GROUPBY and
  PIVOTBY") but ARRAYTOTEXT refuses as a nested array.
- `ERROR.TYPE(INDEX(GROUPBY(A1:A3,B1:B3,_xleta.RANDARRAY),1,2))` is one `#N/A` cell in Excel (J63:J64): the
  engine's element is an array and the ERROR.TYPE spills (J64).

## Acceptance criteria
- [ ] (moved to T-4610 and T-4611, which own these cells) F109, F115, J64 and J65 of probes/t4450-eta-special-forms pass, with a probe of the guess
      (excel-probe skill), recorded in docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY".
- [x] The eval gate shows 0 regressions (probes/groupby-*, probes/t4449-*).
- [x] `make check` green; reviewers approve.

## Out of scope
- SHEET and SHEETS of a LAMBDA parameter (T-4518).

## Outcome
Probed and split. The fixes go to two owner tasks, because they are two separate engine changes.
- Probe probes/t4519-row-column-randarray-over-arrays (excel-graph, 2026-11-06), 11 forms. The guess holds: ROW and COLUMN of an
  array value give an array of `#VALUE!` in the value's shape, so a GROUPBY cell is a nested `#VALUE!` array, and ARRAYTOTEXT
  refuses it with `#CALC!`. RANDARRAY lifts over an array `rows` or `columns` (`RANDARRAY({2})` is 1x1), so GROUPBY's RANDARRAY cell is one
  number per record. Recorded in arrays.md as "ROW, COLUMN and RANDARRAY over an array value (GROUPBY's per-group function)", next to "GROUPBY and PIVOTBY".
- Refused forms: Excel for the web refuses a workbook holding ROW or COLUMN of an array constant, of a function call or of an
  operator: `ROW({1;2})`, `ROW({1,2,3})`, `COLUMN({1,2,3})`, `ROW({7})`, `COLUMN({1;2})`, `ROW(SEQUENCE(2,2))`, `COLUMN(SEQUENCE(1,3))` and
  `ROW(A1:A3*1)` (FileCorruptTryRepair, bisected with single-formula workbooks). The case reaches ROW through LET instead.
  F2:F4, F7 and F8 print random numbers and are not scored; D2:E4 and G2:H4 (shape and type of a volatile call) are scored as `invariant`.
- Owners: T-4610 (opus) owns ROW and COLUMN: t4519 D1:F1, D5:F6, D9:F11 and t4450 F109, F115, J65. T-4611 (sonnet) owns RANDARRAY's lift:
  t4519 D2, D3, E3, G3, H3, D4, D7, D8, E8, G8, H8 and t4450 J64.
- No engine change. Eval delta: probes gain 1 case (50 scored cells: 21 pass, 29 new-coverage failures, all owned); 0 regressions.

## Landing notes (resumed)
- PR #3622 (draft), branch agent/engine/T-4519-groupby-row-column. The head is the commit that adds these notes.
- Round 1 on c35f686: security APPROVE, design APPROVE (MINOR: name the split in the unchecked box; done here), excel-fidelity
  REQUEST_CHANGES. Its two MAJORs: rows 2-4 (direct RANDARRAY) were unscored because the harness skips volatile cells; and T-4611 said
  they "pass already by shape". This commit fixes both. D2:E4 and G2:H4 are now `invariant` and scored (6 new owned failures: D2, D3, E3, G3, H3, D4),
  and T-4611, arrays.md and the Outcome are corrected. The two fidelity MINORs (two refused forms missing from arrays.md, and the
  "top-left" claim not probed) are fixed too.
- The PR conflicted with main in eval/baseline/ only. `make catch-up` (36f2807) cleared it, then a second clean merge of origin/main (847d85e).
  The baselines were then regenerated from main's copy with `make eval-baseline`: probes +1 C line and +29 F lines, visible-parity +1 line, 0 regressions.
- Next step: excel-fidelity round 2 on this head. Security and design approved c35f686 and must re-run only if a reviewer counts
  the case or docs change as theirs; the change is a case-file `invariant` list, docs and baselines. Then mark the PR ready, and enqueue
  it once CI is green.
- Do not restore eval/baseline/ from a stale origin/main ref: merge main first, then regenerate (an earlier attempt pulled in
  newer PRs' baseline lines).
