---
id: T-4132
title: "gen-pivot generator v1: covering arrays over pivot features and data edge pools"
milestone: M16
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4122, T-4125]
---

## Goal
`rustsheet-gen --milestone M16` produces seeded, reproducible pivot cases: pairwise coverage (3-wise for layout x subtotals x Values position x filters x show-as) over the catalog dimensions of the current stage, data from the edge pools (types, blanks, errors, case variants, dates with times, ties, many items), one pivot per sheet and many sheets per workbook.

## Context
docs/research/pivot-tables-2026-11-04/plan.md §6 layer 2. docs/architecture/synthetic-generator.md describes the existing generator. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Generator and suite `gen-pivot` committed; a first batch filled by the oracle (>= 5,000 scored cells).
- [x] Cases tag the catalog ids they cover.
- [x] `make check` green; reviewers approve. (Local gates green, see the Outcome; CI and the review run on the PR.)

## Out of scope
- PV2+ dimensions (added as their stages open).

## Outcome
- Generator: `rustsheet-gen --milestone M16` (or `--policy pivots`), tools/src/rustsheet_tools/gen/pivot_suite.py over a new
  seeded t-wise covering-array builder (gen/covering.py; the repository had no covering-array machinery: arg_pairs.py
  fills report-listed unseen pairs, so it is not reusable here) and per-pivot source blocks from the edge pools
  (gen/pivot_data.py, reusing gen/pools.py's errors, date serials and times). 12 factors; 3-wise over layout x subtotals
  x Values position x filters, pairwise over all; seed 0 needs 104 rows for its 2,120 required combinations (the 3-wise
  floor is 100). Show-values-as and the other PV2+ dimensions are deferred (`DEFERRED`); the page-field and hidden-item
  catalog rows the filter dimension uses are PV2 in the catalog though the plan lists them in PV1. Tests:
  tools/tests/test_gen_pivot_suite.py (coverage, determinism by seed, every case built by the v2 builder, catalog tags).
- Suite eval/suites/gen-pivot (gated, `milestone = ["M16"]`): 9 cases, 104 sheets, 104 pivots (one per sheet, up to 12
  per case, plus a Q sheet of COUNTA controls), tagging 134 catalog ids. Oracle: excel-graph filled all 9 at the first
  try, no refusal. 25,941 scored cells (25,752 pivot-area cells), 22,816 pass (87.95%); 11,378 displayed-text items,
  8,548 pass. 12 pivots pass whole. Baselined by `make eval-baseline`.
- Failures, one owner task each (lane engine, decision 0245 owner lines): T-4914 header labels of the Values field and
  column fields in outline/tabular; T-4915 pivotField@name and item@n captions; T-4916 custom subtotal labels under
  Values on rows; T-4917 date-time items rounded to the second; T-4918 cells whose records hold no value the aggregate
  reads are blank; T-4919 classic layout with no row field; T-4920 record filters on a refreshed date field; T-4921 a
  page field with one visible item; T-4922 missingCaption `-` shows 0; T-4924 no row-label column without a row field;
  T-4925 one custom subtotal on the item's line; T-4926 custom x custom subtotal intersections blank; T-4927 a collapsed
  item with Values on rows. Date items in the source format: T-4807 (amended with the gen-pivot cells). The rules are in
  docs/excel-semantics/pivots.md "What the gen-pivot batch found (T-4132)". The follow-up to grow the batch is the
  existing T-4160 (its 20,000-cell target is met by this batch alone; its filing of failures is done here).
