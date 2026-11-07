---
id: T-3412
title: "GAMMA.DIST and CHISQ.DIST lower tail at a shape above 1: Excel's leading-factor series form (T-3060's), a staircase in x (304 of 330 probed cells, ours 46)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
GAMMA.DIST, GAMMADIST, CHISQ.DIST and CHIDIST's lower tail at a shape above 1 and x below 1 is Excel's leading-factor
series form, as T-3060 made it below shape 1, so the probe cells T-3311 left visibly off reach Excel's value within
half a unit of the 15th digit; or the Outcome lists the forms and regions tried with their counts (decision 0056).

## Context
T-3311 (docs/excel-semantics/statistics.md, T-3311 paragraphs of "Deep lower tail at a shape above 1") found that
Excel's lower-tail cdf at shapes 1.5 to 15 is `EXP(a*LN(x))/EXP(GAMMALN(a+1))` times e^-x and the series (T-3060's
form, `incomplete_gamma::excel_form_lower`). It is within an ulp at 304 of the 330 probed cells
(probes-numeric/t3311-gamma-inv-lower-tail-cdf, sheet S columns B-J and L, sheet W columns C-E), where the engine's
accurate pair is within an ulp at 46. LN(x) moves by an ulp only every few ulps of x, so Excel's cdf is a staircase,
flat across ±4 ulp of x around the probed quantiles; the pair is smooth. The engine's GAMMA.DIST takes the form only
at a shape below 1 (`excel_small_shape_lower`, where a·LN(x) is at most -16). One ordering variant,
`lead * (EXP(-x) * series)`, reaches 307 of 330, and the form's own 1-ulp residue is T-3242's.

Cells. These GAMMA.DIST/CHISQ.DIST cells of probes-numeric/t3311-gamma-inv-lower-tail-cdf moved farther from Excel
with T-3311, because they are evaluated at the quantile in column A, which moved. Ulps against Excel, main to HEAD:
- newly visibly off (5): W!C56 (-31 to +68), W!D56 (-23 to +77), W!E56 (-15 to +84), W!F56 (density, -3 to +43),
  W!F31 (density, -11 to -20);
- farther but within 15 digits (6): W!C31 (-8 to -22), W!D31 (-6 to -20), W!E31 (-3 to -18), W!D23 (-6 to -25),
  W!E23 (-3 to -23), S!L6 (-2.5 to +6).

The probe's other visibly-off cdf cells (sheet S columns B-J and L, sheet W columns C-F at shapes above 1) are this
task's too: at the deep-tail rows (S rows 1, 2, 14) the pair is 100 to 200 ulp from Excel's staircase.

## Visible-parity stage
Decision 0083: the cells this task owns are visibly off, so it is in the campaign. Its target is that every visibly-off
cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value;
bit-level misses left then go to a `defer: phase-1b` follow-up.

## Acceptance criteria
- [ ] The listed probe cells are within half a unit of Excel's 15th digit, or the Outcome lists the forms and regions
      tried with their bit-exact and visible counts on a hold-out split (decision 0188 R4).
- [ ] The region where the form applies is the measured one (shapes, x and a·LN(x)), with a `FITTED` tag if chosen by
      scoring; no gated cell regresses; no new known-failure line in eval/baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The quantiles (T-3311, T-3403, T-3294); shapes below 1 (T-3060, T-3242).

## Outcome
Excel's lower region at a shape above 1 is three forms, now the engine's (`incomplete_gamma::excel_lower`,
`excel_lower_region_upper`, `excel_lower_density`; docs/excel-semantics/statistics.md "Lower region at a shape above 1
(T-3412)"): the cdf up to x/scale = 1 is T-3060's leading-factor form at every shape above 1; the upper tail below
x = a + 1 at shapes 1 to 20 is (0.5 - P) + 0.5; the density up to x/scale = 1 is the leading factor times e^-y a / x.
No new fitted constant (the cuts are `quantile_tail`'s x = 1 and `DIRECT_SHAPE_MAX`).
- Hold-out (30% hash of case and cell, fixed before scoring) and fresh sweeps after each freeze (commits 5d8b1b79,
  56bf7cc5, 2e1d1dbc): probes-numeric/t3412-fresh-sweep (280), t3412-fresh-sweep-2 (110), t3412-density-sweep (120);
  counts in statistics.md and the doc comments.
- The listed probe cells of this task are cdf and density cells at column-A quantiles: at Excel's own quantile the
  form is within 1 ulp of Excel at all of them; the engine's quantile there is 1 to 4 ulp off Excel's, which T-3403
  owns, so 16 cdf and 4 density cells of t3311-gamma-inv-lower-tail-cdf sheet W and S are newly visibly off through
  their input (decision 0343).
- Eval against a9735e10, per cell: gated 34 fixed, 16 closer, 8 visibly fixed, 3 lost by 1 ulp, and in the probes suite GAMMA.DIST(1E-50,2,1,FALSE) bit-exact to 1 ulp (decision 0343, owner
  ruling "Allow, with records"); ungated 270 fixed, 208 visibly fixed. Gate B 465 to 469 of 506.
- Follow-ups: T-4959 (cdf at 1 < x/scale < a + 1, where neither form is Excel's), T-4960 (the density's subnormal
  flush, one cell).
