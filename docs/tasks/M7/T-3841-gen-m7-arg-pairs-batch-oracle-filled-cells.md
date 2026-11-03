---
id: T-3841
title: "gen-m7 arg-pairs batch: oracle-filled cells for the unseen argument-class pairs of PIVOTBY"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
gen-m7 gains an oracle-filled batch from `rustsheet-gen --policy arg-pairs --functions PIVOTBY --spill` that raises PIVOTBY's
pairwise argument coverage, and every Excel cell the engine fails is owned by a task.

## Context
- Split from T-3778, which ran the GROUPBY half (pairwise share 0.11 to 1.0; failures owned by T-3837..T-3840). T-3778's
  Context (the dry run, the layout, why opus) applies unchanged; PIVOTBY had pairwise share 0.088 with 4964 unseen pairs on
  main at the start of T-3778.
- Many PIVOTBY cells will fail on the same rules as GROUPBY (scalar inputs, option order, scalar filter, lifted options:
  docs/excel-semantics/arrays.md "GROUPBY scalar inputs, option order and scalar filters in the gen-m7 arg-pairs batch").
  Check whether the GROUPBY owners' rules carry over (one probe row each) and give those cells to T-3837..T-3840 when they do;
  file new tasks otherwise.
- Steps: `make eval`; `cd tools && uv run rustsheet-gen --policy arg-pairs --milestone M7 --functions PIVOTBY --spill`;
  `uv run rustsheet-oracle fill -b excel-graph --suite gen-m7`; `make eval`, `make eval-baseline`. Assign failing cells to
  owners by anchor row (each call's cells sit below its column-A anchor).

## Acceptance criteria
- [ ] New gen-m7 cases from `--policy arg-pairs --functions PIVOTBY --spill` are committed with their excel-graph sidecars and 0
      fill failures.
- [ ] After `make eval`, PIVOTBY has a `pairwise` share of at least 0.9, or the PR lists the pairs left unseen and why.
- [ ] No scored cell is a layout `#SPILL!`.
- [ ] Failing cells are known failures in eval/baseline/gen-m7.txt, each owned by an open task; `make exit-owners MILESTONE=M7`
      and `make unowned-off ALL=1` report no new unowned cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the cells the batch finds (the owning tasks make them).
