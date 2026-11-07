---
id: T-4793
title: "Probe GROWTH with several x at the factor edges, and a subnormal product of normal factors"
milestone: M9
lane: functions-stat
status: todo
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
- [ ] The probe is filled and recorded in statistics.md, and the engine follows it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The least-squares digits (T-4763, T-2006).
