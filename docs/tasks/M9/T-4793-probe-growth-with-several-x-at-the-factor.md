---
id: T-4793
title: "Probe GROWTH with several x at the factor edges, and a subnormal product of normal factors"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel's GROWTH at the factor edges with several x, and at a subnormal product of two normal factors, is probed and
the engine follows it.

## Context
- T-4765 (statistics.md "GROWTH is #NUM! once LOGEST's b or slopes' factor overflows, and 0 once the slopes' factor
  is subnormal") pinned down the edges for one x. Its Multi rows were `#REF!`, because the formulas paired a 1x4
  known_y with a 4x2 known_x.
- Several x: is the slopes' factor one summed e^(Σ m_j x_j) (H1, the engine) or one factor per variable,
  e^(m_j x_j) (H3)? Probe it with known_y as a column ({..;..;..;..}) against the 4x2 known_x, and with opposing
  large terms (m1·x1 = -720, m2·x2 = +720). Commit the predictions before the fill.
- A subnormal product of two normal factors, for example b = e^-400 and F = e^-310: the engine returns the
  subnormal e^(sum). numbers.md "Subnormal results become 0" suggests that Excel returns 0.
- Code: `growth_prediction` in functions/statistical/trend.rs.

## Acceptance criteria
- [x] The probe is filled and recorded in statistics.md, and the engine follows it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The least-squares digits (T-4763, T-2006).

## Outcome
- Probe: probes/t4793-growth-several-x (excel-graph, 2026-11-07). The predictions were committed before the fill
  (8b1cc66f2, docs/research/t4793). H1 was the engine as T-4765 left it.
  - Multi: with known_y a column against a 4x2 known_x, the factors are per variable (H3), not one summed factor
    (H1). Opposing slope terms of ±720 to ±800 give `#NUM!` on 3 of 3 rows; the ±200 control gives e^1.
  - SubProd: a subnormal prediction from normal b and F is flushed to 0 (H4) on 4 of 4 rows; the normal controls
    match.
- Code: `growth_prediction` (functions/statistical/trend.rs) forms each variable's factor e^(m_j x_j) separately.
  - `#NUM!` if any of them overflows.
  - 0 if any is below DBL_MIN.
  - The prediction e^(sum) is flushed to 0 when subnormal.
  - With one x this is T-4765's rule unchanged. No constants are involved: the cuts are the double range.
- Docs: the statistics.md entry is rewritten. A single factor's zero with two x is marked as inferred from the one-x
  rows.
- Eval delta: the new probe passes 8 of 10. Multi A4 and SubProd A5 are least-squares digits, now listed in T-4763.
  The t4765 and t4590 probes are unchanged (20 of 24, 47 of 61).
