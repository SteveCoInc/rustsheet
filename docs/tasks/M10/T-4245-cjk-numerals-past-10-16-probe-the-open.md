---
id: T-4245
title: "CJK numerals past 10^16: probe the open rest shapes and the 2^53 bracket (after T-3718)"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: opus
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

- Start from branch `agent/functions-text/T-3718-cjk-rows-21-25` (commit c656531). A parallel adoption of #3189
  wrote it, and all three reviewers approved that tree (round 2), but another session's head f2b72f0 merged instead.
  It adds rows 21-25 to probes/t3718-cjk-numeral-unprobed (`[s]` of 1E11, 1.2E11, 1E12, 1.2345E11; `[h]` of 1E15;
  excel-graph 2026-11-04). They pin the cut: 8.64E15 is spelled "八千六百四十兆", 1.0368E16 is "一〇三六八兆". They also
  pin a nonzero rest after a digit-by-digit 兆 count: 27 "一〇六六六兆八百億", 30 "一○六六六兆○八百亿", 31
  "壹零陆陆陆兆零捌佰亿", so 30 and 31 write one gap zero after 兆. The branch has the matching numerals.rs change
  (the trillion arm), unit test rows, and number-formats.md text. Bring that delta onto `main`: merge `origin/main`
  in and keep the branch's side where #3189's merged text is a subset. Then add the probes listed above.

## Acceptance criteria
- [ ] A probe case (excel-probe skill, excel-graph) covers the four forms under bytes 0x1B-0x1F. The code matches
      them, or the differences are recorded and filed.
- [ ] number-formats.md has one Unprobed list. The `number` arm's bindings no longer shadow.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `General` (it switches to scientific from 1E+11).

## Outcome
- Merged `agent/functions-text/T-3718-cjk-rows-21-25` (c656531) onto `main`: the trillion arm of
  `Numerals::number`, its unit test rows, rows 21-25 of probes/t3718-cjk-numeral-unprobed (now 125/125), and the
  number-formats.md text. Conflicts were resolved to the branch's side; the unit tests keep both sides' rows.
- New probe probes/t4245-cjk-numeral-past-1e16 (excel-graph, 35 cells, bytes 0x1B-0x1F). The cut is 10^16, not
  2^53 (9.504E15 is spelled). An empty 億 group past 10^16 takes one gap zero under 30 and 31. A count of 兆
  ending in zero keeps its zero digit. A zero gap inside or after the 兆 group below 10^16 is one zero under 30
  and 31. The T-3378 value with an integer part is spelled with the tens sign. All 35 cells match the existing
  code, so only the docs change.
- Tidy-ups: number-formats.md's "CJK General and elapsed counts" paragraph has one Unprobed list, citing T-4245.
  The trillion arm's bindings are renamed (`below`, `count`) and no longer shadow.
- Eval: probes known failures 4528 -> 4528 (+30 checked cells in t3718, +35 in the new case, all passing), 0
  regressions.

