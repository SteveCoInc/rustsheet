---
id: T-4928
title: "A LET or LAMBDA omission in ISEVEN and ISODD follows Excel (probe the ToolPak #N/A rule)"
milestone: M5
lane: functions-logic
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
ISEVEN and ISODD, which began as Analysis ToolPak functions, give what Excel gives when their argument is a LET value
or LAMBDA parameter bound to nothing (`LET(e,,ISEVEN(e))`, `LAMBDA(a,b,ISODD(b))(1,)`). A small probe decides, with
its predictions committed before the fill.

## Context
- The rule under test: a fixed-arity ToolPak function reads a written-empty argument, or a LET or LAMBDA variable bound
  to nothing, as `#N/A`. Evidence: `LET(e,,QUOTIENT(7,e))` and `LAMBDA(a,b,QUOTIENT(a,b))(7,)` (numbers.md "A
  written-empty argument of a ToolPak function is #N/A", probes/t4873-empty-arg-let-lambda A1:A4), and
  `LET(e,,EDATE(1,e))` (dates.md, T-4870). The non-ToolPak IS functions are different: `ISNUMBER`, `ISBLANK` and the
  others take any value. ISEVEN and ISODD sit between the two groups. Their checks follow the ToolPak functions (a
  boolean is `#VALUE!`; information.md "ISEVEN/ISODD over a range"), so only Excel can say which group they follow.
- A one-argument function cannot be written with an empty argument, so the LET and LAMBDA forms are the only way to
  reach the rule. No eval cell uses them (gap triage 2026-11-07, `make eval` on origin/main 1e0e3f023).
- Ours today (scratch eval, unverified): `LET(e,,ISEVEN(e))` is TRUE, `LET(e,,ISODD(e))` and
  `LAMBDA(a,b,ISODD(b))(1,)` are FALSE, and `LET(e,,ISNUMBER(e))` is FALSE. The omission is read as 0.
- Code: `lifted_parity`/`parity_of` (functions/information/mod.rs). Tell the omission by its node with
  `Model::empty_arguments` and `written_empty_or`/`written_empty_na` (functions/util/empty_args.rs). Never copy them
  (directive 3).
- How it avoids fitting: each probe cell is `#N/A`, another error, or a boolean. The fill decides, and nothing numeric is
  tuned.

## Acceptance criteria
- [ ] A probes case (excel-probe skill, excel-graph) has these rows. Rule rows: `LET(e,,ISEVEN(e))`, `LET(e,,ISODD(e))`,
      `LAMBDA(a,b,ISEVEN(b))(1,)`, `LAMBDA(a,b,ISODD(b))(1,)`. Lift and operator rows: `LET(e,,ISEVEN(e+0))` and
      `ROWS(LET(e,,ISEVEN(IF({1;1},e))))`. Non-ToolPak controls: `LET(e,,ISNUMBER(e))`, `LET(e,,ISBLANK(e))` and
      `LET(e,,TYPE(e))`.
- [ ] The predictions are committed in a commit before the oracle fill: `#N/A` for the four rule rows, and today's
      answers for the rest.
- [ ] After the fill every probe cell passes. information.md gets an entry stating the rule with its cells, or, if
      Excel reads the omission as 0, saying so (then no engine change is made).
- [ ] No new F line in any eval/baseline file; `make check` green; reviewers approve.

## Out of scope
- SQRTPI and FACTDOUBLE (functions-math, T-4923).
- Empty text as an ISEVEN/ISODD argument (T-2872).
