---
id: T-4648
title: "trailing_digits: split the after-point branch into a helper; T-4594 review follow-ups"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`trailing_digits` (crates/rustsheet-engine/src/formatter/fraction.rs) reads as two small state machines, one for each side of the tail's `.`, with no change in output.

## Context
- T-4594 review (PR #3693, design and excel-fidelity MINORs): `trailing_digits` grew to about 100 lines with two nested
  state machines sharing `due`, `pending` and `lacking`; its doc comment repeats number-formats.md "Placeholders after a
  fraction" in one long paragraph; docs/research/t4594/README.md does not record the fifth hold-out's result (392/392);
  the unit test `placeholders_after_a_fraction_with_a_point` asserts `TEXT(12,"# ?/1000|?.#")` without citing
  probes/t4515-fresh-sweep F5.
- Pure refactor: every t4594-* and t4515-* probe case must stay fully passing.

## Acceptance criteria
- [ ] The after-point branch is a helper taking a small state struct; the doc comment is short and points to
      number-formats.md.
- [ ] The README records "the fifth hold-out matched 392 of 392"; the unit test cites t4515-fresh-sweep F5.
- [ ] No eval change (`make eval-check` 0 regressions, no baseline change).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any behavior change, or the uncovered shapes (`?/7|.0`, a second `.`).
