---
id: T-4653
title: "Probe the ERROR.TYPE/CELL mark positions T-2766 left UNVERIFIED (IFERROR array fallback, TREND new_x order, mixed reference grids, CHITEST order)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2766]
---

## Goal
Every rule of statistics.md "ERROR.TYPE and CELL results are #NUM! in the paired functions" that the engine implements
is evidenced: the four positions T-2766 left UNVERIFIED are probed and the engine follows Excel there.

## Context
T-2766 (crates/rustsheet-engine/src/number_mark.rs) carries the mark through the wrappers and nested orders that
probes/t2766-mark-positions and t2766-mark-positions-2 evidence. statistics.md lists what it does unprobed, each a
choice by analogy:
- a marked array as IFERROR's fallback, `FTEST({1;2;3},IFERROR(1/0,ERROR.TYPE({#REF!;#N/A})))` (ours drops the mark,
  as IFERROR of an array does, t2766-mark-positions A27; a number fallback keeps it, -2 A24);
- TREND's and GROWTH's marked known_y against an error new_x, `TREND(ERROR.TYPE(#REF!),{1},1/0)` (ours `#VALUE!` first;
  LINEST's waits for known_x's error, -2 A21; `Model::read_known_y_and_x`, functions/statistical/regression.rs);
- an array of references mixing references and error elements as the regression family's second argument,
  `CORREL({1;2},INDIRECT("B1",{1;0}))` (ours: the reference no number, the `#REF!` kept; `PairedArg::without_numbers`,
  functions/statistical/paired.rs);
- which of CHITEST's `#NUM!` for a marked argument and an error element of the other comes first,
  `CHITEST({1;#N/A;3},ERROR.TYPE({#REF!;#DIV/0!;#VALUE!}))` (ours `#NUM!`; `SecondMark::NumError`).
Write one probe case (excel-probe skill, predictions committed before the fill), then make the engine match.

## Acceptance criteria
- [ ] One probe case covers the four positions above (excel-graph), all its cells pass.
- [ ] statistics.md has no UNVERIFIED line left for the mark's positions above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Why Excel marks these numbers at all.
