---
id: T-4410
title: "BETA.DIST cdf last digits on direct bgrat at a0 != 1/2, x in [0.29, 0.3): 155 x29-probe cells within 15 digits, after T-3933 (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3933]
defer: phase-1b
---

## Goal
Each listed cell of the x29 probe (probes-numeric, T-2464 r2-bratio) returns Excel's double, or the Outcome lists
it with Excel, ours, the 80-digit math (t2464-r2-bratio-x29.math.json) and the reason Excel's last bits cannot be
derived. The cells are
BETA.DIST cdf points on bratio's direct bgrat branch (0 < a0 < 1, a0 != 1/2, b0 > 15). Every one is already within
half a unit of the 15th significant digit, so this is Phase 1b work only (decision 0114).

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 952336450). Ownership was checked with `scripts/exit_owners.py`'s
  `owned` over every failing cell, not only the visibly-off ones. 155 failing cells of the x29 probe sit at
  a = 0.25, 0.75 or 0.9, b = 20 or 50 and x in [0.29, 0.3). There bratio goes straight to bgrat(b0, a0, y0, x0)
  (b0 > 15). No open task names these cells. All are NumericUlp, 1 or 2 ulp from Excel, and none is visibly off.
- Ours comes from today's `incomplete_beta::beta_reg_tails` fallback, because `ported_tails_with` takes
  `Branch::Bgrat` only at a0 = 1/2. Against the 80-digit math, ours is the correctly rounded double on 154 of the 155
  cells and Excel is correctly rounded on 1. Examples:
  - A2097 `BETA.DIST(0.2906,0.75,20.0,TRUE)`: Excel 0.9994689991536878, ours 0.9994689991536877, math
    0.99946899915368773...
  - A2103 `BETA.DIST(0.2912,0.75,20.0,TRUE)`: Excel 0.999478142893949, ours 0.9994781428939491, math
    0.99947814289394904...
- The root cause is Excel's own direct-bgrat arithmetic, and its noise is still unexplained. See statistics.md
  "Incomplete beta: the integrated model", the "Why TOMS bgrat lost at b != 1/2" item: Excel's D is a median 4 units of
  2^-53 low, with white noise of sd 5.7-8.5 units that no model explains (T-3567 and T-3582 negatives).
- T-3933 moves this branch onto `bgrat_increment` (the accurate double-double bgrat) for the bgrat-split probe's
  visibly-off cells, but it does not name these cells. This task scores them once T-3933 lands, hence the dependency:
  some of them may become bit-exact and some may move.
- Reproducibility floor (R4.4): the differences here are 1-2 ulp, inside the 4-ulp web/desktop floor
  (docs/research/oracle-calibration.md). T-3992 found 0 of 5,843 bpser cells changed over three fills, but nobody has
  measured fill-to-fill stability for direct bgrat. So one refill of this case comes before any fitting.
- The evidence is already cached: this case's 800 direct-bgrat cells (excel-graph) and the 2,908 of T-3933's
  bgrat-split probe. A fit needs a fresh sweep after the freeze (R4.2), drawn and oracle-filled inside this task as
  T-3524 did, so no separate research sweep task is needed first.
- T-4411 owns the case's bup + bgrat and w1-bpser cells. T-3592 owns A1616 and A2393.
- Cells:
  - probes-numeric/t2464-r2-bratio-x29.case.json (155): Calc!A552, A558, A560, A562, A563, A564, A566, A571, A577, A582, A584, A586, A588, A590, A591, A597, A599, A604, A605, A607, A621, A622, A624, A626, A632, A634, A640, A644, A646, A649, A661, A663, A667, A672, A681, A682, A689, A699, A702, A706, A708, A710, A711, A715, A716, A723, A727, A731, A735, A737, A738, A739, A740, A741, A742, A744, A751, A760, A2097, A2103, A2106, A2107, A2108, A2110, A2113, A2119, A2125, A2129, A2131, A2132, A2133, A2142, A2144, A2150, A2154, A2158, A2161, A2169, A2172, A2178, A2187, A2188, A2219, A2222, A2223, A2229, A2233, A2235, A2240, A2246, A2247, A2249, A2253, A2260, A2261, A2263, A2267, A2268, A2269, A2276, A2279, A2284, A2285, A2286, A2287, A2295, A2862, A2864, A2867, A2870, A2871, A2875, A2878, A2885, A2886, A2890, A2899, A2902, A2905, A2907, A2908, A2912, A2913, A2914, A2917, A2920, A2922, A2931, A2932, A2943, A2946, A2956, A2958, A2959, A2960, A2972, A2973, A2975, A2987, A2991, A3000, A3002, A3007, A3009, A3018, A3021, A3023, A3027, A3030, A3038, A3049, A3052, A3053, A3056, A3057

## Acceptance criteria
- [ ] After T-3933, a refill of the x29 probe (excel-graph) reports how many of the 155 cells changed bits. Cells
      that vary between fills stop being targets (R4.4, decision 0112) and are listed in the Outcome.
- [ ] One of these holds:
      - a model of Excel's direct bgrat at a0 != 1/2 makes the listed cells bit-exact without moving any cell of
        the x29 probe or T-3933's bgrat-split probe outside 15 digits;
      - or the Outcome lists every cell still off, with Excel, ours and the math, and ranks the candidates tried by
        bit-exact count.
- [ ] Any kept candidate chosen by scoring passes every R4 guard of reviewer-excel-fidelity check 7 (decision 0188),
      each shown in the Outcome:
      - hold-out counts: train and hold-out side by side (bit-exact / within 4 ulp / within 15 digits) for the
        candidate and for the correctly rounded reference, with the margin fixed before scoring;
      - a fresh log-uniform sweep, drawn and oracle-filled after the freeze, and its result;
      - the Gate A envelope count on that sweep, with every point outside the envelope listed;
      - a `FITTED(T-4410, n_cells, range)` tag, applied only inside its range;
      - no step that fits differences below the reproducibility floor;
      - no keep-matched branch.
- [ ] The listed cells leave the probes-numeric failing list. Because probes-numeric is gate = false, it has no
      eval/baseline file. No new failing cell appears in the x29 probe or T-3933's bgrat-split probe, and no
      eval/baseline/*.txt file gains a line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The bgrat-split probe's cells, and the branch switch itself (T-3933).
- This case's bup + bgrat and w1-bpser cells (T-4411) and the cells T-3592 owns.
