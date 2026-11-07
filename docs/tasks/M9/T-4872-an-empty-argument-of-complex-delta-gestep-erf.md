---
id: T-4872
title: "An empty argument of COMPLEX, DELTA, GESTEP, ERF, ERFC, the BESSEL functions, CONVERT and the base conversions follows Excel (probe the Analysis ToolPak #N/A rule)"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A written-empty argument of the fixed-arity Analysis ToolPak engineering functions (COMPLEX, DELTA, GESTEP, ERF, ERFC,
BESSELI/J/K/Y, CONVERT, DEC2BIN/DEC2OCT/DEC2HEX, BIN2*/OCT2*/HEX2*) gives what Excel gives, decided by a small probe
whose predictions are committed before the fill. Today the engine reads it as 0 (or as an empty text) and returns a
number where the ToolPak rule says `#N/A`.

## Context
- Blind spot: the generator never writes an empty argument (`call_args` in tools/src/rustsheet_tools/gen/generator.py
  only drops trailing optionals), and no eval cell calls these functions with one (gap triage 2026-11-07, `make eval`
  on origin/main 3d991bd3d).
- Excel evidence for the rule to test, in this lane already: an empty argument of IMSUB or IMDIV, in either place, is
  `#N/A` (engineering.md, probes/eng-imsub-imdiv-blank-j A3, A4, B3, B4), also through a LAMBDA or LET variable bound
  to nothing (probes/eng-imsub-imdiv-lambda-let-empty A1:A4, A12); the first empty argument of IMSUM and IMPRODUCT is
  `#N/A` and a later one is skipped (same entry, C1:D7). Elsewhere: `MROUND(5,)`, `RANDBETWEEN(5,)` (numbers.md,
  probes/volatile-invariant Calc!I5:I6) and `GCD(,4)` (coercion.md, probes/gcd-lcm-multinomial-negative-precedence
  E40) are `#N/A`. Every probed fixed-arity ToolPak function gives `#N/A` for an empty argument.
- Ours today (scratch eval, unverified): `COMPLEX(,1)` "i", `COMPLEX(1,)` "1", `COMPLEX(1,1,)` "1+i"; `DELTA(,1)` 0,
  `DELTA(0,)` 1; `GESTEP(,1)` 0, `GESTEP(1,)` 1; `ERF(,1)` 0.842700792949715, `ERF(6,)` -1, `ERF(,)` 0;
  `BESSELJ(,1)` 0, `BESSELJ(1,)` 0.765197683754859, `BESSELK(1,)` 0.421024421083418; `CONVERT(,"m","ft")` 0,
  `CONVERT(1,,"m")` `#N/A`; `DEC2BIN(,2)` "00", `DEC2BIN(-331,)`, `DEC2HEX(34618,)`, `BIN2HEX(1,)`, `HEX2BIN("FF",)`,
  `OCT2BIN(17,)` `#NUM!` (places read as 0), `DEC2BIN(,)` `#NUM!`.
- Code: `complex_pair_of` (functions/engineering/complex/mod.rs) holds the evidenced check inline (`Node::EmptyArgKind`
  or `is_omitted_variable`). T-4870 and T-4873 need the same test: search first (`rg 'EmptyArgKind'
  crates/rustsheet-engine/src/functions`) and reuse or extract one helper, never a copy (directive 3). The functions:
  `fn_gestep`/DELTA (engineering/misc.rs), `fn_erf`/`fn_erfc` and the BESSEL functions (engineering/bessel.rs), CONVERT
  (engineering/convert.rs), the base conversions (engineering/number_basis.rs), COMPLEX (engineering/complex/).
  Their lift kinds are `ATP_1_OF_2`, `ATP_2`, `ATP_2_OF_3`, `ATP_3` (functions/lift/kinds/engineering.rs).
- How it avoids fitting: each cell is `#N/A`, another error or the documented default; the probe decides, nothing
  numeric is tuned.

## Acceptance criteria
- [ ] A probes case (excel-probe skill, excel-graph) writes each argument of COMPLEX, DELTA, GESTEP, ERF, ERFC,
      BESSELJ, BESSELK, CONVERT, DEC2BIN, DEC2HEX, BIN2DEC, HEX2OCT and OCT2BIN empty, one at a time and all at once, plus
      `LAMBDA(a,b,DEC2BIN(a,b))(5,)` and `LET(e,,COMPLEX(1,e))` rows.
- [ ] The engine's predictions (`#N/A` for every written-empty argument of these fixed-arity ToolPak functions,
      required or optional) are committed in a commit before the oracle fill.
- [ ] After the fill every probe cell passes; engineering.md states the rule with its cells, including which optional
      arguments (places, upper_limit, step, suffix) follow it if any do not.
- [ ] No new F line in any eval/baseline file; `make check` green; reviewers approve.

## Out of scope
- The IM* functions (already evidenced and implemented, T-2132).
- BITAND/BITOR/BITXOR/BITLSHIFT/BITRSHIFT (Excel 2013 functions, not ToolPak; a follow-up if the rule turns out wider).
