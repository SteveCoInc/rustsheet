---
id: T-4594
title: "Placeholders after a fraction with a decimal point among them: one rule for the digits on each side of the tail's point"
milestone: M10
lane: functions-text
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Digit placeholders after a fraction's denominator with a `.` among them print as Excel does, by one rule for the
placeholders on each side of that point, extending the T-4515 model.

## Context
- The T-4515 model (number-formats.md "Placeholders after a fraction") fits every tail without a `.` (1,300+ cells and
  its hold-out) but not a tail with one. Before the point a `?` seems to take every spilled digit, not one: `# ?/7 ?.?`
  on 0.0123 is " 123/7 0000. " (probes/t4515-fraction-tail-holdout E6). After the point the owed zeros are not paid and
  `#` prints differently: `# ?/7 ?.0` on 0.0123 is " 123/7 0000.0" (one zero, not two), `# ?/1000|?.#` on 0.0123
  " 123/1000|0." but on 0.123456789 " 10/10  |0.0", `# ?/40|?.0#` on 0.0123 " 123/40|000.00"
  (probes/t4515-fresh-sweep rows 1-8, excel-graph 2026-11-06).
- A "take all before the point, owe one" rule was tried in T-4515 and falsified by that sweep. Treat the tail as a
  small number section: the spilled digits as its integer, the lacking digits and owed zeros as its decimals.
  Hold-out: a fresh probe written after the model is frozen.
- Code: `trailing_digits` in crates/rustsheet-engine/src/formatter/fraction.rs.

## Owned cases
- probes/t4515-fraction-tail-holdout E6
- probes/t4515-fresh-sweep A1:G3, A5:G6, A8:G8

## Acceptance criteria
- [x] The owned cells pass, and every T-4515 case still passes, with no regression in any suite.
- [x] A hold-out probe made after the model froze passes.
- [x] number-formats.md "Placeholders after a fraction" states the point rule with its cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- Commas after the decimal point; a literal right after a number section's point (T-4595).

## Outcome
- `trailing_digits` (formatter/fraction.rs) treats a `.` among the placeholders after a fraction like a number's
  point. Before it, the last `?` takes every spilled digit, and zeros are left *pending*: a `?` that prints a space
  adds one, a spilled digit clears them, a `0` or `#` leaves one plus one per placeholder before it (except a `?` that
  printed a lacking zero or a zero for nothing due), and a later `?` that prints a zero for nothing due takes one.
  After the point nothing spills or is paid. A lacking zero prints first, then a pending zero outside a run of blanks,
  then a blank while a space is due (a space for `?`, nothing for `#`, a zero for `0`), which leaves one more pending.
  A blank's run ends at a literal or a placeholder of another kind. number-formats.md "Placeholders after a fraction"
  states the rule with its cells; the T-4594 unit test is `placeholders_after_a_fraction_with_a_point`.
- Evidence (excel-graph 2026-11-06), all in docs/research/t4594:
  - two exploration grids: t4594-fraction-tail-point-explore (320 cells) and -explore-wide (648 cells);
  - five hold-outs, each with the engine's predictions committed before the fill: -holdout 192/210,
    -holdout-2 199/252, -holdout-3 271/280, -holdout-4 332/336, -holdout-5 392/392.
  Each miss was refitted, and the next hold-out was fresh. This is a structural model (rules chosen from ranked
  candidates, no numeric constant), so it carries no `FITTED` tag and Gate A does not apply (decision 0188 R4).
- Eval delta: probes known failures 7650 -> 7624 (26 fixed: t4515-fresh-sweep 25 and t4515-fraction-tail-holdout
  E6, the owned cells among them), 0 regressions in any suite. The seven new probe cases add 2,438 cells, all passing.
- Not covered: a tail whose integer side has no placeholder (`?/7|.0`), and a second `.`.
