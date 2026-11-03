---
id: T-3187
title: "Student t tails at df 1 and 2 for large t: Excel is within an ulp of the exact tail and ours 5 to 24 ulp off (6 visibly-off gen-m9 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
campaign: accuracy
depends_on: []
---

## Goal
T.DIST.RT, T.DIST.2T, TDIST and T.DIST's cumulative tail at a truncated df of 1, and at df 2 below the closed-form cut
(y = 2/(2 + t^2) under 0.1, t above about 4.24), are within an ulp of the exact tail, as Excel's are. That brings the
six visibly-off gen-m9 cells below to visible parity.

## Context
Gap triage 2026-10-30 (`make eval` on origin/main, opus refill). T-0969 owns these cells, but it is blocked because
Excel is noisy and ours is near exact. For these cells it is the other way round: Excel is exact and ours is not. The
reference is mpmath `betainc(n/2, 1/2, 0, n/(n+t^2))/2` at 60-80 digits on the exact t (script logic: the tail L =
P(T > |t|); 2T = 2L; TDIST(t,n,k) = k L).

Visibly-off cells (eval/baseline/gen-m9.txt F lines; Excel, ours, then ulps from the exact tail):
- gen-m9/46ac3ef5e61583ca Calc!A236 `T.DIST.2T(1E15,1)`: Excel 6.366197723675813e-16 (0), ours 6.366197723675822e-16 (+9)
- gen-m9/11eb3ab4f4f8104c Calc!A174 `T.DIST.2T(EVEN(SUM((-2.5),Data!C5:C8)),2)`, t = 1E15: Excel 9.999999999999999e-31
  (-1), ours 1.0000000000000043e-30 (+24)
- gen-m9/5395b8a427764a3f Calc!A125 `TDIST(AVERAGE(Data!E2:E6,(Data!E5+Data!A2)),2,1)`, t = 30864222.95575: Excel
  5.248791352441565e-16 (-1), ours 5.248791352441582e-16 (+16)
- gen-m9/a5342a7e24fd803e Calc!A230 `TDIST(EOMONTH(24332,-7),2,Data!A6)`, t = 24138, one tail: Excel
  8.581583593277439e-10 (+1), ours 8.581583593277429e-10 (-9)
- gen-m9/bbab6562e88b70e8 Calc!A181 `T.DIST.2T((123456789.123-(42*0.2)),2.5)`, df 2: Excel 6.561000999157746e-17 (0),
  ours 6.56100099915777e-17 (+19)
- gen-m9/91e167cd8d7eb652 Calc!A292 `T.DIST.2T(GEOMEAN(Data!A8:A8,Data!B5:B8,(100+Data!D$2)),1)`, t about 1.14E6:
  Excel 5.578729012390279e-07, ours 5.578729012390272e-07 (recompute t with `make eval-show` before scoring)

The same pattern, not visibly off. Every failing df-1 or df-2 cell with literal arguments in the report has Excel within
1 ulp of exact (49 cells):
- gen-m9/334f3dc9ca937f10 Calc!A308 `T.DIST.2T(1E+5,1)`: Excel +1, ours +5
- gen-m9/873f30d2bf76f8db Calc!A37 `TDIST(100,1,1)`: Excel 0, ours +5
- gen-m9/eedd629f3d69fdda Calc!A244 `T.DIST.2T(100,1)`: Excel 0, ours +5
- gen-m9/eedd629f3d69fdda Calc!A210 `T.DIST.2T(38.5,2)`: Excel 0, ours +2
- gen-m9/bcb1105c61a2649d Calc!A209 `T.DIST.2T(40,2)`: Excel 0, ours -3
- gen-m9/d8f44d556674c2fd Calc!A58 `TDIST(40,2,1)`: Excel 0, ours -3
- gen-m9/bbab6562e88b70e8 Calc!A266 `T.DIST.2T(37.5,2)`: Excel 0, ours +1
- probes-numeric m9-t-dist-rt A10 `T.DIST.RT(100,1)` (Excel 0, ours +5), A12 (1E6,1) (0, -2), A25 (100,2) (+1, -3),
  A26 (1000,2) (0, +2), A27 (1E6,2) (+1, -4, visibly off); m9-t-dist-2t A10, A12, A22, A23, A24 (A22 and A24 visibly off).
- At df 1 the failing cells with y >= 0.1 (t <= 3) also have Excel within 1 ulp and ours above it (9 cells).

Semantics already recorded: docs/excel-semantics/statistics.md "T.DIST.RT at df 2 is BETA.DIST's closed form down to
y = 0.1". Below the cut it says "Excel is accurate and no simple form fits". On t0969-t-df2-cut's 352 cells below the
cut plus six m9-t-dist-rt cells, -expm1(log1p(-y)/2)/2 at the rounded y is bit-exact on 208 of 358 and
y/(1 + sqrt(1 - y))/2 on 175. Our current path (Cephes power series) gets 111. Those counts are bit-exact scores; this
task's target is accuracy (within an ulp). Try those forms, and the exact 1/(sqrt(2+t^2)(sqrt(2+t^2)+t)) on t, against
mpmath first. For df 1 the tail is atan(1/t)/pi, or the arcsine form of I_y(1/2,1/2) at the rounded y. Score each
against the oracle cells and against mpmath.

