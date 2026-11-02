---
id: T-3689
title: "Score typed refusals Excel also makes (257 parentheses, 66 nested calls) as blank cells and restore the dropped typed-refusal probes"
milestone: M2
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A typed formula that Excel refuses (Graph PATCH 400 `InvalidArgument`, cell left blank) is refused by the API with
`TYPED_FORMULA_DOES_NOT_PARSE`, so the harness scores it as the blank cell Excel leaves, for the parenthesis cap and
the call cap as T-3338 did for the sign cap. The dropped typed-refusal probes are back and pass.

## Context
- T-3338 made the sign cap a non-limit parse error (`TOO_MANY_SIGNS_MESSAGE`, expressions/parser/depth.rs), so
  `typed_input.rs` reports it as not parsing and the eval adapter (crates/rustsheet-eval/src/engine.rs, decision 0179)
  keeps the cell blank. The 257-parenthesis refusal (`TOO_DEEP_MESSAGE` from `MAX_PARENTHESES`) and the 66-call refusal
  (`Parser::typed_refusal`, `MAX_NESTED_CALLS`) are still errors the harness scores as `EngineError` (decision 0168).
- Excel refuses both typed with PATCH 400 and a blank A1 (docs/excel-semantics/syntax.md "Formula nesting depth cap":
  probes/t3172-paren-257, t3266-nested-paren-257, t3097-nesting-*-66).
- The typed refusals T-3266 and T-3172 dropped because of this can be restored: the 1,024/1,025 sign runs,
  t3266-nested-200-825-typed, t3266-nested-50-975-typed, t3266-nested-paren-257-typed (their sidecars: no A1,
  `ISFORMULA(A1)` FALSE). The sign-cap ones need only the oracle files back, since T-3338.
- Keep `MAX_FORMULA_DEPTH` refusals that Excel would accept as limit errors (directive 5, decision 0014): only a
  refusal Excel also makes may become "does not parse".

## Acceptance criteria
- [x] Typed past the parenthesis cap or the call cap, `set_typed_user_input` errs with `TYPED_FORMULA_DOES_NOT_PARSE`
      and the cell keeps its content; unit tests pin both.
- [x] The dropped typed-refusal probes are restored (re-run the oracle where a sidecar is gone) and pass.
- [x] syntax.md updated; `make check` green; reviewers approve.

## Out of scope
- Stored refusals (Excel refuses the workbook; they keep no sidecar).

## Outcome
- The parenthesis cap is now Excel's syntax, not a resource limit: past `MAX_PARENTHESES` the parser's
  error is `TOO_MANY_PARENTHESES_MESSAGE` (depth.rs, `Parser::too_many_parentheses`, used by
  `Parser::parse_parenthesis_run`), which `is_limit_error` leaves out, as it leaves out the sign cap
  (T-3338). Typed, such a formula is refused as one that does not parse (`TYPED_FORMULA_DOES_NOT_PARSE`)
  and the cell keeps its content; stored (`set_user_input`) it loads as a parse error (`#ERROR!`), as a
  sign-cap formula does, where it was refused before. `MAX_FORMULA_DEPTH` stays a limit error (directive 5).
- The call cap: `Parser::typed_refusal` now prefixes `TOO_MANY_NESTED_CALLS_MESSAGE` with
  `TYPED_FORMULA_DOES_NOT_PARSE`, so the eval adapter scores the cell as the blank Excel leaves (decision
  0179). The xlsx importer's 66-call workbook refusal is unchanged.
- Unit tests: `typed_formulas_past_the_parenthesis_or_call_cap_are_refused_as_not_parsing`
  (test_typed_parse_errors.rs: 257 `(`, 257 `-(`, 66 `ABS(` refused, the cell kept; 256/65 accepted);
  the parenthesis-cap tests in test_nesting_depth.rs and test_nested_formula_stack.rs now pin the new
  message and the stored parse error; the eval runner's refusal-pin test uses spaced groups, which still
  meet the depth limit.
- 19 typed-refusal probes restored, each one past its committed accepted neighbour, filled from
  excel-graph on 2026-11-02 (every one PATCH 400 `InvalidArgument`, A1 blank, `ISFORMULA(A1)` FALSE):
  t3097-nesting-{abs,if,sum,mixed,op-abs,paren-abs,let,lambda}-66-typed, t3172-paren-257-, -minus-1025-,
  -plus-1024-, -alt-1024-typed, t3266-plus-num-1025-, -minus-ref-1024-, -minus-true-1024-,
  -minus-spaced-1024-, -nested-200-825-, -nested-50-975-, -nested-paren-257-typed. All pass.
- syntax.md "Formula nesting depth cap" updated (the rule, the restored cases, the API's refusal).

Eval delta: 19 new passing probe cases (19 cells, `S!B1`), 0 regressions, known failures unchanged (probes 3,881).
