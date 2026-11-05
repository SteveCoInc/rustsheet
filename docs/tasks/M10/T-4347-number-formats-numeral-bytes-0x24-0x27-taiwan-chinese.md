---
id: T-4347
title: "Number formats: numeral bytes 0x24-0x27 (Taiwan Chinese, traditional formal, byte-29 variant, Korean hangul)"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`[$-24000409]` to `[$-27000409]` (and the `,2400` to `,2700` suffixes) print Excel's numerals: Chinese with ０ for zero, traditional formal, a byte-29 variant, and Korean hangul.

## Context
T-4107 found numeral bytes 0x20-0x27 are CJK systems, not ASCII, and implemented 0x20-0x23. Failing cells: probes/t4107-numeral-2000-suffix A22, A42:A69 (expected values are in the oracle sidecar; the rules are in docs/excel-semantics/number-formats.md "Bytes 32-39"). Code: `Numerals::from_byte` (formatter/numerals.rs). Probe date fields and elapsed tokens first (excel-probe); Korean needs its own digit and unit tables.

## Acceptance criteria
- [ ] The cells above pass and a date/clock/elapsed probe for each byte is evidenced.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
