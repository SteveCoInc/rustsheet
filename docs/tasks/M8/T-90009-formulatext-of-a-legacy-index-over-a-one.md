---
id: T-90009
title: "FORMULATEXT of a legacy INDEX over a one-row range with a MATCH index shows no @"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
usage: expert
depends_on: []
---

## Goal
FORMULATEXT of a legacy INDEX over a one-row or one-column range with a MATCH index shows the formula without @, as Excel does.

## Context
Owns: probes/legacy-index-match-one-row (every visibly-off cell).
- C9-C13, C16, C17: Excel's FORMULATEXT is `=INDEX($D$64:$O$64,MATCH(...))` (and ROUND(INDEX(..)) bare); ours adds `@` (`=@INDEX(...)`). The controls (B14 literal index bare, B15 cell index with @) agree. Pin the rule (when a legacy INDEX gets @ in FORMULATEXT) in docs/excel-semantics; compare probes/t3856-legacy-at-name-body.
- Source: (excel-graph fill, 2026-11-11, #4125 wave 2; Excel's value first, ours second).

## Acceptance criteria
- [ ] Every visibly-off cell of probes/legacy-index-match-one-row matches Excel, or its verdict is recorded in docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only behaviour (M14, decision 0039).
