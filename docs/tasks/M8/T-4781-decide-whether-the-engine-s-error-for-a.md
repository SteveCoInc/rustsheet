---
id: T-4781
title: "Decide whether the engine's #ERROR! for a name Excel refuses at load needs a DEVIATION entry"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Every form Excel refuses at load (createSession 501 `FileCorruptTryRepair`) that the engine loads instead, with
`#ERROR!` in the cell, is either covered by a `DEVIATION` entry under decision 0014's rules or recorded as a decided
non-deviation, so the policy is written once rather than implied per section.

## Context
- references.md "Characters of a name" (T-1387, T-1856, T-4446, T-4753): `LET(Ⓐ,1,Ⓐ+1)`, `=Ⓐ+1`, duplicate LET or
  LAMBDA names, a defined name starting with `Ⓐ` or a mark, an unquoted `Ⓐ!B1`, and `SUM(Ⓐ[h])` make Excel refuse the
  whole workbook; the engine loads it and gives `#ERROR!` (a parse error) in the cell. "Implemented in" records this,
  but no `DEVIATION` tag does (reviewer-excel-fidelity MINOR on T-4753, PR #3841).
- docs/excel-semantics/README.md (decision 0014) says when a `DEVIATION` entry is needed; check whether refusing a
  workbook counts as behavior the engine must match (it could fail the import instead) or as out of scope for a
  headless engine, and grep for other refused-at-load forms (syntax.md, xlsx-storage.md).

## Acceptance criteria
- [ ] One place states the rule for Excel-refused workbooks (a `DEVIATION` entry, or a decision record if the
      choice is a policy), and the sections above point to it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine to refuse such workbooks (a follow-up, if the decision says so).
