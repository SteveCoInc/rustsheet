---
id: T-4391
title: "A sheet-scoped name added through the API re-binds some unresolved uses on its sheet: SUM(Span) and Rate2 resolve, Rate stays #NAME?"
milestone: M8
lane: engine
status: done
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
- [x] The rule is probed and recorded; the owned cells pass, or stay known failures with the reason recorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Which sheet an unqualified body binds to (T-4353: the first).
- Typing cells after the add (T-4348).

## Outcome
- No clean rule, so the owned cells stay known failures and move to T-4617 with the probes' cells. Ten probe
  cases (probes/t4391-api-rebind-*, predictions committed before each fill, excel-graph 2026-11-06) vary the
  name (function names `Rate`, `N`, `Abs`, `Npv`, `Sln` against plain ones), the add order, the body (cell, range,
  constant), the using formula (`=X`, `=X+0`, `=SUM(X)`), the active sheet (First, S2, Last), qualified `=S2!Rate`
  uses and a closing workbook add. Every one of those books re-binds every unresolved use; each hypothesis
  (function name; first add; qualified use; first add with a qualified use) was refuted. T-4049's, T-4014's and
  T-4353/T-4394-s2's books keep `#NAME?` in every repeat. One input did not repeat: the first fill of
  t4391-api-rebind-first was `#NAME?` in all 13 cells (commit 681082d1a), and three later sessions re-bound them
  all; its sidecar is refilled with the repeated answer. Recorded in docs/excel-semantics/syntax.md "A sheet-scoped
  name added after load" ("Most books re-bind instead; no rule found").
- Engine unchanged apart from a doc comment on `Model::bind_added_sheet_name`. Re-binding everywhere would break the
  passing T-4049/T-4014 cells, and a narrower rule would be special-casing without evidence.
- Eval delta: new coverage 10 cases, 72 checked cells: 12 pass (the qualified `=S2!Rate` and workbook `=W` uses),
  60 known failures owned by T-4617 (recorded in eval/baseline/probes.txt by `make eval-baseline`); 0 regressions.
- Follow-up: T-4617 (bisect t4391-api-rebind-s2-qualified-first-add against t4394-api-name-active-s2, with at least
  three runs per book).
