---
id: T-4002
title: "Debug of a Node loops over sign runs"
milestone: M2
lane: engine
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`format!("{node:?}")` of a formula holding a 1,023-sign run under the deepest nesting keeps
the same 4x stack headroom on a 1 MiB optimized stack as `Clone` and `PartialEq` have since
T-3253.

## Context
T-3253 hand-wrote `Clone` and `PartialEq` for `Node` (`node_walks.rs`) so they loop over runs
of signs, whitespace and parentheses; `Debug` stays derived and recurses once per node: 635 KiB
of stack for a spaced run at the cap (349 KiB unspaced), measured in T-3253. `Debug` only serves
diagnostics (test failure messages, logs), so this is P3. A hand-written `Debug` could print a
run's shells flat (prefix text in a loop, then the operand, then the closing braces), generated
from the same variant list as `clone_one`/`eq_one` in `node_walks.rs`; it must keep the derived
output format byte for byte, `{:#?}` included, or show that no test depends on it.

## Acceptance criteria
- [ ] A test in `test_nesting_depth.rs` formats such a tree with `{:?}` and `{:#?}` on a
      256 KiB release stack.
- [ ] The output for ordinary formulas is unchanged (a test compares it with the derived format
      on a few trees), or the change is justified.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any Excel-visible behavior.
