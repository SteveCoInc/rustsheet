---
id: T-4210
title: "Probe the T-1683 hand generalizations: unqualified [@col] case, column-range spelling, non-space whitespace after @"
milestone: M8
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence (excel-graph) for the printed spellings that T-1683 implemented by hand generalization, with an
engine task filed for every shape where ours differs.

## Context
- T-1683 (PR #3173; docs/excel-semantics/references.md "Structured references: FORMULATEXT") probed FORMULATEXT of
  structured references from another sheet (probes/t1683-formulatext-structured-refs). The entry names what it
  generalized by hand: `StructuredReference::use_table_spelling` and `prints_bare_after_at`, both in
  crates/rustsheet-engine/src/expressions/parser/structured_reference.rs.
- Unprobed, ours shown:
  1. An unqualified reference inside its table, typed in another case: `=[@qty]*2` and `=SUM([qty])` in a data
     row of `Sales`. Ours prints `=[@Qty]*2` and `=SUM(Sales[Qty])`, the table's spelling.
  2. Both ends of a column range in another case: `=SUM(Sales[[#This Row],[qty]:[UNIT PRICE]])` and
     `=SUM(Sales[[qty]:[unit price]])`. Ours prints `=SUM(Sales[@[Qty]:[Unit Price]])` and
     `=SUM(Sales[[Qty]:[Unit Price]])`.
  3. A column name with whitespace other than a space, after `@`: a header holding a tab or a no-break space
     (`A` + U+00A0 + `B`) in a stored `Sales[[#This Row],[A B]]`. Ours keeps the brackets for any whitespace.
- A file whose table `name` and `displayName` differ in spelling is T-2320's question, not this task's.

## Acceptance criteria
- [ ] A probe case (excel-probe skill) with excel-graph oracle values covers items 1 to 3, typed and stored, with
      FORMULATEXT beside each.
- [ ] references.md "Structured references: FORMULATEXT" records each result and drops it from the hand
      generalizations.
- [ ] An engine task is filed for every shape where ours differs (or none differs).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The header-row, totals-row and copied-out spellings (T-3240).
