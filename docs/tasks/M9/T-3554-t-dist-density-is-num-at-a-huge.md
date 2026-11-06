---
id: T-3554
title: "T.DIST density is #NUM! at a huge x in Excel where the engine returns 0"
milestone: M9
lane: functions-stat
status: in-progress
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
- [ ] A probe case (M9, excel-graph) brackets the x where the density turns `#NUM!` at several df, for positive and
      negative x; the rule is in the statistics.md entry.
- [ ] The engine matches it; the seven known-failure lines of eval/accuracy/known-failures.tsv are removed as stale.
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
