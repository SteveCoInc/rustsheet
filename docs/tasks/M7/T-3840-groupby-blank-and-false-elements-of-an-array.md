---
id: T-3840
title: "GROUPBY: blank and FALSE elements of an array option are #VALUE!, and an early error lifts over an array field_relationship"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
GROUPBY lifted over an array `field_headers` or `field_relationship` gives Excel's `#VALUE!` for a blank or FALSE element, and an error in an earlier argument fills the lifted shape as Excel does.

## Context
- Found by T-3778 (gen-m7 arg-pairs batch for GROUPBY, excel-graph oracle, 2026-11-02). Every case's Data sheet has G1=7, G2=0, G3=-2.5, G4=1.5, G5="12" (text), G6="x", G7="" (empty text), G8=FALSE, G9=#DIV/0!, and H1, H2 blank. Data!A1:B2 is {"a*",<blank>;"?b",FALSE}; Data!C3:D4 is {"1",-2.5;0.1,-0.0001}.
- `Model::lift_options` (lifted_options.rs) and `Model::lifted_groupby` / `lifted_error` (groupby.rs) lift these options per element.
  The batch shows:
  - An element that is a blank cell or FALSE is `#VALUE!` in the lifted result (06c84cc76de04bae B3783:B3784, field_headers
    `Data!A1:B2`; 06c84cc76de04bae B5736 and 7ab51243a366f57b B8054, field_relationship `Data!A1:B2`); ours evaluates them as 0. A scalar
    FALSE field_relationship is 0 (probes/groupby-pivotby-followup G!A941), so this is lift-specific.
  - An error `row_fields` or `values` (`GROUPBY(#NULL!,..,Data!A1:B2)`, `GROUPBY(Data!D1:D12,#REF!,..,Data!A1:B2)`,
    `GROUPBY(Data!A1:C12,Data!G9,..,Data!C3:D4)`) fills the 2x2 lifted shape with that error in Excel; ours returns one cell.
- Owned cells:
- gen-m7/06c84cc76de04bae.case.json Calc!B3783:B3784, Calc!B5736
- gen-m7/45c197dc0b22cf2b.case.json Calc!A7565:B7566
- gen-m7/53473f95ff58029d.case.json Calc!A9090
- gen-m7/7ab51243a366f57b.case.json Calc!B8054
- gen-m7/eb01a9867300331b.case.json Calc!A2563:B2564, Calc!A2624:B2625
- gen-m7/f310ddb6bc8ec33a.case.json Calc!B4150

## Acceptance criteria
- [ ] The owned cells match Excel, spill extent included; unit tests cover a blank and a FALSE element and an early error under an array option.
- [ ] Record the rule in docs/excel-semantics/arrays.md ("GROUPBY and PIVOTBY lift over an array field_headers or total depth").
- [ ] Visible parity: every owned cell has Excel's type and error.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong error, or a spilled table where Excel returns `#VALUE!`), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.

## Out of scope
- PIVOTBY (its arg-pairs batch is T-3778's second PR) and the other GROUPBY rules of the batch, owned by T-3837..T-3840.
