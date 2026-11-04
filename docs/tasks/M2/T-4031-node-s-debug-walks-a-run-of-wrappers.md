---
id: T-4031
title: "Node's Debug walks a run of wrappers in a loop, so a 1,022-sign spaced run formats on a 2 MiB debug stack"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3691]
---

## Goal
`format!("{:?}", node)` for the heaviest wrapper shapes fits a 2 MiB stack in an unoptimized build, like every other
walk over `Node`. An embedder that logs a parsed tree on a small debug thread then cannot abort (directive 5).

## Context
- T-3691 pinned the heaviest shapes in `heaviest_wrapper_shapes_take_every_walk_on_a_2_mib_stack`
  (expressions/parser/tests/test_nesting_depth.rs): set, evaluate, clone, compare, print and drop. It moved
  `walk_names` to a loop. `Node`'s `Debug` is still derived and recurses once per node. A run of 1,022 spaced signs
  before `B1` (about 2,000 `UnaryKind`/`UnaryPlus`/`Spaced` wrappers) overflows 2 MiB unoptimized (measured in
  T-3691). The test leaves `Debug` out with a `TODO(T-4031)`.
- Approach: a hand-written `Debug` that prints a wrapper run in a loop, as node_walks.rs does for `Clone` and
  `PartialEq`. It keeps the derived output byte for byte (tests may compare `{:?}` text) and uses the per-variant
  field list that node_walks.rs generates, so a new variant cannot be forgotten.

- Carry-over (T-4013 review, MINOR): `fold`'s doc comment in functions/util/collation.rs still calls the `<` order of
  class members UNVERIFIED. probes/t4013-class-equality-order Calc!C1:E18 verifies it, so cite that and drop the
  word.

- Carry-over from T-3691's review (PR #3092, MINOR findings):
  - docs/excel-semantics/syntax.md's nesting paragraph still says the heaviest shapes need about 16 MiB unoptimized
    and that "T-3691 makes those walks iterative". Since T-3691 every walk fits 2 MiB unoptimized except `{:?}`;
    say so, and drop the `{:?}` exception once this task lands.
  - The run in T-3691's test doc, its Outcome note and the `walk_names` comment is 1,023 signs (`MAX_SIGNS - 1`,
    `MAX_SIGNS` = 1024), not 1,022.
  - `walk_names`'s wrapper arms after its peel loop are unreachable. Fold them into the leaf arm with a comment, or
    share one `&mut` wrapper-child accessor with `Node::shell` (node_walks.rs).

## Acceptance criteria
- [ ] The T-3691 test formats every shape with `{:?}`. Its `TODO(T-4031)` is gone.
- [ ] `{:?}` output is unchanged for a sample of formulas (a test pins it against the derived form).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any accepted limit.
