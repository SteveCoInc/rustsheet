---
id: T-4407
title: "BETA.DIST cdf last digits at a < 1, x in [0.29, 0.301): 382 unowned t2464-r2-bratio-x29 cells within 15 digits (bup+bgrat, direct bgrat, complement bpser)"
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
The 382 BETA.DIST cells of probes-numeric/t2464-r2-bratio-x29 listed below, all within 15 digits of Excel and 1 to 2
ulp off it, either return Excel's bits or are listed in the Outcome. For each listed cell the Outcome gives Excel, ours
and the 80-digit math, and the reason Excel's digits cannot be derived (decision 0267 accepts them then, with no
follow-up).

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 15371221a; every failing cell checked against every open task by
  `scripts/exit_owners.py`'s ownership rule): no open task names these cells. T-3592 owns two other cells of the case
  (A1616, A2393). Every cell is NumericUlp and within half a unit of Excel's 15th digit, so the group is deferred
  (decision 0114). It is incomplete-beta family work at the 15th digit or below, which decision 0267 item 4 cuts
  until Phase 1 exits. It is opus because matching it means modelling Excel's arithmetic.
- The case: BETA.DIST(x, a, b, TRUE) at a in {0.25, 0.5, 0.75, 0.9}, b in {2.5, 5, 7.5, 12, 14.5, 20, 50}, x in
  [0.29, 0.301) (T-2464, r2-bratio/make_probe_x29.py). The math is in t2464-r2-bratio-x29.math.json (80 digits).
  statistics.md "Incomplete beta is TOMS 708 `bratio`" settles the x0 >= 0.3 dispatch on these cells.
- Groups by `bratio/mod.rs` `small_shape_branch` (a0 = a < 1 < b0 = b, no swap). Distances are in ulp from the math,
  medians, Excel then ours:
  - **Direct bgrat, b0 > 15, x0 < 0.3 (155 cells; a = 0.25, 0.75, 0.9 at b = 20 and 50).** Ours is correctly rounded
    on all 155 and Excel on 2 (0.61 against 0.39). Excel is about 0.6 ulp off the math, the unexplained noise of
    T-3567 and T-3582. a = 1/2 has no failing cell here, because `bgrat_half` is ported. T-3933 moves this branch
    onto `bgrat_increment` for a0 != 1/2, so this task depends on it: re-measure these cells after T-3933 lands.
    Example A2097 `BETA.DIST(0.2906,0.75,20.0,TRUE)`: Excel 0.9994689991536878, ours 0.9994689991536877, math
    0.99946899915368773112.
  - **bup + bgrat, b0 <= 15, x0 < 0.3, a0 != 1/2 (161 cells).** This is T-3665's arm: 20 TOMS bup terms plus the
    double-double `bgrat_increment`. Excel is correctly rounded on 71 and ours on 54 (0.59 against 0.72), with a
    difference of at most 2 ulp. Example A1 `BETA.DIST(0.2900,0.25,2.5,TRUE)`: Excel 0.8978847192944112, ours
    0.8978847192944113, math 0.89788471929441110577.
  - **bup + bgrat at a0 = 1/2 (58 cells; b = 2.5, 5, 7.5).** On `bgrat_half`. Excel is correctly rounded on 26 and
    ours on 24 (0.60 against 0.68). Example A1005 `BETA.DIST(0.2914,0.5,7.5,TRUE)`: Excel 0.9746909208143628, ours
    0.9746909208143629, math 0.97469092081436283618.
  - **w1 = bpser(b0, a0, y0), x0 >= 0.3 (8 cells).** Excel is correctly rounded on 0 and ours on 4 (1.41 against
    0.69). Example A109 `BETA.DIST(0.3008,0.25,2.5,TRUE)`: Excel 0.9032052314021777, ours 0.9032052314021778, math
    0.90320523140217783405.
- On 236 of the 382 cells ours is the correctly rounded value. So an `accuracy` stage would not move them; only a model
  of Excel's own rounding would. T-3992 measured the family's bits as stable over three fills (0 of 5,843 changed), so
  the differences are above decision 0112's floor, but the T-3567 and T-3582 negatives found no model for the noise in
  bgrat's D.
