---
id: T-4196
title: "Owner ruling: BESSELI at a large x and order 2 or more, and BESSELJ near a root, past R3's 1e-4 (T-4083 R3 candidates)"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Record the owner's ruling on whether Excel's BESSELI at a large |x| with an order of 2 or more, and BESSELJ next to a root,
are a decision 0188 R3 (`MATH-OVER-EXCEL`) case. Until the owner rules, the engine keeps matching Excel.


## Context
- T-4083's Gate A tables (eval/accuracy/tables/BESSELI/s1, BESSELJ/s1; excel-graph, 2026-11-04) found 9 points where
  Excel is more than G = 1e-4 relative off the 50-digit math (engineering.md "BESSELI at a large x and an order of 2 or
  more is far from the math (T-4083)"):
  - BESSELI, 8 points, all at |x| >= 250 and 2 <= n <= 11, up to 53% off (`BESSELI(709,3)` is 5.748E+305 against
    1.224E+306);
  - BESSELJ, 1 point: `BESSELJ(11.0627245753426,4)`, 0.14% off next to a root of J_4.
- The engine matches Excel bit for bit at all of them (directive 1), so Gate A holds through Excel's 15 digits and
  nothing waits on this ruling.
- The owner's options: (a) keep matching Excel, so no R3 case; (b) an R3 case over a bounded region (which functions,
  which |x| and n), with a decision record, R3 marks and an engine change that follows the math there.

## Acceptance criteria
- [x] The owner's ruling is recorded (a decision record for an R3 case), and the engine and tables follow it.
- [x] `make check` green; reviewers approve.

## Out of scope
- Excel's ~1E-7 BESSELJ/BESSELY imprecision below G (decision 0188 R3 keeps it as the target).

## Outcome
- Owner ruling 2026-11-06, decision 0300: keep matching Excel. This is not an R3 case: no `MATH-OVER-EXCEL`
  entry, no R3 mark, no engine change (the engine already matches Excel here).
- Eval delta: none.
