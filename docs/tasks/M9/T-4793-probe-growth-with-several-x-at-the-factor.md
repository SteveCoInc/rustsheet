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
- Probe 1, probes/t4793-growth-several-x (excel-graph, 2026-11-07). The predictions were committed before the fill
  (8b1cc66f2, docs/research/t4793).
  - Multi: with known_y a column against a 4x2 known_x, opposing slope terms of ±710 to ±800 give `#NUM!` on 3 of 3
    rows. A summed factor would give e^0, so the factors are per variable. The ±200 control gives e^1.
  - SubProd: a subnormal prediction from normal b and factors is 0 on 4 of 4 rows. The normal controls match.
- Probe 2, probes/t4793-growth-one-factor-underflow, filled after excel-fidelity review round 1. The predictions were
  committed before the fill (f21a5304e, predictions2.tsv).
  - With two x, a first factor of e^-720 or e^-709 gives 0 (One A1, A3).
  - e^600 followed by e^-720 gives a number (One A2), with the subnormal factor's precision in it. So Excel
    multiplies the factors left to right in doubles and flushes each partial product once it is subnormal. The
    engine's first rule, 0 for any underflowing factor, missed One A2, so it was replaced.
- Code: `growth_prediction` (functions/statistical/trend.rs) forms the product of the variables' factors left to
  right with `flush_subnormal`.
  - A non-finite product (overflow, or 0·∞) is `#NUM!`.
  - A product of 0 is 0.
  - Otherwise the result is e^(sum), flushed to 0 when subnormal.
  - With one x this is T-4765's rule unchanged. No constants are involved; the cuts are the double range.
- Docs: the statistics.md entry is rewritten. Three or more x, and a zero product meeting a later normal factor
  below 1, are marked as not probed.
- Eval delta:
  - probe 1 passes 8 of 10 and probe 2 passes 3 of 4;
  - Multi A4, SubProd A5 and One A2 are digit cells, added to T-4763;
  - the t4765 and t4590 probes are unchanged (20 of 24, 47 of 61).
