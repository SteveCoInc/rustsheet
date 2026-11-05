---
id: T-4343
title: "Typed _XLPM. prefix: lowercase it only before a name a LET or LAMBDA binds (probes/t4331-typed-xlpm-remaining C2:C3)"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Typed `=_XLPM.x` and `=LET(x,2,_XLPM._XLPM.x*3)` read back in FORMULATEXT with their prefix case unchanged, as Excel does.

## Context
probes/t4331-typed-xlpm-remaining C2 (`=_XLPM.x`, Excel `=_XLPM.x`, ours `=_xlpm.x`) and C3 (Excel `=LET(x,2,_XLPM._XLPM.x*3)`, ours lowercases the first prefix). Excel lowercases the typed prefix only when the rest is a name the LET or LAMBDA binds (t4299 C2). `name_operand` in crates/rustsheet-engine/src/expressions/parser/operands.rs lowercases it unconditionally; see docs/excel-semantics/syntax.md.

## Acceptance criteria
- [ ] probes/t4331-typed-xlpm-remaining C2:C3 pass, and t4299 C2:C3 still pass; baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
