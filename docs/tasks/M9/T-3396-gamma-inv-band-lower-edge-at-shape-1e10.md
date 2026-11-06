---
id: T-3396
title: "GAMMA.INV band lower edge at shape 1E10: GAMMA.INV(1E-80,1E10,1) is #NUM! below the cdf-ratio edge"
milestone: M9
lane: functions-stat
status: done
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
  S!A129:A140, S!A151:A161. Column B (CHISQ.INV(3E-k,2a)) matches everywhere (all `#NUM!`), but its df, 1.4E10 to 2.4E10,
  is above CHISQ.INV's deg_freedom limit of 1E10, so it is the domain limit and not a hold-out for any band rule (T-4202). Next: T-3409's search model, tested on these cells; this task now waits on it.

## Acceptance criteria
- [x] A filled probe pins the lower edge at shapes 7E9 to 1.2E10; the rule separates every huge-shape quantile cell
      in eval/, A122 included, or the remaining misses are listed with the reason.
- [x] No gated cell regresses.
- [x] `make check` green; reviewers approve.

## Out of scope
- A model of Excel's whole search (T-3409). If T-3409's model explains A122, close this task as its duplicate.
- The band's upper edge and the closed-form branch.

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 3 probes-numeric cells that no open task listed: 3 with the wrong type or error.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/fit-audit-t3070-band-upper.case.json (3): S!A47-A49

- [x] Every cell listed above matches Excel within half a unit of the 15th significant digit, or has Excel's type and error. Otherwise the task is blocked under decision 0056 and the rest is accepted (decision 0267).

## Visible-parity stage
Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit, or has Excel's type, error and text (decisions 0083, 0267).

## Outcome
Closed as a duplicate of T-3409, as "Out of scope" says to when T-3409's model explains A122. No code change.
T-3409 (#3175, merged) replaced the band cuts with a model of Excel's search: Newton's method on the
odds from the Wilson-Hilferty start, which gives up with `#NUM!` after 207 steps (`huge_shape.rs`
`search_converges`). On main 966090b73 (`rustsheet-eval show`) that model gives every cell this task names Excel's type:
- probes/t3350-gamma-inv-band-lower-edge A122 `GAMMA.INV(1E-80,1E10,1)` is `#NUM!`, and A121 is a value.
- Every attempt-1 cell of probes/t3396-gamma-inv-band-huge (A35, A55:A56, A75:A77, A95:A98, A107:A108, A114:A119,
  A129:A140, A151:A161) is `#NUM!`. The case has no type or error mismatch left.
- The decision 0267 cells, probes-numeric/fit-audit-t3070-band-upper A47:A49, are `#NUM!`.

Across every eval case, the only huge-shape gamma quantile cells left with the wrong type are T-4206's:
fit-audit-t3070-band-upper A39 and t3409-r4-gamma-inv-search-sweep A177 and A191. This comes from scanning the
`make eval` report on b450050f8, which already includes T-3409. The case's remaining 33 known failures are
`NumericUlp`, 1 to 3 ulp off at the 16th or 17th digit, so none is visibly off. They are the band's true-quantile
last digits, which belong to T-3272; that task's Context now names the case. Column B's wording is T-4202's.

Eval delta: none. No code changed, and T-3409's PR already updated the baseline.
