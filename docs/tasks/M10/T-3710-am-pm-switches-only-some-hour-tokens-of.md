---
id: T-3710
title: "AM/PM switches only some hour tokens of a section to the 12-hour clock"
milestone: M10
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(45000.5432,"h|hh|mm|ss|h:mm:ss.00 AM/PM")` prints "13|13|02|12|1:02:12.48 PM" as Excel does: only some hour tokens of a section with `AM/PM` take the 12-hour clock.

## Context
Found by T-3532: probes/t3532-numeral-digit-fields row 10, all columns (numeral bytes 2-31; the quirk is not about digits, A10 is "١٣|١٣|٠٢|١٢|١:٠٢:١٢.٤٨ PM"). We switch every hour token ("1|01|..."). Semantics: number-formats.md "Numeral-system digits", the hour bullet. Probe first which hours switch (the hour nearest the marker, the hours after the last `|`-like literal, or those in the same time run as the marker) with ASCII codes (`h|h AM/PM`, `h AM/PM|h`, `h:mm|h:mm AM/PM`). Code: formatter/format.rs `TextToken::Hour`/`HourPadded` (`p.use_ampm`).

## Acceptance criteria
- [x] probes/t3532-numeral-digit-fields row 10 passes (S!A10:G10, 7 cells); a probe covers the ASCII forms.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Done (general rule). Probed 36 hour/marker layouts (probes/t3710-ampm-hour-tokens): the last hour before each
`AM/PM`/`A/P` and the section's last hour take the 12-hour clock, the others stay 24-hour
(`elapsed::twelve_hour_tokens`, replacing `DatePart::use_ampm`). The probe also showed that an hour read after a minute
makes the next `mm` a minute again (`h:mm|h:mm`), fixed with the parser's `MinuteSlot` state machine. number-formats.md "AM/PM
switches only some hours" and "`mm` after a minute is a month". Eval: t3532-numeral-digit-fields S!A10:G10 pass, the
new case 36/36.
