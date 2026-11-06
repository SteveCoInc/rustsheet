---
id: T-4617
title: "Which unresolved uses a sheet-scoped API-added name re-binds: Excel re-binds in most books but not in T-4049's or T-4353's, with no isolated cause"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
An unresolved use loaded from the file re-binds, or stays `#NAME?`, after a Graph `worksheets('S')/names/add`
exactly as Excel does. That needs a rule the evidence supports, and T-4391's probes did not find one.

## Context
- docs/excel-semantics/syntax.md "A sheet-scoped name added after load" lists the evidence. In 11 books of
  T-4391's probes, Excel re-binds every unresolved use: any name, any body, any using formula, with First, S2 or
  Last active. The engine pins them all `#NAME?` (`Model::bind_added_sheet_name`, T-4049's rule). Yet `=N` in
  t4049-sheet-api-name-plain and t4014-sheet-api-name-65, and `=Rate` in t4353-api-name-unqualified-ref and
  t4394-api-name-active-s2, stay `#NAME?` in every repeat. Function names (N, RATE), add order, a qualified `=S!N`
  loaded beside them and the active sheet were each ruled out alone. t4391-api-rebind-s2-qualified-first-add and
  t4394-api-name-active-s2 differ only in the other adds (Span, Rate2, W vs Qa, Ql), the `=W` and `=SUM(Span)` uses
  and the cell positions: bisect between them.
- The oracle did not repeat itself once. t4391-api-rebind-first's first fill (commit 681082d1a) was `#NAME?` in
  every cell; three fresh sessions of the same input re-bound every use. Run each discriminating book at least three
  times, using `--force` with its `eval/.cache/oracle` entry moved aside, or a direct `backend.recalc`, before
  trusting it.
- Changing the engine to always re-bind would regress the passing T-4049/T-4014 cells; do not special-case.
- Owned cells (the engine gives `#NAME?` at each):
  - probes/t4353-api-name-unqualified-ref (2): S2!A3, Last!A1
  - probes/t4394-api-name-active-s2 (2): S2!A3, Last!A1
  - probes/t4394-api-name-active-last (3): S2!A1, S2!A3, Last!A1
  - probes/t4391-api-rebind-first (13): First!A1:A2, S2!A1:A9, Last!A1:A2
  - probes/t4391-api-rebind-s2 (13): First!A1:A2, S2!A1:A9, Last!A1:A2
  - probes/t4391-api-rebind-last (13): First!A1:A2, S2!A1:A9, Last!A1:A2
  - probes/t4391-api-rebind-first-no-first-add (3): S2!A1:A2, Last!A1
  - probes/t4391-api-rebind-first-then-book (3): S2!A1:A2, Last!A1
  - probes/t4391-api-rebind-first-t4353-unqualified (3): S2!A1, S2!A3, Last!A1
  - probes/t4391-api-rebind-s2-qualified-use (3): S2!A1:A2, Last!A1
  - probes/t4391-api-rebind-last-qualified-use (3): S2!A1:A2, Last!A1
  - probes/t4391-api-rebind-s2-qualified-first-add (3): S2!A1:A2, Last!A1
  - probes/t4391-api-rebind-last-qualified-first-add (3): S2!A1:A2, Last!A1

## Acceptance criteria
- [ ] A rule that holds across every probe above and the T-4049/T-4014 cases is found and recorded in syntax.md,
      and implemented in `Model::bind_added_sheet_name`. Otherwise the cells stay known failures and the
      bisection's result is recorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Uses inside API-added name bodies (T-4587).
- Typing cells after the add (T-4348).
