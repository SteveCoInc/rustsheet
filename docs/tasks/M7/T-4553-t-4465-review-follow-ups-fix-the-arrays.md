---
id: T-4553
title: "T-4465 review follow-ups: fix the arrays.md section title cited in sequence.rs, the eps23 miss range, and probe SEQUENCE extents near 1048576 with the near-integer offsets"
milestone: M7
lane: functions-math
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The citations and numbers T-4465 left slightly wrong are right, and SEQUENCE's near-integer rule is evidenced at the top
of its extent range, next to 1048576.

## Context
- PR #3547 (T-4465) review round 2, MINOR findings (security, excel-fidelity):
  - crates/rustsheet-engine/src/functions/math_and_trigonometry/sequence.rs (the `extent` closure comment) and random.rs
    (the RANDARRAY row/column comment in `fn_randarray`, if any) cite the arrays.md section as "SEQUENCE and RANDARRAY
    read an extent near an integer"; the heading is "Array-constructor extents near an integer".
  - docs/excel-semantics/arrays.md, that section's Evidence line: "a 2^-23 epsilon 16 to 22 per column" should read
    "16 to 21 per column on the fixed cells" (docs/research/t4465/predictions.tsv).
  - The fresh sweep (probes/t4465-extent-sweep) drew bases up to 185976 only; 1048576 itself is evidenced only by
    probes/empty-array-args-sequence near 1048577. The rule is `near_integer_whole` (an absolute bound), unprobed between
    about 2E5 and 1048576.

## Acceptance criteria
- [ ] The code comments cite the real arrays.md heading.
- [ ] arrays.md's Evidence line gives the eps23 range as recomputed from predictions.tsv.
- [ ] A probe case (excel-probe skill, excel-graph, tag M7) steps `ROWS(SEQUENCE(1048576-x))`,
      `COLUMNS(SEQUENCE(1,1048576-x))` and `ROWS(SEQUENCE(524288-x))` by x = k*2^-30 for k in {96, 128, 160, 256, 257,
      264, 288, 320} and by -x (1048576+x is `#VALUE!`), and every cell passes; arrays.md cites it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- RANDARRAY past 16384 columns (the engine's array-size cap; a separate question).
