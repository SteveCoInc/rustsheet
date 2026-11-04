---
id: T-3718
title: "Probe the unprobed CJK numeral forms (large integers, padded elapsed, myriad zero gap, General with literals)"
milestone: M10
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence, and the matching implementation, for the CJK numeral forms T-3532 left unprobed.

## Context
T-3532 implemented numeral-system digits (formatter/numerals.rs). number-formats.md "Numeral-system digits" lists these as unprobed: integers of 10^12 and more (reached through elapsed counts; we print them digit by digit, `MAX_SPELLED`), a padded elapsed count under 10 (`[$-1B000409][hh]` on 0.2; we print "〇四"), a zero gap across a myriad under bytes 30 and 31 (`[$-1E000409]General` on 100005 and 100000005; we write one zero), and `General` beside literals or in a date section (`[$-1B000409]General "x"` on 105, `[$-1B000409]d General` on 5.5). Write one probe case (excel-probe skill), fill it with excel-graph, and fix whatever differs.

## Acceptance criteria
- [x] A probe case covers the four forms with oracle values; the code matches them, or the differences are recorded and filed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe case `probes/t3718-cjk-numeral-unprobed` (100 cells, excel-graph values; rows 16-20 added and the case
refilled on 2026-11-04 after the steward's excel-fidelity review) covers the four forms. Findings recorded in
docs/excel-semantics/number-formats.md. Rows 1-15 already matched. Rows 16-20 (elapsed counts of 10^11 and more)
showed that Excel spells 10^12 with 兆 under every CJK byte and, past 10^16, writes the count of 兆 digit by digit
before 兆; `formatter/numerals.rs` now does the same (`myriads` gains 兆, `MAX_SPELLED` 10^16 - 1, unit test
`spells_trillions_with_zhao`). Adopted as an orphan PR (#3189) by an opus run. Eval delta: probes new case, 100/100
passing; no regressions.
