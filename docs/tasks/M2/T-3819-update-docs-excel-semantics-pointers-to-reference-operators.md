---
id: T-3819
title: "Update docs/excel-semantics pointers to reference_operators.rs after the T-3439 split"
milestone: M2
lane: quality
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The "Implemented in" pointers in `docs/excel-semantics/` name the module's real files after T-3439 split `reference_operators.rs` into `reference_operators/mod.rs` and `reference_operators/areas.rs`.

## Context
Found by the excel-fidelity review of PR #2826 (T-3439). Stale pointers: references.md lines 14, 84, 86, 910, 918, 933, 1097, 1099; lookup.md:1184; text.md:164. `MAX_AREAS` (the DEVIATION at references.md:1097, decision 0014) now lives in `reference_operators/areas.rs`; `impl Model` stays in `mod.rs`. `rg 'reference_operators\.rs' docs/excel-semantics` finds them. The parser-side `expressions/parser/reference_operators.rs` is unchanged.

## Acceptance criteria
- [ ] No `docs/excel-semantics/` pointer names `crates/rustsheet-engine/src/reference_operators.rs` or a bare `reference_operators.rs` for the engine evaluator; each names `mod.rs` or `areas.rs`, whichever holds the item.
- [ ] The `MAX_AREAS` DEVIATION entry points at `reference_operators/areas.rs`.
- [ ] Docs-only change.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Repointed every engine-evaluator `reference_operators.rs` pointer in `docs/excel-semantics/` (references.md, lookup.md,
text.md) to `reference_operators/mod.rs`, or to `reference_operators/areas.rs` for `MAX_AREAS` (the DEVIATION entry),
`Binding::Areas`, `RefArg::Grid` and `RefGrid`. Also fixed `MAX_HELD_AREAS`/`VariableStack`, which live in
`variable_stack.rs`. Parser-side `expressions/parser/reference_operators.rs` pointers are untouched. Docs-only; no eval delta.
