---
id: T-4411
title: "BETA.DIST cdf last digits on bup + bgrat at a0 < 1 and w1 bpser, x near 0.3: 227 x29-probe cells within 15 digits that no task names (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
Each listed cell of the x29 probe (probes-numeric, T-2464 r2-bratio) returns Excel's double, or the Outcome lists
it with Excel, ours, the 80-digit math (t2464-r2-bratio-x29.math.json) and the reason Excel's last bits cannot be
derived. The cells are
BETA.DIST cdf points on bratio's bup + bgrat branch at a0 < 1 and on its w1 = bpser(b0, a0, y0) branch at x0 >= 0.3.
Every one is already within half a unit of the 15th significant digit, so this is Phase 1b work only (decision 0114).

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 952336450). Ownership was checked with `scripts/exit_owners.py`'s
  `owned` over every failing cell, not only the visibly-off ones. 227 failing cells of the x29 probe (3,080
  cells, excel-graph) have no owner. All are NumericUlp, 1 to 3 ulp from Excel. None is visibly off. They split by
  bratio branch (`bratio/mod.rs` `dispatch` and `ported_tails_with`) as follows; "CR" means correctly rounded against
  the 80-digit math.
  - **bup + bgrat at a0 != 1/2 (161 of 1,500 cells; b = 2.5 to 14.5, x < 0.3).** This is T-3665's arm: 20 bup terms
    plus `bgrat_increment` in double-double. Excel is CR on 71 of these cells and ours on 54. Examples:
    - A1 `BETA.DIST(0.2900,0.25,2.5,TRUE)`: Excel 0.8978847192944112, ours 0.8978847192944113,
      math 0.89788471929441110...
    - A10 `BETA.DIST(0.2909,0.25,2.5,TRUE)`: Excel 0.8983384307273808, ours 0.898338430727381,
      math 0.89833843072738091...
  - **bup + bgrat at a0 = 1/2 (58 of 500 cells).** This uses `bgrat_half` (T-2509). Excel is CR on 26 and ours on 24.
    Example: A1005 `BETA.DIST(0.2914,0.5,7.5,TRUE)`: Excel 0.9746909208143628, ours 0.9746909208143629, math
    0.97469092081436283...
  - **w1 = bpser(b0, a0, y0) at x0 >= 0.3 (8 of 280 cells).** Excel is CR on none of them and ours on 4. Example: A107
    `BETA.DIST(0.3006,0.25,2.5,TRUE)`: Excel 0.9031091164245724, ours 0.9031091164245725, math 0.90310911642457246...
- statistics.md "Incomplete beta is TOMS 708 `bratio`" (the T-3665 bullet) records that Excel is within 1.7 ulp of the
  math in this region. So both values are faithful, and the remaining bits come from Excel's order of operations
  inside bup, bgrat's increment and the final `(0.5 - w) + 0.5`, not from a less accurate kernel.
- Reproducibility floor (R4.4): every difference is 1 to 3 ulp, inside the 4-ulp web/desktop floor
  (docs/research/oracle-calibration.md). T-3992 measured the bpser family stable over three fills (0 of 5,843 cells
  changed), but nobody has measured bup + bgrat. So one refill of this case comes before any fitting.
- The evidence is already cached: the case's 2,280 bup+bgrat and w1-bpser cells. A fit needs a fresh sweep after the
  freeze (R4.2), drawn and oracle-filled inside this task, so no separate research sweep task is needed first.
- Related tasks: T-3871 and T-3712 own L140 bup+bgrat at min(a, b) > 1, a different branch. T-3956 owns bpser last
  digits on other probes. T-4410 owns this case's direct-bgrat cells (b0 > 15). T-3592 owns A1616 and A2393.
- Cells:
  - probes-numeric/t2464-r2-bratio-x29.case.json (161): Calc!A1, A4, A10, A12, A29, A32, A41, A43, A50, A51, A53, A70, A78, A94, A114, A165, A166, A169, A229, A244, A247, A297, A1547, A1548, A1552, A1557, A1558, A1562, A1564, A1568, A1570, A1575, A1576, A1579, A1581, A1583, A1584, A1585, A1588, A1589, A1591, A1594, A1596, A1600, A1602, A1603, A1606, A1608, A1610, A1611, A1612, A1613, A1615, A1617, A1618, A1619, A1621, A1622, A1624, A1627, A1630, A1632, A1640, A1655, A1658, A1660, A1661, A1664, A1666, A1677, A1680, A1684, A1690, A1695, A1703, A1705, A1713, A1718, A1726, A1732, A1736, A1741, A1746, A1747, A1800, A1822, A2314, A2316, A2318, A2319, A2321, A2323, A2325, A2326, A2328, A2329, A2331, A2335, A2340, A2349, A2350, A2351, A2353, A2355, A2358, A2359, A2360, A2363, A2365, A2366, A2367, A2369, A2370, A2377, A2378, A2380, A2383, A2384, A2385, A2388, A2396, A2399, A2401, A2402, A2403, A2404, A2406, A2407, A2421, A2422, A2431, A2434, A2443, A2451, A2455, A2463, A2475, A2479, A2483, A2485, A2490, A2492, A2497, A2498, A2503, A2504, A2516, A2542, A2561, A2562, A2568, A2593, A2601, A2608, A2616, A2622, A2674, A2716, A2770, A2818, A2839
  - probes-numeric/t2464-r2-bratio-x29.case.json (58): Calc!A779, A780, A782, A788, A790, A792, A793, A796, A797, A798, A800, A801, A802, A809, A811, A812, A814, A816, A819, A821, A822, A823, A828, A832, A835, A837, A839, A840, A842, A843, A845, A846, A849, A850, A854, A856, A858, A861, A864, A865, A926, A939, A950, A951, A959, A960, A961, A963, A965, A966, A970, A972, A973, A977, A980, A1005, A1025, A1063
  - probes-numeric/t2464-r2-bratio-x29.case.json (8): Calc!A107, A109, A1641, A1646, A1647, A1648, A1650, A1757

## Acceptance criteria
- [ ] A refill of the x29 probe (excel-graph) reports how many of the 227 cells changed bits. Cells that vary
      between fills stop being targets (R4.4, decision 0112) and are listed in the Outcome.
- [ ] One of these holds for each branch group:
      - a model of Excel's arithmetic on that branch makes its listed cells bit-exact without moving any cell of the
        case outside 15 digits;
      - or the Outcome lists every cell still off, with Excel, ours and the math, and ranks the candidates tried by
        bit-exact count.
- [ ] Any kept candidate chosen by scoring passes every R4 guard of reviewer-excel-fidelity check 7 (decision 0188),
      each shown in the Outcome:
      - hold-out counts: train and hold-out side by side (bit-exact / within 4 ulp / within 15 digits) for the
        candidate and for the correctly rounded reference, with the margin fixed before scoring;
      - a fresh log-uniform sweep, drawn and oracle-filled after the freeze, and its result;
      - the Gate A envelope count on that sweep, with every point outside the envelope listed;
      - a `FITTED(T-4411, n_cells, range)` tag, applied only inside its range;
      - no step that fits differences below the reproducibility floor;
      - no keep-matched branch.
- [ ] The listed cells leave the probes-numeric failing list. Because probes-numeric is gate = false, it has no
      eval/baseline file. No new failing cell appears in the x29 probe, the w2f unit-shape probe or the F, T and
      BETA.INV callers of the same arms, and no eval/baseline/*.txt file gains a line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Direct bgrat at b0 > 15 (T-4410 and T-3933) and L140 at min(a, b) > 1 (T-3871, T-3712).
