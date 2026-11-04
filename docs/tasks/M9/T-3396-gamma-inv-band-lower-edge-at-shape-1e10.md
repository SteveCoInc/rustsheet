---
id: T-3396
title: "GAMMA.INV band lower edge at shape 1E10: GAMMA.INV(1E-80,1E10,1) is #NUM! below the cdf-ratio edge"
milestone: M9
lane: functions-stat
status: todo
priority: P0
size: S
tier: opus
depends_on: [T-3409]
campaign: semantics
---

## Goal
GAMMA.INV, CHISQ.INV and the other gamma quantiles at a huge shape are `#NUM!` exactly where Excel is, at the band's lower edge when the start lies above the mean (shapes from about 7E9).

## Context
- T-3350 replaced the band's lower edge `offset - z < 39` with a cdf ratio: Excel finds the true quantile only while
  ln P(a, x0) - ln p < 205.3 (incomplete_gamma/huge_shape.rs `BAND_LOWER_EDGE`). It separates 548 of 549 huge-shape
  quantile cells; the miss is probes/t3350-gamma-inv-band-lower-edge A122 `GAMMA.INV(1E-80,1E10,1)`, Excel `#NUM!`
  at 184.2, while A121 `(1E-70,1E10,1)` is a value at 161.2. At 7E9 and 1E10 the start's lower tail is about 1
  (the start is above the mean), so the cap there may count other steps (docs/excel-semantics/statistics.md
  "GAMMA.INV and CHISQ.INV at a huge shape").
- Probe p between 1E-70 and 1E-90 at shapes 7E9 to 1.2E10 (and CHISQ.INV at df 2a) to pin the edge there, with a
  hold-out split (decision 0188 R4).
- More of the same at 1.1E10, from T-3530's fit audit (probes-numeric/fit-audit-*, excel-graph; scored on main 4e2f5f3df after T-3350;
  docs/research/fit-audit/README.md): fit-audit-t3070-band-upper A47 `GAMMA.INV(4.11E-29,11000000000,1)`, A48
  `(3.38E-27,..)` and A49 `(2.89E-26,..)` are `#NUM!` in Excel and values in the engine. T-3409 (a model of
  Excel's search) may explain them together with A122; check it first.

- **Attempt 1 (2026-11-04, decision 0056 time-box).** probes/t3396-gamma-inv-band-huge (256 cells, excel-graph)
  pins the band at 7E9 to 1.2E10: it closes from both sides as the shape grows and is empty from 1.15E10
  (statistics.md "GAMMA.INV and CHISQ.INV at a huge shape"). Neither the cdf ratio nor the distance o - z is constant
  along the lower edge, and a 2-D cut fitted to these points would be an overfit (T-3409's owner ruling), so no rule
  changed. The probe's off cells, all GAMMA.INV values where Excel is `#NUM!`, are this task's:
  probes/t3396-gamma-inv-band-huge.case.json S!A35, S!A55:A56, S!A75:A77, S!A95:A98, S!A107:A108, S!A114:A119,
  S!A129:A140, S!A151:A161. Column B (CHISQ.INV(3E-k,2a), the hold-out fixed before scoring) matches everywhere
  (all `#NUM!`). Next: T-3409's search model, tested on these cells; this task now waits on it.

## Acceptance criteria
- [ ] A filled probe pins the lower edge at shapes 7E9 to 1.2E10; the rule separates every huge-shape quantile cell
      in eval/, A122 included, or the remaining misses are listed with the reason.
- [ ] No gated cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A model of Excel's whole search (T-3409). If T-3409's model explains A122, close this task as its duplicate.
- The band's upper edge and the closed-form branch.

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 3 probes-numeric cells that no open task listed: 3 with the wrong type or error.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/fit-audit-t3070-band-upper.case.json (3): S!A47-A49

- [ ] Every cell listed above matches Excel within half a unit of the 15th significant digit, or has Excel's type and error. Otherwise the task is blocked under decision 0056 and the rest is accepted (decision 0267).

## Visible-parity stage
Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit, or has Excel's type, error and text (decisions 0083, 0267).
