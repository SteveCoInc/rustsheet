---
id: T-3407
title: "IRR returns #DIV/0! on long conventional series near -0.2 to -0.5 (9 hold-out cells; the engine is #NUM!)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
IRR returns `#DIV/0!` where Excel does on conventional cash flows, instead of `#NUM!`.

## Context
- probes-numeric/t3347-irr-conventional-holdout A123, A126, A130, A131, A132, A134, A135, A138 and A143 (9 cells of
  the neg-long stratum): long series (36-60
  flows, uneven or level, scales 1 to 1e10) after one outlay, exact root r = -0.23 to -0.47. Excel (excel-graph,
  2026-10-31) is `#DIV/0!`; the engine is `#NUM!`, with or without T-3347's stage. The other 15 cells of the stratum
  are `#NUM!` in both.
- A Python replica of the secant (T-3347 Outcome) shows that on 7 of the 9 the first secant step lands where NPV
  overflows (no finite value), and on A131 and A138 the secant goes flat at step 3 and both Newton runs overflow; the
  `#NUM!` cells of the stratum never overflow. A first guess: an overflowing NPV inside Excel's iteration surfaces as
  `#DIV/0!` instead of failing the run. Probe it with short series built to overflow at the first step, and with a
  guess that avoids it.
- docs/excel-semantics/financial.md "IRR on conventional flows near 0 and -10%" (the hold-out paragraph).
- Code: `irr_npv`, `irr_secant`, `compute_irr` (functions/financial/financial_util.rs).

## Acceptance criteria
- [ ] A probe case (excel-graph) that isolates when IRR is `#DIV/0!`, its cells listed here before the fix.
- [ ] The engine returns `#DIV/0!` on those cells and on the 9 hold-out cells, with no gated cell regressing.
- [ ] The rule in docs/excel-semantics/financial.md.
- [ ] `make check` green; reviewers approve.


## Cells of T-3411's fresh sweeps
T-3411's R4 sweeps 3 and 4 (cases added by PR #3063, scored on its round-4 model), where Excel is #DIV/0!, the engine #NUM!:
- probes-numeric/t3411-irr-r4-fresh-3.case.json (12): Calc!A6, A18, A19, A21, A23, A24, A78, A79, A81, A85, A89, A93
- probes-numeric/t3411-irr-r4-fresh-4.case.json (16): Calc!A2, A3, A4, A11, A13, A14, A15, A54, A68, A77, A85, A86, A89, A94, A103, A147

## Cells left by T-3411's close (gap triage 2026-11-04b)
T-3411 owned these until it closed (PR #3063); no open task named them after it. Excel is `#DIV/0!`, the engine
`#NUM!` (`make eval` on origin/main a7b115e9e; the engine stays `#NUM!` on all 38 with the fallback's Newton cap
lifted, so they are not a cap effect). On t3406 these are the level annuities where T-3411's replica put
log10 |NPV(d2)| above 228 (T-3411 progress notes, 2026-11-02):
- probes/t3406-irr-num-boundary.case.json (21): P!A66, A67, A68, A69, A70, A71, A72, A81, A82, A84, A85, A86, A87, A88, A89, A90, A91, A92, A93, A94, A95
- probes-numeric/t3411-irr-r4-fresh.case.json (7): Calc!A3, A5, A77, A83, A85, A88, A91
- probes-numeric/t3411-irr-r4-fresh-2.case.json (10): Calc!A1, A6, A11, A16, A19, A52, A82, A91, A92, A98

## Out of scope
- Which series return a number (T-3406).
