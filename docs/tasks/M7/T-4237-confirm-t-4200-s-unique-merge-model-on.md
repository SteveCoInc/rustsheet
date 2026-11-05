---
id: T-4237
title: "Confirm T-4200's UNIQUE merge model on a probe made after its freeze: 6-text sets mixing the A3-A5 cycle with A6-A8, and two-column rows where both columns cycle"
milestone: M7
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: [T-4200]
---

## Goal
Fresh Excel evidence, made after T-4200 froze its UNIQUE grouping model, that the model predicts: a pass confirms it,
a miss reopens it.

## Context
- Follow-up of T-4200 (excel-fidelity review, MINOR). T-4200 chose `coalesce_equal_lines`
  (crates/rustsheet-engine/src/functions/util/pairwise_order.rs) from about eleven simulated sort and grouping
  variants scored on the same 348 cells of probes/t4200-unique-cycle-permutations that it now passes. No probe
  was made after the freeze (T-3418's SORT model had one: t3418-sort-cycle-permutations). docs/excel-semantics/
  coercion.md "Not a total order", "UNIQUE's rule" has the model.
- Probe (excel-probe skill, excel-graph, tag M7), predictions written into the task before the fill:
  - UNIQUE and exactly_once of a sample of orders (e.g. 60 seeded permutations) of 6-text sets mixing the
    {A3,A4,A5} cycle with A6, A7 and A8 of the t3211 Data sheet, read as t4200 reads them;
  - two-column rows where both columns hold a cycle, in several orders;
  - a number or a boolean column beside a cycling text column (the cross-type order the model takes from SORT,
    unprobed).
- Compute the engine's predictions before the oracle fill (`rustsheet-eval show` of the unfilled case, or a
  release build on the case JSON) and record them, so the score is a true hold-out.

## Acceptance criteria
- [x] The probe case is filled on excel-graph and its predictions were recorded before the fill.
- [x] Every cell passes, or the misses are recorded in coercion.md with an owner task.
- [x] `make check` green; reviewers approve.

## Out of scope
- GROUPBY and PIVOTBY (T-4088).

## Outcome
- probes/t4237-unique-cycle-fresh (tag M7, 120 rows: UNIQUE in A, exactly_once in B, the input order in C) was
  written by docs/research/t4237/gen_probe.py and committed with the engine's values
  (docs/research/t4237/predictions.tsv) before the excel-graph fill. Groups: 60 seeded orders of the six texts
  A3..A8; 36 two-column rows with an order of the {A3,A4,A5} cycle in both columns; 24 rows of a number, boolean or
  mixed-type first column beside an order of the cycle.
- Every one of the 240 cells matches Excel, so T-4200's merge model holds out of sample. coercion.md "UNIQUE's
  rule" records the confirmation. Which line represents a merged group is still not separated (not this probe's
  question).
- Eval delta: probes new case 1, 248 of 248 passing; fixed 0, regressions 0. No engine change.
