---
id: T-3089
title: "TEXT: aaa as the first token before b after B2[40409] gives code 51, not ddd's 15"
milestone: M10
lane: functions-text
status: done
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(45000,"B2[$-40409]aaa|b")` is "週三|23" as in Excel (b's second character is code 51), and `aaaa` first gets its
own code once probed.

## Context
Found by T-3028 (docs/excel-semantics/number-formats.md, Taiwan bullet under "`B2` before a locale tag and calendar
digit 6"). After `B2[$-40409]` a short Minguo year's `b` prints a character set by the section's first date token
(`calendar_dates::first_token_code`). `aaa` lexes as `TextToken::DayNameShort`, whose code is `ddd`'s 15, but Excel
gives 51 (probes/t3028-b2-taiwan-cuts A65). The token needs to remember that it was written `aaa` (or `aaaa`; probe
its code first) without changing what it prints anywhere else.

## Acceptance criteria
- [ ] probes/t3028-b2-taiwan-cuts C65 passes and leaves eval/baseline/probes.txt; `aaaa` is probed and matched.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any other effect of `aaa` (T-1880).

## Outcome
`aaa`/`aaaa` now lex to their own tokens and map to `TextToken::DayNameShort(true)` / `DayName(true)`; `first_token_code` gives
51 and 52 (probed: probes/t3089-b2-aaa-code A1:A6, excel-graph 2026-11-04; `aaaa`, `AAAA` and `aaaAAA` are 52). Printing unchanged.
probes/t3028-b2-taiwan-cuts A65 and C65 leave eval/baseline/probes.txt.
