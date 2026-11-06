---
id: T-4093
title: "Gate A tables, PERCENTRANK, PERCENTRANK.INC and PERCENTRANK.EXC"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
PERCENTRANK, PERCENTRANK.INC and PERCENTRANK.EXC each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
Every failing table point has an owner.

## Context
- Gate A (decision 0188 R1(a), SPEC-0029): decision 0271 (T-3799) classified these functions as needing a table.
- The generator's one-row array literals (T-3586) and 2-D literals (T-3686) drive the data arguments. Variadic data
  follows T-3549's SUM and AVERAGE definitions (the draw of a variable-length array and the near-zero snap rule):
  reuse it, do not copy it (directive 3).
- An Excel error region is part of the domain or a `domain` rule, never a math value. Where Excel's number is more than
  G = 1e-4 off the math, list the points in the Outcome as R3 candidates for an owner ruling and do not mark them
  (decision 0188 R3).
- Tools: as in T-3925 (`rustsheet-accuracy` table generation, oracle fill, `rustsheet-eval accuracy`).
- Sources (statistics.md): "PERCENTRANK: a huge significance is #DIV/0!; truncation does not lose a digit at a cut
  (T-2256)", "Order statistics: the scalar argument's error before the no-number result; PERCENTRANK's #N/A before
  significance (T-2256)", "PERCENTRANK: a non-numeric-text or error significance before the #N/A (T-3781)".
- The result is truncated (not rounded) to `significance` digits, so the math reference truncates the 50-digit rank.

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing samplers and
      kernels, each citing its sources, with unit tests against Microsoft's published examples or the semantics
      entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other sample-statistics groups of decision 0271.

## Outcome
- **Definitions:** `accuracy/defs/percentrank.py` defines PERCENTRANK, PERCENTRANK.INC and PERCENTRANK.EXC, exactly,
  on the literals' doubles as rationals.
  - The position of x is c, the count of values below it, for an x equal to a value. Between two neighbours it is
    interpolated. INC takes p/(n - 1) and EXC (p + 1)/(n + 1).
  - The rank is cut to `significance` digits by Excel's decimal rule (statistics.md T-2256), as decisions 0223 and
    0244 read ROUND and CEILING. The rank times 10^significance is cut to 15 digits, floored and divided back.
    Without the cut, `PERCENTRANK.EXC({1E+15,1,-1E+15},0.3)` would be 0.499, where Excel gives 0.5.
  - A point whose digit changes within the rounding noise of Excel's double scaled rank has no value. The noise is
    twice the gap to the double in the engine's fitted operation order, plus two ulps.
  - Domain errors: x outside the data is `#N/A`, a significance below 1 is `#NUM!`, and 309 or more is `#DIV/0!`.
    One value equal to x ranks 1, whatever numeric significance. With a significance of 309 or more there it is
    unprobed, so it has no value. All 27 drawn error points carry Excel's code for their region (10 `#N/A`, 8
    `#NUM!`, 9 `#DIV/0!`).
- **Reuse (directive 3):** `array_of` and `sorted_values` (AGGREGATE 14-19, T-3693), `cut15` (CEILING/FLOOR,
  T-3673), and `by_stratum`, `domain_of`, `one_of`, `scalars` and `fraction_to_mpf` (`_common.py`). The edge
  stratum draws the error regions, ties, a constant array, a single value, Microsoft's examples and the t2473 cells.
  Tests are in tools/tests/test_accuracy_percentrank.py: Microsoft's examples, t2473 A1-A15 and D1-D23,
  percentrank-single-value, the domains and the draws.
- **Tables:** `eval/accuracy/tables/{PERCENTRANK,PERCENTRANK.INC,PERCENTRANK.EXC}/s1` have 200 points each. They were
  filled from excel-graph with 0 failures.
  - `rustsheet-eval accuracy` puts all 600 points inside the envelope (598 on both sides, 2 through the math only).
    0 failing, 0 table defects, 0 known failures added. `missing tables` went from 88 to 85 (76 to 73 after merging main).
  - The engine is bit-exact with Excel on 598 of 600. The other two are `PERCENTRANK.EXC({0,3},2,17)` (A128, A140).
    Excel gives 0.555555555555556 and the engine 0.555555555555555, 1.8E-16 apart. The engine's double rank of
    5/9 is below the correctly rounded one. This is filed as T-4556.
- **Excel kept as the target (decision 0300):** no table point and no probe sidecar cell is more than 1e-4 relative
  off the math. The largest table gap is 1.8E-16.
- **Semantics:** new entry, statistics.md "PERCENTRANK ranks x by the count of values below it (T-4093)". It states
  the position and rank rule, the ends, and the ties. A constant array with x equal to its value ranks 0 (INC) or
  1/(n + 1) (EXC), while a single value ranks 1. It cites the table cells, and the definitions cite it.
- **Math sidecars:** made with `rustsheet-accuracy math --function` for the three functions, in probes and
  probes-numeric. There are 121 new math cells: stat-order-floor-batch (49), stat-argument-modes (6) and
  t2473-percentrank-significance (66, new sidecar). probes-numeric has none, because its cells pass scalars as the
  data.
  - The change is additive: no existing entry changed.
  - All 121 cells agree with Excel's oracle value: 85 numbers within 1e-9, and 36 errors on both sides.
  - Cells with a fractional significance (t2473 A13, B13, C13) get no value, because the parameter is an integer.
  - `make eval-math-check` reports current.
- **Review round 1 (Excel fidelity):**
  - New probe probes/t4093-percentrank-tied-neighbour, 27 cells, filled from excel-graph. When x lies between two
    values and the lower neighbour is tied, Excel interpolates from that neighbour's last copy:
    `PERCENTRANK({1,1,1,3},2)` is 0.833. That is the rule the definition and the engine use, and it matches all 27
    cells.
  - The probe's 27 math sidecar cells agree with the oracle. The statistics.md entry cites the probe.
  - The Ties bullet now names PERCENTRANK and PERCENTRANK.INC (0) and gives n = 5, with EXC 1/6 at A113.
  - The docstring marks the order of the `#N/A` against the `#DIV/0!` as UNVERIFIED.
  - A non-integral or non-finite significance now gets no math value (`_significance_of`) instead of being
    truncated by `int()`.
- **Follow-up:** T-4556 (PERCENTRANK.EXC's last digit at 16 or more digits).
- **Eval delta:** none in the gated suites (tools, tables, docs and sidecars only).
