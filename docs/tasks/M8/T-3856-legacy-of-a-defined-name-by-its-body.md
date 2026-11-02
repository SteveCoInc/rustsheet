---
id: T-3856
title: "Legacy @ of a defined name by its body: a one-cell name as an IF arm and a constant name as an INDEX index show no @ (probe the other bodies first)"
milestone: M8
lane: engine
status: todo
priority: P2
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
- [ ] A probe (excel-graph) of a name whose body is a constant number, text, boolean, a one-cell reference, another
      sheet's one-cell reference, a one-element and a larger array constant, and a range, each at the root, under ABS,
      under SUM, as an IF arm and as an INDEX index, with FORMULATEXT beside each.
- [ ] The engine matches every cell of that probe and probes/t3297-legacy-if-index-arm-name-table I35, K35, M35, I36,
      K36; references.md records the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- LAMBDA-valued names and named calls (T-2440 settled them).
