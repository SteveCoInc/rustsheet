---
id: T-4200
title: "UNIQUE groups texts that hold a comparison cycle by adjacency in SORT's order, not by equality with a kept row (t3211 G4:I4)"
milestone: M7
lane: engine
status: done
priority: P1
size: S
tier: opus
depends_on: [T-3418]
---


## Goal
UNIQUE (and its `exactly_once` count) gives Excel's rows on texts whose comparison is not transitive, as in
probes/t3211-collation-sort-cycles Calc!G4, H4, I4: `UNIQUE(VSTACK(a̧, a͂, a͂͂))` (Data!A5, A3, A4) keeps three rows,
where ours keeps two because a̧ = a͂͂ pairwise.

## Context
- Split from T-3418, which fitted SORT's order (`functions::util::sort_indices_pairwise`, a natural merge of
  non-descending runs) and owned these three cells.
- Candidate model, checked in T-3418 by a Python simulation over the probed comparison matrix (Calc!L1:S8): sort
  the lines with `sort_indices_pairwise`, group neighbours in that order that compare equal, keep each group's
  first line, output the kept lines in input order; `exactly_once` keeps the one-line groups. It reproduces every
  one-column UNIQUE cell of t3211 whose texts are Data!A1:A8 (G1:G5, G14, G15, H1:H5, H14, H15, I1:I4, I14, I15).
  In G4 the sorted order is a̧, a͂, a͂͂: no neighbours are equal, so three rows.
- Ours (`unique_of`, crates/rustsheet-engine/src/functions/spill_functions.rs) groups by hash bucket, then by
  pairwise equality with the first kept line of the bucket. A change must keep the hash fast path for totals
  (numbers, booleans, texts without cycles) and the work charge (`spend_work`), and must say how multi-column
  lines compare (probe it: two-column rows where one column holds the cycle).
- Probe first (excel-probe skill): larger sets (every order of {A3, A4, A6, A7} and {A4..A8}, as
  probes/t3418-sort-cycle-permutations does for SORT) to confirm the adjacency model before implementing it.

## Acceptance criteria
- [x] probes/t3211-collation-sort-cycles Calc!G4, H4, I4 pass, with a new probe case backing the model.
- [x] docs/excel-semantics/coercion.md "Not a total order" states UNIQUE's rule with the evidence.
- [x] No regressions; `make check` green; reviewers approve.

## Out of scope
- SORT and SORTBY (T-3418), GROUPBY and PIVOTBY (T-4088).

## Outcome
- Probe `probes/t4200-unique-cycle-permutations` (excel-graph, canary-verified, 2026-11-04, tag M7): UNIQUE and
  exactly_once of every order of {A3,A4,A6,A7} (24), {A4..A8} (120) and {A3,A4,A5} (6) of the t3211 Data sheet, and
  24 two-column rows where one column holds the {A3,A4,A5} cycle (348 formula cells).
- The candidate model (group the neighbours of SORT's order) fits the 4- and 5-text sets but misses rows 146 and
  150 ({A3;A5;A4}, {A5;A4;A3}). The rule that fits all 348 cells: SORT's natural merge, where two groups `=` finds
  equal become one as soon as they meet (neighbours in a run, heads of two runs, or the last group out and the next
  one), each compared through the line that started it; lines compare column by column. A Python simulation
  separated eleven sort variants (bottom-up, top-down, insertion, descending, ties to the right) before the engine
  change; none but this one fits.
- Engine: `coalesce_equal_lines` (functions/util.rs, beside `sort_indices_pairwise`) implements it with a
  union-find, O(n log n) comparisons; `unique_of` uses it when its data holds a non-ASCII text, comparing cells with
  `unique_cell_cmp` (texts by `cmp_text_pairwise`, others equal exactly when their `unique_cell_key`s are), each
  comparison charged to the work budget. All-ASCII data keeps the hash buckets (`first_equal_lines`), which give
  the same groups on a transitive comparison. GROUPBY/PIVOTBY are unchanged.
- File size: both touched files would pass 1000 lines, so `sort_indices_pairwise` and `coalesce_equal_lines` moved
  to functions/util/pairwise_order.rs and UNIQUE's grouping (`first_equal_lines`, `unique_of`, `unique_cell_key`,
  `unique_cell_cmp`) to functions/spill_functions/unique.rs, re-exported at their old paths. Duplicated lines
  2,880 -> 2,878 (`check_duplication.py --update`).
- coercion.md "Not a total order" states UNIQUE's rule with the cells; the ours note is updated.
- Eval delta: probes/t3211-collation-sort-cycles Calc!G4, H4, I4 fixed (3 known failures fewer); the new case
  passes 356/356 checked cells. No regressions.
- Not separated by the probe: which line represents a merged group (first, smallest or last all fit); with
  numbers or booleans beside a cycling text column the order across types is SORT's, unprobed.
