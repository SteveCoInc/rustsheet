---
id: T-4700
title: "BETA.INV at a huge shape below sqrt(DBL_MAX): Excel's 0.999999, 1 - 1/e and tiny-quantile values off the crossing (T-3862 split)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
BETA.INV matches Excel's values at a huge shape below the sqrt(DBL_MAX) cut where they are not the crossing of the
cdf, by a rule derived from Excel's search (owner rule: no fitted values).

## Context
- At beta 1E-100 or 0.5 and alpha from 1E+110 to 1.34E+154, Excel is 0.999999 at p = 1E-10 and 0.4, and
  0.6321205588285577 = 1 - 1/e at p = beta = 1E-100; the engine is 1 - 2^-53 (the true quantile rounds to 1).
  Cells: probes/t3862-beta-inv-huge-alpha A1:A9, B1:B9, C1:C9; probes/t3862-beta-inv-sqrt-dbl-max A1:A3, B1:B3,
  C1:C3, and the flipped D1:D3 `BETA.INV(0.6,0.5,a)` = 1.0000000000287557E-06 (ours 2^-53). Gate A
  accuracy/BETA.INV/s1 A183 (`BETA.INV(3.64865612912771E-25,3.42800394078672E+20,3.65494379813492E-53)`) is
  0.999999 too.
- Lead (UNVERIFIED): 1 - exp(-p/beta) is AS 109's start for a tiny second shape, 1 - exp((ln((1 - p) qq) + ln B)/qq),
  when ln B rounds to -ln qq; 0.999999 looks like that start capped at 1 - 1E-6, and Excel's search not moving from
  it (the Newton factor overflows there).
- A huge second shape with a tiny quantile: probes/t3862-beta-inv-sqrt-dbl-max E1:E6 `BETA.INV(0.1,0.5,a)`, a from
  1.3E+154 to 1.4E+154: Excel 5.85776711227727E-157 at a = 1.34078079299426E+154, ours 5.888648679911079E-157, the math (Excel 0.5% below;
  compare statistics.md "BETA.INV at a huge shape is AS 109's Newton search", early stop where tiny, T-2555).
- docs/excel-semantics/statistics.md "BETA.INV is #NUM! once the square of AS 109's first shape overflows (T-3862)".
- Code: functions/statistical/beta.rs `beta_quantile`, inverse_beta.rs `beta_quantile_crossing`.

## Visible-parity stage
Owner of the visibly-off cells of probes/t3862-beta-inv-huge-alpha A1:C9 and probes/t3862-beta-inv-sqrt-dbl-max A1:D3
and E1:E6, and of Gate A BETA.INV/s1 A183. Here Excel is the less accurate side (its 0.999999 and 1 - 1/e sit where the
math rounds to 1; at a huge second shape the engine's quantile is the math and Excel's is 0.5% below it), so the target
is reproducing Excel's own AS 109 start and stop (task-format.md: campaign `visible`).

## Acceptance criteria
- [ ] The start and stop that give 0.999999 and 1 - 1/e are derived and probed (predictions before the fill), or the
  negative result is recorded in statistics.md.
- [ ] The cells above pass where the rule covers them; the rest stay owned here.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `#NUM!` above sqrt(DBL_MAX) (T-3862) and at a beta above 1 (T-4713).
