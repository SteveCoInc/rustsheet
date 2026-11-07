---
id: T-4738
title: "A lifted INDEX of nested GROUPBY cells gives each cell's top-left (INDEX(G,{2;3},2) is {3;1})"
milestone: M7
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A lifted INDEX that selects nested GROUPBY cells gives each selected cell's top-left value, as Excel does:
`INDEX(G,{2;3},2)` is `{3;1}`, and SUM over it is 4.

## Context
Found by T-2609 (docs/excel-semantics/arrays.md "Operators over nested cells, and Excel's change of 2026-11"; SPEC-0018
rule 4, revised by T-4661). Ours gives `#VALUE!` for each element. Excel's top-left here differs from the error that an
element-wise lift reads in November (T-4737), so the two reads must stay separate.

Cells this task fixes (probes): t2609-nested-operators-and-lifts P!A56:A59. No October fill covers them, so nothing is
superseded.

## Acceptance criteria
- [ ] P!A56:A59 pass, and no other cell regresses.
- [ ] The lifted INDEX path (functions/lookup_and_reference INDEX with array row/column numbers) cites A56:A59 where it
      reads the top-left.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A scalar INDEX that selects one nested cell: it keeps returning the cell's array (SPEC-0018 rule 3).
