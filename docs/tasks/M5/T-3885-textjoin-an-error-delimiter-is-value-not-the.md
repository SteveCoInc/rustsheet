---
id: T-3885
title: "TEXTJOIN: an error delimiter is #VALUE!, not the delimiter's own error"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
TEXTJOIN returns #VALUE! when the delimiter is an error value, as Excel does.

## Context
Found by the T-3512 arg-pairs batch (`make eval-show CASE=gen-m5/d0c5e5dff134d787.case.json`). `TEXTJOIN(<#DIV/0! ref>,<scalar>,<values>)`: Excel `#VALUE!`, ours `#DIV/0!`. Probe whether this holds for every error kind and when the ignore_empty argument or the values are errors too, then record it in docs/excel-semantics/text.md.
- d0c5e5dff134d787: Calc!A310, A316, A322.

## Acceptance criteria
- [ ] d0c5e5dff134d787 Calc!A310, A316 and A322 leave eval/baseline/gen-m5.txt; no new known failure.
- [ ] The probed rule is in docs/excel-semantics/text.md.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Every cell this task owns has the right type, error and text (decision 0083).

## Out of scope
-
