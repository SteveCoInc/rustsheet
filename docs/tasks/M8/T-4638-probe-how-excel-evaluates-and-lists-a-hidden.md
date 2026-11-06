---
id: T-4638
title: "Probe how Excel evaluates and lists a hidden defined name loaded from an xlsx"
milestone: M8
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what Excel does with `<definedName hidden="1">` loaded from an xlsx: a
formula that uses the hidden name evaluates as with a visible one, and the name keeps its
hidden flag and `comment` when Excel saves the workbook. The finding is recorded in
`docs/excel-semantics/`.

## Context
T-3301 (PR #3669) keeps `hidden` and `comment` through import and export, assuming Excel still
evaluates a hidden name and only Name Manager hides it. The fidelity reviewer noted that no
oracle cell backs this, and that `docs/excel-semantics/` has no hidden-name entry. Nothing reads
the flag during evaluation, so the claim gates no code today. Use the `excel-probe` skill. The
spec→xlsx builder (tools/) may not write `hidden`/`comment` on a defined name yet; if so, add
that to the builder in this task.

## Acceptance criteria
- [ ] A probe case has a workbook-scoped and a sheet-scoped hidden name, one with a comment, used by formulas (plain, inside SUM, and via INDIRECT of the name). It is filled by the oracle.
- [ ] A `docs/excel-semantics/` entry cites the cells, VERIFIED or with the observed difference. A mismatch with RustSheet is filed as a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Name Manager UI behaviour, which the oracle cannot observe.
