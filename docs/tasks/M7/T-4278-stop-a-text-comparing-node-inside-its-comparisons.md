---
id: T-4278
title: "Stop a text-comparing node inside its comparisons once the formula work budget runs out (T-4233 follow-up)"
milestone: M7
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A single node whose text comparisons alone would spend past the formula work budget stops at the budget, instead of
running to its end and only then being `#VALUE!`.

## Context
T-4233 charges the ticks a comparison reads (`functions/util/collation/compared.rs`) after each node
(`Model::spend_compared`, area_budget.rs). One node can still compare far more than the limit before the charge lands:
`=ROWS(UNIQUE(REPT("a",100)&SEQUENCE(1048576)))` runs about 30 s, and a UNIQUE of 1,000 texts of 16,383 `é` about 60 s,
before they are `#VALUE!` (release build, 2026-11-05; docs/excel-semantics/arrays.md "One formula evaluation does at
most 268,435,456 work units"). Find a way for the long-running loops (UNIQUE's keys and buckets, SORT's comparator, the
array `=`, the lookups' scans) to see that the pending ticks already exceed what the run has left and stop early, for
example a check in the readers' loops against a remaining-work snapshot, without a panic or unwinding (directive 5).

## Acceptance criteria
- [x] The two formulas above are `#VALUE!` (`TOO_MUCH_WORK`) after at most about the limit's worth of ticks (an engine
      test that counts ticks or units, not time).
- [x] No eval formula changes value; `make check` green; reviewers approve.

## Out of scope
- Making the comparisons faster (T-4279).

## Outcome
- `functions/util/collation/compared.rs`: a thread-local allowance, the ticks the run can still pay for at its next
  charge, set by `Model::spend_compared` after each node (work left x `COMPARED_TICKS_PER_UNIT`, 0 once exceeded) and
  by `Model::with_area_budget` for each run, which puts the interrupted run's back. `out_of_ticks` is true once the
  pending ticks pass it, inside `compared::stoppable` only (UNIQUE's grouping, SORT's and SORTBY's sort):
  `cmp_text_pairwise` then returns `Equal` without reading and `collation_key` the empty key. Readers that build a
  pass-shared index (XMATCH/XLOOKUP's exact-match index, the *IFS criteria index) never stop: review round 1 found
  that their stand-in keys reached the next formulas of the pass (`=XMATCH(A200,A1:A200)` #N/A in a neighbour cell);
  `a_formula_past_the_limit_leaves_no_stand_in_in_a_shared_index` covers it. Pending ticks past the allowance make the next charge fail, and a cut-short mark also
  forces the run's `TOO_MUCH_WORK` in `with_area_budget`, so a value computed from those stand-ins never shows. The
  total order (`cmp_text`) is not cut: GROUPBY sorts with `slice::sort_by`, which may panic on a comparator that is
  not an order. SORT/SORTBY use `sort_indices_pairwise`, which takes any comparator.
- Test: `a_reader_stops_reading_once_its_ticks_pass_what_the_run_has_left` (test_formula_work.rs) counts the ticks
  read (a test-only counter) under a 2^20-unit limit. UNIQUE of 65,536 105-byte texts, UNIQUE of 200 8,003-byte `é`
  texts and SORT of 65,536 texts are each `TOO_MUCH_WORK` within twice the allowance plus one key or comparison.
  Unstopped, they read 100M to 440M ticks; the test fails with the stop disabled.
- Full size, release build: `=ROWS(UNIQUE(REPT("a",100)&SEQUENCE(1048576)))` is `#VALUE!` in 5.3 s (about 30 s
  before), and a UNIQUE of 1,000 texts of 16,383 `é` in 1.7 s (about 60 s). arrays.md is updated.
- Eval: no change (`make eval-check`: 0 fixed, 0 regressions in every suite).
