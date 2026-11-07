---
id: T-4006
title: "rustsheet-gen --policy arg-pairs for GETPIVOTDATA: build the case pivot so its unseen argument-class pairs get oracle-filled gen-m6 cells"
milestone: M6
lane: harness
status: in-progress
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
- [ ] `rustsheet-gen --policy arg-pairs --functions GETPIVOTDATA` writes cases with a `Pivot` sheet and one call per
      forced pair; a tools test covers a forced `range` pair and a forced non-range pair (`uv run pytest` green,
      ruff and mypy strict clean).
- [ ] The new gen-m6 cases are committed with their excel-graph oracle sidecars and 0 fill failures (a workbook
      Excel refuses is a generator bug to fix, not a case to drop).
- [ ] After `make eval`, GETPIVOTDATA has a `pairwise` share of at least 0.9 in report.json `argument_coverage`, or
      the PR lists the pairs left unseen and why (a hard domain, a shape Excel refuses to load).
- [ ] Failing cells are known failures in eval/baseline/gen-m6.txt (`make eval-baseline`, 0 regressions), each owned
      by an open task, and `make exit-owners MILESTONE=M6` and `make unowned-off ALL=1` report no new unowned cell
      from the batch; the gen-m6 exit row (no visibly-off cell beyond its allowance) stays met or the PR names the
      owning tasks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the cells this batch finds (the owning functions-lookup tasks make them).
- New pivot shapes (several data fields, filters, dates, other layouts) beyond what pivots.py builds today.
- The other pivot-reading paths (PIVOTBY, GROUPBY: their M7 batches are done) and other milestones.
