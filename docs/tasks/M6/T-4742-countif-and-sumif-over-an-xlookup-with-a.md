---
id: T-4742
title: "COUNTIF and SUMIF over an XLOOKUP with a text if_not_found spill a 2x1 result (probes/t4704 B64:C64, A65, B67:C67, A68, B70:C70, A71, B73:C73, A74)"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: [T-4704]
---

## Goal
`COUNTIF(XLOOKUP({2;9},A1:A3,B1:B3,"x"),">0")` and the SUMIF forms, with a miss or with every lookup a hit, spill a 2x1 result of `#VALUE!` elements (the range argument is a value array, not a range), not a scalar `#VALUE!`.

## Context
- Evidence: probes/t4704-mixed-xlookup-grid-readers (excel-graph, 2026-11-07, T-4704): probes 22-25 (Calc A64:C65 COUNTIF with a miss, A67:C68 COUNTIF all hits, A70:C71 SUMIF with a miss, A73:C74 SUMIF all hits): ROWS 2, COLUMNS 1, both elements `#VALUE!`. Ours gives a scalar `#VALUE!` (B64:C64, B67:C67, B70:C70, B73:C73; A65, A68, A71, A74 empty).
- This is the per-element-caller bullet of T-4704; t4569 A26 (`COUNTIF` over an XLOOKUP with a reference if_not_found) already spills.

## Acceptance criteria
- [x] Calc B64:C64, A65, B67:C67, A68, B70:C70, A71, B73:C73 and A74 of probes/t4704 pass.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other readers of a mixed XLOOKUP grid (T-4704).

## Outcome
A lifted XLOOKUP in a per-element caller's reference argument (COUNTIF, SUMIF, COUNTIFS, AVERAGEIF and the rest of `Function::per_reference_element`) is now always a grid of its elements, whatever its `if_not_found`: `Model::xlookup_reference_elements` (xlookup.rs, the `XlookupGrid::Elements` read) keeps the all-hits grid and wraps a value-`if_not_found` result in `RefGrid::of_values`; a value element is `#VALUE!` in the call, an error element its error. `lifts_into_reference_elements` and `criteria_and_grid_static_result` take any XLOOKUP, not only one with a reference `if_not_found`. New probe probes/t4742-countif-array-range-arg (excel-graph, 2026-11-07) covers the three-argument form, all misses, a 1x1 and a 1x2 lookup, a scalar lookup, SUMIF's sum_range, COUNTIFS, AVERAGEIF and an array criterion; COUNTIF over SEQUENCE, arithmetic and SORT arrays made the workbook fail to open on Excel for the web, so those probes were dropped (recorded in lookup.md). Unit test added; lookup.md and SPEC-0014 updated.

Eval delta: probes known failures 7603 -> 7594 (fixed 12: probes/t4704 Calc B64:C64, A65, B67:C67, A68, B70:C70, A71, B73:C73, A74); the new case passes 38 of 41 cells, regressions 0. Follow-up: T-4826 owns probes/t4742 A40, C40, A41 (an array criterion beside the grid broadcasts to 2x2).

## Landing notes
- PR #3890. Security, design and excel-fidelity approved the code head a91ff5c66. Their MINOR findings are recorded in the PR body, and follow-up T-4863 is filed in #3898.
- After the catch-up merge (ad8efcf2d), CI went red on `rust (test 3/3)`. The parser test `an_xlookup_grid_needs_a_reference_if_not_found` still asserted the old T-4705 rule. Commit 4bda9bd2f replaces it with `an_xlookup_grid_has_the_lookup_value_shape`, which asserts the probes/t4742 rule. The full engine lib suite passes locally (5701 passed). 7d182c75b merges the catch-up commit back in.
- Next step:
  1. Wait for CI to go green on the head after this commit.
  2. Re-run reviewer-design and reviewer-excel-fidelity on the test change. It is a test-only diff, but it changes what is asserted.
  3. Once both approve, call `enable_pr_auto_merge` (SQUASH).
- Do not restore the old assertion: Excel's evidence (t4742 A10:C22) contradicts it.
- One more fix in the task file: its Outcome says "38 of 41 cells"; the right count is 38 passing of 40 checked cells.
- Update, 09:2xZ: design and excel-fidelity re-approved 1cfcbd459 and the PR was enqueued. The queue then removed it with `MERGE_CONFLICT`: T-4741 (#3882) merged first and added a test at the same place in `test_xlookup_ref_element.rs`. `make catch-up` refused because the conflict is outside the baselines.
- The manual merge of main (the commit before this notes commit) keeps both tests (T-4741's first) and takes main's baseline lists, then re-records them with `make eval-baseline NEW_CASES=<t4742 case>`. `make eval-check` showed 0 regressions and the xlookup_ref_element tests pass (9).
- Next step: wait for CI to go green on the head after this commit. Have reviewer-design confirm the merge resolution: its only non-baseline change is concatenating the two tests. Then enqueue (SQUASH).
