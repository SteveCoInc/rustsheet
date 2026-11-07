---
id: T-4438
title: "T-4272 review follow-ups: probe deleting the start of a two-sheet 3-D span and a backwards span in a defined name; share the sheet-edit stringify closure"
milestone: M8
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The two 3-D span rules T-4272 recorded by inference have excel-graph evidence, and the sheet-edit formula rewrites
share one stringify helper.

## Context
- Review MINORs on T-4272's PR #3457 (Excel-fidelity and design reviewers).
- docs/excel-semantics/references.md ("3-D references when sheets move or are renamed") says a two-sheet span left
  with one sheet becomes a plain reference. Only deleting the end was probed (`Jan:Feb` without Feb is `Jan!A1`,
  probes/sheet-ops/t4272-3d-span-delete-two-sheet-end). Deleting the start (Jan out of `Jan:Feb`, predicted
  `Feb!A1`) is inferred.
- The parser reads a backwards span (`Mar:Jan`) in tab order everywhere, including a defined name's text
  (`get_sheet_span`, crates/rustsheet-engine/src/expressions/parser/sheet_span.rs). The probe evidences cell
  FORMULATEXT only (t4272-3d-span-backwards B1, B2). A name's cells check only values.
- `take_sheet` (sheet_undo.rs) and `move_sheet_rewriting` (new_empty.rs) each pass `rewrite_formulas` the same
  closure: `match context { Some(c) => to_english_string(&node, c), None => to_rc_format(&node) }`. Add one helper
  in formula_rewrite.rs (for example `rewrite_formulas_with(skip, |node| ...)`) and use it at both sites.
- Probe layout: copy probes/sheet-ops/t4272-3d-span-delete-two-sheet-end.case.json. For the name text, the case
  format cannot read a name's body back, so check how Graph reports it (the oracle backend's names readout, if
  any), or record that no backend can show it.

## Acceptance criteria
- [ ] `t4272-3d-span-delete-two-sheet-start` is filled on excel-graph and passes, and references.md cites it.
- [ ] A backwards span in a defined name is either evidenced and cited, or references.md says its text is unprobed.
- [ ] One stringify-by-context helper serves both sheet-edit rewrites; no eval cell changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other sheet edits (T-4272 settled them).
