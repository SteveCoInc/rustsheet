---
id: T-4394
title: "An unqualified reference in an API-defined name: first sheet or active sheet"
milestone: M8
lane: research
status: todo
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
- [ ] A probe with the active sheet not first is filled, or the entry records why it cannot be; syntax.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Uses typed before the add re-binding (T-4391).
