---
id: T-4066
title: "Lowercase day halves for en-IE, en-BZ and en-HK hex LCIDs (1809 2809 3C09)"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`[$-1809]`, `[$-2809]` and `[$-3C09]` print lowercase day halves ("6 pm|6 pm|6 pm") as Excel does.

## Context
probes/text-locale-invalid-sublanguage S!J3, S!K3, S!L3 (found by T-3666); ours prints "6 PM". Code: formatter/locale_tag.rs LOCALES tables.

## Acceptance criteria
- [ ] S!J3, S!K3, S!L3 pass; `make check` green.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
