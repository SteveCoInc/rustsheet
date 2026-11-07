---
id: T-2788
title: "YIELD after the fitted start: the last ulps at the root (3 regressed probe cells, 9 lab cells)"
milestone: M9
lane: research
status: done
priority: P3
size: M
tier: opus
depends_on: [T-1592]
---

## Goal
YIELD (more than one coupon left) matches Excel bit for bit on the cells T-1592's fitted Newton leaves a few ulps off
at the root, including the three cells T-1592 regressed by 1 ulp under decision 0083's allowance.

## Context
Split from T-2777 on 2026-10-26 (T-1592 review): the ulp part of the follow-up, with no campaign key (decision 0083).
T-1592 fitted the start (100R + (100-P)/T)/((100+P)/2 - (100-P)/(4T)) and the stop (a step below 1e-10 is not
taken). The engine follows it (`excel_yield_newton`, `yield_start` in
crates/rustsheet-engine/src/functions/financial/financial_bonds/yield_solver.rs). financial.md "YIELD stops short of
PRICE's root", T-1592 fourth-round paragraph; lab fit `uv run rustsheet-fitlab yield-caret`.
- Ulp regressions T-1592 took under decision 0083's allowance, at the root, 1 ulp low in the engine and the lab
  alike (no PRICE summation order tried moves them): probes/yield-start-one-input S!A5
  (Excel 0.10352912862429842, ours 0.1035291286242984), probes/yield-start-rate-term S!B34 (0.11099781701293797,
  ours ...795) and S!B35 (0.153546637344011, ours 0.15354663734401097).
- Lab cells 7 to 33 ulp off: solver-iteration-traces Y!E2; yield-solver-sweep S!C7, S!C11;
  yield-start-one-input S!B11, B17, B31, B54, B62, B69.
- gen-m9 cells within 4 ulp after T-1592: A229 (3aa168259335be54), A333 (5ded7594e597e31e), A376 (6376e27be7a05871),
  A231 (9a1fa5366bc50918), A122 and A269 (b73c6cc96e093f03), A23 (bc04c1f478c9896c), A223 (c0681a8b150ce301).
- Gate A (T-4113, eval/accuracy/known-failures.tsv): accuracy/YIELD/s1 A230, A252 and A261 (extreme stratum, bases 3,
  2 and 0) are engine values outside Excel's 15 digits, both short of the root (A252 engine 5.680114950882204E-6, Excel
  5.680114950874831E-6, math 5.680145374838903E-6); the lines go when these pass. They are 130 to 8,700 ulp off
  Excel, not last ulps, and their prices and redemptions are far from 100 (redemptions 0.014 to 0.048), the region of
  T-4523, whose start finding may be what moves them: re-check them there before scoring candidates here.
- Handed over by T-2777 (2026-10-26) once its term fix left them ulp-level: gen-m9/81bcdc0bec0c8e6b A189
  (basis 2, Excel -0.0008265806588318691, ours ...682, 8 ulp), gen-m9/a91bad4e84f49739 A102 (-0.0008665128023479636,
  ours ...645) and gen-m9/e168be7f6d3f84c2 A194 (0.05369271319672304, ours ...308), which miss Excel's 15th
  significant digit by a rounding boundary; and probes/yield-start-term-basis A9, C11, C20, A26 and A38, at the
  root and 4e-15 to 1e-14 off, plus B50 and A52 (sub-year, 10 to 12 ulp in the engine, 1-2 ulp in the lab) and the
  probe's 73 cells 1-2 ulp off.
- Gap triage 2026-10-27 (`make eval` on 7dba3bfe): one more cell 8 ulp off at the root. It is just past half a
  unit of the 15th digit and no task named it: probes/yield-start-one-input S!B67
  `YIELD(DATE(2096,3,1),DATE(2097,6,30),0.05,98,100.5,1,0)`, Excel 0.06966784545910434, ours 0.06966784545910445.
