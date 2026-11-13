---
id: T-90011
title: "CHISQ.DIST.RT and CHIINV at df 1 match Excel on the T-5524 fresh sweep"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
usage: expert
depends_on: []
---

## Goal
The fresh sweep's visibly-off cells (69) are explained against docs/research/t5524/ and fixed, or their verdict recorded in docs/excel-semantics/statistics.md. The request's T-5524 comes from the requesting repository's task list.

## Context
- excel-graph fill, 2026-11-13, probe request val11 (#4128). Owns: probes-numeric/t5524-chisq-df1-complement-fresh (every visibly-off cell of each). Run `make eval-show CASE=<case>` for the cells, and score them against the request's frozen predictions where it has them.

## Acceptance criteria
- [ ] Every visibly-off cell of the owned cases matches Excel, or its verdict is recorded in docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only behaviour (M14, decision 0039).
