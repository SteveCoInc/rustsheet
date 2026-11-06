---
id: T-4439
title: "Probe DEVSQ's overflow (1E+200 twice, +-1E+200, 1E+308 twice) and record it in statistics.md; word dispersion.py's _xlfn comment as the evidence shows"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's DEVSQ overflow behavior is evidenced and recorded in statistics.md, and the Gate A DEVSQ domain
(`deviations_overflow` in tools/src/rustsheet_tools/accuracy/defs/dispersion.py) matches it.

## Context
- T-4089 (PR #3465) gave DEVSQ a `#NUM!` domain inferred from its two-pass form (statistics.md "DEVSQ is a two-pass
  sum"): the sum of the numbers, a squared deviation, or the sum of the squared deviations beyond the double range.
  It has no oracle cell. Its unit test asserts `DEVSQ(1E+200,1E+200)` is 0 (no `#NUM!`), which is inferred too. No
  Gate A table point reaches either case, because the draws keep |x| <= 1E+150.
- The engine (crates/rustsheet-engine/src/functions/statistical/devsq.rs) follows the same two-pass form.
- The excel-fidelity review of #3465 (head 12bf9ab8a) also asked for the comment above `_draw_xlfn` in
  dispersion.py to be reworded. It says "a file holding a newer (`_xlfn.`) function with 255 arguments does not
  open", but the evidence (syntax.md "The repeating maxima", docs/research/t4089/xlfn-arguments.md) covers only
  VAR.S and STDEV.S.

## Probe
excel-probe skill, excel-graph. Use the case probes/t4439-devsq-overflow, with an `ERROR.TYPE` companion for each cell.
- `DEVSQ(1E+200,1E+200)`: two-pass gives 0, one-pass gives `#NUM!`.
- `DEVSQ(1E+200,-1E+200)`: the squared deviation 1E+400 overflows.
- `DEVSQ(1E+308,1E+308)`: the sum of the numbers overflows.
- `DEVSQ(1E+154,-1E+154,1E+154,-1E+154)`: the squared deviations are finite, but their sum may overflow.
- Controls: the same arguments in VAR.S.

## Acceptance criteria
- [ ] probes/t4439-devsq-overflow is committed with excel-graph oracle values and passes, or every failing cell has
      an owner task.
- [ ] statistics.md "DEVSQ is a two-pass sum" states the overflow rule and cites the cells.
- [ ] `deviations_overflow` and its unit test agree with the evidence. If the evidence differs, change them and
      regenerate nothing: the committed DEVSQ table has no point near overflow.
- [ ] The `_draw_xlfn` comment is limited to VAR.S and STDEV.S.
- [ ] `make check` green; reviewers approve.

## Out of scope
- VAR's overflow, which is already evidenced (`VAR.P` of 1E+200 twice is `#NUM!`).
