---
id: T-4278
title: "Stop a text-comparing node inside its comparisons once the formula work budget runs out (T-4233 follow-up)"
milestone: M7
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A single node whose text comparisons alone would spend past the formula work budget stops at the budget, instead of
running to its end and only then being `#VALUE!`.

## Context
T-4233 charges the ticks a comparison reads (`functions/util/collation/compared.rs`) after each node
(`Model::spend_compared`, area_budget.rs). One node can still compare far more than the limit before the charge lands:
`=ROWS(UNIQUE(REPT("a",100)&SEQUENCE(1048576)))` runs about 30 s, and a UNIQUE of 1,000 texts of 16,383 `é` about 60 s,
before they are `#VALUE!` (release build, 2026-11-05; docs/excel-semantics/arrays.md "One formula evaluation does at
most 268,435,456 work units"). Find a way for the long-running loops (UNIQUE's keys and buckets, SORT's comparator, the
array `=`, the lookups' scans) to see that the pending ticks already exceed what the run has left and stop early, for
example a check in the readers' loops against a remaining-work snapshot, without a panic or unwinding (directive 5).

## Acceptance criteria
- [ ] The two formulas above are `#VALUE!` (`TOO_MUCH_WORK`) after at most about the limit's worth of ticks (an engine
      test that counts ticks or units, not time).
- [ ] No eval formula changes value; `make check` green; reviewers approve.

## Out of scope
- Making the comparisons faster (T-4279).
