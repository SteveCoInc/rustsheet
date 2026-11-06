---
id: T-4494
title: "LAMBDA parameter ranges and unions: a:v between two parameters is their range (not columns A:V), (a,v) is a union, AREAS of a REDUCE result, and a union REDUCE initial value stays a reference"
milestone: M7
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The cells of probes/t4478-reduce-scan-reference-accumulator that T-4478 left failing pass. They cover:
- a range written between two LAMBDA parameters;
- a union of parameters;
- AREAS over a REDUCE call;
- a union as REDUCE's initial value.

## Context
- From T-4478 (excel-graph, 2026-11-06; docs/excel-semantics/arrays.md "REDUCE keeps a returned reference").
- `a:v`, where `a` and `v` are LAMBDA parameters, is the range between the two references they hold in Excel.
  `REDUCE(A1,A2:A3,LAMBDA(a,v,a:v))` is A1:A3 (3x1). The engine parses it as the column range A:V
  (1048576x22). The same probe with names that cannot be column letters (`p_a:p_v`, Calc!D20:H27) already passes.
  Cells: Calc!F5, H5, F6, G6, H6, F9, G9, H9, H14, G15, H15, J1, J2, J3.
  - In SCAN, step 2's `a` is a value, so `ROWS(a:v)` there is `#VALUE!` (H14 `{2;#VALUE!}`) and SCAN's
    `a:v` output is `#CALC!` (G15, H15).
  - The parser has to tell a LAMBDA/LET name from a column label inside the LAMBDA's scope.
- `AREAS(REDUCE(..))` is `#ERROR!` in ours, so the formula does not parse or evaluate. In Excel it is 3 for
  `REDUCE(A1,A2:A3,LAMBDA(a,v,(a,v)))` and for the `p_a` form, and 2 for `REDUCE((A1,A3),{1},LAMBDA(a,v,a))`.
  Cells: Calc!E7:H7, E16:H16, E22:H22.
- A union initial value stays a reference: `REDUCE((A1,A3),{1},LAMBDA(a,v,ISREF(a)))` is TRUE (Calc!H17).
  Ours reads it (`lambda_initial_value`, crates/rustsheet-engine/src/functions/logical/map_reduce.rs).

## Acceptance criteria
- [x] Every cell named above passes; no regressions.
- [x] arrays.md "REDUCE keeps a returned reference" and the `lambda_initial_value` comment are updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- SCAN's and REDUCE's accumulator rule itself (T-4478).

## Outcome
- The parser reads `a:v`, text the lexer took for a column range, as the range between two names an enclosing LET
  or LAMBDA binds (`Parser::bound_names_range`). Only when both sides are bound; `a:C` stays a column range (not
  probed, pinned by a unit test).
- AREAS accepts a REDUCE argument (`Parser::function_call`). REDUCE is not added to `Function::may_be_reference`:
  that would also make it an intersection, range and SUBTOTAL operand and change `yields_cells` (SUBTOTAL over TAKE
  of a value REDUCE would stop spilling), none of it probed (review round 1). In reference
  context it is `Model::reduce_reference`, which keeps a union accumulator as its areas from step to step and a
  union initial value as its areas (`lambda_initial_reference`); `fn_reduce` is its single-area view.
  `call_lambda_caller_binding` is the one step call both share.
- ISREF of a parameter bound to a union is TRUE (`Model::is_areas_variable`).
- `union_areas` charges each area it copies to the formula's work budget (`spend_cells`), so a union accumulator
  that grows a step at a time is bounded (`AREAS(REDUCE(A1,A2:A70000,LAMBDA(a,v,(a,v))))` is a fast `#VALUE!`,
  review round 1, security).
- Follow-up: T-4656 probes REDUCE as an intersection, range, call and SUBTOTAL operand, `a:v` between LET names and
  SCAN's union initial value.
- arrays.md "REDUCE keeps a returned reference" and the `lambda_initial_value` comment are updated.
- Eval delta: probes/t4478-reduce-scan-reference-accumulator 90/117 -> 117/117 (27 cells fixed: every cell the task
  names plus the F6:H6 and G9 forms); probes known failures 8312 -> 8285; 0 regressions in any suite.
- Not probed, left as is: SCAN with a union initial value; `a:C` with one bound name.

## Landing notes
- PR #3704, head after this commit (merge head 11c21881b). Not enqueued when the 2026-11-06 opus B run's window closed.
- State: CI never ran on e7b560227 because the PR conflicted with main (`mergeable_state: dirty`). The 18:53 merge of
  origin/main resolved one conflict in functions/logical/lambda.rs: T-2766 made `call_prepared`'s bind closure
  return `(Binding, bool)` (the ERROR.TYPE/CELL mark), and `call_lambda_caller_binding` now passes `false`, as
  main's `call_lambda_caller` did. `cargo test --lib -- reduce lambda` passed (154); `make check-fast` was not re-run
  after the merge, and CI is pending on the new head.
- Reviewers: security and excel-fidelity APPROVE on e7b560227, design APPROVE on 68735810e (round 1; round 2
  changed what security and fidelity asked for). The merge is a conflict resolution only and needs no new review.
- Next step: once CI is green on the latest head, enqueue (`enable_pr_auto_merge`, SQUASH). If eval-gate fails,
  check that probes/t4478-reduce-scan-reference-accumulator is still 117/117 after the merge.
- Do not repeat: putting REDUCE in `Function::may_be_reference` (fidelity BLOCKING: it changes `yields_cells`,
  the intersection, `:`, call and SUBTOTAL grammar, none of it probed); union copies without a work-budget charge
  (security BLOCKING: quadratic).
