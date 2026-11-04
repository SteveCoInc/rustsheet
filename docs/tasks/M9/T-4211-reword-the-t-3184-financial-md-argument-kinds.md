---
id: T-4211
title: "Reword the T-3184 financial.md argument-kinds entry to its evidence and probe whether a scalar text flow is skipped or read as a number"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/financial.md "M9 financial argument kinds (T-3184)" claims only what its cells show. Its
scalar-text-flow rule is either backed by discriminating Excel cells or stated without a cause.

## Context
- Reviewer finding on PR #3184 (T-3184, excel-fidelity, MINOR). The entry says a scalar text or numeric-text flow is
  "skipped". The cells it cites give the same result if `"5"` were read as 5:
  - `MIRR("5",0.1,0.12)` is `#DIV/0!` (probes/t3184-m9-financial-argument-kinds-c A42), but MIRR(5,..) has no
    negative flow either.
  - `XIRR("5",<5 dates>)` (-e A49) and `XNPV(..,<5 values>,"5")` (-e A60) are `#NUM!` from the count mismatch either way.
  - `IRR("",0.1)` `#NUM!` (-c A18) and `MIRR("",..)` `#DIV/0!` (-c A44) likewise.
- Also: the entry lists IPMT's nper among "TRUE is 1", but `IPMT(0.008,1,TRUE,8000,0,0)` (-c A7) is period 1's
  interest, rate*pv, for any valid nper. That cell shows TRUE is accepted, not that it is 1.
- Discriminating probes: matching lengths and a flow whose sign matters, e.g. `XIRR({-100,"110"},{0,365})` against
  `XIRR({-100,110},{0,365})`, `XNPV(0.1,"-100",{0})` against `XNPV(0.1,-100,{0})`, and `IPMT(0.008,2,TRUE,8000)`
  against nper 1 and 2. (`IRR` of one flow is `#NUM!` either way, so it cannot tell.)

## Acceptance criteria
- [ ] A probes case (excel-probe skill, excel-graph, tag M9) with the discriminating cells.
- [ ] financial.md's T-3184 entry states the rule the new cells show, or drops the "skipped" cause, and drops IPMT nper
      from the "TRUE is 1" list unless a new cell shows it.
- [ ] Any engine difference gets a fix or an owner task; `make eval-baseline` 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other kinds of the T-3184 entry (verified by its cells).