- Cells (probes-numeric/t2464-r2-bratio-x29, Calc sheet):
  - Direct bgrat (155), probes-numeric/t2464-r2-bratio-x29 Calc!A552, A558, A560, A562, A563, A564, A566, A571, A577,
    A582, A584, A586, A588, A590, A591, A597, A599, A604, A605, A607, A621, A622, A624, A626, A632, A634, A640, A644,
    A646, A649, A661, A663, A667, A672, A681, A682, A689, A699, A702, A706, A708, A710, A711, A715, A716, A723, A727,
    A731, A735, A737, A738, A739, A740, A741, A742, A744, A751, A760, A2097, A2103, A2106, A2107, A2108, A2110, A2113,
    A2119, A2125, A2129, A2131, A2132, A2133, A2142, A2144, A2150, A2154, A2158, A2161, A2169, A2172, A2178, A2187,
    A2188, A2219, A2222, A2223, A2229, A2233, A2235, A2240, A2246, A2247, A2249, A2253, A2260, A2261, A2263, A2267,
    A2268, A2269, A2276, A2279, A2284, A2285, A2286, A2287, A2295, A2862, A2864, A2867, A2870, A2871, A2875, A2878,
    A2885, A2886, A2890, A2899, A2902, A2905, A2907, A2908, A2912, A2913, A2914, A2917, A2920, A2922, A2931, A2932,
    A2943, A2946, A2956, A2958, A2959, A2960, A2972, A2973, A2975, A2987, A2991, A3000, A3002, A3007, A3009, A3018,
    A3021, A3023, A3027, A3030, A3038, A3049, A3052, A3053, A3056, A3057
  - bup + bgrat, a0 != 1/2 (161), probes-numeric/t2464-r2-bratio-x29 Calc!A1, A4, A10, A12, A29, A32, A41, A43, A50,
    A51, A53, A70, A78, A94, A114, A165, A166, A169, A229, A244, A247, A297, A1547, A1548, A1552, A1557, A1558, A1562,
    A1564, A1568, A1570, A1575, A1576, A1579, A1581, A1583, A1584, A1585, A1588, A1589, A1591, A1594, A1596, A1600,
    A1602, A1603, A1606, A1608, A1610, A1611, A1612, A1613, A1615, A1617, A1618, A1619, A1621, A1622, A1624, A1627,
    A1630, A1632, A1640, A1655, A1658, A1660, A1661, A1664, A1666, A1677, A1680, A1684, A1690, A1695, A1703, A1705,
    A1713, A1718, A1726, A1732, A1736, A1741, A1746, A1747, A1800, A1822, A2314, A2316, A2318, A2319, A2321, A2323,
    A2325, A2326, A2328, A2329, A2331, A2335, A2340, A2349, A2350, A2351, A2353, A2355, A2358, A2359, A2360, A2363,
    A2365, A2366, A2367, A2369, A2370, A2377, A2378, A2380, A2383, A2384, A2385, A2388, A2396, A2399, A2401, A2402,
    A2403, A2404, A2406, A2407, A2421, A2422, A2431, A2434, A2443, A2451, A2455, A2463, A2475, A2479, A2483, A2485,
    A2490, A2492, A2497, A2498, A2503, A2504, A2516, A2542, A2561, A2562, A2568, A2593, A2601, A2608, A2616, A2622,
    A2674, A2716, A2770, A2818, A2839
  - bup + bgrat at a0 = 1/2 (58), probes-numeric/t2464-r2-bratio-x29 Calc!A779, A780, A782, A788, A790, A792, A793,
    A796, A797, A798, A800, A801, A802, A809, A811, A812, A814, A816, A819, A821, A822, A823, A828, A832, A835, A837,
    A839, A840, A842, A843, A845, A846, A849, A850, A854, A856, A858, A861, A864, A865, A926, A939, A950, A951, A959,
    A960, A961, A963, A965, A966, A970, A972, A973, A977, A980, A1005, A1025, A1063
  - w1 bpser (8), probes-numeric/t2464-r2-bratio-x29 Calc!A107, A109, A1641, A1646, A1647, A1648, A1650, A1757
- Code: crates/rustsheet-engine/src/functions/statistical/bratio/ (`mod.rs` `small_shape_branch` and
  `ported_tails_with`, `bgrat.rs` `bgrat_increment`, `bup.rs`). Read docs/research/excel-kernel-playbook.md and
  docs/research/t2464/r3-bgrat/RESULTS.md first. Decision 0056's time-box applies.

## Acceptance criteria
- [ ] Each of the 382 cells above passes, or the Outcome lists it under its branch with Excel, ours and the math, and
      names the model tried for that branch with its bit-exact count (decision 0267 then accepts the cell).
- [ ] probes-numeric is not gated, so the Outcome gives t2464-r2-bratio-x29's before and after counts (today 2,696 of
      3,080 passed) from `rustsheet-eval show --all`. No cell of the case leaves 15 digits. No new known-failure lines
      appear in any eval/baseline/<suite>.txt (`make eval-check` 0 regressions), except as decision 0267 item 6 allows
      with its counts in the PR. `make eval-baseline` records the fixes.
- [ ] Any form, constant or branch choice ranked on Excel cells meets the decision 0188 R4 guards of
      reviewer-excel-fidelity check 7, shown in the Outcome:
      - hold-out: train and hold-out counts (bit-exact, within 4 ulp, within 15 digits) for the chosen candidate and the
        correctly rounded one, on a split fixed before scoring, with its margin;
      - fresh sweep: an `r4-T-4407` batch of BETA.DIST's Gate A table (eval/accuracy/tables/BETA.DIST; SPEC-0029 "The
        R4 guards"), drawn after the freeze over a < 1 < b and x in [0.25, 0.35]. Decision 0267 item 5 allows no new
        probes-numeric case;
      - Gate A envelope: that batch's count inside the envelope, with every point outside listed;
      - `FITTED(T-4407, n_cells, range)` on each fitted piece, applied only inside its range, accurate math outside;
      - reproducibility floor: no step fits differences below the oracle's floor (decision 0112, T-3992);
      - no keep-matched branch (decisions 0056 and 0082).
- [ ] `make check` green; reviewers approve.

## Out of scope
- t2512-r3-bgrat-split's direct-bgrat cells and the kernel move itself (T-3933); L140 with both shapes above 1
  (T-3712, T-3871); BETA.INV's search (T-3867).
