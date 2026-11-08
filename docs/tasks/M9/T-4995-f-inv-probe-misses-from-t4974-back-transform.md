---
id: T-4995
title: "F.INV probe misses from t4974: back-transform and tiny-tail neighbourhood"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
F.INV / F.INV.RT / FINV match Excel on the visibly-off cells of the t4974 F probes (PR #4002 groups G08, G09).

## Context
Found by T-4974, which filled the probe suggestions of PR #4002 (docs/research/probe-suggestions-2026-11-07, merged to main as 8001d80b1)
with excel-graph. The predictions there came from a prototype of the 2026-11-07 Excel-parity fixes that is not on main;
compare against them before designing a fix.
- G09: `F.INV(1E-300,11,3)` (E7) and `F.INV(1E-250,11,3)` (E11) are `#NUM!` in Excel, numbers in the engine; B9, D8 differ in the
  9th digit.
- G08: the back-transform columns `d1*p/(d2*(1-p))` differ visibly where the engine's beta quantile does (AE3, AE9, C8, W8), and
  C7 / AE7 are `#DIV/0!` in Excel against the engine's `#NUM!`.

Owned cells (28, visibly off against Excel):
- probes/t4974-f-inv-back-transform.case.json (24): P!B7, C7:C8, D8, L1, O1, P1, W8, X8, AA1, AA5, AA8:AA9, AB5, AB8:AB9, AD7, AE3, AE7, AE9, AE11, AF3, AF9, AF11
- probes/t4974-f-inv-tiny-tail-neighbourhood.case.json (4): P!B9, D8, E7, E11

## Acceptance criteria
- [ ] The owned cells are within half a unit of Excel's 15th significant digit (or the same error) and leave
      eval/baseline/probes.txt (`make eval-baseline`), with no new known-failure lines.
- [ ] docs/excel-semantics/statistics.md (financial.md for IRR) records the rule the probe settles.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The last-digit cells of the same cases.
