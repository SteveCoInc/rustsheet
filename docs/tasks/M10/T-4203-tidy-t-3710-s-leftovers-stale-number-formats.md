---
id: T-4203
title: "Tidy T-3710's leftovers: stale number-formats.md lines and the duplicated parser hour comment"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/number-formats.md and the formatter parser agree with what T-3710 (PR #3134) found, with no stale statement left.

## Context
MINOR review findings on PR #3134 (T-3710), deferred after every reviewer approved:
- number-formats.md, entry "`mm` after a minute is a month": its Rule says every later `m`/`mm` in the section is a
  month whatever time token came before. probes/t3710-ampm-hour-tokens B3 (`h:mm|h:mm AM/PM` = "13:02|1:02 PM") and
  B14 show an hour token in between makes the next `mm` a minute again (parser `minute_read` reset). Say so and cite them.
- number-formats.md, the t3532 note (numeral digits, the hour bullet) still says which hours switch is unprobed and
  points at T-3710: point it at the "Hours with AM/PM" entry instead.
- crates/rustsheet-engine/src/formatter/parser.rs: the same three-line comment sits in the `Token::Hour` and
  `Token::HourPadded` arms; keep it once (or merge the arms with an or-pattern).
- Optional: formatter/test/test_dates.rs could also assert `AM/PM h h` ("PM 13 1") and `h AM/PM|hh|h` ("1 PM|13|1"),
  which the probe already covers.

## Acceptance criteria
- [ ] The two stale number-formats.md passages are corrected, citing t3710-ampm-hour-tokens B3 and B14.
- [ ] The duplicated parser comment appears once; no behavior change (t3710-ampm-hour-tokens still 26/26).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
