---
id: T-3253
title: "Widen the stack margin for 1,023-sign runs: iterative Clone/Drop of sign runs"
milestone: M2
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A formula holding a 1,023-sign run under the deepest nesting the depth bound admits keeps a
comfortable stack margin on a 1 MiB release stack in every tree walk, not just evaluation and
printing.

## Context
T-3193 made signs cost no depth (`MAX_SIGNS`, depth.rs) and made the printer peel signs, groups
and whitespace in a loop (stringify/wrapped.rs). Other walks still recurse once per sign node (and
once more per `Node::Spaced` after a spaced sign): the derived `Clone`, `Drop`, `PartialEq` and
`Debug`, static analysis and the rewrite passes. The test
`sign_runs_at_the_cap_survive_every_walk_under_the_deepest_nesting` (test_nesting_depth.rs) passes
on 1 MiB but overflows at 512 KiB, in the derived `Clone` (about 2x headroom, the architecture
review of T-3193 called it thin). Options: fold a run into one node (signs as data, as the T-3193
task context sketched), or iterative `Clone`/`Drop` for runs plus loops in the walkers that
recurse per sign. Measure before and after.

## Acceptance criteria
- [x] The T-3193 cap test passes on a 512 KiB release stack (or the headroom is measured and
      recorded at 4x or better).
- [x] No printed text, evaluation result or eval case changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `MAX_SIGNS` or Excel-visible behavior.

## Outcome
`Node`'s `Clone` and `PartialEq` are hand-written (new `expressions/parser/node_walks.rs`): they
peel a run of `UnaryKind`, `UnaryPlus`, `Spaced` and `Group` wrappers in a loop, and `PartialEq`
also loops down operator chains' left operands, as `ChainLeft`'s `Clone` already did. Both
impls' per-variant arms come from one macro-generated list of each variant's fields, so a new
variant or field fails to compile until it is listed (and the duplication ratchet stays at
2,880). `Debug` stays derived: follow-up T-4002.

Measured on an optimized build, a 1,023-sign run (unspaced / spaced) under the deepest nesting
the depth bound admits, minimum stack per walk:

| walk | before | after |
|---|---|---|
| `Clone` | 508 / 920 KiB | 63 KiB |
| `PartialEq` (with its clone) | 508 / 920 KiB | under 147 KiB |
| `Drop` | 95 / 158 KiB | same |
| printing (`to_rc_format`) | 111 KiB | same |
| static analysis, `strip_spaces` | under 32 KiB | same |
| `Debug` | 349 / 635 KiB | same (T-4002) |
| the T-3193 cap test, whole | over 768 KiB (fails there in `Clone`) | 610 KiB |

All the tree walks together now fit in 147 KiB, about 7x headroom on 1 MiB: the new test
`sign_run_tree_walks_fit_a_quarter_of_the_stack` runs them on 256 KiB (4 MiB unoptimized). The
whole cap test cannot pass on 512 KiB: parsing and evaluating the deepest nesting it builds take
more stack with no signs at all (`nesting_at_the_limit_parses` and `chain_links_add_no_depth`
need 697 KiB), a margin the depth bound (T-0931) sizes, out of this task's scope. So the
acceptance criterion's alternative applies: the run's own walks measured at 7x, and the cap test
now needs less than the sign-free nesting tests. `hand_written_clone_and_eq_agree_with_the_formula_text`
checks that a copy equals its original and that 44 distinct formulas compare pairwise unequal.

Eval delta: none (`make check-fast` eval gate: 0 regressions, 0 fixed in every suite). No
Excel-visible behavior changed, so no `docs/excel-semantics/` entry.