Code: crates/rustsheet-engine/src/functions/statistical/student_t.rs. In `unflushed_upper_tail` the df-2 cells below
the cut fall through `closed_form_tail` (None below y = 0.1) to `tail_at(rounded_y, n)` beyond t = 1, or to
`tail_from`. df 1 takes the same paths.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-30): the six gen-m9 cells above are visibly off, so the task is in the campaign. Its
target: each is within half a unit of the 15th significant digit of Excel's value, and so are probes-numeric
m9-t-dist-rt A27 and m9-t-dist-2t A22 and A24. Bit-level criteria still unmet then move to a follow-up task without
the key and with `defer: phase-1b`.

## Stage (decision 0114)
`campaign: accuracy`: the mpmath reference above shows Excel is within an ulp of the exact tail on every failing df-1
and df-2 cell, and the engine is not. Aim for a tail that is correctly rounded or within an ulp, scored against mpmath
and the oracle. Do not fit Excel's last bits.

## Acceptance criteria
- [x] The six gen-m9 cells above leave eval/baseline/gen-m9.txt, or are within half a unit of Excel's 15th significant
      digit, with no new baseline line in any suite.
- [ ] probes-numeric m9-t-dist-rt and m9-t-dist-2t: no df-1 or df-2 cell more than 1 ulp from Excel. The t0969-t-df2-cut
      bit-exact count does not drop below 560 of 801 (this triage's eval).
- [x] The df-2 cells at y >= 0.1 keep the closed form (statistics.md entry above; `df_two_tail_is_the_closed_form_down_to_y_one_tenth`
      still passes).
- [x] statistics.md records the df-1 and below-cut df-2 form that ships, citing the cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- df 3 and above (T-2509, T-2126, T-0969); the density (T-2585); T.TEST's rounded-y path at df 1 and 2 beyond what the
  shared tail gives.

## Outcome
- Past the closed-form cut (fl(1 - y) > 0.9 at the worksheet's rounded y, the cut df 2 already had, which is t above
  about 3 at df 1 and 4.24 at df 2), df 1 and df 2 take the exact tail at t, rounded once: atan(1/t)/pi at df 1 and
  1/(s (s + t)) with s = sqrt(2 + t^2) at df 2, in double-double (`exact_low_df_tail` in student_t.rs; the cut is
  incomplete_beta.rs `A1_CLOSED_FORM_MAX_Y`, now `pub(crate)`). Inside the cut nothing changed: df 2 keeps its closed
  form (`df_two_tail_is_the_closed_form_down_to_y_one_tenth` passes) and df 1 the tail at the rounded y, which gives
  Excel's bits at `T.DIST.RT(3,1)` (1.8 ulp from exact) and `TDIST(1.005,1,2)`. An exact form there regressed those.
- Scored against mpmath (50 digits) and the oracle: our new tails are within 0.3 ulp of the exact value on the
  probe cells checked. Excel is within about 1 ulp of exact past the cut. This is not a fit: no constant or form was
  chosen by scoring against Excel's bits, and the cut is the existing one.
- The six gen-m9 cells: A236, A181 and A292 are bit-exact; A174, A125 and A230 are 1 ulp off, where Excel is itself 1
  ulp from exact. All six are within half a unit of Excel's 15th digit, so visibly fixed.
- probes-numeric (not gated): m9-t-dist-rt 97 -> 99 of 150 bit-exact, m9-t-dist-2t 73 -> 75 of 123, t0969-t-df2-cut
  560 -> 602 of 801. Every df-1 and df-2 cell there is within 1 ulp of Excel except `T.DIST.RT(2,1)` and
  `T.DIST.2T(2,1)` (A5 of each, 2 ulp, inside the cut, unchanged by this task), moved to T-3877 with `defer: phase-1b`
  (decision 0083: bit-level criteria not met at visible parity).
- Eval delta: gen-m9 fixed 14, regressions 1; ironcalc-calc regressions 2; no other suite moves; visible parity
  unchanged or better. The three regressions are numeric, 1 ulp each and not visibly off, so they fall under decision
  0083's standing ulp-regression allowance (`update-baseline --allow-regressions`). The PR fixes 6 visibly-off cells
  and makes none visibly off. Regressed cells (all `T.DIST.RT(t,1)`/`T.DIST.2T(t,1)`, where Excel is 0.7 ulp from
  exact): ironcalc-calc/STATISTICAL/T_DIST.xlsx `T.DIST T.INV T.DIST.RT!F2` and `T.DIST.2T T.INV.2T!C2` (t = 60),
  gen-m9/bbab6562e88b70e8 Calc!A113 (t = 7.1). Follow-up: T-3877.
- Test: `low_df_tails_past_the_cut_are_exact_at_t`. statistics.md "T.DIST.RT at df 2 is BETA.DIST's closed form down
  to y = 0.1" records the form that ships and the cells.
- Also filed T-3878 (follow-up of T-3102's reviews: keep A1-mode TAB/NBSP probe cases).
