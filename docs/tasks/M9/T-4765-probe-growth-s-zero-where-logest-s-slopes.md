---
id: T-4765
title: "Probe GROWTH's zero where LOGEST's slopes' factor is subnormal: the cut, several x, a subnormal b, const FALSE"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Pin down where Excel's GROWTH is 0 because LOGEST's slopes' factor is subnormal, then make the engine follow it.
probes/t4590-growth-overflow Int A2:A7 show GROWTH is 0 where the prediction is about 1.

## Context
- T-4590 found it (statistics.md "GROWTH is #NUM! once LOGEST's b overflows; it can be 0 where the slopes' factor is
  subnormal"). In the Int rows, y = e^(c - c/2·x) at x = 1, 2, 3, predicted at x = 2. GROWTH is 0 for c = 709 to
  709.7827, where the slopes' factor is e^(-709) or below, and 1 at c = 700. T-4590 left this unimplemented because
  the evidence covers only const TRUE, one x and a slope sum in [-709.78, -709].
- Owns probes/t4590-growth-overflow Int! A2, A3, A4, A5, A6, A7 (ours about 1, Excel 0).
- To probe, with predictions committed before the fill:
  - the cut, with slope sums between -700 and -709 (the DBL_MIN cut -708.396 is the natural hypothesis, numbers.md
    "Subnormal results become 0");
  - several x with opposing large terms (m1·x1 = -720, m2·x2 = +720: one summed factor or one per variable);
  - a subnormal b (intercept below -708.4) with an offsetting slope;
  - const FALSE with a subnormal slopes' factor;
  - a slopes' factor that overflows at the new x while b keeps the prediction finite (const TRUE,
    y = e^(-c + c/2·x), c >= 710).
- Code: `growth_prediction` in functions/statistical/trend.rs.

## Acceptance criteria
- [x] The probe is filled and its rule recorded in statistics.md. The engine follows it, and Int A2:A7 pass.
- [x] `make check` green; reviewers approve.

## Out of scope
- The least-squares digits (T-4763, T-2006).

## Outcome
- Probe: probes/t4765-growth-factor-edges (excel-graph, 2026-11-07). The predictions, H1 (two factors as doubles,
  subnormal ones flushed) and H2 (the engine before T-4765), were committed before the fill (053f5525b,
  docs/research/t4765). H1 matched 19 of 24 cells and H2 12.
  - Cut: the zero starts between a slope sum of -708.39 (1) and -708.4 (0). That is ln(DBL_MIN) = -708.396, so the
    slopes' factor F is flushed once it is subnormal.
  - NoConst: const FALSE is 0 too.
  - Over: F overflowing is `#NUM!` though the prediction is finite.
  - SmallB: a subnormal b is not flushed (A3 is about 1), which goes against H1.
  - Multi: `#REF!` from a shape mismatch in my formulas, so several x stay unprobed. Follow-up T-4793 re-probes
    them, with a subnormal product of normal factors.
- Code: `growth_prediction` (functions/statistical/trend.rs) adds two rules to T-4590's b-overflow `#NUM!`: `#NUM!`
  where F overflows, and 0 where F is below DBL_MIN (`f64::MIN_POSITIVE`). The value elsewhere is unchanged. The cuts
  are the double range, so there are no constants.
- Docs: the statistics.md entry is rewritten as "GROWTH is #NUM! once LOGEST's b or slopes' factor overflows, and 0
  once the slopes' factor is subnormal". New unit test `growth_slopes_factor_edges`.
- Follow-up: T-4793 (several x; a subnormal product of normal factors). statistics.md marks both UNVERIFIED.
- Eval delta:
  - probes/t4590-growth-overflow Int A2:A7 now pass (6 cells), and the new probe passes 20 of 24.
  - The other 4 are least-squares digits, about 1E-13 off, added to T-4763.
  - Gate A unchanged, 0 failing. eval-check: 0 regressions.
