---
id: T-3675
title: "Gate A slice 4h: reference tables for COMBIN, COMBINA, FACT, FACTDOUBLE, MULTINOMIAL, QUOTIENT and MOD"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-3551]
---

## Goal
COMBIN, COMBINA, FACT, FACTDOUBLE, MULTINOMIAL, QUOTIENT and MOD each get a committed, oracle-filled `s1` Gate A table with a cited mpmath definition, and
`make check` runs them (decision 0228 classified them as needing a table).

## Context
- Filed by T-3551 (decision 0228): these in-scope numeric functions have a floating-point kernel, so neither `exact`
  nor `composite` fits and Gate A (decision 0188 R1(a), SPEC-0029) needs their tables.
- numbers.md: "COMBIN is inexact", "COMBINA is COMBIN(n+k-1, k)", "FACT and FACTDOUBLE of a huge argument",
  "QUOTIENT, FACTDOUBLE and SERIESSUM argument modes", "MOD is an exact remainder" and "MOD is #NUM! for a huge
  quotient". The math is the exact combinatorial value (or remainder); Excel's product order makes large results
  inexact, which Gate A's envelope judges.
- The arguments are documented to truncate to integers (Microsoft's FACT, COMBIN and MULTINOMIAL pages); no oracle cell
  pins a fractional argument yet, so the table's edge stratum should include one and the definition reads the truncated
  value only once a filled point confirms it.
- MULTINOMIAL is variadic: use `Definition.variadic` (T-3549).
- Tools: as in T-3549 and T-3586 (`rustsheet-accuracy generate`, `rustsheet-oracle fill -b excel-graph --suite
  accuracy`, `rustsheet-eval accuracy`, then `rustsheet-accuracy math --suite probes-numeric`).

## Acceptance criteria
- [x] One Gate A definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, each citing its sources, with unit
      tests against published or semantics-entry values.
- [x] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each function, generated, filled (excel-graph) and
      committed; `rustsheet-eval accuracy` reports `missing tables` lower by 7. The Outcome names every failing point
      with its owner task (`known-failures.tsv`, T-3379's rules) and any R3 candidate (decision 0188 R3 needs an owner
      ruling).
- [x] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's kernels: each failing point gets its own task (`known-failures.tsv`).

## Outcome
- Definitions (decision 0228): `defs/combinatorics.py` (FACT, FACTDOUBLE, COMBIN, COMBINA, MULTINOMIAL, the exact
  combinatorial value of the floored or truncated arguments, with the `#NUM!` rules of numbers.md as domain errors) and
  `defs/remainder.py` (QUOTIENT, the truncated double quotient; MOD, numbers.md's exact-remainder, huge-quotient,
  shift-back-to-d and smallest-normal rules). MULTINOMIAL is `variadic`. The per-stratum draw dispatch (`by_stratum`,
  `one_of`) lives in `defs/_common.py`. Unit tests: `tools/tests/test_accuracy_combinatorics.py`, against the values
  numbers.md cites.
- Tables: `s1` for each of the seven (120 points each), filled from excel-graph 2026-11-03; `rustsheet-accuracy verify`
  0 differences. `rustsheet-eval accuracy`: 10844 points in 77 tables, 0 failing, 0 table defects; missing tables
  220 -> 213 (-7). No failing point, so no `known-failures.tsv` line.
- Fractional arguments: the edge and typical strata hold 100+ fractional points (`FACT(13.9)`, `FACTDOUBLE(27.9)`,
  `COMBIN(17.9,7)`, `COMBINA(26,0.5)`, `MULTINOMIAL(0,4,3,0,4.25,3)`); every one matches the floor or truncation reading, so
  the definitions read the integer parts.
- Excel against the math: every Excel error is a math domain error and every math domain error an Excel error (FACT 42,
  FACTDOUBLE 40, MULTINOMIAL 32, COMBIN 5, COMBINA 5, MOD 5). The widest gap is 7.0e-13 relative
  (`MULTINOMIAL(64,89,16)`, its log10 sum), so there is no R3 candidate.
- New quirk recorded: numbers.md "QUOTIENT truncates the double quotient (T-3675)" (`QUOTIENT(-46.22552,8E-05)` is -577819,
  not the exact ratio's -577818).
- probes math sidecars: 28 new `eval/suites/probes/cases/*.math.json` files, for the probe cases that call these
  functions (the MOD, COMBIN, FACT and MULTINOMIAL probes among them). `t2714-mod-near-smallest-normal-b` is wrong
  (see Landing notes).
- probes-numeric math sidecars regenerated: `double-rounding-ops` gains math for its MOD and QUOTIENT cells (A84:A87). The
  run also rewrote BETA.INV cells of `t1538-f-inv-huge-df` (A153, A155) and `t1543-inverse-cdf-tiny-p` (A91, A92) to "no
  single value", the effect of T-3578's BETA.INV definition change, not of this task's; they are left as committed.
- `fitlab.sweep.arithmetic` (used by `eval_math.py` for exact inputs) now follows numbers.md "Subnormal results become
  0": a nonzero result of `*`, `/` or `^` below the smallest normal double, and such a literal, is 0, while `+` and `-`
  keep it (`_flush_subnormal`; unit tests in tools/tests/test_fitlab.py). Before, it evaluated `2^-1074` and
  `1E-307/10` exactly, so 57 of the 68 cells of `t2714-mod-near-smallest-normal-b.math.json` disagreed with the oracle
  (round 1's excel-fidelity finding). Regenerating the probes math sidecars changed only cells with such an argument:
  t2714-mod-near-smallest-normal-b (57), subnormal-function-results (5), t2756-roundup-smallest-normal-threshold (3),
  radians-sqrtpi-munit (2) and trunc-decimal-stages (1), each now 0 where Excel flushes. The same run also filled
  CHISQ.INV and GAMMA.INV cells of t3350-chisq-inv-huge-df and t3505-gamma-inv-budget-band that had timed out before;
  that depends on solver load, not on this change, so they are left as committed.
- Also filed T-3880 (follow-up of T-3839's excel-fidelity review).
- Eval delta: none (no engine change).

