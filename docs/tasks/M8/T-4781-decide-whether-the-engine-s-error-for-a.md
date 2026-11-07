---
id: T-4781
title: "Decide whether the engine's #ERROR! for a name Excel refuses at load needs a DEVIATION entry"
milestone: M8
lane: engine
status: done
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
- [x] One place states the rule for Excel-refused workbooks (a `DEVIATION` entry, or a decision record if the
      choice is a policy), and the sections above point to it.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the engine to refuse such workbooks (a follow-up, if the decision says so).

## Outcome
- Decided: this is not a `DEVIATION`. Decision 0014 covers only the limits directive 5 needs (decision 0052). Decisions 0045
  and 0047 already make the engine's cell-level parse error the stand-in for a form Excel refuses at load, and 0047
  calls matching Excel's refusal not a deviation. The choice was made in those decisions, so no new decision record
  was written.
- The rule is now stated once, in docs/excel-semantics/README.md "Forms Excel refuses at load". A refused form keeps
  `VERIFIED` with "Ours: `#ERROR!`" and needs no gap task. A refused form that the engine evaluates to a value is a
  known gap with a task. Desktop loading stays M14 (decision 0039).
- The sections point to it: references.md (duplicate LET and LAMBDA names, T-1387; "Characters of a name" table names
  and "Implemented in", T-1856, T-4446, T-4753) and syntax.md "Formulas over 8,192 characters refuse the workbook".
- Eval delta: none (docs only).
