---
id: T-4468
title: "Near-integer snap at the upper limit: SUBSTITUTE instance_num, FIXED/DOLLAR decimals, CHAR and FIND start just below their caps; tidy T-4453's citations"
milestone: M5
lane: functions-text
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's answer is on record for each snapping M5 text argument at a value just below its upper cap. Since T-4453 the
engine snaps these values up to the cap and answers `#VALUE!`, where the old floor accepted them. T-4453's citation
nits are fixed in the same PR.

## Context
- T-4453 (PR #3485) made FIND/SEARCH start_num, CHAR, SUBSTITUTE instance_num and FIXED/DOLLAR decimals snap within
  2^-22 + 2^-33 below an integer (probes/t4453-near-integer-text-args). The probe covered only interior bases. At the
  cap, the snap now gives `#VALUE!` with no Excel cell behind it:
  - `SUBSTITUTE(t,"a","b",2147483647-2^-23)` (2^31-1 is rejected; text.md "SUBSTITUTE instance_num" accepts up to
    2147483646)
  - `FIXED(1,128-2^-23)` and `DOLLAR(1,128-2^-23)` (128 is `#VALUE!`)
  - `CHAR(256-2^-23)` (256 is `#VALUE!`)
  - `FIND("a","abc",5-2^-23)` (a start past LEN+1)
  The existing REPLACE 32767.9999999 caveat in text.md is the same kind of open question.
- Excel-fidelity review MINORs on #3485, both docs-only:
  - char_code.rs cites "D1:E21". UNICHAR's column ends at E14, so it should read "D1:D21, E1:E14".
  - text.md "DOLLAR and FIXED decimals": the Evidence and Tasks lines should add
    probes/t4453-near-integer-text-args G1:J21 and T-4453.

## Acceptance criteria
- [ ] A committed excel-graph probe case covers each cap above: n - 2^-23 (snaps) and n - 2^-22 - 9*2^-36 (does not).
- [ ] text.md's "LEFT, RIGHT and REPLACE round counts up near an integer" entry records the result. If Excel differs
      from the engine, the engine is fixed and every probe cell passes.
- [ ] Both citation nits above are fixed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Interior bases (T-4453 has them).
- TEXTBEFORE/TEXTAFTER and REGEX occurrence arguments (M7).
