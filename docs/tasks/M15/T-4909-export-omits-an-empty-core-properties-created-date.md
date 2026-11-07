---
id: T-4909
title: "Export omits an empty core-properties created date (Excel refuses the package)"
milestone: M15
lane: engine
status: blocked
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A regenerated save of a workbook whose source records no creation date opens in Excel. Today Excel for the web
refuses it.

## Context
Found by the T-4890 probe (docs/research/xlsx-round-trip/excel-resave-probe-2026-11-07.md §3;
docs/excel-semantics/xlsx-storage.md "An empty W3CDTF date makes Excel refuse the package").
- `crates/rustsheet-xlsx/src/export/doc_props.rs` `get_core_xml` always writes
  `<dcterms:created xsi:type="dcterms:W3CDTF">{created}</dcterms:created>`. When the source has no
  `docProps/core.xml`, or no `created` in it, `created` is empty.
- Excel for the web refuses that package with `FileCorruptTryRepair`. Removing only that element makes the same
  package open (fixtures valueless-cell-forms R1-E5).
- In the same function, `creator` is written without XML escaping. A `<` or `&` in the source's creator would
  break the part.

## Acceptance criteria
- [ ] An empty `created` is not written. `creator` is escaped. Unit tests cover both.
- [ ] The fixtures valueless-cell-forms R1-all save opens in Excel for the web (`rustsheet-oracle resave`), recorded
      in the semantics entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Writing a real creation date when the source has none.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`.
