---
id: T-2805
title: "SPEC-0024 step 3: charge the other whole-array readers a LAMBDA can repeat"
milestone: M7
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-2803, T-2804]
---

## Goal
Every function that reads a whole array argument charges the cells it reads to the formula work budget (SPEC-0024). No reader
is left that a LAMBDA can repeat over a LET-bound array without bound.

## Context
SPEC-0024 rollout step 3. Audit the whole-array readers and charge each the cells it reads through `Model::spend_work`. The
list to check: MATCH/XMATCH/XLOOKUP/LOOKUP/VLOOKUP/HLOOKUP linear search; SORT/SORTBY; UNIQUE; FILTER; TEXTJOIN/CONCAT;
TOCOL/TOROW; SUMPRODUCT; MMULT inputs; the *IF(S) over arrays; INDEX over arrays (O(1), so no charge; record that). A reader also charges the text elements it reads by their length (⌈bytes / `TEXT_BYTES_PER_UNIT`⌉, SPEC-0024 section 1). Prefer a
shared helper at the point where an array argument is read whole. Do not add a per-function copy (directive 3). List the
covered readers in the SPEC-0024 DEVIATION entry in arrays.md.

## Acceptance criteria
- [x] For each charged reader, `LET(s,SEQUENCE(100000),REDUCE(0,SEQUENCE(100000),LAMBDA(a,b,a+<reader over s>)))` is
      `#VALUE!` within the harness timeout (engine tests, counters, no timing).
- [x] The DEVIATION entry lists the covered readers.
- [x] No baseline gains a line.
- [x] `make check` green; reviewers approve.

## Out of scope
- The aggregate walker (T-2804).

## Outcome
The audit found one gap shared by every uncharged reader. A read of a LET variable or LAMBDA parameter copies the bound
value (`Binding::value` clones the array). `evaluate_node_in_context` charged that copy (`Model::charged`), but the
reference-context reads did not: `evaluate_node_with_reference`'s variable arm and the fallthrough of
`variable_reference`. XMATCH, XLOOKUP, LOOKUP, the *IF(S) family, TEXTJOIN, CONCAT, LARGE, SMALL, PERCENTILE,
QUARTILE, PERCENTRANK, NPV and AGGREGATE read through those two paths, so a LAMBDA could repeat them over a LET-bound
array without bound. MATCH, VLOOKUP, HLOOKUP, SORT, SORTBY, UNIQUE, FILTER, TOCOL, TOROW, SUMPRODUCT and MMULT already
paid in value context. Both reference-context sites now charge the copied value through `Model::charged`, the same
definition as value context, so there is no per-function charge (directive 3).

INDEX, ROWS and COLUMNS are charged too, unlike the task's "INDEX: no charge". Their own work is O(1), but the variable
read under them copies the whole array, so an uncharged INDEX(s,i) in a LAMBDA was just as unbounded.

Text elements of arrays are not charged by length at a read: their bytes are shared (T-2806). Charging them would refuse
`IF(SEQUENCE(1024,1024),REPT("a",32767))`. Readers that compare text bytes are follow-up T-4233.

DEVIATION entry: arrays.md "One formula evaluation does at most 268,435,456 work units" lists the audited readers.

Tests (test_formula_work.rs):
- `every_whole_array_reader_pays_the_cells_it_reads` checks 32 readers.
- `a_lambda_repeating_any_whole_array_reader_is_bounded` runs the acceptance formula for each reader under a 2^24
  limit, to keep the debug suite fast. At the default limit the same charge stops it after about 900 calls; one
  full-size run takes 8 s in a debug build.

The three GROUPBY/PIVOTBY visit-cap tests now lift the formula and pass budgets (`recalc_limits`), as the first of
them already lifted the formula budget. Each call's copy of its group is now charged, so the 2^30 pass pool ran out
before the visit cap they test.

Eval: 0 regressions and no baseline change (`make check-fast` eval gate).

