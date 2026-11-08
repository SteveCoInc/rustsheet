---
id: T-5081
title: "Probe lower-case newer and engine-only formula error literals"
milestone: M1
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the lower-case formula literals the engine now reads in any case without a probe: `=#spill!`, `=#calc!`, and the engine-only `=#error!`, `=#circ!` and `=#n/impl!`.

## Context
T-4852 (probes/t4852-error-name-case E1:G8) showed Excel reads the seven older formula error literals in any case, and `formula_error_prefix` (expressions/token.rs) now folds case for every localized name. docs/excel-semantics/syntax.md "Error names are read in any case" lists `=#spill!` and `=#calc!` as unprobed. The engine-only names are not Excel literals at all; a probe should confirm how Excel treats `=#error!`, `=#circ!` and `=#n/impl!` when typed (entered through the input parser, `{"entered": ...}`, so a refusal does not fail the whole workbook load).

## Acceptance criteria
- [ ] A probe case with oracle values for the typed formulas `=#spill!`, `=#Calc!`, `=#error!`, `=#circ!` and `=#n/impl!` (value, FORMULATEXT, ERROR.TYPE).
- [ ] syntax.md records the result; if Excel differs from the engine, the engine matches it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Non-ASCII case folding of localized error names.
