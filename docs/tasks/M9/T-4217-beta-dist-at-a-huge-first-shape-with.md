---
id: T-4217
title: "BETA.DIST at a huge first shape with b of 40 (bratio's bfrac side): Excel is 1.3E-9 to 4.6E-8 relative off the math, ours the math (26 t4080 cells, split from deferred T-3209)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
`BETA.DIST(x,a,40,TRUE)` at a first shape of 2E9 to 1E10 with x near 1 (I about 1E-6 and 0.05) gives Excel's value to
within half a unit of the 15th significant digit, from a model of the arithmetic Excel runs there (TOMS 708 bratio's
branch for b0 >= 40, a0 > b0, b0 <= 100: `bfrac`, with `brcomp`'s prefactor), not from a correction fitted to these
cells. Excel is the inaccurate one here, so this is decision 0114's `visible` stage.

## Context
- Gap triage 2026-11-04 (`make eval` on origin/main 0516b489). `make unowned-off ALL=1` lists these cells as owned only
  by T-3209, which is `defer: phase-1b`. T-3209's context says a point outside Gate A's envelope "gets its own task
  without the key". These cells are visibly off under decision 0188 R1(c): Excel is more than 1E-9 relative off the
  50-digit math, and no R3 mark applies, because the error is far below R3's 1E-4. That puts them in M9's probe exit
  row. T-3209 keeps the bgrat-side (b < 40) digits; this task takes the b = 40 cells.
- Evidence: probes-numeric/t4080-beta-num-band-edge (T-4080, excel-graph 2026-11-04). x = 1 - q/a is chosen so that I
  is about 1E-6 (column A) and 0.05 (column B). The relative errors below are against 60-digit mpmath
  `betainc(a, 40, 0, x)` at the double x, computed during this triage and not committed:
  - A21 (a 2E9): Excel 9.999984224817014E-07 (-2.8E-9), ours 9.999984252839257E-07 (+2.8E-15)
  - B105 (a 3E9): Excel 0.05000000149710333 (-4.3E-9), ours 0.05000000171154521 (+1.9E-15)
  - B294 (a 1E10): Excel 0.049999949775626405 (-4.6E-8), ours 0.049999952069376706 (+9.3E-16)
  - At every b = 40 cell in columns A and B, ours is within 1E-14 of the math. Excel is 4.5E-10 (A63) to 4.6E-8
    (B294) off it, and the sign varies from cell to cell. The error grows with a: about 3E-9 at 2E9 and up to 4.6E-8
    at 1E10.
  - At the same a, columns C (I near 0.5, lambda < 0, so bratio swaps the shapes) and E (the density) are within
    15 digits already. So the loss is specific to the unswapped b0 = 40 point.
- Which branch Excel takes at these points: lambda = (a + b)y - b > 0 with a0 = a > b0 = 40 <= 100. TOMS 708 bratio
  goes to `bfrac` there (the `b0 < 40` test is false). docs/excel-semantics/statistics.md "Incomplete beta at a huge
  first shape" ("The edge (T-4080)") already records that b = 40 leaves L140 and is a number at all 14 a.
- Ours: `large_shape_lower` (`large_shape_beta.rs`) stops below `MAX_SECOND_SHAPE` = 40. At b = 40 the point goes
  through BETA.DIST's kernel (`bratio::beta_dist_tails`; `Branch::Bfrac` in `bratio/mod.rs` `dispatch`, the T-2508
  port), or through the accurate fallback where the port declines. Find out which one runs first. Then compare the
  port's `bfrac` and `brcomp` against Excel at these points, step by step. Excel's errors are 1E-9 to 5E-8 relative,
  far above the 4-ulp reproducibility floor, so a right structural model shows up clearly in bit-exact counts.
  Candidates: bfrac's stopping test (15·eps relative on a fraction that converges slowly at a huge a); brcomp's
  e = lambda/b, v = e - ln(y/y0) form against `rlog1`; h = b/a, x0 = 1/(1 + h), y0 = h/(1 + h) rounded as written;
  x87 extended steps.
