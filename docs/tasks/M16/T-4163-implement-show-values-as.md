---
id: T-4163
title: "Implement show-values-as"
milestone: M16
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4161, T-4149]
---

## Goal
PivotResult applies all fourteen show-values-as modes as the evidence says (leaves, subtotals, grand totals).

## Context
SPEC-0032; mog's three-pass dispatcher (oss-engines.md §1) is a design idea only. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

Owner of these visibly-off pivot cells (decision 0245; moved from T-4152, whose adapter loads the
  case builder's parts through the importer, which skips a pivot with a `showDataAs` data field, "a data
  field shown as a calculation"): probes/t4123-pivot-capture T!A3:E12, T2!A3:E12, T3!A3:E12,
  T4!A3:E12.
- From T-4129's capability matrix (each mode on rows Region and Product, columns Year; pivots.md "Show-values-as
  modes as Excel for the web renders them"):
  - probes/t4129-pivot-show-as DF!A3:A12, DF!B3:B4, DF!B7:B8, DF!B11, DF!C4, DF!C7:C8, DF!C11, DF!D4, DF!D7:D8,
    DF!D11, DP!A3:A12, DP!B3:B4, DP!C4:C12, DP!D4, IX!A3:A12, IX!B3:B12, IX!C4:C12, IX!D4:D12, PC!A3:A12,
    PC!B3:B4, PC!B6:B8, PC!B10:B11, PC!C4, PC!C6:C8, PC!C10:C11, PC!D4, PC!D6:D8, PC!D10:D11, PCo!A3:A12,
    PCo!B3:B12, PCo!C4:C12, PCo!D4:D12, PD!A3:A12, PD!B3:B4, PD!B7:B8, PD!B11, PD!C4, PD!C7:C8, PD!C11, PD!D4,
    PD!D7:D8, PD!D11, PR!A3:A12, PR!B3:B12, PR!C4:C12, PR!D4:D12, PT!A3:A12, PT!B3:B12, PT!C4:C12, PT!D4:D12,
    RT!A3:A12, RT!B3:B4, RT!B6:B8, RT!B10:B11, RT!C4, RT!C6:C8, RT!C10:C11, RT!D4, RT!D6:D8, RT!D10:D11
  - probes/t4129-pivot-show-as-x14 PP!B5:B12, PP!C5:C12, PP!D5:D12, PPC!B5:B12, PPC!C5:C12, PPC!D5:D12,
    PPR!B5:B12, PPR!C5:C12, PPR!D5:D12, PRT!B5:B12, PRT!C5:C12, PRT!D5:D12, RA!B5:B10, RA!B12, RA!C5:C7,
    RA!C9:C12, RA!D5:D12, RD!B5:B7, RD!B9:B10, RD!B12, RD!C5:C7, RD!C9:C12, RD!D5:D7, RD!D9:D12

## Acceptance criteria
- [ ] p-showas probes pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GETPIVOTDATA of show-as values unless evidenced.
