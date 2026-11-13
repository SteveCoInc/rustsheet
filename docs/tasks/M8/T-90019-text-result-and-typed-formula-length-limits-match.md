---
id: T-90019
title: "Text result and typed formula length limits match Excel (val11 probes)"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Every visibly-off cell matches Excel, or its verdict is recorded. Excel for the web refused four typed entries of typed-formula-length-utf16-and-open-call (A6, A16, A21, A26; REFUSED in the fill log): the engine should refuse them alike.

## Context
- excel-graph fill, 2026-11-13, probe request val11 (#4128). Owns: probes/text-result-limit-builders, probes/typed-formula-length-utf16-and-open-call (every visibly-off cell of each). Run `make eval-show CASE=<case>` for the cells, and score them against the request's frozen predictions where it has them.

## Acceptance criteria
- [ ] Every visibly-off cell of the owned cases matches Excel, or its verdict is recorded in docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only behaviour (M14, decision 0039).
