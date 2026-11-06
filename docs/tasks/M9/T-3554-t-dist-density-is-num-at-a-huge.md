---
id: T-3554
title: "T.DIST density is #NUM! at a huge x in Excel where the engine returns 0"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
`T.DIST(x,df,FALSE)` returns Excel's `#NUM!` where Excel gives it at a huge x, instead of 0.

## Context
- Found by T-3358's Gate A table (excel-graph): `T.DIST(1.01778553276442E+236,2,FALSE)`,
  `T.DIST(1.06518240138577E+269,9,FALSE)`, `T.DIST(3.05696424373289E+292,824733515,FALSE)` and
  `T.DIST(1.43991809793873E+307,20225998,FALSE)` are `#NUM!` in Excel. The true density underflows to 0 there, and the
  engine returns 0. These are Gate A known failures accuracy/T.DIST/s1 A85, A95, A105 and A110.
- T-3425's batch adds three: accuracy/T.DIST/e-T-3425 A116 (`T.DIST(4.20674629135061E+220,2.42593594707487E+249,FALSE)`)
  and, at a negative x, A103 (`T.DIST(-4.62236657470614E+224,11,FALSE)`) and A90 (`T.DIST(-1.3895161258545E+237,16,FALSE)`).
  A negative x is in scope: its cut lies between |x| 7.6E+87 and 4.6E+224.
- statistics.md, "T.DIST's density is #NUM! at a huge x, and T.INV.2T and TINV at p = 2.2E-308 (T-3358)": the cut
  point is not yet probed. T-2752 probes TDIST's x cut, which may share the rule.
- Code: `fn_t_dist` and the density in crates/rustsheet-engine/src/functions/statistical/ (t_dist.rs).

## Acceptance criteria
- [x] A probe case (M9, excel-graph) brackets the x where the density turns `#NUM!` at several df, for positive and
      negative x; the rule is in the statistics.md entry.
- [x] The engine matches it; the seven known-failure lines of eval/accuracy/known-failures.tsv are removed as stale.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cumulative T.DIST (unaffected).

## Probe predictions (committed before the fill)
Case probes/t3554-tdist-density-huge-x (excel-graph). Every hypothesis is even in x, so each row predicts the same for
A (x > 0) and B (-x). "N" is `#NUM!`, "n" a number (0 included). Hypotheses:
- (a0) `#NUM!` exactly when the density underflows to 0 in double; (aMIN) when it falls below DBL_MIN 2.2E-308;
  (c) a ln-space density whose exp underflows: same cells as (a0).
- (b1) when `x*x` overflows, |x| >= 2^512 (about 1.3408E+154), whatever the df;
- (b2) when x²/df overflows (|x| > sqrt(df * DBL_MAX)); (b3) when `x*x + df` overflows.

Prior evidence already refutes (a0), (aMIN) and (c) (s1 A97, `T.DIST(9.93E+137,24,FALSE)` is 0 with a density far
below 1E-324) and (b2) (e-T-3425 A116, x 4.2E+220 at df 2.4E+249, is `#NUM!`); the probe tests them again.

| Row | x | df | a0/c | aMIN | b1 | b2 | b3 |
|---|---|---|---|---|---|---|---|
| 1 | 2^512 pred. | 1 | n | N | n | n | n |
| 2 | 2^512 | 1 | n | N | N | N | N |
| 3 | 2^512 pred. | 2 | N | N | n | n | n |
| 4 | 2^512 | 2 | N | N | N | n | N |
| 5 | 2^512 pred. | 24 | N | N | n | n | n |
| 6 | 2^512 | 24 | N | N | N | n | N |
| 7 | 4 * 2^512 | 24 | N | N | N | n | N |
| 8 | 8 * 2^512 | 24 | N | N | N | N | N |
| 9 | 2^512 pred. | 1E+10 | N | N | n | n | n |
| 10 | 2^512 | 1E+10 | N | N | N | n | N |
| 11 | 2^512 pred. | 1E+300 | N | N | n | n | N |
| 12 | 2^512 | 1E+300 | N | N | N | n | N |
| 13 | 1E+200 | 1E+300 | N | N | N | n | N |
| 14 | 1.339E+154 | 1E+307 | N | N | n | n | N |
| 15 | 1 | 1E+307 | n | n | n | n | n |
| 16 | 1E+120 | 2 | N | N | n | n | n |
| 17 | 1E+100 | 2 | n | n | n | n | n |
| 18 | 1E+154 | 1 | n | N | n | n | n |
| 19 | 2E+153 | 1 | n | n | n | n | n |
| 20 | 1E+160 | 1 | n (c) | N | N | N | N |
| 21 | 1E+300 | 1 | N | N | N | N | N |
| 22 | 1E+120 | 9 | N | N | n | n | n |
| 23 | 2^512 pred. | 1E+15 | N | N | n | n | n |
| 24 | 2^512 | 1E+15 | N | N | N | n | N |

