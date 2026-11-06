---
id: T-4622
title: "Probe which function the countA and count custom pivot subtotals compute"
milestone: M16
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-4137]
---

## Goal
Evidence for what the `countASubtotal` (`item t="countA"`) and `countSubtotal` (`item t="count"`) custom subtotals of a pivot field compute and how Excel labels each line: over a field mixing numbers, text, blanks and errors, does one count every value (COUNTA) and the other only numbers (Count Numbers), as ECMA-376 reads them?

## Context
probes/t4137-pivot-captions K5!A8 labels a `t="count"` line `East Count` and counts 4 records, but its Sales field holds only numbers, so it cannot tell COUNTA from COUNT. The model maps `countA` to `Aggregate::Count` and `count` to `Aggregate::CountNums` (ECMA-376) without evidence (docs/excel-semantics/pivots.md "Which function the countA and count custom subtotals compute", UNVERIFIED; T-4139). T-4168 renders custom subtotals and needs the rule. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] A probe case with sidecar: one pivot per variant (and both together) over a field with numbers, text, a blank and an error; the subtotal values and labels captured.
- [x] pivots.md entry rewritten from the evidence (VERIFIED or corrected), and the mapping in crates/rustsheet-xlsx/src/import/pivots/table.rs (`subtotal_function`) fixed if it is wrong.
- [x] `make check` green; reviewers approve.

## Out of scope
- Rendering custom subtotals (T-4168).

## Outcome
probes/t4622-pivot-count-subtotals (excel-graph, 2026-11-06) holds seven tabular builder-v2 pivots, rows
Region (custom subtotals only) and Product, over Val (East: 10, `abc`, a blank, `#N/A`, 5; West: 7, `x`,
TRUE, 3) and variants. ECMA-376's reading holds, so `subtotal_function` in table.rs is unchanged (its doc
comment and tests now cite the evidence, and a new test covers `countA`):
- **Functions (B2, no error):** `countA` is COUNTA (East 4, West 4: text, TRUE counted, blank skipped);
  `count` is Count Numbers (East 3, West 2).
- **Labels:** both lines read `<item> Count` (`East Count`, `West Count`; B1!A6:A7 are both
  `East Count`). There is no `Count Numbers` label.
- **The subtotal function overrides the data field's aggregate**, and the lines stay counts while the
  Sum cells and Grand Total are `#N/A`.
- **Errors (CA, CN, B1, B4, B5):** under Sum of Val, an error ends the count within its inner-item cell
  in record order. countA counts the error itself (East 3 and 1 in B1 and B4, 4 and 2 in B5). Under Count
  of Val (B3) nothing is cut (4 and 2). The scope of this is open: T-4636.
The case builder now takes custom subtotals by their ST_ItemType names (`countA`, `avg`, `stdDevP`,
`varP`); before, it offered the data-field names, and `countNums` or `average` would have written
attributes the schema does not define. Recorded in docs/excel-semantics/pivots.md. The case's pivot
cells are owned by T-4149, and T-4168's label criterion is updated.
