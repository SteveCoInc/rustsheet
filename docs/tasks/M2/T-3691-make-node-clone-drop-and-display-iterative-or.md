---
id: T-3691
title: "Make Node Clone, Drop and Display iterative (or bound the node count) so small-stack debug embedders cannot abort"
milestone: M2
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
No formula the parser accepts can overflow a small stack in any build in the derived recursive walks of `Node`
(`Clone`, `Drop`, `Display`/`Debug`, `PartialEq`), or their node count is bounded so the at-limit tests fit a 2 MiB
debug stack.

## Context
- T-3338 measured: 256 levels of spaced `- ( ` around `B1` overflow a 2 MiB stack in an unoptimized build, in the
  derived `Clone` of the stored tree (formula_cells.rs `stored_formula`), about 4,000 frames for ~1,000 nodes; the
  existing 1,023-sign spaced runs (test_nesting_depth.rs `sign_runs_at_the_cap_survive_every_walk_under_the_deepest_nesting`)
  are the same. Release builds fit 1 MiB, the size the bounds are set for; debug builds need about 16 MiB for the
  heaviest spaced shapes (`on_small_stack`).
- An embedder running an unoptimized build on a small thread could abort (directive 5).
- Options: an iterative `Clone`/`Drop` for the unary/group/spaced wrapper chains (a manual impl that loops over
  `UnaryKind`, `UnaryPlus`, `Group`, `Spaced`), or a bound on wrapper nodes per path.
- Record the debug-stack requirement in docs/architecture (or syntax.md "Formula nesting depth cap") until fixed.

## Acceptance criteria
- [x] The heaviest shapes (spaced 256-level `-(`, 1,023 spaced signs, 200 levels plus 824 signs) set, clone,
      drop and print on a 2 MiB stack in a debug build; tests pin it.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing any accepted limit.

## Outcome
- Measured in an unoptimized build on a 2 MiB thread:
  - `Clone`, `PartialEq`, `Drop`, printing (`to_rc_format`), `strip_spaces` and static analysis already fit. T-3253's
    loops cover `Clone` and `PartialEq`.
  - Setting a formula overflowed for a run of 1,022 spaced signs before `B1`. The overflow was in
    `stringify::bound_variables::walk_names`, the LET/LAMBDA name walk `Model::set_user_input` runs through
    `shared_formula_index`, which recursed once per wrapper (gdb backtrace).
- `walk_names` now peels a run of signs, unary pluses, whitespace, parentheses and `@`/`#` wrappers in a loop before
  its per-variant match.
- New test `heaviest_wrapper_shapes_take_every_walk_on_a_2_mib_stack` (expressions/parser/tests/test_nesting_depth.rs).
  It takes each shape through parse, clone, eq, print, strip, analysis, drop, then set, evaluate and print in a model,
  all on 2 MiB. The shapes are: spaced 256-level `- ( `, 200 `-(` levels around 824 signs, and 1,022 signs bare,
  spaced and newline-spaced, plus their `-(...)` forms.
- `signed_parentheses_at_the_parenthesis_cap_evaluate_on_a_small_stack` now gives every row 2 MiB. Before, the heavy
  rows got 16 MiB unoptimized.
- Not done: the derived `Debug` still recurses per node and overflows 2 MiB on the spaced sign run. Follow-up T-4031,
  which also carries a MINOR doc fix from T-4013's review.
- Eval delta: none (no behavior change).
