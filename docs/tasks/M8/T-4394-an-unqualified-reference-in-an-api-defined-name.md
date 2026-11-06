---
id: T-4394
title: "An unqualified reference in an API-defined name: first sheet or active sheet"
milestone: M8
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: [T-4353]
---

## Goal
Know whether an unqualified reference in a name added through the API binds to the workbook's first sheet or to its
active sheet.

## Context
- T-4353 (probes/t4353-api-name-unqualified-ref, docs/excel-semantics/syntax.md "An unqualified reference in an
  API-defined name binds to the first sheet"): `=$B$1` added with Graph `names/add` reads First!B1, but First was
  both the first and the active sheet of the probe's workbook. The engine binds to the first sheet.
- Needs a workbook whose active sheet is not the first: check whether spec2xlsx (tools/) can write
  `<workbookView activeTab=...>` and whether excel-graph honours it. If it cannot, say so in the entry and close.
- If Excel uses the active sheet, file an engine/api task for `Txn::define_name` and `Txn::define_sheet_name`.

## Acceptance criteria
- [x] A probe with the active sheet not first is filled, or the entry records why it cannot be; syntax.md updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- Uses typed before the add re-binding (T-4391).

## Outcome
- Excel binds an unqualified reference in an API-defined name to the **active** sheet, not to the first sheet.
  probes/t4394-api-name-active-s2 and probes/t4394-api-name-active-last (excel-graph, 2026-11-06) are T-4353's book with S2
  or Last as the active sheet. Every use reads the active sheet's B1 (7, or 9) or B1:B2 (77, or 99), whether the name
  is workbook-scoped or sheet-scoped. With Last active, even S2's `=Rate`, typed before the add, resolves; it stays
  `#NAME?` with First or S2 active.
- Tooling: spec2xlsx takes a top-level `"active_sheet"`, which sets `<workbookView activeTab>` and the only
  `tabSelected` sheet (`tools/tests/test_spec2xlsx_active_sheet.py`). The Rust `CaseSpec` ignores the key: it is not
  `deny_unknown_fields`, and the engine adapter has no active sheet.
- syntax.md: the T-4353 entry now says "binds to the active sheet", with the new evidence.
- Follow-ups: T-4445 (api) gives the API an active sheet and binds API names to it. It owns 10 cells: First!A1,
  S2!A5, S2!A7, Last!A3 and Last!A4 of each case. T-4391 now also owns the typed-before cells of both cases.
- Eval delta: 16 new cells, 1 passing (t4394-api-name-active-s2 S2!A1, `#NAME?` in both) and 15 known failures, all owned as above; 0 regressions. No engine change.
