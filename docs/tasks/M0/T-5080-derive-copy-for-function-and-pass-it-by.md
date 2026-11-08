---
id: T-5080
title: "Derive Copy for Function and pass it by value (the four lift/fold sites T-4981 left on a reasoned expect)"
milestone: M0
lane: quality
status: todo
priority: P3
size: M
tier: sonnet
depends_on: [T-4981]
---

## Goal
`Function` (crates/rustsheet-engine/src/functions/mod.rs) is a fieldless enum that derives `Copy`, and the
four `needless_pass_by_value` expects T-4981 left on a `kind: Function` parameter are gone.

## Context
T-4981 removed the fork's file-level `needless_pass_by_value` debt. Four sites take `kind: Function` by value
without consuming it: `Model::lift_call_costed` and `Model::lift_call_beside` (functions/lift/call.rs),
`Model::fold_complex` (functions/engineering/complex/mod.rs) and the test helper `amortization_units`
(functions/lift/tests/kernel_work.rs). They carry a statement-level `#[expect(clippy::needless_pass_by_value)]`
naming this task. Taking `&Function` would touch every caller's `Function::X` constant; the clean fix is
`#[derive(Copy)]` on `Function`. T-4981 tried that and found it makes about 75 existing `&Function` parameters
fail `trivially_copy_pass_by_ref` (a 2-byte type) and turns many `kind.clone()` calls into `clone_on_copy`
failures, so it is its own change: derive `Copy`, pass `Function` by value where it is taken by reference,
drop the `.clone()`s (`cargo clippy --fix` applies those), then delete the four expects.

## Acceptance criteria
- [ ] `Function` derives `Copy`; no `&Function` parameter is left in the engine.
- [ ] The four T-5080 expects are deleted; no new lint expect or allow is added.
- [ ] `make eval-check` shows 0 regressions and 0 fixes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `token::Error` (also fieldless and not `Copy`; same cascade, separate change).
