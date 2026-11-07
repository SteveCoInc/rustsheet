---
id: T-4006
title: "rustsheet-gen --policy arg-pairs for GETPIVOTDATA: build the case pivot so its unseen argument-class pairs get oracle-filled gen-m6 cells"
milestone: M6
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`rustsheet-gen --policy arg-pairs` can target GETPIVOTDATA, and gen-m6 gains an oracle-filled batch that raises
its pairwise argument coverage from 0.05 to at least 0.9, with every Excel cell the engine fails owned by a task.
GETPIVOTDATA is the only M2-M7 function outside the volatile and desktop-only ones (RANDBETWEEN, RANDARRAY,
FILTERXML) whose pairwise share is below 0.3.

## Context
- Gap triage 2026-11-04 (report of main cd41e050; report.json `argument_coverage[].pairwise` and `unseen_pairs`;
  decision 0031 evidence gaps). Every failing Excel cell of the eval has an open owner (`make unowned-off ALL=1`
  lists only cells T-3978 owns and cells owned by deferred tasks), and GETPIVOTDATA is above the evidence floor
  (240 gen-m6 cells from T-2674), but 378 of its argument-class pairs have no Excel cell: pairwise 0.05, the lowest
  of any in-scope function. Catalog `prior` 0.3, milestone M6, so it ranks before every M9 batch.
- First unseen pairs: data_field/pivot_table = (zero, range), (negative, range), then data_field as numeric text,
  empty text, boolean, error or blank against the pivot reference, and field/item pairs of the same classes.
- Why the batch policy skips it today: tools/src/rustsheet_tools/gen/arg_pairs.py drops `pivot: true` functions
  ("GETPIVOTDATA's arguments are names from a pivot"), so no case gets the `Pivot` sheet that
  tools/src/rustsheet_tools/gen/pivots.py builds for the `--focus` path (T-2396). T-3496 (the M6 lookup batch, done)
  left it out for that reason. The change: in arg-pairs mode, build the case's pivot source once (as `Gen.call`
  does for `pivot: true`), force `pivot_table`'s `range` class to a cell or range of that pivot and its other classes
  to the forced forms, and draw the parameter of the pair that is not forced from `pivot_call`'s valid names, so
  each call isolates the forced pair against an otherwise valid query.
- Constraints already recorded: Excel for the web refuses a workbook with an odd GETPIVOTDATA argument count of 5
  or more (docs/excel-semantics/syntax.md, T-2403), so a forced field must come with an item; arrays of items spill
  (pivots.py leaves them out). The `#REF!` paths Excel showed are in docs/excel-semantics/web.md ("GETPIVOTDATA
  against a pivot table").
- Why opus: the generator change has to keep each call a valid pivot query apart from the forced pair, and the
  batch's failing cells (a number or boolean data_field, an error field, a blank item) will be new coercion and
  error-order rules of a function whose code is in `functions/lookup_and_reference/`; sorting them into owners and
  recording each rule takes judgment.
- Steps: run `make eval` so the policy reads current unseen pairs; generate with
  `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M6 --functions GETPIVOTDATA`; fill with
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m6`; score. Own each failing cell in the same PR: an
  existing open task that covers the rule, or a new task via `make new-task` in `functions-lookup`. A wrong type,
  error or text gets `campaign: semantics` and a `## Visible-parity stage` section; name the case file stem and the
  A1 cells in one bullet (decision 0159). Record each new Excel quirk in docs/excel-semantics/web.md beside the
  GETPIVOTDATA entry, citing the cells.

## Acceptance criteria
- [x] `rustsheet-gen --policy arg-pairs --functions GETPIVOTDATA` writes cases with a `Pivot` sheet and one call per
      forced pair; a tools test covers a forced `range` pair and a forced non-range pair (`uv run pytest` green,
      ruff and mypy strict clean).
- [x] The new gen-m6 cases are committed with their excel-graph oracle sidecars and 0 fill failures (a workbook
      Excel refuses is a generator bug to fix, not a case to drop).
- [x] After `make eval`, GETPIVOTDATA has a `pairwise` share of at least 0.9 in report.json `argument_coverage`, or
      the PR lists the pairs left unseen and why (a hard domain, a shape Excel refuses to load).
- [x] Failing cells are known failures in eval/baseline/gen-m6.txt (`make eval-baseline`, 0 regressions), each owned
      by an open task, and `make exit-owners MILESTONE=M6` and `make unowned-off ALL=1` report no new unowned cell
      from the batch; the gen-m6 exit row (no visibly-off cell beyond its allowance) stays met or the PR names the
      owning tasks.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the cells this batch finds (the owning functions-lookup tasks make them).
- New pivot shapes (several data fields, filters, dates, other layouts) beyond what pivots.py builds today.
- The other pivot-reading paths (PIVOTBY, GROUPBY: their M7 batches are done) and other milestones.

## Outcome
- `rustsheet-gen --policy arg-pairs` now targets GETPIVOTDATA (tools/src/rustsheet_tools/gen/arg_pairs.py,
  `ArgPairs.pivot_call`). Each case gets the function's own `Pivot` sheet (built once from its own seeded stream by
  gen/pivots.py). Arguments the pair does not force are valid names from that pivot. A forced `range` pivot
  reference is the pivot's top-left cell or the whole pivot (`corner_ref`/`whole_ref`, now shared with
  `pivot_call`). The field/item pair always comes whole.
- Each pivot case opens with a valid control query at Calc!A1. Nearly every forced pair is a miss, so an all-error
  batch could not otherwise tell a miss from a pivot Excel never built. Excel answers all four controls with 6.75.
- Tests: `test_arg_pairs_getpivotdata_queries_the_case_pivot` covers a forced `range` pair (data_field/pivot_table)
  and a forced non-range pair (field/item), plus the control. Design doc §6.7 is updated.
- Batch: gen-m6 16b67941f34e0346, ccc550562ee5fd06, 37eb2ebaf9386ade and 2963992505456e99 (seed 4006, 721 calls
  over the 360 unseen pairs of the 2026-11-07 report). All are excel-graph filled with 0 fill failures.
- Coverage: GETPIVOTDATA's pairwise share goes from 0.09 to 0.997 on gen-m6 alone and single from 0.47 to 1.0. The
  one pair gen-m6 lacks (field=number/item=number) was already seen in another suite, so it was not generated.
- Eval delta: gen-m6 +4 cases, 764 checked cells, 65 new known failures (`make eval-baseline`, 0 regressions;
  visible-parity lock +65 on the 4 cases). They are all wrong errors under two new Excel rules, recorded in
  docs/excel-semantics/web.md "GETPIVOTDATA error arguments (T-4006)" and owned by T-4823 (functions-lookup,
  sonnet):
  - an error in `field` is the result (43 cells);
  - a referenced error as `data_field` is `#REF!`, a typed one the result (22 cells).
  `make exit-owners MILESTONE=M6` reports 0 unowned, and `make unowned-off ALL=1` lists no gen-m6 cell.
