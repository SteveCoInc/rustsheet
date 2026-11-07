---
id: T-4873
title: "An empty argument of QUOTIENT and SERIESSUM, and an empty significance of CEILING.MATH, FLOOR.MATH and the PRECISE roundings, follow Excel"
milestone: M5
lane: functions-math
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A written-empty argument of the ToolPak math functions QUOTIENT and SERIESSUM, and a written-empty `significance` or
`mode` of CEILING.MATH, FLOOR.MATH, CEILING.PRECISE, FLOOR.PRECISE and ISO.CEILING, give what Excel gives, decided by
a small probe whose predictions are committed before the fill.

## Context
- Blind spot: the generator never writes an empty argument (`call_args` in tools/src/rustsheet_tools/gen/generator.py
  only drops trailing optionals), and no eval cell calls these functions with one (gap triage 2026-11-07, `make eval`
  on origin/main 3d991bd3d).
- Two rules to test, each with Excel evidence in sibling functions:
  - ToolPak functions give `#N/A` for a written-empty argument: `MROUND(,1)` and `MROUND(5,)` (arrays.md "MROUND keeps
    its scalar rules", probes/ceiling-floor-mround-array-lift), `RANDBETWEEN(5,)` (numbers.md, probes/volatile-invariant
    Calc!I5:I6), the first argument of GCD, LCM and MULTINOMIAL (coercion.md, probes/gcd-lcm-multinomial-negative-
    precedence E40:E61), `IMSUB("3+4j",)` (engineering.md). QUOTIENT and SERIESSUM are ToolPak functions
    (arrays.md "The other Analysis ToolPak-era functions").
  - Functions added since Excel 2010 read an empty optional as omitted: TAKE/DROP (arrays.md "TAKE and DROP with an
    omitted count"), SORT's sort_order (arrays.md "SORT: omitted sort_index / sort_order take their defaults"). If
    that holds for CEILING.MATH/FLOOR.MATH (2013) and CEILING.PRECISE/FLOOR.PRECISE/ISO.CEILING (2010), an empty
    significance is 1, not 0.
- Ours today (scratch eval, unverified): `QUOTIENT(,5)` 0, `QUOTIENT(5,)` and `QUOTIENT(,)` `#DIV/0!`;
  `SERIESSUM(2,,1,{1,2})` 5, `SERIESSUM(2,1,,{1,2})` 6, `SERIESSUM(2,1,1,)` 0; `CEILING.MATH(2.675,,0.1)` 0,
  `CEILING.MATH(2.5,,)` 0, `FLOOR.MATH(-5,,{0,1})` 0, `CEILING.PRECISE(2,)` 0, `ISO.CEILING(1.005,)` 0,
  `FLOOR.PRECISE(2.5,)` 0. The defaults in `numbers_fn` (functions/math_and_trigonometry/mathematical.rs) apply only to
  an absent argument; an `EmptyArg` is cast to 0.
- Code: `fn_quotient` (mathematical.rs, via `two_numbers`), `fn_seriessum` (seriessum.rs), `fn_ceiling_math`,
  `fn_floor_math`, `fn_ceiling_precise`, `fn_iso_ceiling` (ceiling_floor.rs, all through `numbers_fn`). MROUND's
  evidenced `#N/A` lives in `fn_mround` (mathematical.rs) and the IMSUB/IMDIV one in `complex_pair_of`
  (engineering/complex/mod.rs); T-4870 and T-4872 need the same test, so reuse or extract one helper (`rg
  'EmptyArgKind' crates/rustsheet-engine/src/functions`), never a copy (directive 3).
- How it avoids fitting: each cell is `#N/A`, the default-argument result or the 0-argument result; the probe decides,
  nothing numeric is tuned.

## Acceptance criteria
- [x] A probes case (excel-probe skill, excel-graph) writes each argument of QUOTIENT and SERIESSUM empty (one at a
      time and all), and the significance and mode of CEILING.MATH and FLOOR.MATH and the significance of
      CEILING.PRECISE, FLOOR.PRECISE and ISO.CEILING empty, on a positive, a negative and a fractional number, plus a
      `LET(e,,QUOTIENT(7,e))` row and CEILING(2.5,) / FLOOR(2.5,) as legacy controls.
- [x] The engine's predictions (`#N/A` for the ToolPak pair, empty = omitted for the five rounding functions) are
      committed in a commit before the oracle fill.
- [x] After the fill every probe cell passes; numbers.md states the rule with its cells.
- [x] No new F line in any eval/baseline file; `make check` green; reviewers approve.

## Out of scope
- MROUND, GCD, LCM, MULTINOMIAL, RANDBETWEEN (already evidenced and implemented).
- AGGREGATE and SUBTOTAL (their empty-argument cells are already in probes and pass).

## Predictions (committed before the oracle fill)
Cases `probes/t4873-empty-arg-quotient-ceiling` (Calc) and `probes/t4873-empty-arg-let-lambda` (Calc).
- ToolPak rule: every QUOTIENT and SERIESSUM cell with a written-empty x, n, m, numerator or denominator is `#N/A`
  (main A1:A8, A10, B1:B3, B5:B7), except that an error argument before the empty one wins (A9 `#DIV/0!`). B4 (empty
  coefficients) is the open question: predicted `#N/A`. B8 control: 10.
- Empty = omitted for the 2010+ roundings: C1 3, C2 -2, C3 7, C4 3, C5 -3, C6 -2, C7 -2, C8 0, C9 3, C10 -2; D1 2,
  D2 -3, D3 7, D4 2, D5 -2, D6 -3, D7 -3, D8 0, D9 2, D10 -4; E1 3, E2 -2, E3 7, E4 0; F1 2, F2 -3, F3 7, F4 0;
  G1 3, G2 -2, G3 7, G4 0 (an empty number is 0, as today).
- Legacy controls keep empty = 0: H1 0, H2 `#DIV/0!`, H3 0, H4 `#DIV/0!`, H5 0.
- LET/LAMBDA case: an empty LET value or omitted LAMBDA parameter acts as a written-empty argument: A1:A4 `#N/A`,
  A5 3, A6 -3, A7 3, A8 -3, A9 3, A10 -2, A11 0.

## Outcome
Every prediction held (66 cells, excel-graph 2026-11-07). A written-empty QUOTIENT or SERIESSUM argument (x, n, m and
the coefficients too), or an empty LET value / omitted LAMBDA parameter handed in, is `#N/A` after an earlier argument's
error or boolean; the five 2010+ roundings read an empty significance or mode as omitted (significance 1), while legacy
CEILING/FLOOR keep empty = 0. MROUND's `#N/A` test moved into one helper, `Model::atp_number` (functions/lift/mod.rs),
now shared by MROUND, QUOTIENT and SERIESSUM; `numbers_fn_empty_omitted` (mathematical.rs) carries the rounding rule.
Emptiness is judged on the written argument (`Model::empty_arguments`/`is_empty_argument`, logical/let.rs, also used by
IMSUB/IMDIV's `complex_pair_of`), not on a blank value: a first cut on the value turned SERIESSUM's evidenced `#VALUE!`
at gen-m5/6efa59fc1257c475 Calc!A229 and gen-m5/74b183ca6fc1d190 Calc!A16 (an IFERROR / VLOOKUP of an empty cell as x)
into `#N/A`.
numbers.md: "A written-empty argument of a ToolPak function is #N/A" and "An empty significance of the 2010+ roundings
is omitted". Eval delta: two new probe cases, 66/66 cells pass (45 were off before the fix); no baseline line added or
removed. Note: Calc!A7:A8 spill into occupied cells (`#SPILL!` both sides), so they say nothing about array lifting.
