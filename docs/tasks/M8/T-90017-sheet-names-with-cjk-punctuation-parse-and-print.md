---
id: T-90017
title: "Sheet names with CJK punctuation parse and print as Excel does, quoted and unquoted"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Every visibly-off cell matches Excel, or its verdict is recorded (corpus rows #15-#17 of the requesting repository wait on this).

## Context
- excel-graph fill, 2026-11-13, probe request val11 (#4128). Owns: probes/sheet-names-cjk-punctuation, probes/sheet-names-cjk-punctuation-unquoted-circled-digit-first, probes/sheet-names-cjk-punctuation-unquoted-corner-brackets, probes/sheet-names-cjk-punctuation-unquoted-double-angle-brackets, probes/sheet-names-cjk-punctuation-unquoted-double-arrow, probes/sheet-names-cjk-punctuation-unquoted-middle-dot (every visibly-off cell of each). Run `make eval-show CASE=<case>` for the cells, and score them against the request's frozen predictions where it has them.

## Acceptance criteria
- [ ] Every visibly-off cell of the owned cases matches Excel, or its verdict is recorded in docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only behaviour (M14, decision 0039).