Row 20 separates (c) (a ln-space density of 3.2E-321 is a number) from a direct (a0) (1/(pi*(1+inf)) is 0, so N).

## Probe result 1 and predictions for probe 2 (committed before its fill)
probes/t3554-tdist-density-huge-x: every `#NUM!` cell is where `x*x` overflows ((b1): 2^512 and above at df 1, 2, 24,
1E+10, 1E+15 and 1E+300, both signs), and every cell with a finite `x*x` is a number (0 for an underflowed density),
except A1, B1, A18 and B18: at df 1 a finite `x*x` with |x| 1E+154 or 2^512's predecessor is `#DIV/0!`, while 2E+153
is the number 7.96E-308. (a0), (aMIN), (c), (b2) and (b3) each miss cells (rows 3, 16, 22; 19; 20; 4, 13; 11, 14).

Probe 2, probes/t3554-tdist-density-df1-div0, separates two rules for that df-1 `#DIV/0!`:
- (d1) the denominator fl(pi fl(1 + fl(x x))) overflows: `#DIV/0!` from 7.6E+153 (A9, A10), a number up to 7.55E+153 (A1:A8);
- (d2) the density falls below DBL_MIN: `#DIV/0!` from 3.8E+153 (A3:A10), a number at 3.7E+153 and 3.75E+153 (A1, A2).
Both predict the same for B (negated x) and for C1, C2 at df 1.5 (as A5 and A9); C3, C4 follow A-column 1E+154 and A5.

## Probe result 2
probes/t3554-tdist-density-df1-div0 confirms (d1) and refutes (d2): `#DIV/0!` at 7.6E+153 and 8E+153 (A9, A10, B9, B10,
and C2 at df 1.5), a number at 3.7E+153 to 7.55E+153 (2.3251269991511373E-308 and 2.2635369684180667E-308 at 3.7E+153
and 3.75E+153, 0 from 3.8E+153 on, where the quotient is subnormal and flushed).

## Outcome
- Rule (structural, no fit; both probes written with their predictions before the fill): `T.DIST(x,df,FALSE)` is
  `#NUM!` exactly when fl(x x) overflows (|x| >= 2^512), at every df and either sign; at a df truncating to 1 it is
  `#DIV/0!` where fl(x x) is finite but fl(pi fl(1 + fl(x x))) overflows (|x| about 7.565E+153 up to 2^512), and a
  subnormal df-1 quotient is 0. Every cell of both probes (58 + 24) matches the rule.
- Engine: `t_dist_of` applies `t_x_overflows` to the density too and returns `#DIV/0!` through the new
  `loader::excel_t_density_df_one_overflows`; `excel_t_density` flushes the df-1 quotient (`flush_subnormal`).
- Gate A: the 18 T-3554 known failures (T.DIST s1 4, r4-T-2585 11, e-T-3425 3) pass and their lines are removed;
  `rustsheet-eval accuracy` 0 failing. The T.DIST definition's density brackets become the exact cut
  (`DENSITY_X_SQUARED_CUT`) plus the df-1 `#DIV/0!` domain; `rustsheet-accuracy verify T.DIST` reports 0 differences
  (no committed table point changes), and only the two new probes get math sidecars.
- Eval: +2 probe cases passing (82 cells), no other baseline change.
- statistics.md "T.DIST's density is #NUM! at a huge x" records both cuts and the refuted hypotheses.
