---
id: T-4645
title: "Calipers formulas_math_trig: 17 Mac-cached one-ulp ATAN, ATAN2, TAN, COT and CSC cells the web does not give; record a ruling like T-3434's"
milestone: M5
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
The 17 calipers formulas_math_trig cells whose only gap is one ulp of Mac desktop digits have a recorded ruling and an
owner. Either Excel for the web gives the engine's value and a decision names them as desktop-only evidence, as T-3434
does for the same ATAN2 digits in COMPLEXs.xlsx, or the web disagrees with the engine and a functions-math task is
filed for each cell. Today they are known failures (eval/baseline/calipers.txt) that no open task names.

## Context
- Gap triage 2026-11-06 (report of `make eval` at 37389d18). A scan of every failing cell against open-task ownership
  (exit_owners `owned`) leaves these 17 unowned. All are NumericUlp, 1 ulp (`ulp1`), so they are not visibly off and
  do not count toward a Phase 1 exit. Hence `defer: phase-1b` (decision 0114).
- The golden was saved by Excel for Mac 16.0300 (decision 0315). Decision 0315 excepts its visibly-off trig cells at
  multiples of pi/2 (T-4534's probes/t4534-desktop-golden-web-values). These one-ulp cells were out of that probe's
  scope. Cells, Excel (Mac cache) vs ours (`rustsheet-eval show calipers/roundtrip/formulas_math_trig/golden.xlsx`;
  C6 = PI()/6, C7 = PI()/4, C8 = PI()/3, C9 = PI()/2, C10 = PI(), C12 = 2*PI(), C13 = -PI()/4, C34 = -0.5, C36 = 0.5):
  - Sheet1!E19 `TAN($C$6)`: 0.5773502691896256 vs 0.5773502691896257
  - Sheet1!G19 `CSC($C$6)`: 2.0000000000000004 vs 2
  - Sheet1!H19 `COT($C$6)`: 1.7320508075688776 vs 1.7320508075688774
  - Sheet1!H20 `COT($C$7)`: 1.0000000000000002 vs 1; Sheet1!H26 `COT($C$13)`: -1.0000000000000002 vs -1
  - Sheet1!E46 `ATAN($C$34)` and E48 `ATAN($C$36)`: ±0.46364760900080615 vs ±0.4636476090008061
  - Sheet1!E60 `ATAN($C$10)`: 1.2626272556789115 vs 1.2626272556789118
  - ATAN2 cells whose quotient is 1/2 or 2: Sheet1!F68, G69, H71, J72 (0.46364760900080615 vs ...061), G75
    (-0.46364760900080615 vs ...061), D70, E71, G72, H74 (1.1071487177940906 vs ...904)
- Web evidence already in eval/ agrees with the engine on several of these inputs, but not on all of them:
  numbers.md (trig reciprocal entry, probes/trig-reciprocal-special-angles, excel-graph) gives `COT(PI()/4)` = 1,
  `CSC(PI()/6)` = 2 and `COT(PI()/6)` on the engine's value. probes/t2198-m5-lift-argument-kinds D17/D18
  (`ATAN2(2,1)`, `ATAN2(1,2)`) give 0.4636476090008061 and 1.1071487177940904, as does probes/atan2-quadrants.
  `ATAN(0.5)`, `ATAN(PI())`, `TAN(PI()/6)`, `COT(-PI()/4)` and the ATAN2 calls on PI() multiples have no web cell yet.
- Same digits, same root cause as T-3434 (COMPLEXs.xlsx IMARGUMENT H2, H3, H10: Mac 0.46364760900080615 and
  1.1071487177940906) and decision 0086 (ATAN2_POWER.xlsx D26). One decision record may cover both tasks.
- Precedents: decisions 0074, 0086, 0196, 0315. Decision 0039 item 4: web-vs-desktop conflicts match the web.
  Decision 0266: desktop work is out of scope. eval/milestones/ is protected (decision 0044).

## Acceptance criteria
- [ ] Fresh web evidence: a probes case (excel-probe skill, excel-graph) with the 17 cells' own formulas on literal
      inputs equal to the golden's C cells (`PI()/6`, `0.5`, ...), filled. Ours passes every cell, or the task files a
      functions-math task for each cell it fails.
- [ ] If the web agrees with ours: a decision record (`make new-decision`, or an amendment of T-3434's record if it
      has merged) names the 17 cells as desktop-only evidence. It gives each cell's Mac and web value. The cells stay
      known failures: the 17 lines stay in eval/baseline/calipers.txt and no new line appears there. numbers.md gets a
      sentence on Mac's one-ulp-up atan and tan digits, citing the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The visibly-off trig cells at multiples of pi/2 (decision 0315).
- COMPLEXs.xlsx IMARGUMENT (T-3434), except as a shared decision.
