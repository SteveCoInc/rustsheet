---
id: T-4590
title: "GROWTH is #NUM! where the ln fit's intercept is past the double range (Gate A GROWTH s1 A104)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
GROWTH gives Excel's `#NUM!` on the Gate A point where Excel is `#NUM!` though the prediction is a finite number, and
the rule behind it is probed and recorded: today the engine (and the exact math) give 8.1389E-18 there.

## Context
Found by T-4096's Gate A tables (excel-graph, 2026-11-06). `accuracy/GROWTH/s1!Calc!A104` is a two-variable GROWTH
(known_y twelve values near 1E-17, x1 near 7.3E+7, x2 near 77.9407 with a spread of 2E-4, const TRUE), read at
`INDEX(..,1,2)`. Excel is `#NUM!`; the exact least-squares prediction is 8.13890815167933E-18, and the engine gives
8.138904702791959E-18. The ln fit's intercept there is 1289.95, so b = e^1289.95 is past the double range while the
prediction e^(ln b + m1'x1 + m2'x2) is not. That is the one point of the four tables whose intercept is past
ln(max double) = 709.78; two const-FALSE GROWTH points whose slope e^coefficient underflows (A119, slope -710.56;
A127, -1430.22) are numbers in Excel that match the math, so a multiplier that underflows is not an error. The
working hypothesis (UNVERIFIED, one point): Excel forms LOGEST's b = e^intercept and is `#NUM!` when it overflows.
statistics.md "LINEST, LOGEST, TREND and GROWTH: Gate A tables (T-4096)" records the cell.
Code path: `trend_or_growth` in crates/rustsheet-engine/src/functions/statistical/trend.rs.

## Acceptance criteria
- [ ] A probe (excel-probe skill) separates the candidate rules: GROWTH and LOGEST with an intercept just below and
      just above ln(max double), with const TRUE and FALSE, and a slope's e^coefficient past the double range.
      The finding goes into statistics.md.
- [ ] The engine follows the probed rule; `accuracy/GROWTH/s1!Calc!A104` leaves eval/accuracy/known-failures.tsv.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The least-squares solve's digits (T-2006).
