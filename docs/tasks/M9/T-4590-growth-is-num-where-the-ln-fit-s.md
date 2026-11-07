---
id: T-4590
title: "GROWTH is #NUM! where the ln fit's intercept is past the double range (Gate A GROWTH s1 A104)"
milestone: M9
lane: functions-stat
status: done
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
- [x] A probe (excel-probe skill) separates the candidate rules: GROWTH and LOGEST with an intercept just below and
      just above ln(max double), with const TRUE and FALSE, and a slope's e^coefficient past the double range.
      The finding goes into statistics.md.
- [x] The engine follows the probed rule; `accuracy/GROWTH/s1!Calc!A104` leaves eval/accuracy/known-failures.tsv.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The least-squares solve's digits (T-2006).

## Outcome
- **Probe.** probes/t4590-growth-overflow (excel-graph, 2026-11-07), with predictions in docs/research/t4590 committed
  before the fill (dc6d154df).
  - H1 holds on all 23 GROWTH cells: GROWTH is `#NUM!` exactly when LOGEST's b = e^intercept overflows (intercept
    709.7828 or more; 709.7827 is a number).
  - H2 (any factor's overflow) is refuted: a slopes' factor past the double range is a number in GROWTH, though
    LOGEST's own m is `#NUM!`.
  - Not predicted: GROWTH is 0 where the slopes' factor is subnormal (c = 709 to 709.7827), though the prediction is
    about 1.
- **Code.** `growth_prediction` (functions/statistical/trend.rs) applies the two edges: `#NUM!` when e^intercept
  overflows, and 0 when the slopes' factor e^(Σ m_j x_j) is subnormal or 0. Elsewhere it keeps the single
  exponential.
  - The product of the two factors was tried. It moves ironcalc-calc `TREND_GROWTH.xlsx` K2, K3 and K5 from exact to
    one ulp off, so it was not kept, and how Excel combines the factors between the edges is not identified.
  - No constants are involved: the cuts are the double range itself.
- **Docs and test.** New statistics.md entry "GROWTH is #NUM! once LOGEST's b overflows ...", and the T-4096 entry's
  A104 note is resolved. Unit test `growth_edges_of_logest_factors`.
- **Eval delta.**
  - Gate A loses accuracy/GROWTH/s1 A104 (`eval-gates`: 0 failing).
  - eval-check has 0 regressions; no other GROWTH cell changes.
  - The new probe adds 14 known failures, which are least-squares digits (1E-13 to 1E-12 off) owned by the new
    T-4763, after T-2006.
