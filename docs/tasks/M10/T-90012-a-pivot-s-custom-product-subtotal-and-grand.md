---
id: T-90012
title: "A pivot's custom Product subtotal and grand total render as Excel's (t5483 probe)"
milestone: M10
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The probe's pivot layout and cells match Excel's, or each difference is recorded in docs/excel-semantics.

## Context
- excel-graph fill, 2026-11-13, probe request val11 (#4128). Owns: probes/t5483-pivot-custom-product-grand (every visibly-off cell of each). Run `make eval-show CASE=<case>` for the cells, and score them against the request's frozen predictions where it has them.

## Acceptance criteria
- [ ] Every visibly-off cell of the owned cases matches Excel, or its verdict is recorded in docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only behaviour (M14, decision 0039).
