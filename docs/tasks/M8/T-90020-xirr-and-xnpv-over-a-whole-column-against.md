---
id: T-90020
title: "XIRR and XNPV over a whole column against a range match Excel"
milestone: M8
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
usage: expert
depends_on: []
---

## Goal
Every visibly-off cell matches Excel, or its verdict is recorded.

## Context
- excel-graph fill, 2026-11-13, probe request val11 (#4128). Owns: probes/xirr-xnpv-whole-column-against-a-range (every visibly-off cell of each). Run `make eval-show CASE=<case>` for the cells, and score them against the request's frozen predictions where it has them.

## Acceptance criteria
- [ ] Every visibly-off cell of the owned cases matches Excel, or its verdict is recorded in docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only behaviour (M14, decision 0039).
