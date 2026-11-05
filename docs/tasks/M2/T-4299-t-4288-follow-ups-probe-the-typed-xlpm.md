---
id: T-4299
title: "T-4288 follow-ups: probe the typed _xlpm. extrapolations; saturating corner order; refusal docs"
milestone: M2
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: [T-4288]
---

## Goal
The places where T-4288 extrapolated past probes/t4224-doubled-xlpm-prefix have Excel evidence, the engine follows
it, and the reviewers' MINOR findings on PR 3292 are closed.

## Context
- T-4288 (PR 3292) made typed text, in A1 or R1C1 mode, keep `_xlpm.` as part of a name, refused a typed LET name
  written with it (`Parser::note_prefixed_declaration`), and replaced `r1c1_formula_to_a1` with
  `Model::set_typed_r1c1_user_input` / `Input::ParseR1C1`.
- Unprobed extrapolations (Excel-fidelity review of PR 3292):
  (a) the `_xlpm.` match is case-sensitive: a typed `=LET(_XLPM.x,1,_XLPM.x+1)` is not refused, and `_XLPM.x` is the
      name as written;
  (b) a LAMBDA body use `_xlpm.x` of a parameter `x` no longer binds, typed (`=LAMBDA(x,_xlpm.x*2)(3)`), extending
      A9's LET evidence to LAMBDA; and a LAMBDA parameter typed `_xlpm.x` binds under that name
      (`=LAMBDA(_xlpm.x,_xlpm.x*2)(3)`), where a LET name so written is refused;
  (c) typed R1C1 `=LET(x,2,_xlpm._xlpm.x*3)` (only the A1 form, A14, is probed);
  (d) a structured reference typed in R1C1 mode now parses against the workbook's tables (the converter used to have
      none, T-3113), e.g. `=SUM(T[Col])` typed in R1C1 mode next to a table `T`.
  docs/excel-semantics/syntax.md lists only the LAMBDA-parameter case as unprobed.
- Security review MINOR: `Parser::range_operand`'s typed-R1C1 corner ordering adds offsets in `i32`
  (`base + n`); it is safe only because `expand_short_forms` bounds every relative offset first. Use `i64` or
  `saturating_add` so a change to the pre-pass cannot overflow.
- Excel-fidelity review MINOR: docs/excel-semantics/references.md ("LET and LAMBDA names spelled like a cell", Ours)
  still describes the typed cell-shaped refusal only as `EditError::InvalidInput`; it now starts with
  `TYPED_FORMULA_DOES_NOT_PARSE`, so the harness leaves such an `entered` cell blank, as Excel's PATCH 400 does.

## Acceptance criteria
- [ ] A probe case (excel-probe skill, excel-graph) answers (a) to (d), each typed cell with its ISBLANK and
      FORMULATEXT; syntax.md records each answer, and the engine follows it (or the cells get owner tasks).
- [ ] The corner-ordering arithmetic cannot overflow; references.md's Ours paragraph names the blank-cell result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- What Excel writes to a file for these typed formulas.
