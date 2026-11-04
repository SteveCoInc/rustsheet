---
id: T-4002
title: "Debug of a Node loops over sign runs"
milestone: M2
lane: engine
status: done
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
- [x] A test in `test_nesting_depth.rs` formats such a tree with `{:?}` and `{:#?}` on a
      256 KiB release stack.
- [x] The output for ordinary formulas is unchanged (a test compares it with the derived format
      on a few trees), or the change is justified.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any Excel-visible behavior.

## Outcome
- `Node`'s `Debug` is hand-written in node_walks.rs: `fmt_one`, generated from the same variant list as
  `clone_one`/`eq_one`, prints one node as the derived impl did; `{:?}` prints a run of signs, unary pluses,
  whitespace and parentheses in a loop (each wrapper's head, the node under them, then ` }` per wrapper) through the
  caller's formatter, so every flag reaches the values. `{:#?}` loops for a run of 32 wrappers or more through one
  `Indent` writer that stands for std's nested pad adapters; a shorter run, or `{:#?}` with width, precision, sign or
  fill flags, recurses per node exactly as before. The one difference left: a `{:#x?}` hex flag does not reach the
  values of a long run (std has no stable way to read it).
- Tests: `debug_keeps_the_derived_format` pins `-+ (A1)` against the derived output captured before the change and
  builds the derived form of 1 to 40 nested levels from it (both paths, `{:?}` and `{:#?}`, also mid-line under an
  operator, a call and an `@`, and with width/precision flags). `sign_run_tree_walks_fit_a_quarter_of_the_stack`
  now formats the deepest sign-run trees with `{:?}` and `{:#?}` on its 256 KiB release stack, and T-3691's
  `heaviest_wrapper_shapes_take_every_walk_on_a_2_mib_stack` formats every shape (its `TODO(T-4031)` is gone).
- Also closes T-4031 (the same change) and its carry-overs: syntax.md's nesting paragraph, the 1,023-sign count in
  T-3691's test doc and `walk_names`'s comment, and `walk_names`'s unreachable wrapper arms folded into the leaf arm.
  The collation `fold` carry-over was already done on main.
- Eval delta: none (no Excel-visible behavior).