- Handed over by T-2741 (2026-10-28): the bit-level residue of T-2741's owned YIELD cells, all within 15 digits after
  T-1592. That is every remaining known-failure line of probes/yield-start-rate-term, yield-start-sweep,
  yield-solver-sweep, solver-iteration-traces, date-system-1900-financial-readers and its -shifted twin,
  fin-coupon-walk-bounds-c, fin-remaining-later-argument-ranges, fin-bond-negative-basis, coup-calendar-edges,
  fin-argument-modes and fin-yield-odd-lift, and of ironcalc-calc FINANCIAL/PRICE_YIELD.xlsx's I column, whose
  outermost call is YIELD. The three cells T-2974 owns are excluded. For example, date-system-1900-financial-readers
  S!E8, its -shifted twin S!E8 and yield-start-rate-term S!B20 are each 1 ulp off
  (`fn_yield_matches_excel_off_the_fit`).

- Added by T-2974 (2026-10-29): probes/yield-price-at-iterates P!A5 to P!A12, which repeat yield-start-one-input B54,
  B62, B69, B11, A54 and A62, yield-solver-sweep C11 and solver-iteration-traces Y!E2. The same probe shows Excel's PRICE
  at our iterates equals the lab's `^` PRICE to the bit (financial.md "YIELD stops short of PRICE's root", T-2974).

## Acceptance criteria
- [x] The three regressed probe cells pass bit for bit (their lines leave eval/baseline/probes.txt), or the task
      records the PRICE kernels and stop rules tried with their lab bit-exact counts (decision 0056).
- [x] The lab's "Newton, fitted start" family raises its bit-exact count above 240/370, or the task is blocked with
      the candidates tried.
- [x] The 40 probe cells added by gap triage 2026-10-30 leave eval/baseline/probes.txt, or the Outcome lists each with its ulp distance and the candidate it was scored with.
- [x] `make check` green; reviewers approve.

## Out of scope
- The visibly-off basis 1 and 2 cells (T-2777); YIELD's `#NUM!` cells and its 100-step cap (T-2391).