- A related set: the F digits at df1 80 and 100 with a huge df2, probes-numeric/t4030-f-dist-rt-df1-boundary A39,
  A40, A61, A62. T-4044 owns these cells for the type rule. Their digits depend on how the F caller rounds u = df2/(df2
  + df1·x): at a = 5E9 one ulp of x moves I by about 5E-7. So they are not this task's acceptance cells. Report them
  if the model moves them.
- The playbook: docs/research/excel-kernel-playbook.md. Fit lab and the T-2464 bratio models:
  docs/research/t2464/r2-bratio/.

Cells this task owns (split from T-3209, which keeps the b < 40 cells):
- probes-numeric/t4080-beta-num-band-edge.case.json (28): Calc!A21, B21, A42, B42, A63, B63, A84, B84, A105, B105, A126, B126, A147, B147, A168, B168, A189, B189, A210, B210, A231, B231, A252, B252, A273, B273, A294, B294

Needs an oracle sweep only if the cached cells do not settle the model. The 28 cells above plus columns C and E give
14 values of a and two tails. If no structural candidate reproduces at least most of them, stop and file a
research-lane sweep before fitting anything. The sweep: b from 40 to 100 (bfrac's whole a0 > b0 range), a from 1E6 to
1E10, I from 1E-12 to 0.5 on the unswapped side, plus F.DIST.RT at df1 >= 80 and df2 >= 1E8, which T-4017 never
probed (it stopped at df1 79). Then block this task on that sweep.

## Acceptance criteria
- [ ] The 28 cells above are within half a unit of Excel's 15th significant digit. The 26 visibly-off lines of
      probes-numeric/t4080-beta-num-band-edge disappear from `make unowned-off ALL=1`. No other t4080, t4017, t4030,
      t2528 or t1530 cell moves away from Excel (`rustsheet-eval show --all`, cell by cell before and after).
- [ ] No new known-failure lines in any eval/baseline/<suite>.txt (`make eval-baseline` reports 0 regressions).
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7), with each result stated in the Outcome:
  - [ ] Hold-out: train and hold-out counts side by side (bit-exact, within 4 ulp, within 15 digits) for the chosen
        candidate and for the correctly rounded value. Use the fit lab's split, or a manual split of about 30% by cell
        hash, fixed before any scoring. The chosen candidate beats the correctly rounded value on hold-out.
  - [ ] Fresh sweep: after the model is frozen, a new log-uniform probes-numeric case over a 1E6..1E10 and
        b 40..100 is oracle-filled, and its result is recorded here.
  - [ ] Gate A envelope: the fresh sweep's count inside the envelope (R2), with every point outside it listed.
  - [ ] `FITTED(T-4217, n_cells, range)` tag and range on every fitted constant or cut. Outside the range the code
        falls back to accurate math. A purely structural model (a TOMS 708 operation order or rounding) needs no
        tag; say which kind it is.
  - [ ] Reproducibility floor: no step targets differences below 4 ulp (web against desktop) or bits that vary
        between fills.
  - [ ] No keep-matched branch: no band or special case whose domain is tuned to the cells that already match.
- [ ] docs/excel-semantics/statistics.md "Incomplete beta at a huge first shape" gains a b >= 40 rule with these cells
      as evidence (or records why no model fits).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The bgrat-side digits at b < 40 (T-3209, deferred) and the `#NUM!` band (T-4044).
- The F callers' argument rounding at a huge df2 (T-4018, T-4044).

## Visible-parity stage
Target (decision 0114, `visible`): every cell this task owns is within half a unit of the 15th significant digit of
Excel's value. Decision 0056's time-box applies: two attempts that do not move the cells block the task as
time-boxed, with the attempts recorded. Bit-level criteria still unmet after that move to a follow-up filed with
`defer: phase-1b`.
