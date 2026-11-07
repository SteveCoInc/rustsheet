---
id: T-4932
title: "Probe unprobed element-wise lifts (N, T, ISBLANK, ISREF, ISEVEN, IFERROR) over nested GROUPBY/PIVOTBY cells"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for how N, T, ISBLANK, ISREF, ISEVEN and IFERROR's value argument, lifted over a GROUPBY or PIVOTBY
result whose function returns its array, read each nested cell. Today the engine reads every nested cell as `#VALUE!`
under every lift (T-4737, `ArrayNode::scalar` in nested_array.rs), generalized from thirteen probed lifts.

## Context
T-4737 probed ERROR.TYPE, LEN, ABS, UPPER, ROUND, NOT, ISLOGICAL and others (probes/t4737-nested-cell-lift-error-type,
excel-graph 2026-11-07); docs/excel-semantics/arrays.md says the unprobed lifts follow by generalization. Its reviewers
(PR #3863) asked for a probe of the lifts whose handling of non-error input differs most: N and T (which do not error on
text or numbers), ISBLANK and ISREF (type tests), ISEVEN (a ToolPak number reader) and IFERROR (which would catch the
`#VALUE!`). Use the `excel-probe` skill; reuse the GROUPBY shapes of the t4737 probe (one-level, two-level, a
reference body, PIVOTBY).

## Acceptance criteria
- [ ] A new probe case under eval/suites/probes with excel-graph values for each of the six lifts over each shape.
- [ ] docs/excel-semantics/arrays.md states the result; any cell that differs from the engine gets an owner task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (a follow-up task does it if a lift differs).
