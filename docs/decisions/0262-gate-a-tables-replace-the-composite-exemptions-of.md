# 0262 — Gate A tables replace the composite exemptions of MDETERM, MINVERSE and MMULT

- **Status:** accepted (2026-11-04, T-3686). Amends `eval/accuracy/exempt.tsv`, a protected path (decision 0199).
- **Context:**
  - Decision 0228 (T-3551) exempted MDETERM, MINVERSE and MMULT as `composite` for one reason only: the generator's
    array literal wrote one row, and MINVERSE and MMULT return arrays. It said that a generator able to drive them
    takes the lines out under a new record.
  - T-3686 teaches the generator 2-D array literals (`{a,b;c,d}`, parameter kind `matrix`) and indexed definitions,
    whose cell reads one element of the result (`=INDEX(MINVERSE({...}),row,column)`).
  - Their `s1` tables are oracle-filled (excel-graph, 2026-11-04): MDETERM 120 points, MINVERSE 200 and MMULT 200.
    All 520 are inside the envelope, with 0 failing and 0 table defects. Excel is off the exact math by more than
    1e-9 on 1 MDETERM point and 14 MINVERSE points (near-singular and wide-range matrices). The engine matches
    Excel's elimination on every one, so they are inside on Excel's 15 digits.
- **Decision:** remove the three `composite` lines from `exempt.tsv`. The functions are covered by their tables.
- **Consequences:**
  - `rustsheet-accuracy` reports 3 fewer missing tables, and no exemption is stale.
  - An indexed definition puts INDEX's `row_num` and `column_num` after the function's catalog parameters. The
    catalog-order test skips those two.
  - The eval-cell math sidecars (`rustsheet-accuracy math`) still skip matrix arguments, because the sweep reader
    flattens array literals and ranges. T-4025 adds them.
