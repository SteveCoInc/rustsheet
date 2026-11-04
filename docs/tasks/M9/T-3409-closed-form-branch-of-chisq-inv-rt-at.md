---
id: T-3409
title: "Model Excel's gamma inverse search at a huge shape: its start, step rule, step limit and give-up value"
milestone: M9
lane: functions-stat
status: done
priority: P0
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
GAMMA.INV, GAMMAINV, CHISQ.INV, CHISQ.INV.RT and CHIINV at a large and huge shape return Excel's value because the engine
reproduces how Excel's inverse search actually works (its start, step rule, step limit and what it returns when it gives
up), not because of more fitted cuts. In particular `CHISQ.INV.RT(1E-300,df)` for df 5E6 to 1E9 is Excel's value.

## Context
- **Owner ruling (2026-10-31, on T-3350's review): R3 is NOT widened.** Asked whether decision 0188 R3's closed-form
  case should cover these cells, the owner said: "I want you to match Excel here but not overfit. Can you just explore
  further and crack how Excel works in this region?" So the target at the cells below is Excel's value (directive 1),
  and no MATH-OVER-EXCEL case is added or widened.
- The cells: probes/t3350-chisq-inv-huge-df A205, A217, A229, A241, A253, A265, A277, A289 (`CHISQ.INV.RT(1E-300,df)`,
  df 5E6 to 1E9). Excel's value is its Wilson-Hilferty start x0 (the T-3070 constants, within 3.4E-9), 2.6E-4 to
  2.7E-4 above the 50-digit math, where the gamma density at x0 underflows. At df 2E6 (A193) Excel is the true
  quantile. The engine returns the true quantile, so these stay F lines of eval/baseline.
- **Hypothesis to test (UNVERIFIED).** Excel runs one search: it starts from x0, takes Newton-like steps, stops after a
  step limit, and on giving up returns x0 (or `#NUM!`). Three facts so far look like that one search:
  - T-3350's cut `BAND_LOWER_EDGE` (huge_shape.rs: ln P(a, x0) - ln p < 205.3) reads as Newton from the inner side of
    a convex tail giving up after about 200 steps, each dividing the tail by about e.
  - The x0 cells above look like the same give-up, where the density at x0 underflows and a step cannot move.
  - The closed-form branch from shape 1E9 (T-3070) and the band's upper edge (`BAND_UPPER_SLOPE`, `BAND_UPPER_EDGE`)
    may be the same thing.
  If one model explains all of them, it replaces the fitted cuts `BAND_UPPER_SLOPE`, `BAND_UPPER_EDGE` and
  `BAND_LOWER_EDGE`, and T-3396's miss (`GAMMA.INV(1E-80,1E10,1)`, `#NUM!` where the cut says a value) is the test of
  it.
- Evidence to start from: probes/t3396-gamma-inv-band-huge (the band closing from both sides at 7E9 to 1.2E10,
  T-3396's attempt 1), probes/t3350-chisq-inv-huge-df, probes/t3350-gamma-inv-band-lower-edge,
  probes-numeric/t3350-r4-chisq-inv-sweep, probes/t3070-gamma-inv-band, probes/t2982-gamma-inv-huge-shape;
  docs/excel-semantics/statistics.md "GAMMA.INV and CHISQ.INV at a huge shape"; docs/research/t3350/sweep.py (the
  mpmath reference).
- **Wrong-type cells from T-3530's fit audit (probes-numeric/fit-audit-*, excel-graph; scored on main 4e2f5f3df after T-3350;
  docs/research/fit-audit/README.md).** These are semantic (an error against a number), so the model must place them:
  - `HUGE_SHAPE_MIN` (1E9) is not Excel's edge. Excel is already `#NUM!` from shape 6E8 at p = 1E-300, 7E8 at 1E-250
    and 9E8 at 1E-200 (A20, A21, A24 and A27 are numbers), where the
    engine (below the huge-shape path) gives the true quantile: fit-audit-t3070-shape-min A19, A22, A23, A25, A26,
    A28, A29, A30 (`GAMMA.INV(1E-300,6E+8,1)` to `GAMMA.INV(1E-200,9E+8,1)`).
  - The band's upper edge (`BAND_UPPER_SLOPE`, `BAND_UPPER_EDGE`): Excel is `#NUM!` where the engine is inside the band
    in the upper tail at shapes 3E9 to 5E9: fit-audit-t3070-band-upper A1, A2, A3, A5, A6, A7, A12, A13 (for example
    `GAMMA.INV(1-3.84E-12,3000000000,1)`).
  - The reverse, Excel a value where the engine is `#NUM!`, in the lower tail at shapes 7E9 to 9E9:
    fit-audit-t3070-band-upper A26, A32, A33, A38, A39 (for example `GAMMA.INV(0.00434,7000000000,1)`, Excel
    6999780428.321592).
  - At 1.1E10 in the lower tail Excel is `#NUM!` where the engine has a value: A47, A48, A49 (T-3396's edge).
  - The lower edge (`BAND_LOWER_EDGE` after T-3350) separates all 42 cells of fit-audit-t3070-band-lower.

## Acceptance criteria
- [x] A written model of Excel's search (start, step rule, step limit, the value on giving up), with probes designed
      to tell it apart from the alternatives (for example p just either side of a predicted step count, and shapes
      from 1E6 to 1E10), filled by the oracle.
- [x] Reviewer check 7 (decision 0188 R4): the model's free parameters are frozen on a training set and stated before a
      held-out set is scored; then a fresh probes-numeric sweep, drawn after the freeze, is scored against Excel and
      the 50-digit math. The Outcome puts train, held-out and fresh-sweep results side by side.
- [x] The fit audit's wrong-type cells above have Excel's type, except fit-audit-t3070-band-upper A39 (moved to T-4206).
- [x] No cell of the probes above that matches Excel now regresses. Cells A205-A289 being Excel's value within 15
      digits moved to T-4205 (the start's digits below shape 1E9; decision 0267's time-box).
- [x] Where the model replaces a fitted constant, its FITTED tag goes and quality/fitted.json's count falls; any constant
      the model still needs carries a FITTED tag with its hold-out result.
- [x] docs/excel-semantics/statistics.md describes the model, and its status (VERIFIED only where the probes show it).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Widening decision 0188 R3, or any new MATH-OVER-EXCEL case (the owner ruled against it).
- The digits of the closed form at shape 1E9 and up (T-3272) and its move to the math under R3 (T-3364), unless the
  model shows they are the same mechanism; then say so in those tasks.
- Small-shape quantiles (below shape 1E5).

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 21 probes-numeric cells that no open task listed: 21 with the wrong type or error.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/fit-audit-t3070-band-upper.case.json (13): S!A1-A3, A5-A7, A12-A13, A26, A32-A33, A38-A39
- probes-numeric/fit-audit-t3070-shape-min.case.json (8): S!A19, A22-A23, A25-A26, A28-A30

- [x] (20 of 21; S!A39 of fit-audit-t3070-band-upper moved to T-4206) Every cell listed above matches Excel within half a unit of the 15th significant digit, or has Excel's type and error. Otherwise the task is blocked under decision 0056 and the rest is accepted (decision 0267).

## Visible-parity stage
Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit, or has Excel's type, error and text (decisions 0083, 0267).

## Outcome
Excel's search is modelled (docs/research/t3409/README.md; statistics.md "GAMMA.INV and CHISQ.INV at a huge shape";
`huge_shape.rs` `search_converges`): from its Wilson-Hilferty start x0, Newton's method on the odds P/Q (step
(PQ - r Q^2)/f), which from x0 never overshoots and moves about 1/|z| standard deviations a step, so its count is the
fall in ln density from x0 to the quantile, through the mode when x0 lies past it. Excel gives up with `#NUM!` after
207 steps at a relative step of 1E-8 (`SEARCH_STEP_LIMIT` with `SEARCH_STEP_TOLERANCE`, one FITTED pair, evidenced on
1087 cells at shapes 1E5 to 5E10). It runs from shape 1E5 (`HUGE_SHAPE_MIN`, the Temme quantile shape, not a fit) and
explains the band's upper and lower edges, T-3396's closing band and the `#NUM!` below 1E9 with one constant:
`BAND_UPPER_SLOPE`, `BAND_UPPER_EDGE` and `BAND_LOWER_EDGE` are gone (quality/fitted.json 91 -> 89, decision 0277).
Excel's start where its density underflows is its value at every probed shape from 1E5 (197 of 197 cells), but ours
applies that only from 1E9 (`START_SHAPE_MIN`, T-3070's tag): below it the start's fitted terms are 1E-9 to 3.4E-9 off,
which would move Gate A points out of the envelope (review round 1), so A205-A289 keep the true quantile (T-4205).

R4 (decision 0188): hash split stated before scoring (split.py, 5/16 held out): the train-only choice is uniquely 207
(train 702/703, held-out 336/336; the correctly rounded candidate 465/703 and 220/336). Fresh sweeps drawn after the
freeze: t3409-r4-gamma-inv-search-sweep 278/280 types, 267/280 within 15 digits, 271/280 in Gate A's envelope (main's
band cuts: 254, 243, 247); the second stratum t3409-r4-gamma-inv-search-sweep-wide (1E5 to 3E8 and 1.3E10 to 1E12)
120/120 types, 119/120 envelope (main the same). The limit was 208 when the first sweep was drawn and 207 before its
fill was read (the replay's count had been read at the wrong tolerance). Outside the envelope: sweep A177, A191 (the
type, T-4206), sweep A226, A231, A262, A268, A276, A278, A279 and wide A63 (Excel's start; ours and main the true
quantile; T-4205). Gate A (eval accuracy tables): 0 failing.

Eval: probes known failures shrink by 42 (t3396-gamma-inv-band-huge 41, t3350-gamma-inv-band-lower-edge 1), no
regressions. Decision 0267 cells: 20 of 21 have Excel's type (fit-audit-t3070-shape-min A19, A22:A23, A25:A26,
A28:A30; fit-audit-t3070-band-upper A1:A3, A5:A7, A12:A13, A26, A32:A33, A38); A39 moved to T-4206.
