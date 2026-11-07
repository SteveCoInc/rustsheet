---
id: T-5014
title: "Which Newton iterate Excel returns once its quantile steps cycle (T-3403 remainder: six probe quantiles 6 to 12 ulp off)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3403]
---

## Goal
Excel's choice among the cycling Newton iterates of its lower-tail gamma quantile, so the cells below reach Excel's
value within half a unit of the 15th digit; or the Outcome lists the rules tried with their counts (decision 0056).

## Context
T-3403 made the quantile at shapes 1 to 20 and p up to 1E-10 the Newton steps from the leading-factor root on Excel's
staircase cdf, keeping the iterate whose cdf is nearest p (`incomplete_gamma::closed_form::excel_newton_quantile`).
Excel's value is one of the iterates at most points but not always the nearest one. Rules already tried and scored
below nearest-F (T-3403 Outcome): a fixed step count 0 to 120, a step under 1E-16 to 1E-8 of x, a residual under
1E-16 to 1E-12 of p, and the root first when its cdf is within 5E-14 of p.

Cells that moved past the 15th digit with T-3403 (ungated, ulps of Excel, before to after):
- probes-numeric/t3311-gamma-inv-lower-tail-cdf W!A37 `GAMMA.INV(1E-16,3.5,1)` 6 to 12; W!G55 `CHISQ.INV(1E-30,10)`
  and W!H55 `GAMMA.INV(1E-30,5,2)` 4 to 6; W!G71 `CHISQ.INV(1E-30,15)` and W!H71 `GAMMA.INV(1E-30,7.5,2)` 2 to 8;
- probes-numeric/t3311-fresh-sweep-2 S!A321 `CHISQ.INV(6.81E-34,11)` 2 to 6.
On T-3403's own fresh sweep two cells stay past the 15th digit:
- probes-numeric/t3403-fresh-sweep Calc!A31 `GAMMA.INV(2.6E-67,15.15,2)` (2 to 5 ulp; Excel 5.218691678707849E-4);
- probes-numeric/t3403-fresh-sweep Calc!A114 `CHISQ.INV(3.6E-298,19)` (47 to 48 ulp; Excel 4.250751250226025E-31),
  near p = 1E-298 where Excel is erratic (T-3294).
The 12 GAMMA.DIST cells of t3311-gamma-inv-lower-tail-cdf sheet W that take these quantiles as input (rows 37, 38,
46, 55, 71, 80, 87 and their neighbours) move with them. Lead: Excel may stop on a step-size test in x87 extended
precision, or iterate on the density's own staircase.

## Acceptance criteria
- [ ] The six quantile cells within 15 digits of Excel, or the Outcome lists the rules tried.
- [ ] Any rule passes decision 0188 R4; no gated cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- p above 1E-10 (the search) and the moderate-p right tail (`CHIINV(0.975,5)`, T-3146).
