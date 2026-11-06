---
id: T-4556
title: "PERCENTRANK.EXC: match Excel's last digit at 16 or more digits of significance"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
PERCENTRANK.EXC gives Excel's value, to the last digit, where its rank is cut to 15 digits at a significance of 16 or more.

## Context
- T-4093's Gate A table found two cells where the engine is one unit off in the 15th digit. Both are inside the envelope, so neither is a failing point.
  - `PERCENTRANK.EXC({0,3},2,17)` (eval/accuracy/tables/PERCENTRANK.EXC/s1 A128 and A140): Excel 0.555555555555556,
    engine 0.555555555555555. At 16 digits (A126) both give 0.555555555555556.
  - The rank is 5/9. The engine forms it as `(2/3 + 1)/3` in doubles, which is 0.55555555555555547. Scaled by 1E+17
    and cut to 15 digits (`cut_to_15_digits`), that gives ...555. Excel's double rank must lie above it, as the
    correctly rounded 5/9 (0.55555555555555558) does.
  - PERCENTRANK and PERCENTRANK.INC match Excel on every T-4093 table cell, including `PERCENTRANK({0,3},2,17)`
    0.6666666666666671 (t2473 D22).
- The semantics are in statistics.md, "PERCENTRANK ranks x by the count of values below it (T-4093)".
- The code path is `percentrank_exc_impl` in crates/rustsheet-engine/src/functions/statistical/percentile.rs.
- A probe that varies n, x and the significance (16-20) over EXC would show Excel's operation order. Use the excel-probe skill.

## Acceptance criteria
- [ ] A probe records Excel's PERCENTRANK.EXC last digit over several n, x and significances of 16 or more, and the
      entry states the fitted operation order.
- [ ] The engine is bit-exact with Excel on PERCENTRANK.EXC/s1 A128 and A140 and on the probe's cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PERCENTRANK and PERCENTRANK.INC, which already match.
