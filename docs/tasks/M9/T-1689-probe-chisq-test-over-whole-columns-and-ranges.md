---
id: T-1689
title: "Probe CHISQ.TEST over whole columns and ranges with blanks: degrees of freedom and blank pairs"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
CHISQ.TEST over whole columns, and over ranges whose cells are partly blank, text or booleans, has
excel-graph evidence. The engine matches it, or each failing cell has a filed task.

## Context
Found in T-1681 review. `fn_chisq_test` (`functions/statistical/chisq.rs`) turns a pair with a
non-number side into `(1, 1)`, which adds 0 to the statistic. It takes the degrees of freedom from
the shape of the ranges, so `CHISQ.TEST(A:A,B:B)` counts 1,048,575 degrees of freedom whatever the
used area. The unit test `chisq_test_over_whole_columns_counts_every_cell` in
`test/test_clipped_range_walks.rs` pins this behavior, but nothing shows that Excel does the same.
Probe these with the `excel-probe` skill:
- `CHISQ.TEST(A:A,B:B)`, and the same over `A1:A10` with blanks inside.
- 2-D ranges with blanks.
- Text and booleans in either range.

## Acceptance criteria
- [x] A probe case (≥ 8 cells) covers whole columns, blanks, text and booleans in 1-D and 2-D
      ranges, and is oracle-filled.
- [x] The unit test's expectation agrees with the evidence, or is corrected with the fix.
- [x] `make check` green; reviewers approve.

## Out of scope
- CHISQ.INV and CHISQ.DIST accuracy.

## Outcome
Probe `probes/t1689-chisq-test-blanks-whole-columns` (24 cells, excel-graph filled, Gate A sidecar
written). Findings, all matched by the engine except last-digit differences:
- Whole columns count every cell for the degrees of freedom: `CHISQ.TEST(W!A:A,W!B:B)` =
  `CHISQ.DIST.RT(stat,1048575)` = 0.5124423901432269 (A1, A19); a 2-row range gives 0 (A2). The
  existing unit test `chisq_test_over_whole_columns_counts_every_cell` is therefore right as it
  is (it uses a statistic of 1, so the new probe adds a discriminating one).
- A pair with a blank, text or boolean cell on either side is dropped from the statistic but the
  degrees of freedom still come from the full range shape (A1, A3, A4, A6:A11, A13:A15). A18 shows
  the df of different shapes comes from the first (1-D) array.
- `CHISQ.TEST(A:B, C:D)` with C:D empty is `#DIV/0!` (A24).
- Seven cells are 1 to 4 ulp off (the engine matches on all 24 cells within that) (A6:A8, A10:A11, A17:A18): follow-up T-4864.
Eval delta: probes +24 cells, 17 pass, 7 numeric-ulp (owned by T-4864); no existing case changed.
