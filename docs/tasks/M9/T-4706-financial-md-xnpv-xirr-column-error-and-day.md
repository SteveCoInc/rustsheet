---
id: T-4706
title: "financial.md XNPV/XIRR column-error and day -1..0 entries say only what T-4607's cells isolate"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The docs/excel-semantics/financial.md entries that T-4607 (PR #3697) added claim only what the cells of
probes/t4607-xnpv-xirr-column-errors-and-day-minus-one isolate.

## Context
- The excel-fidelity review of #3697 (head 9041bcfa9, adopted by an opus run) approved the PR with two MINOR wording
  findings:
  - The day -1..0 entry says these dates are `#NUM!` "for XNPV and XIRR as the first, a later or a last date". For
    XNPV only rows 15-16 (the bad date is the first) isolate the rule: rows 17-22 put -0.5 or -1E-9 after an earlier
    first date, and XNPV is `#NUM!` for any date before the first anyway. The XIRR cells of rows 17-22 do isolate it.
    The phrase "as a date truncated to -1 would be" explains rather than observes: the cells show only `#NUM!`.
  - "cell errors win over the empty-cell #NUM! in whole columns and ranges alike" (financial.md and T-4607's Outcome)
    overstates the range form: the A1:A3/B1:B3 ranges (columns C and D) hold no empty cell, so they show only that
    the error passes through.
- Wording only. No engine or case change.

## Acceptance criteria
- [ ] financial.md says the XNPV part of the day -1..0 rule rests on the first-date cells only, and drops the
      truncation explanation (or marks it UNVERIFIED).
- [ ] financial.md says only the whole-column cells show cell errors winning over the empty-cell `#NUM!`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New probe cells (a range holding an empty cell and an error could be a later probe).

## Outcome
Wording only in docs/excel-semantics/financial.md (and T-4607's Outcome sentence): the day -1..0 XNPV rule rests on rows 15-16 only, the truncation explanation is marked UNVERIFIED, and only the whole-column cells show cell errors winning over the empty-cell `#NUM!`. No engine, case or eval change (delta 0).
