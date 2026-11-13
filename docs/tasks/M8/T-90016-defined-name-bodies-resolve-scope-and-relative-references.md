---
id: T-90016
title: "Defined-name bodies resolve scope and relative references as Excel does (val11 name probes)"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Every visibly-off cell matches Excel, or its verdict is recorded. Compare T-90008 (the first-sheet case).

## Context
- excel-graph fill, 2026-11-13, probe request val11 (#4128). Owns: probes/defined-name-body-scope-file-order, probes/defined-name-body-scope-first-sheet-inactive, probes/defined-name-relative-wraps, probes/plain-reader-of-api-range-name (every visibly-off cell of each). Run `make eval-show CASE=<case>` for the cells, and score them against the request's frozen predictions where it has them.

## Acceptance criteria
- [ ] Every visibly-off cell of the owned cases matches Excel, or its verdict is recorded in docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only behaviour (M14, decision 0039).
