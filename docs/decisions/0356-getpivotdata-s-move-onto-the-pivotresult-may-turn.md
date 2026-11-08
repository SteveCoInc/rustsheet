# 0356 — GETPIVOTDATA's move onto the PivotResult may turn eight accidental t4177-date-flags-needed passes into T-4941 known failures

- **Status:** accepted (2026-11-08, owner ruling; T-4154 in the M6 probe-cell PR)
- **Context:**
  - T-4154 moves GETPIVOTDATA from folding cache records onto the pivot's `PivotResult`, the values the grid writer
    renders (docs/excel-semantics/web.md "GETPIVOTDATA against a pivot table"). On a fresh eval it fixes 88 probe
    cells (M6 70 to 3 visibly off), 30 pivot-export cells and the M16 probe cells of t4130 and t4170, with no other
    regression.
  - Eight pivot-export cells that passed before now fail: pivot-export/t4177-date-flags-needed.export.json Calc!B1:B3,
    Calc!B5, Calc!B6, Calc!B9, Calc!B11, Calc!B14. Excel answers `#REF!` there because its refresh on open drops the
    date grouping of a cache whose Date field lacks `containsNonDate="0"`, `minDate` and `maxDate`, so no month or
    level exists (pivots.md "Excel for the web opens engine-exported grouped pivot caches"). The engine keeps the
    grouping, which is T-4941 (M16): its render already differs from Excel's in 125 cells of the same workbook.
  - The old GETPIVOTDATA answered `#REF!` for any pair on a pivot with two fields on an axis (SPEC-0015 unknown 10),
    so these eight cells matched Excel by accident, not by the rule Excel follows. The new answers are the values the
    engine's own pivot shows there, which T-2085 established is what GETPIVOTDATA reads.
  - Pivot tables are a Phase 2 milestone since decision 0354, so these cells count toward no Phase 1 exit row.
- **Decision:**
  1. The implementing PR runs `rustsheet-eval update-baseline --allow-regressions` once, and the baseline diff may add
     F lines for exactly the eight cells above and no other cell; reviewers check the diff line by line.
  2. T-4941 owns the eight cells: when its fix drops the grouping as Excel does, they pass again.
  3. No special case keeps the old `#REF!` for these pivots: GETPIVOTDATA follows the render everywhere.
- **Consequences:**
  - pivot-export's known failures rise by eight and fall by the 30 cells T-4154 fixes in the same suite.
  - T-4941's owner list names Calc!B1:B3, Calc!B5, Calc!B6, Calc!B9, Calc!B11 and Calc!B14 beside its B16, B18, B20.
  - A later cell that passes only because an unprobed shape answered `#REF!` follows the same route: a decision
    record that lists it, and the owner task of the real cause.
