---
id: T-4534
title: "Own the cells T-4207 listed that decision 0308 does not except"
milestone: M9
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Each cell below has an owner that matches what it needs: a fix, a Phase 1 rate exception backed by its own evidence and
decision, or a recorded "not visibly off". None of them is left out of the Phase 1 rate without such a decision.

## Context
T-4207 named these cells for the M14 tasks decision 0266 dropped (T-1075, T-2663, T-2704). Decision 0308 excepts only the
FORECAST.ETS family and CELL's desktop-only info types, so these stay counted. A first draft of the 0308 PR put them in
T-4373 (blocked, M14), which made decision 0143's desktop-owned path leave 19 of them out of the Phase 1 rate with no
ruling (excel-fidelity review): SEC 2, COT 4, TAN 4, COS 2, CSC 2, SIN 2, SUMPRODUCT 1, CHAR 1, FVSCHEDULE 1. They are
owned here instead, an open non-M14 task, so they count. Many are calipers goldens (desktop-authored) or ironcalc-calc and
poi cells where the cached value is desktop Excel's; some are already exit exceptions (eval/milestones/exit-exceptions.tsv)
or not visibly off. Triage each against excel-graph: web-fixable cells get a fix or a functions task; desktop-only ones
need a decision like 0308 with per-cell oracle evidence.
Cell lists:
- probes/forecast-ets-longer-series.case.json (2): Sheet1!K1:K36, K39:K42
- ironcalc-calc/ENGINEERING/COMPLEXs.xlsx (4): Sheet1!AL12, AL26, K25, N6
- ironcalc-calc/ENGINEERING/BESSEL.xlsx (2): All!D18, F18
- ironcalc-calc/LOGICAL/IF_ARRAY.xlsx (3): Sheet1!C4, C11, E4
- ironcalc-calc/MATH_AND_TRIGONOMETRY/ATAN2_POWER.xlsx (1): Sheet1!D14
- poi/FormulaEvalTestData_Copy.xlsx (5): EverythingTests!G936, H936, J136, J1328, K936
- calipers/pv_goldens/pv26_011/golden.xlsx (1): CD market model!B3; UC market model!B3
- calipers/pv_goldens/pv26_2_011/golden.xlsx (1): CD market model!B3; UC market model!B3
- calipers/roundtrip/formulas_engineering/golden.xlsx (4): Sheet1!D26, D27, J26, J27

## Acceptance criteria
- [ ] Every cell above is triaged (fix, task, exception by decision, or not visibly off), recorded in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Decision 0308's cells (T-4373).

## Progress (2026-11-06, partial PR)
probes/t4534-desktop-golden-web-values (excel-graph): Excel for the web gives the engine's value on the calipers
formulas_math_trig trig cells at multiples of pi/2, formulas_text C190 `CHAR(255)` and formula_stress_test Information!B3
`INFO("release")`. Decision 0315 lists those 18 cells as Phase 1 rate exceptions (owner T-4373), and they left this list.
Still here: poi EverythingTests (J136, G936, H936, K936 are exceptions under 0134; J1328 SUMPRODUCT), ironcalc-calc
ENGINEERING, IF_ARRAY and ATAN2_POWER (exit exceptions or not visibly off), the forecast-ets-longer-series cells
(decision 0308's too) and calipers formulas_engineering.

## Progress (2026-11-06, second partial PR)
probes/t4534-bessel-convert-web-values (excel-graph, 14 cells) settles three more groups:
- ironcalc-calc ENGINEERING/BESSEL.xlsx All!D18 and F18 (`BESSELJ(100000,5)`, `BESSELY(100000,5)`; Mac desktop cache
  0.001846551246313725 and 0.001719431947654535): Excel for the web gives the engine's 0.0018465512463137278 and
  0.0017194319476545319 (A1:A2, F1:F2). Both are within 15 digits (NumericClose), so not visibly off, and
  `scripts/exit_owners.py --all-off` lists neither with this task closed.
- calipers roundtrip/formulas_engineering Sheet1!D26, D27, J26, J27 (CONVERT mi to km, in to cm): the web gives the
  golden's values and the engine is one ulp off (B1:B4, D1:D4), so they are web-fixable: T-4630 owns them and the
  eight probe cells.
- calipers pv26_011 and pv26_2_011 `CD market model!B3`, `UC market model!B3` (`CELL("filename")`): already decision
  0308's lines (the web gives the engine's `#VALUE!`, probes/t4534-desktop-golden-web-values).
- poi EverythingTests!J1328 `SUMPRODUCT(G1330:G1330,I1330:I1330)`: Microsoft 365 gives the engine's 0
  (probes/t2921-sumproduct-one-cell-numeric-text A45; coercion.md, CONFLICTING with the Excel 15 cache).
Still owned here, and why this task stays open: with it closed, the 40 probes/forecast-ets-longer-series cells have no
open M9 owner (T-4373 is M14) and the four ironcalc-calc COMPLEXs exit-exception cells (N6, AL12, AL26, K25) have no
open owner (`exit_owners.py --milestone M9` and `--all-off` against a copy of docs/tasks with this task done). Closing
it needs an open M9 owner for those cells and the owner column of the T-4534 lines of eval/milestones/
exit-exceptions.tsv and phase1-exceptions.tsv moved (a protected path: a decision). J1328 needs a line like J136's
(decision 0134 mechanism) or an owner the same way.
