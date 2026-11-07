---
id: T-5036
title: "Probe a linked workbook's structured reference when the link part caches the table's cells (T-4144 follow-up)"
milestone: M11
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/references.md says what Excel returns for a structured reference into a linked workbook's
table, `[1]!Table3[[#This Row],[Col]]` and `[1]!Table3[#Data]`, when the link part caches that sheet's cells.

## Context
- references.md "A structured reference into a linked workbook is `#REF!`" is VERIFIED only for a link part that
  caches no cells (corpus-bench 58994 F8). corpus-bench 33094 Sheet1!B5,
  `VLOOKUP([1]!Table3[[#This Row],[Resource Name]],[1]Data!E:J,6,0)`, has a link part caching the Data sheet's
  cells, a cached value of 20, and no oracle value; the engine gives `#REF!` (T-4144).
- T-4144's excel-fidelity review (PR #4016, MINOR) asked for this probe. Use the `excel-probe` skill: a hand-built
  workbook with an externalLink part caching cells under a defined table range, recalculated by the oracle.

## Acceptance criteria
- [ ] A probe case gives Excel's value for both forms with a link part that caches the table's cells.
- [ ] references.md states the result with its evidence; if it is not `#REF!`, an engine task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (a follow-up task if the probe disagrees).
