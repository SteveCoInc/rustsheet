---
id: T-4430
title: "Legacy @ of a defined name: probe one-element, 1-D and out-of-range INDEX over array bodies, #REF! and #DIV/0! bodies, +3 and --3 bodies"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-4273]
---

## Goal
The legacy-formula `@` of a defined name is evidenced for the body kinds T-4273's rules cover but did not probe.

## Context
- T-4273 (PR #3458) probed and implemented, in references.md "A defined name takes the `@` by its body"
  (probes/t4273-legacy-at-name-body-more):
  - `-3` and `#N/A` bodies read as their literal, without the `@`;
  - `INDEX(n2d,1,1)` over n2d = `{1,2;3,4}` is one element.
- The code applies those rules more widely than the evidence (review MINORs on #3458):
  - `is_array_body` (static_analysis/name_body.rs) and `indexes_one_reference_cell` (if_arm.rs) make every
    three-argument INDEX over an array-constant body a scalar. Unprobed: a one-element `{3}`, a one-row `{3,4}`,
    a one-column `{3;4}`, and out-of-range indexes (`INDEX(n2d,5,5)`). Also unprobed and kept with the `@`: one
    index into a 1-D body (`INDEX(nArrC,1)`), and an area number.
  - `scalar_body` reads every error literal as a literal; only `#N/A` was probed. `#REF!` matters most, since
    Excel writes `=#REF!` into a name whose range is deleted. Also probe `#DIV/0!`.
  - `+3` and `--3` bodies fall through to the `@` (unprobed).
- Use t4273's layout and generator (docs/research/t4273/gen_probe.py) for a new case
  `t4430-legacy-at-name-body-rest`. Commit the engine's predictions before the fill.

## Acceptance criteria
- [ ] The new probe is filled on excel-graph. Every cell passes, or each off cell is fixed in this task or recorded in
      references.md and owned.
- [ ] references.md states the probed rule for each body kind above (cells cited).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The redraw question (needs a harness name-edit op).

## Outcome
- New probe `probes/t4430-legacy-at-name-body-rest` (181 scored cells, excel-graph, 2026-11-06), generator
  `docs/research/t4430/gen_probe.py` (helpers shared with T-4273's). First fill: 174 pass, 7 off, all INDEX over an
  array-constant name: the IF arm with one index kept no `@` (Excel keeps it), and the area number `INDEX(n2d,1,1,1)`
  got an `@` (Excel none). Fixed in `static_analysis/if_arm.rs` (`one_index_into_array_body`, 3-4 argument
  `indexes_one_reference_cell`); now 181/181. The engine's predictions were not committed before the fill (deviation
  from the task text).
- Probed and recorded in references.md ("A defined name takes the `@` by its body"): `#REF!` and `#DIV/0!` bodies read
  as literals; `+3` reads as the literal, `--3` takes the `@`; one index into `{3}`, `{3,4}` and `{3;4}` keeps it, a
  row and column index (or an area number, or out of range) is one element.
- Eval delta: probes +181 passing cells in 1 new case, no regressions, baseline lists the new case.
