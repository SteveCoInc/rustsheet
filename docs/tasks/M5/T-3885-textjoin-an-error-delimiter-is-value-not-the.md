---
id: T-3885
title: "TEXTJOIN: a text ignore_empty is #VALUE! even when the delimiter is an error"
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
TEXTJOIN returns #VALUE! for a text ignore_empty even when the delimiter is an error value; an error delimiter with a numeric, boolean, blank or error ignore_empty keeps its own error.

## Context
Found by the T-3512 arg-pairs batch (`make eval-show CASE=gen-m5/d0c5e5dff134d787.case.json`). `TEXTJOIN(<#DIV/0! ref>,<text ignore_empty>,<values>)`: Excel `#VALUE!`, ours `#DIV/0!`. Do not change the passing cells: A286, A292, A298, A304, A328, A334 (number, boolean, blank or error ignore_empty) stay `#DIV/0!`; A349 and A352 (text ignore_empty, blank delimiter) are `#VALUE!` and pass. Rule recorded in docs/excel-semantics/text.md.
- d0c5e5dff134d787: Calc!A310, A316, A322.

## Acceptance criteria
- [ ] d0c5e5dff134d787 Calc!A310, A316 and A322 leave eval/baseline/gen-m5.txt; no new known failure.
- [ ] The probed rule is in docs/excel-semantics/text.md.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Every cell this task owns has the right type, error and text (decision 0083).

## Out of scope
- Other TEXTJOIN argument kinds (T-2863).