## Cells added by gap triage (2026-10-29)
`make eval` on main a2ecf005b (excel-graph), `python3 scripts/exit_owners.py --all-off`: visibly off, no open task named them, same cause as this task's cells.
- probes/yield-start-term-basis S!A18 `YIELD(DATE(2013,5,10),DATE(2024,2,29),0.05,98,100.5,2,2)`: Excel 0.05279263039071429, ours 0.05279263039071384 (8.5E-15 relative, the size of this probe's A9, C11, C20, A26 and A38 above)

## Cells added by gap triage (2026-10-30)
`make eval` on origin/main 6a26b992a (excel-graph), `scripts/exit_owners.py`'s `owned` rule over every failing cell and the task files of origin/main and every pushed branch: 40 YIELD cells 1 to 4 ulp off, none visibly off, that no open task names. T-2391's Outcome left the yield-newton-domain ones to this task ("last-digit misses"); the others sit on the same fitted start and root. Re-run the list after T-3215 before fitting. Examples, Excel vs ours: yield-newton-domain P!B13 `YIELD(36665,44491,30,106,100,4,0)` 14.303206872934162 vs 14.303206872934156; yield-start-term-basis S!A21 `YIELD(DATE(2015,6,10),DATE(2016,5,20),0.05,98,100.5,2,2)` 0.07759929343433138 vs 0.0775992934343314; date-system-1904-financial S!A14 0.05932842042333485 vs 0.05932842042333486.
- probes/date-system-1904-financial.case.json (1): S!A14
- probes/yield-newton-domain.case.json (5): P!B6, B10, B11, B12, B13
- probes/yield-start-one-input.case.json (8): S!A18, A31, A68, A76, B5, B18, B68, B76
- probes/yield-start-term-basis.case.json (26): S!A3, A5, A8, A15, A21, A29, A47, A48, A53, B3, B5, B8, B15, B18, B21, B29, B47, B48, B53, C2, C8, C21, C30, C44, C46, C48

## Cells added by T-2747 (2026-10-31)
probes/yield-deep-discount-sweep (excel-graph, 2026-10-31): `YIELD(39681,45662,R,P,102,4)` over coupon rate R 1 to 100
and price P 1 to 20. Every cell matches Excel in type (T-2747's rule); 37 value cells are 1 to 5 ulp off on the same
fitted start and root as this task's cells. Example: S!B1 `YIELD(39681,45662,1,2,102,4)` 15.009570961865686 vs ours
15.009570961865677 (5 ulp).
probes/yield-deep-discount-cut adds 13 more at coupon rates 20.5 to 23, two of them 100th iterates far off the root
(S!B3 `YIELD(39681,45662,21.5,3.14159,102,4)` -2.6837489023653673 vs ours -2.6837489023654952, 5e-14 relative; S!C4
6e-15 relative), where the last-digit gap grows over 100 steps.
- probes/yield-deep-discount-cut.case.json (13): S!A2, B1, B2, B3, C1, C2, C3, C4, D1, D2, D3, D5, D6
- probes/yield-deep-discount-sweep.case.json (37): S!A1, A2, A5, A9, B1, B2, B4, B6, C1, C2, C3, C4, C5, C6, C8, D5,
  D6, D7, D9, E2, E3, E4, E6, E7, E8, E9, F1, F2, F4, F9, G5, G6, G8, H3, H7, H8, H9

## Outcome
Done 2026-11-07. Most of this task's cells had already been fixed by T-3215's start (`100·T - q·T/2 - q/4`); on
origin/main the YIELD known failures left in probes were 14 cells. Of those, three were 8 to 11 ulps off in the engine
and bit-exact in the fit lab: the engine's PRICE discounted the redemption at `(DSC/E + n) - 1`, the lab at
`(n - 1) + DSC/E`, and the two round differently (financial.md "YIELD stops short of PRICE's root", T-2788 paragraph).
The `^` factors were identical (928 compared) and the accrued order made no difference.
- Change: `CouponSchedule::price` takes the redemption's exponent as `n - 1.0 + dsc_over_e` (structural model, decision
  0188 R4 class (c): an operation order; no `FITTED` tag).
- Eval delta: probes 6744 -> 6740 known failures (fixed yield-start-one-input S!B67, yield-start-term-basis S!A52 and
  S!B50, coup-calendar-edges S!J12), 0 regressions in every suite.
- Hold-out (R4.1, the fit lab's fixed split, 127 of 370 `yield-caret` cells by hash): `(n - 1) + DSC/E` train 242/243
  bit-exact, test 127/127 (all within 4 ulps and 15 digits); `(DSC/E + n) - 1` train 241/243, test 127/127. YIELD
  has no correctly rounded candidate, so the lab's verdict column refuses every row on that ground alone.
- Fresh sweep (R4.2): accuracy/YIELD/r4-T-2788, 300 points generated and filled (excel-graph) after the change was
  fixed. 288 bit-exact against 284 under the old order.
- Gate A envelope: 296 of 300 inside (old order 295; A237 comes inside). Outside, the same under both orders and now
  listed in eval/accuracy/known-failures.tsv: A194 and A239 are `#NUM!` in Excel (T-4523); A252 and A267 stop short of
  the root outside Excel's 15 digits (T-4758). `rustsheet-eval accuracy`: 0 failing.
- Criterion 1: yield-start-one-input S!A5 and yield-start-rate-term S!B34 and S!B35 pass (not in the baseline).
- Criterion 2: the "Newton, fitted start" family is 369/370 bit-exact (T-3215's start; unchanged by this task).
- Criterion 3: of the 40 cells of 2026-10-30, only yield-newton-domain P!B13 is left: 1 ulp low
  (Excel 14.303206872934162, ours 14.30320687293416), scored with the `100T - qT/2` start and the short stop, which the
  lab also leaves 1 ulp low.
- Left to T-4758 (follow-up): yield-deep-discount-cut S!A2, B2, C2, D5, D6 (1 to 2 ulps), B3 and C4 (100th iterates);
  yield-deep-discount-sweep S!D7, D9, F3, H9 (1 to 2 ulps; the lab misses each by 1 to 2 ulps too, D7 and F3 where the
  engine's `^` and the lab's differ at exponents 28 to 65); P!B13; Gate A YIELD/s1 A230, A252, A261 (moved from
  this task) and r4-T-2788 A252, A267.
