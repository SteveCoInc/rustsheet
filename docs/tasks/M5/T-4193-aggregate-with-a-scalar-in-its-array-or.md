---
id: T-4193
title: "AGGREGATE with a scalar in its array or reference argument, under every option, and a range against a later error argument: probe and match"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
AGGREGATE gives Excel's value or error in three unprobed cases. First, its data argument is a scalar (a number, an
error, text or a boolean) instead of a reference or an array, in the reference form (functions 1-13) and in the
array form (14-19). Second, the scalar is an error under each option, including the ignore-errors options 2, 3, 6
and 7. Third, a range holding an error comes before a later error argument, and the task finds which error wins.

## Context
- Gap triage 2026-11-04, opus pass (`make eval` on main 5941bb95). Every visibly-off cell has an open owner, and the
  functions-math lane has no ready task (its remaining tasks are deferred last-digit fits). docs/excel-semantics/
  numbers.md "AGGREGATE" ends: "Also unprobed: a scalar in the array form (the engine takes a number as one value,
  returns a scalar error under every option, and gives `#VALUE!` for text or a boolean), and which error wins between
  a range and a later error argument." AGGREGATE is M5 (catalog `prior` 0.3) and covered by Gate A (eval/accuracy/
  tables/AGGREGATE), so only the semantics are open.
- Already evidenced (probes/aggregate-args Misc, excel-graph, T-0993): `AGGREGATE(9,0,"3")` A18, `(9,0,TRUE)` A19,
  `(9,0,Data!E1,"3")` A20, `(4,0,-1,Data!G1)` A49, `(9,6,Data!D1:D11,1/0)` A34 and `(9,6,Data!D1:D11,NA())` A46 (an
  error later argument under option 6), `(14,6,Data!E1:E6,NA())` A14 (an error k). Do not re-probe these. Cite them.
- Unprobed, to probe:
  - A plain number as the data argument: `AGGREGATE(9,0,5)`, `(1,0,5,7)`, `(4,0,-1)`, `(14,0,5,1)`, `(15,6,5,1)`,
    `(16,0,5,0.5)`, `(19,0,5,1)`.
  - A scalar error as data under options 0 through 7: `AGGREGATE(9,k,1/0)`, `(9,k,NA())`, `(14,k,NA(),1)`,
    `(14,k,1/0,1)`, and a one-cell reference to an error cell (`Data!D7`) against the literal error, because a
    one-cell reference is a reference.
  - Text and booleans in the array form: `AGGREGATE(14,0,"3",1)`, `(14,0,TRUE,1)`, `(14,6,"x",1)`.
  - Which error wins: with option 0, `AGGREGATE(9,0,Data!D1:D11,NA())` (D7 = `#DIV/0!`), the reverse order,
    `(9,0,NA(),Data!D1:D11)`, and two literal errors `(9,0,1/0,NA())` and `(9,0,NA(),1/0)`. Under option 4
    (ignore nothing) and option 5 (hidden rows only), these must not change.
  Wrap each error in `ERROR.TYPE`. Add an `ISREF`/`TYPE` control where a scalar might come back as an array.
- Ours: `fn_aggregate` and `aggregate_scalar` (crates/rustsheet-engine/src/functions/aggregate.rs), as the numbers.md
  sentence above describes. The scalar path shares its error handling with SUBTOTAL's readers in
  crates/rustsheet-engine/src/functions/subtotal.rs. If a fix is needed there, change that shared helper (directive 3).
- Related open tasks, not duplicated: T-4068 (AGGREGATE 14-19 Gate A domain), T-1461 and T-1195 (perf),
  T-2862 (M5 math argument kinds, research), T-4194 (the nested-call skip, same module; whichever lands second
  rebases).

## Acceptance criteria
- [x] A probes case (excel-probe skill, excel-graph, canary-verified, tag M5) holds the unprobed cells above.
- [x] numbers.md "AGGREGATE" states the scalar and error-order rules with the new cells, and the "Also unprobed"
      sentence is removed or narrowed to what stays unprobed.
- [x] The engine matches every new cell. probes/aggregate-args, aggregate-grid, aggregate-array-args,
      aggregate-range-args, aggregate-negative-option-range-k and subtotal-hidden-filtered still pass.
- [x] Eval known failures only shrink (`make eval-baseline`, 0 regressions); `make check` green; reviewers approve.

## Out of scope
- Hidden and filtered rows (evidenced), AGGREGATE's k rounding (T-1034's entry), Gate A numerics (T-4068), and the
  nested SUBTOTAL/AGGREGATE skip (T-4194).

## Outcome
- New probe probes/t4193-aggregate-scalar-data-error-order (94 cells, excel-graph 2026-11-04, M5): a number as data
  (reference form `#VALUE!`, array form one value), literal and one-cell-reference errors under options 0-7, text and
  booleans in the array form, `TYPE` controls, and error order between a range and a direct error and between two
  literal errors under options 0-7.
- Findings in docs/excel-semantics/numbers.md "AGGREGATE": a direct error argument beats a range's error in either
  order, the first of two literal errors wins, a literal error is never skipped, and a text or boolean scalar in the
  array form is skipped (`#NUM!`). The "Also unprobed" sentence is replaced; what stays unprobed is named.
- Engine: `aggregate_array_form` skips a text or boolean scalar instead of returning `#VALUE!` (3 cells, Calc!A8:A10);
  the other 91 cells already matched.
- Eval delta: +94 checked cells, all passing (the 3 fixed cells were never in the baseline: new case). aggregate-args,
  aggregate-grid, aggregate-array-args, aggregate-range-args, aggregate-negative-option-range-k and
  subtotal-hidden-filtered still pass.
