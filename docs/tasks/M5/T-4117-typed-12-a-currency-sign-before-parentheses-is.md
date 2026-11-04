---
id: T-4117
title: "Typed $(12): a currency sign before parentheses is a negative number in Excel ($(١٢) is -12), text in ours"
milestone: M5
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Text `$(12)` (a currency sign, then a parenthesised amount) typed into a cell, and read by coercion if Excel does,
becomes the number Excel makes of it.

## Context
probes/t3891-typed-script-digit-shapes B29:C29 (excel-graph 2026-11-04): typed `$(١٢)` is -12 with the long
currency format `[$$-ar-SA,200] #,##0.00_);[Red]([$$-ar-SA,200]#,##0.00)`; the engine keeps it text because
the ASCII reading `$(12)` fails `parse_formatted_number` (formatter/parse_formatted.rs; T-3891 maps the
script digits and reuses the ASCII parser). The ASCII form is unprobed: probe first (excel-probe) typed
`$(12)`, `$(1,234.5)`, `$ (12)`, `($12)`, `(12$)`, and `"$(12)"+0`, VALUE and NUMBERVALUE of each, with the
ENTERED formats.

## Acceptance criteria
- [ ] probes/t3891-typed-script-digit-shapes B29:C29 and the new probe's cells pass; the rule is in
      docs/excel-semantics/coercion.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Script-digit tags (T-3891).
