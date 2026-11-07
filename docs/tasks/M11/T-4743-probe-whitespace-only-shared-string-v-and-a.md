---
id: T-4743
title: "Probe whitespace-only shared-string <v> and a bare <is/>; fix the clear_covered_cells doc comment"
milestone: M11
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The two value-less cell forms T-4148 decided without evidence are probed, and the importer and the snapshot reader
follow what Excel does with them. A misleading doc comment is corrected.

## Context
Review follow-up of T-4148 (PR #3773, reviewer-excel-fidelity MINORs). docs/excel-semantics/xlsx-storage.md "A cell
that stores no value is blank" rests on fixtures/valueless-cell-forms, which does not cover two forms:
- `<c t="s"><v> </v></c>` (a whitespace-only index). `is_valueless_constant`
  (crates/rustsheet-xlsx/src/import/worksheets.rs) reads it as blank because it trims the index; that is a guess.
- `<c t="inlineStr"><is/></c>` (an `<is>` with no `<t>`). The importer and `rustsheet-eval snapshot`
  (crates/rustsheet-eval/src/xlsx_snapshot.rs, `has_is`) read it as the empty string, and the snapshot test
  `valueless_str_and_inline_constants_are_blank` asserts that (F1); that is also a guess.
Also, the doc comment of `clear_covered_cells` (crates/rustsheet-xlsx/src/import/merged_cells.rs) says "Excel leaves
them empty", but fixtures/merged-covered-values shows Excel keeps covered content. The comment should say the
clearing is an engine invariant that differs from Excel, and point to xlsx-storage.md "A merged range keeps the values
of its covered cells" and T-4722.
To probe, extend the raw-XML generator described in eval/suites/fixtures/README.md (valueless-cell-forms) with these
rows in a new fixture, then run `rustsheet-oracle fill -b excel-graph --suite fixtures`.

## Acceptance criteria
- [ ] A fixture probes both forms (ISBLANK, ISTEXT, COUNTA, LEN, TYPE), and its oracle sidecar is committed.
- [ ] The importer and the snapshot reader match the evidence. The snapshot test's F1 assertion cites it or is changed.
- [ ] The xlsx-storage.md entry lists both forms, and the `clear_covered_cells` comment is corrected.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Keeping covered content (T-4722).
