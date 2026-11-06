---
id: T-4639
title: "The volatile-cell detector sees eta-form calls (_xleta.RANDARRAY), so t4519 D7:H8 can be scored as invariant"
milestone: M7
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4519]
---

## Goal
The eval harness treats a formula that names a volatile function in eta form (`_xleta.RANDARRAY`, `_xleta.RAND`,
`_xleta.RANDBETWEEN`, as GROUPBY/PIVOTBY/BYROW write it) as volatile, the same as a direct call, so such cells are
either unscored or listed as `invariant`, never scored by accident.

## Context
- `RANDOM` in `crates/rustsheet-eval/src/volatile.rs` matches only `(?:_xlfn\.)?(RAND|RANDBETWEEN|RANDARRAY)\s*\(`.
  An eta argument such as `GROUPBY(..,_xleta.RANDARRAY)` has no `(` after the name, so the cell counts as non-volatile.
- Found by T-4519's excel-fidelity review (PR #3622). The rows 7-8 cells of
  `probes/t4519-row-column-randarray-over-arrays` (D7:E7, G7:H7, D8:E8, G8:H8) use `_xleta.RANDARRAY`. They are scored
  today only through this gap. They read shape and type only, so their values are stable. But once the detector is fixed,
  they drop out of scoring unless they are listed in the case's `invariant`, and `spec_volatile_cells` refuses an
  `invariant` cell it does not see as volatile.
- T-4611 owns D7, D8, E8, G8 and H8; their ownership must not be lost.

## Acceptance criteria
- [ ] `is_volatile` returns true for an eta-form RAND, RANDBETWEEN or RANDARRAY argument (`_xleta.NAME` not followed by
      `(`), with unit tests for the direct, `_xlfn.` and `_xleta.` forms and a non-volatile lookalike.
- [ ] t4519 D7:E7, G7:H7, D8:E8 and G8:H8 move to the case's `invariant` list and stay scored, with the same pass and
      fail results (`make eval-check`: 0 regressions; T-4611's owned cells still owned).
- [ ] Any other case whose scored cells change because of the fix is listed in the Outcome, and each change is handled
      without losing coverage.
- [ ] `make check` green; reviewers approve.

## Out of scope
- RANDARRAY's lift itself (T-4611).
