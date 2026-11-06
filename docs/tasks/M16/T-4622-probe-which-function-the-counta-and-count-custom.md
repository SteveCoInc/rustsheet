---
id: T-4622
title: "Probe which function the countA and count custom pivot subtotals compute"
milestone: M16
lane: research
status: todo
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
- [ ] A probe case with sidecar: one pivot per variant (and both together) over a field with numbers, text, a blank and an error; the subtotal values and labels captured.
- [ ] pivots.md entry rewritten from the evidence (VERIFIED or corrected), and the mapping in crates/rustsheet-xlsx/src/import/pivots/table.rs (`subtotal_function`) fixed if it is wrong.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Rendering custom subtotals (T-4168).
