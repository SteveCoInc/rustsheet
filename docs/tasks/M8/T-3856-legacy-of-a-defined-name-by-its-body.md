---
id: T-3856
title: "Legacy @ of a defined name by its body: a one-cell name as an IF arm and a constant name as an INDEX index show no @ (probe the other bodies first)"
milestone: M8
lane: engine
status: done
priority: P0
size: M
tier: opus
depends_on: []
---

## Goal
In a legacy (no `cm`) formula, the engine decides whether a defined name takes the implicit-intersection `@` from
the name's body, as Excel does. The case Excel showed: a name whose body is one cell is a one-cell IF arm, and a
name whose body is a constant is a scalar INDEX index.

## Context
- Found by T-3297 (probes/t3297-legacy-if-index-arm-name-table, excel-graph, 2026-11-02). With One = Sheet1!$B$3
  and Three = 3, Excel shows `=IF(TRUE,One)` and `=IF(TRUE,INDEX(B1:B6,Three))` with no `@`, also under ABS and SUM.
  Ours shows `=@IF(TRUE,One)` and `=@IF(TRUE,INDEX(B1:B6,@Three))`. The visibly-off cells are
  probes/t3297-legacy-if-index-arm-name-table I35, K35, M35, I36, K36.
- T-2440 (probes/legacy-at-defined-name-dynamic-funcs) found that a name takes the `@` in a value position whatever its
  body, for bodies that are SWITCH, a range, LET, XLOOKUP or `ABS(range)` (`=ABS(@swOne)`, `=@IF(TRUE,swOne)`).
  references.md lists a name whose body is a single cell, a constant, an array constant or another sheet's reference
  as unprobed. This finding says the single-cell and constant bodies differ.
- Code: `add_legacy_intersection`'s `Node::DefinedNameKind` arm (static_analysis.rs), `is_range_arm`
  (static_analysis/if_arm.rs), and `Node::DefinedNameKind(_) => StaticResult::Unknown` in
  `run_static_analysis_on_node`. The parser holds each name's body (`new_parser(worksheets, names, ..)`). A name's
  body can change after the formula is parsed, so the design must say when the display is recomputed.
- Why opus: a design question (parse-time name-body lookup) and a rule that needs a probe first.

## Acceptance criteria
- [x] A probe (excel-graph) of a name whose body is a constant number, text, boolean, a one-cell reference, another
      sheet's one-cell reference, a one-element and a larger array constant, and a range, each at the root, under ABS,
      under SUM, as an IF arm and as an INDEX index, with FORMULATEXT beside each.
- [x] The engine matches every cell of that probe and probes/t3297-legacy-if-index-arm-name-table I35, K35, M35, I36,
      K36; references.md records the rule.
- [x] `make check` green; reviewers approve.

## Out of scope
- LAMBDA-valued names and named calls (T-2440 settled them).

## Outcome
- Probe `probes/t3856-legacy-at-name-body` (excel-graph, 2026-11-04, 261 scored cells): twelve name bodies (3, "x",
  TRUE, Sheet1!$Z$3, S2!$A$3, {3}, {3;4}, {3,4}, Sheet1!$Z$1:$Z$6, 1+2, Sheet1!$Z$3:$Z$3, S2!$A$1:$A$6) at the root,
  under ABS and SUM, as an IF arm (alone, under ABS, under SUM), as an INDEX index (alone and inside an IF arm, under
  ABS and SUM), FORMULATEXT beside each; row 45 adds literal INDEX shapes.
- Rule (references.md "A defined name takes the `@` by its body"): a name whose body is one number, text or boolean
  literal or one cell reference reads as that body; every other body takes the `@`. Found in passing: a single-cell
  INDEX (literal whole-number indexes) shows no `@` on its own either (`=INDEX(Z1:Z6,3)`), not only as an IF arm.
- Engine: `scalar_body` (new static_analysis/name_body.rs) lexes a name's stored body (at most 520 characters, two
  tokens); `add_legacy_intersection`, `run_static_analysis_on_node` and `is_range_arm` read it, and the new
  `indexes_one_cell` (if_arm.rs) accepts a number-bodied name as a literal index; `indexes_one_reference_cell` makes a
  single-cell INDEX a scalar shape in `fallback_shape` only over a range, a union or a reference-bodied name (review
  round 1: over a 2-D array-bodied name INDEX is a row, unprobed, so it keeps its `@` and spill as before).
- When the display is decided: when the legacy formula is read (xlsx import, `set_cell_with_legacy_formula`), from the
  name's body then; the `@` stays in the stored formula after that. Whether Excel redraws it when a body changes later
  is unprobed (recorded in references.md).
- Eval delta: probes +1 case (261 cells, all pass), 5 known failures fixed (t3297-legacy-if-index-arm-name-table I35,
  I36, K35, K36, M35), 0 regressions.
