---
id: T-4923
title: "A LET or LAMBDA omission in SQRTPI and FACTDOUBLE follows Excel (probe the ToolPak #N/A rule)"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
SQRTPI and FACTDOUBLE, the two single-argument Analysis ToolPak math functions, give what Excel gives when their
argument is a LET value or LAMBDA parameter bound to nothing (`LET(e,,SQRTPI(e))`,
`LAMBDA(a,b,FACTDOUBLE(b))(1,)`). A small probe decides, with its predictions committed before the fill.

## Context
- The rule under test: every fixed-arity ToolPak function probed so far reads a written-empty argument as `#N/A`. That
  covers MROUND, QUOTIENT, SERIESSUM, RANDBETWEEN, GCD/LCM/MULTINOMIAL (first argument) and IMSUB/IMDIV, and the date
  family of T-4870. The same holds when the argument is a LET or LAMBDA variable bound to nothing:
  `LET(e,,QUOTIENT(7,e))` and `LAMBDA(a,b,QUOTIENT(a,b))(7,)` are `#N/A` (numbers.md "A written-empty argument of a
  ToolPak function is #N/A", probes/t4873-empty-arg-let-lambda A1:A4), and so are `LET(e,,EDATE(1,e))` and
  `LAMBDA(a,b,EDATE(a,b))(1,)` (dates.md, T-4870).
- A one-argument function cannot be written with an empty argument (`SQRTPI(,)` has two). So the LET and LAMBDA forms
  are the only way to reach the rule, and no eval cell uses them for SQRTPI or FACTDOUBLE. This is a blind spot, not a
  failing cell (gap triage 2026-11-07, `make eval` on origin/main 1e0e3f023).
- Ours today (scratch eval, unverified): `LET(e,,SQRTPI(e))` and `LAMBDA(a,b,SQRTPI(b))(1,)` are 0, and
  `LET(e,,FACTDOUBLE(e))` is 1, because the omission is read as 0. The variadic ToolPak functions already follow the
  rule: `LET(e,,GCD(e,4))`, `LET(e,,MULTINOMIAL(e,4))` and `LAMBDA(a,b,LCM(a,b))(,4)` are `#N/A`, while
  `LET(e,,GCD(4,e))` and `LAMBDA(a,b,LCM(a,b))(4,)` are 4, a later empty argument being skipped (coercion.md,
  probes/gcd-lcm-multinomial-error-order).
- Code: `fn_sqrtpi` (functions/math_and_trigonometry/mathematical.rs) and `fn_factdouble`
  (math_and_trigonometry/combinatorics.rs) both read through `lift_call`. Reuse `Model::empty_arguments` with
  `written_empty_or`/`written_empty_na` (functions/util/empty_args.rs), or `Model::atp_number` (functions/lift/mod.rs),
  the T-4873 reader. Never copy them (directive 3).
- How it avoids fitting: each probe cell is `#N/A`, another error kind, or the value the engine gives today. The fill
  decides which, and no numeric constant is involved.

## Acceptance criteria
- [x] A probes case (excel-probe skill, excel-graph) has these rows. Rule rows: `LET(e,,SQRTPI(e))`,
      `LAMBDA(a,b,SQRTPI(b))(1,)`, `LET(e,,FACTDOUBLE(e))`, `LAMBDA(a,b,FACTDOUBLE(b))(1,)`. Lift rows:
      `ROWS(LET(e,,SQRTPI(IF({1;1},e))))` and `LET(e,,SQRTPI(e+0))` (does an operator turn the omission into 0?). Control
      rows: `LET(e,,GCD(e,4))`, `LET(e,,GCD(4,e))`, `LAMBDA(a,b,LCM(a,b))(4,)` and `LET(e,,MULTINOMIAL(e,4))`.
- [x] The predictions are committed in a commit before the oracle fill: `#N/A` for the four rule rows, and today's
      answers for the rest.
- [x] After the fill every probe cell passes. numbers.md "A written-empty argument of a ToolPak function is #N/A"
      names SQRTPI and FACTDOUBLE with their cells, or states that they read the omission as 0 if Excel says so (then
      no engine change is made).
- [x] No new F line in any eval/baseline file; `make check` green; reviewers approve.

## Out of scope
- RANDBETWEEN's LET form: the harness skips volatile cells.
- ISEVEN and ISODD (functions-logic, T-4928).
- MROUND of a formula-returned blank and the newer roundings (T-4901).

## Outcome
- **Finding:** SQRTPI and FACTDOUBLE follow the `ToolPak` rule. A LET value or LAMBDA parameter bound to nothing is
  `#N/A`. An operator over it (`e+0`) or a blank cell reads as 0.
- **Evidence:** probes/t4923-sqrtpi-factdouble-omitted, 13 cells, excel-graph, 2026-11-07. Predictions were committed in
  819775fd6, before the fill. H_na scored 13/13 and H_zero (the engine before this change) 9/13. The GCD, LCM and
  MULTINOMIAL controls passed as the engine already gave them. Nothing is fitted: each cell is `#N/A` or the value the
  engine already gave.
- **Code:**
  - `fn_sqrtpi` reads through `Model::atp_number`, which also folds its boolean `#VALUE!`.
  - `fn_factdouble` swaps its argument through `written_empty_or`.
  - Both take the empty flag from `Model::empty_arguments::<1>`.
  - Unit test: test/test_sqrtpi_factdouble_omitted.rs.
- **Docs:** numbers.md "A written-empty argument of a ToolPak function is #N/A" now names SQRTPI and FACTDOUBLE with
  their cells. It also fixes the entry's stale path for `empty_arguments`, which T-4870 moved to
  functions/util/empty_args.rs.
- **Eval:** one new case, 13/13.
