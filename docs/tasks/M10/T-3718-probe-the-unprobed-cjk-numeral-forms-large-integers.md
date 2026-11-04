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
Probe case `probes/t3718-cjk-numeral-unprobed` (95 cells, excel-graph values) covers the four forms. Rows 1-15 (75 cells)
matched the existing implementation. Rows 16-19 (added when the orphaned PR was adopted, after an excel-fidelity review
asked for elapsed counts of 10^11 and more) showed that counts from 10^12 are spelled with 兆 in every style, not digit
by digit: `formatter/numerals.rs` gains 兆 as a third myriad and spells up to 10^16 - 1 (beyond is unprobed). Findings
recorded in docs/excel-semantics/number-formats.md (the 10^12 claim limited to `General`; the 億 zero gap is per style).
Eval delta: +95 passing probe cells (15 of them fixed by the 兆 change), no regressions.
