---
id: T-4174
title: "Implement calculated fields"
milestone: M16
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-4505, T-4142]
---

## Goal
Calculated fields are evaluated on the aggregated sums with the engine's own parser and evaluator, in a sandbox with no references, names or I/O, bounded by the work budget.

## Context
SPEC-0032; directive 3 (reuse the parser), directive 5. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

Excel for the web refuses a file whose calculated field calls NOW, TODAY, RAND or INDIRECT, holds an array
constant or a cell reference, or names an unknown field (pivots.md, T-4505): what the importer does with such
a file is this task's choice, recorded as a DEVIATION if it opens it.

## Acceptance criteria
- [x] p-calc field probes pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Calculated items.

## Outcome
Calculated fields import, compute, render and export (crates/rustsheet-engine/src/pivot/calculated.rs, `result.rs`
`calculate`; crates/rustsheet-xlsx/src/import/pivots.rs, export/pivots/cache.rs).
- Model: a calculated field is a `CacheField` with `formula: Some(..)`, no items and no records, after every other
  field; its pivotField and data field are ordinary ones (the import keeps their attributes verbatim). A refresh
  carries the calculated fields over; the export writes `<cacheField name numFmtId="0" formula databaseField="0"/>`
  and no record values for them.
- Formula: parsed by the engine's own parser, with a new `set_quoted_names` lexer/parser switch so `'Unit Cost'` and
  `'Bob''s'` read as names, then checked: fields (bare, quoted, or spelled like a cell, `Rev2`), constants, operators
  and function calls only. Refused (the import skips the cache, web.md's skip list): what Excel for the web refuses
  (NOW, TODAY, RAND, INDIRECT, an array constant, a cell reference, an unknown field) and, unprobed, RANDBETWEEN,
  RANDARRAY, OFFSET, CELL, INFO, GETPIVOTDATA, IMAGE, STOCKHISTORY, FIELDVALUE, FILTERXML, LET, LAMBDA, ranges,
  unknown functions. The function denylist is pinned by a test over every `Function` variant, so a new one must be
  reviewed for the sandbox. A value or Top-N filter by a calculated data field has no result (explicit check in
  `filter/rank.rs`, UNVERIFIED).
- Evaluation: per cell, every field in the checked tree is replaced by the cell's Sum of it (database fields read
  from extra accumulator slots after the data fields; calculated fields from earlier steps, ordered by Kahn's
  algorithm, a cycle and what depends on it `#NAME?`), and the tree is evaluated by the engine's evaluator
  (`evaluate_outside_cell`) in a sandbox: an empty `Model` of its own on a fixed clock, so no workbook cell, clock,
  file or network is reachable. Its work comes from a `RecalcBudget` sized to what the pivot's budget has left and is
  charged back (`Sandbox::evaluate`), plus the tree clone; limits `MAX_CALCULATED_FORMULA_CHARS` (8,192) and
  `MAX_CALCULATED_FIELDS` (256), depth by the parser (DEVIATION entry extended in pivots.md).
- Count, Average, Max, Min (any function) of a calculated field show the formula's value; a boolean is 1/0, a text
  `#VALUE!`; an intersection no record reaches is the formula over sums of 0 (new observation: LC B7 is 0).
- GETPIVOTDATA naming a calculated data field stays `#REF!` (UNVERIFIED). Calculated items are untouched (T-4175).
- Eval: every cell of probes/t4505-pivot-calc-fields (305), t4505-pivot-calc-field-functions (206) and
  t4130-pivot-calc-items (39, CF included) passes; t4128-pivot-spike K passes but the cells its descending label sort
  reorders (moved to T-4768). 482 known value failures and 466 text failures fixed; no regressions.
- Unit tests: calculated/tests.rs (every sheet of both t4505 probes and t4130 CF against the sidecars via
  `sidecar_differences`, the refusals, the limits, the budget, a 255-field chain) and import tests.
- Follow-up: T-4897 probes the shapes the engine chose without evidence (GETPIVOTDATA, custom subtotals, show-as,
  filters by a calculated field, name case, operand errors, the extra refusals, empty intersections).
