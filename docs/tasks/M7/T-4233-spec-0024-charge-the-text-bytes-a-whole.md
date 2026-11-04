---
id: T-4233
title: "SPEC-0024: charge the text bytes a whole-array reader compares (lookups, MATCH, UNIQUE, SORT, the *IF(S)) over a bound array of long texts"
milestone: M7
lane: engine
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A LAMBDA that repeats a text-comparing reader over a LET-bound array of long texts stays within the formula work budget:
the bytes the reader compares are charged, not only the cells it copies (SPEC-0024 section 1, "text values").

## Context
T-2805 charges each read of a bound array by its cells (`evaluate_node_with_reference`, `variable_reference`), but
not by the bytes of its text elements. Those bytes are shared since T-2806, so the copy is cheap, and charging every read
by length would refuse `IF(SEQUENCE(1024,1024),REPT("a",32767))`, which computes today. A reader that compares texts
byte by byte does more work than the copy, though. For example, in
`LET(s,IF(SEQUENCE(1E5),REPT("a",32767)),MAP(SEQUENCE(900),LAMBDA(i,MATCH(REPT("a",32766)&"b",s,0))))`, each call compares
up to 32 KB per element, which is about 3 GB a call within the cell charge. Find the comparing readers (MATCH/XMATCH/XLOOKUP/LOOKUP and
the V/HLOOKUP searches, UNIQUE, SORT/SORTBY, the *IF(S) criteria matching, GROUPBY keys) and charge the bytes they
compare, at a shared comparison chokepoint if one exists (directive 3). See docs/excel-semantics/arrays.md "One formula
evaluation does at most 268,435,456 work units".

## Acceptance criteria
- [ ] The MATCH formula above, and one formula per other comparing reader, is `#VALUE!` (`TOO_MUCH_WORK`) under the
      default limit (engine tests that count units, not time).
- [ ] No baseline gains a line; the DEVIATION entry lists the text-byte charge.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cell charges for array reads (T-2805, done).

## Outcome
- One chokepoint: `functions/util/collation.rs` counts what each text comparison reads (`counted` wraps the passes'
  iterators, so a comparison pays up to its first difference, not the texts' lengths) and what each collation key or
  lookup fold is built from (`note_compared_bytes`); the ASCII arms that bypass it (`ExactKey::text_is` of MATCH,
  VLOOKUP and HLOOKUP's exact match, `folded_units`, `WildcardPattern::matches`) note theirs too. `Model::spend_compared`
  (area_budget.rs) charges the count after each node (`Model::charged`, which `evaluate_node_in_context` ends with), one unit per 4 characters, and
  `with_area_budget` keeps an outer run's pending count apart from a dependency cell's run.
- Tests (test_formula_work.rs, units not time): `every_text_comparing_reader_pays_the_bytes_it_compares` (MATCH exact,
  wildcard and approximate, XMATCH, XLOOKUP, LOOKUP, VLOOKUP, HLOOKUP, UNIQUE, SORT, SORTBY, GROUPBY, `=`),
  `a_comparison_pays_for_what_it_reads_not_for_the_texts_lengths`, and
  `a_lambda_repeating_a_text_comparing_reader_is_bounded` (the task's MATCH shape and XMATCH over 100 texts under a
  2^24 limit; over the task's 1E5 texts the first MATCH call alone notes about 8.2E8 units, past 2^28, so the MAP is
  `#VALUE!` at the default limit).
- The *IF(S) functions are not readers of a bound array: they take ranges (`COUNTIF(s,..)` over a LET-bound array is an
  error before any comparison), and a criterion over 255 characters is `#VALUE!` first; their comparisons go through
  the same count.
- arrays.md "One formula evaluation does at most 268,435,456 work units" (DEVIATION) lists the charge.
- Eval delta: none (no baseline line changes; no eval formula past the limit).

## Landing notes (2026-11-04 23:10 UTC, end of the run's landing window)
Data for the run that resumes this branch; check it against the PR and the code.
- PR #3244 (draft), head 9d7a46638. CI green on that head (every job, eval gate included); `make check-fast` green
  locally (eval-gates: 0 regressions, no baseline change). Review round 1: security APPROVE (MINOR: the charge lands
  after the node, so one node's compares run before the budget stops it; bounded), design APPROVE (MINOR:
  `check_duplication.py` reports 2878 vs the 2880 baseline: refresh with `--update` in the next push),
  excel-fidelity REQUEST_CHANGES.
- Fidelity's blocking findings (round 1 of 3):
  1. The weight is uncalibrated: compared characters reuse `TEXT_BYTES_PER_UNIT` (4 per unit), so single formulas
     that compute on main become `#VALUE!`: `=ROWS(SORT(REPT("a",80)&SEQUENCE(1048576)))` (main: 1048576 in 4.9 s),
     the same with 100 and 300 a's (6.1 s, 10 s), `ROWS(SORTBY(SEQUENCE(1048576),REPT("a",300)&SEQUENCE(1048576)))`,
     `ROWS(SORT(REPT("a",3000)&SEQUENCE(100000)))` (6.7 s). Fine: 60-char prefix x 1M, 100-char x 300k.
     Fix: a constant of its own (compared characters per unit) fitted per SPEC-0024 section 4 (worst case to the
     limit under 10 s on a release build; heaviest eval formula's units recorded). Rough figure from the timings
     above: about 0.6-0.7 G compared chars/s, so 16-24 chars a unit puts the all-compare worst case near 7-10 s;
     time it (a MAP of `XMATCH(REPT("a",32766)&"b",s)` over a bound 32,767-byte array until `#VALUE!`), then add an
     engine test that a 1M-row SORT with a ~100-char shared prefix still computes (units, not time: assert its
     spent units stay under the limit).
  2. arrays.md's DEVIATION entry must say the charge applies to any comparison (single SORT/SORTBY/UNIQUE/lookups over
     ranges or arrays with long shared prefixes, not only bound arrays), and record the calibration figures.
  - MINOR: reset the count at the top-level run start (`take_compared_bytes()` when no run encloses), with a comment.
- The reviewer's scratch probe (one-cell release evaluator, formula as its argument) was in this run's scratchpad and
  will not survive; rebuild one with a path dependency on crates/rustsheet-engine.
- Next step: add the constant, time it, adjust tests/docs, push once, re-run all three reviewers (round 2), then mark
  #3244 ready and enqueue.

