---
id: T-4621
title: "Probe which sheet's definition a bare name binds to in a sheet-scoped API-added name's body: its own sheet or the first sheet by name"
milestone: M8
lane: api
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A bare defined name in the body of a *sheet-scoped* name added through the API (`Txn::define_sheet_name`) binds to
the definition Excel picks, with oracle evidence that separates "the name's own sheet first" from "the first
defining sheet by name" (T-4587's rule).

## Context
- T-4587 (PR #3640) qualifies a bare name in any API body with the first defining sheet by lower-cased name
  (`Parser::api_body_sheet_name`, crates/rustsheet-engine/src/expressions/parser/defined_names.rs), including the body
  of a sheet-scoped add. Its only evidence for a sheet-scoped add's body is probes/t4554-api-name-local-name-body, where
  Last is both the name's own sheet and the first sheet by name, so the two readings agree there. syntax.md marks the
  point UNVERIFIED (Excel-fidelity review of T-4587).
- A probe that separates them: sheets First, S2, Last; `Dup` defined on First (1) and on S2 (7); an API add of `SD`
  scoped to S2 with body `=Dup`; read `=SD` on S2. Own sheet first gives 7, first sheet by name gives 1. Vary the active
  sheet, and add the same on Last with `Dup` defined on First and Last.

## Acceptance criteria
- [ ] The probe above (excel-graph, predictions committed first) is filled and its cells pass, changing
      `api_body_sheet_name` for sheet-scoped adds if Excel prefers the own sheet.
- [ ] syntax.md's UNVERIFIED note on a sheet-scoped add's body becomes VERIFIED or is corrected.
- [ ] `make check` green; reviewers approve.

## Out of scope
- probes/t4554-api-name-local-name-body Last!D3 (T-4618).
