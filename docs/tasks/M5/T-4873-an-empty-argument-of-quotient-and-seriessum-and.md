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

## Outcome
- Probe probes/t4873-empty-arg-toolpak-roundings (55 formulas, 57 checked cells, excel-graph 2026-11-07). The engine's
  predictions and unit tests were committed (0fbd24317 on the branch) before the fill; all 57 cells matched.
- Rules (numbers.md "An empty argument of a ToolPak function is #N/A; an empty significance of the newer roundings is 1"):
  a written-empty argument of QUOTIENT or SERIESSUM (x, n, m, coefficients) is `#N/A`, judged left to right; a
  written-empty significance or mode of CEILING.MATH, FLOOR.MATH, CEILING.PRECISE, FLOOR.PRECISE and ISO.CEILING is
  read as omitted; CEILING and FLOOR keep 0.
- Code: one helper, `Model::atp_numbers` (cast.rs), now reads MROUND's, QUOTIENT's and SERIESSUM's numbers; it decides
  "empty" on the argument node through the new `Model::is_empty_argument` (let.rs), which the complex functions' check
  now reuses. `numbers_fn_empty_as_omitted` (mathematical.rs) serves the five roundings. The node test matters: a blank a
  formula returns (VLOOKUP of an empty cell) reaches the kernel as the same value, and Excel reads it as 0 (gen-m5
  74b183ca6fc1d190 A16, 6efa59fc1257c475 A229 regressed under a value-based first version). MROUND now reads such a
  blank as 0, not `#N/A` (unprobed for MROUND; noted UNVERIFIED in numbers.md).
- Eval delta: fixed 0 / regressions 0 / new case 1 (57 of 57 pass, no F lines). No math sidecar: the Gate A generator
  skips formulas with an empty argument.
