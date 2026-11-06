---
id: T-4391
title: "A sheet-scoped name added through the API re-binds some unresolved uses on its sheet: SUM(Span) and Rate2 resolve, Rate stays #NAME?"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Uses of a sheet-scoped name that the file left unresolved re-bind after a Graph `worksheets('S')/names/add` exactly
as Excel does, which T-4049's rule (they stay `#NAME?`) does not cover.

## Context
- probes/t4353-api-name-unqualified-ref (T-4353): three sheets First, S2, Last; API adds in order Rate (`=$B$1`, S2),
  Span (`=$B$1:$B$2`, S2), Rate2 (`=$B$1`, Last), W (`=$B$1`, workbook). Cells typed before the adds: S2!A1 `=Rate`
  stays `#NAME?` as T-4049 found for a constant body, but S2!A3 `=SUM(Span)` is 11 and Last!A1 `=Rate2` is 1 in
  Excel; the engine gives `#NAME?` for both (`Model::bind_added_sheet_name` pins them).
- What differs between Rate and the others is unknown: the order of the adds (Rate first), the body, or the using
  formula. Probe it (vary the order, the body kind and the using formula) before changing the binding, and record the
  rule in docs/excel-semantics/syntax.md "A sheet-scoped name added after load".
- Owned cells: probes/t4353-api-name-unqualified-ref (2): S2!A3, Last!A1.
- The same uses with another sheet active (T-4394): probes/t4394-api-name-active-s2 (2): S2!A3, Last!A1 (77 and 7 in
  Excel), and probes/t4394-api-name-active-last (3): S2!A1, S2!A3, Last!A1 (9, 99 and 9). With Last active even S2's
  `=Rate` resolves, so the active sheet takes part in the rule. Ours is `#NAME?` at each.

## Acceptance criteria
- [ ] The rule is probed and recorded; the owned cells pass, or stay known failures with the reason recorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Which sheet an unqualified body binds to (T-4353: the first).
- Typing cells after the add (T-4348).
