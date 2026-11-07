---
id: T-4482
title: "Chained LAMBDA calls with the wrong argument count at a later link: LAMBDA(x,LAMBDA(y,x+y))(1)(), (1)(2,3) and an outer link with too many, probe and match"
milestone: M7
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A chained LAMBDA call (`f(..)(..)`, T-2038) whose later or earlier link has too few, too many or no arguments gives
Excel's error kind, in a cell and through ERROR.TYPE, IFERROR, ISERROR and a LAMBDA helper, as evidenced on
excel-graph.

## Context
- docs/excel-semantics/references.md, the chained-calls entry: "`LAMBDA(x,LAMBDA(y,x+y))(1)()` gives an error the
  oracle could not decode (unprobed)". The bare cell returns a rich error type the oracle client cannot decode, so
  the probe must read it through wrappers. T-1224 did this for the one-link form: `=LAMBDA(f,f(2))()` is a call with
  too few arguments, `ERROR.TYPE` 3 (`#VALUE!`), and IFERROR catches it (probes/let-name-call-forms H15:H17).
- Every chained form in eval/suites has the right argument count at each link: probes/chained-calls-lambda,
  chained-calls-name-curried and lambda-escaping-closures (`LAMBDA(a,LAMBDA(b,a*b))(3)(4)`,
  `LAMBDA(x,x)(1)(2)`, three-link curries). No case calls a later link with too few, too many or no arguments, or
  gives an earlier link too many.
- Ours: each link is a call of the LAMBDA value the previous link returned (expressions/parser/lambda.rs, T-2038;
  `Model::call_prepared`, crates/rustsheet-engine/src/functions/logical/lambda.rs). The arity check is whatever that
  path does for a single call, and it is unprobed whether Excel reports the inner link's count the same way, or
  reports an outer link's extra argument first.
- Probe (excel-probe skill, excel-graph, tag M7). Read each form through `ERROR.TYPE(..)`, `IFERROR(..,"e")` and
  `ISERROR(..)`, and through `MAP({1;2},LAMBDA(v,ERROR.TYPE(<form>)))`. A bare copy goes in a cell the case leaves
  unscored, in case the oracle decodes it:
  - a later link with no arguments: `LAMBDA(x,LAMBDA(y,x+y))(1)()`; with too many: `LAMBDA(x,LAMBDA(y,x+y))(1)(2,3)`;
  - an outer link with too many: `LAMBDA(x,LAMBDA(y,x+y))(1,2)(3)`; with none: `LAMBDA(x,LAMBDA(y,x+y))()(3)`;
  - an error argument beside the bad count: `LAMBDA(x,LAMBDA(y,x+y))(1/0)()`, `LAMBDA(x,LAMBDA(y,x+y))(1)(NA(),2)`;
  - through a LET name and a defined name: `LET(f,LAMBDA(x,LAMBDA(y,x*y)),f(3)())`, `MYFN(3)(4,5)` with `MYFN` =
    `=LAMBDA(x,LAMBDA(y,x*y))`;
  - an optional parameter at the later link, to separate "too few" from "omitted": `LAMBDA(x,LAMBDA(y,[z],
    x+y+IF(ISOMITTED(z),0,z)))(1)(2)` and `..(1)()`. If Excel for the web refuses that workbook (as it refuses
    `=LAMBDA([f],f(2))()`, T-1224), record the refusal and drop the form.
- Write the engine's values as predictions before the fill (docs/research/t4482/).

## Acceptance criteria
- [x] probes/t4482-chained-call-arity (excel-graph, canary-verified) covers every loadable form in Context through
      the wrappers. Its predictions are committed before the fill, and each refused form is named in the Outcome.
- [x] Every scored cell passes, or each one that does not is recorded in references.md and owned by a filed task
      named in the Outcome.
- [x] references.md: "gives an error the oracle could not decode (unprobed)" is replaced by the evidenced rule, with
      cells cited. A unit test pins each wrong-count link.
- [x] No regressions in any baseline; the chained-calls-* and let-name-call-forms cases still pass; `make check`
      green; reviewers approve.

## Out of scope
- Chained calls of built-ins (`IF(TRUE,LAMBDA(..))(3)`, T-2405) and calls of non-LAMBDA values (`LET(f,{1,2},f(2))`).
- Decoding the rich error type in the oracle client (harness lane).

## Outcome
- Probes probes/t4482-chained-call-arity (10 forms, 50 cells) and probes/t4482-chained-call-arity-optional (3 forms, 15
  cells), filled on excel-graph 2026-11-06 (canary-verified fill). Excel for the web loaded both workbooks, the
  optional-parameter one included, so no form was refused. The engine's predictions
  (docs/research/t4482/predictions.md) were committed before the fill.
- Excel: a wrong argument count at any link is `#VALUE!` (ERROR.TYPE 3, caught by IFERROR, ISERROR and inside MAP). That covers a
  later link with none or too many, an outer link with too many or none, a LET name and a defined name. The count wins over an error
  argument (`(1/0)()` and `(1)(NA(),2)` are `#VALUE!`). An optional parameter at a later link may be omitted
  (`..(1)(2)` is 3), but `..(1)()` is `#VALUE!`. The bare cells decode as `#VALUE!`, so column E is scored too.
- Engine: already matched every cell (65 of 65), so no code change was needed. references.md replaces "an error the oracle
  could not decode (unprobed)" with the rule and its cells. Unit test
  `a_chained_call_with_a_wrong_count_at_a_link_is_value_error` pins each link.
- Eval delta: probes +2 cases, all passing; 0 regressions.
- Also files T-4596 (MINORs from the #3610 T-4469 review).
