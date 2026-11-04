---
id: T-4245
title: "CJK numerals past 10^16: probe the open rest shapes and the 2^53 bracket (after T-3718)"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3718]
---

## Goal
Oracle evidence for the CJK elapsed-count forms that T-3718 left open, with the code and number-formats.md following
it.

## Context
- T-3718 (PR #3189) taught `formatter/numerals.rs` the 兆 sign and the 10^16 cut, from
  probes/t3718-cjk-numeral-unprobed rows 16-25. Row 21 (`[s]` of 1E11, 8.64E15) is spelled out and row 22 (`[s]` of
  1.2E11, 1.0368E16) is digit by digit. Its reviewers left these open (MINOR):
  - The cut also fits 2^53 (about 9.007E15). Probe `[s]` of 1.1E11 (9.504E15) under bytes 0x1B-0x1F. Spelled means
    10^16; digit by digit means a precision cut.
  - Past 10^16 only one shape of the rest below 兆 is probed (row 25, 0800億, where 30 and 31 write a gap zero). Probe
    an empty 億 group (for example a rest of 5万, `…兆○五万` under 30?) and a 兆 count that ends in zero.
  - A zero gap inside the 兆 group under 30 and 31 below 10^16 (for example 10^12 + 5).
  - The T-3378 value in a date section with an integer part (十/百 signs).
- Tidy-ups from the same reviews: merge the two Unprobed lists in number-formats.md's "CJK General and elapsed
  counts" paragraph into one, citing this task; rename the bindings in `Numerals::number`'s trillion arm that shadow
  `text` and `rest`.

## Acceptance criteria
- [ ] A probe case (excel-probe skill, excel-graph) covers the four forms under bytes 0x1B-0x1F. The code matches
      them, or the differences are recorded and filed.
- [ ] number-formats.md has one Unprobed list. The `number` arm's bindings no longer shadow.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `General` (it switches to scientific from 1E+11).
