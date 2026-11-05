---
id: T-4331
title: "T-4299 follow-ups: one _xlpm. prefix constant in the parser; probe the remaining typed _XLPM. forms"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-4299]
---

## Goal
The parser names the `_xlpm.` prefix once, and the typed `_XLPM.` forms T-4299 left unprobed are settled by an
excel-graph probe (the engine matches it, or the cells are owned by a task).

## Context
MINOR findings from the PR #3305 review (T-4299):
- Design: T-4299 added `XLPM_PREFIX` and `has_xlpm_prefix` (crates/rustsheet-engine/src/expressions/parser/
  operands.rs), but about ten bare `"_xlpm."` literals remain (parser `call_names.rs`, `operands.rs`, `mod.rs`,
  and `stringify/bound_variables.rs`). Move the constant to the parser module and use it at those sites
  (CLAUDE.md directive 3). Keep the case-sensitive sites case-sensitive: only the typed-input check ignores case.
- Excel fidelity: the engine extrapolates past probes/t4299-typed-xlpm-extrapolations for a mixed-case `_Xlpm.`
  prefix, a bare `=_XLPM.x`, a doubled `_XLPM._XLPM.x` (the engine lowercases only the first prefix) and a called
  `_XLPM.f(3)`. docs/excel-semantics/syntax.md lists them as unprobed. Use the `excel-probe` skill.

## Acceptance criteria
- [x] One prefix constant serves every parser and stringify site; `rg '"_xlpm\."' crates/rustsheet-engine/src`
      finds only that definition (and tests).
- [x] An excel-graph probe covers the four typed forms above; syntax.md records each result with its cell.
- [x] The engine matches every probe cell, or the cells are known failures owned by a task.
- [x] `make check` green; reviewers approve.

## Out of scope
- Prefix handling in xlsx-stored formulas (file input is unchanged by T-4299).

## Outcome
One `XLPM_PREFIX` constant (parser/mod.rs) now serves every parser, stringify and lambda site; `rg '"_xlpm\."'` finds only its definition. Probe `probes/t4331-typed-xlpm-remaining` (excel-graph) covers the four forms; results are in docs/excel-semantics/syntax.md. Engine matches all cells except C2 and C3 (bare `_XLPM.x` and doubled prefix keep their case in Excel), owned by T-4343. A1 and A5 were refused by Excel (blank) and match.
