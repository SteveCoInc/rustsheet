---
id: T-3677
title: "Gate A slice 4j: reference tables for SUMPRODUCT, SUMX2MY2, SUMX2PY2 and SUMXMY2"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3551, T-3549, T-3586]
---

## Goal
SUMPRODUCT, SUMX2MY2, SUMX2PY2 and SUMXMY2 each get a committed, oracle-filled `s1` Gate A table with a cited mpmath definition, and
`make check` runs them (decision 0228 classified them as needing a table).

## Context
- Filed by T-3551 (decision 0228): these in-scope numeric functions have a floating-point kernel, so neither `exact`
  nor `composite` fits and Gate A (decision 0188 R1(a), SPEC-0029) needs their tables.
- Each takes two (SUMPRODUCT: two or more) equal-length array literals (T-3586's array kind). The math is the exact sum
  of the products (or of x²-y², x²+y², (x-y)²) of the doubles.
- SUMPRODUCT never snaps near zero (numbers.md, near-zero snap, "Summing functions": T-3048,
  probes/t3048-snap-conditional-sums), so its math is the plain exact sum. Whether the SUMX family snaps, and whether
  each overflows at a running total, is to be probed before writing the domain.
- Unequal lengths are `#N/A` (SUMX2MY2 family) or `#VALUE!` (SUMPRODUCT): domain errors, an edge point each.
- numbers.md mentions trig-odd-even-sumxmy2 probes (probes-numeric/trig-odd-even-sumxmy2-sweep): reuse their evidence.
- Tools: as in T-3549 and T-3586 (`rustsheet-accuracy generate`, `rustsheet-oracle fill -b excel-graph --suite
  accuracy`, `rustsheet-eval accuracy`, then `rustsheet-accuracy math --suite probes-numeric`).

## Acceptance criteria
- [x] One Gate A definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, each citing its sources, with unit
      tests against published or semantics-entry values.
- [x] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each function, generated, filled (excel-graph) and
      committed; `rustsheet-eval accuracy` reports `missing tables` lower by 4. The Outcome names every failing point
      with its owner task (`known-failures.tsv`, T-3379's rules) and any R3 candidate (decision 0188 R3 needs an owner
      ruling).
- [x] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's kernels: each failing point gets its own task (`known-failures.tsv`).

## Outcome
- **Probe first** (the task's open questions): probes/t3677-sumx-running-overflow-snap, 20 cells plus ERROR.TYPE companions,
  filled from excel-graph on 2026-11-03 and recorded in numbers.md ("Overflow in `+ - * /` is #NUM! where it happens" and
  "Subnormal flush edges"):
  - SUMPRODUCT, SUMX2MY2 and SUMX2PY2 overflow at a running sum, as SUM does (A1, A3, A4, A7). SUMX2MY2 adds one x^2-y^2 per
    pair, not SUMX2 - SUMY2 (A5 1.44E+308, A6 0). A term past the range is `#NUM!` too (A19, A20).
  - SUMX2MY2 never snaps near zero (A8, A10, A11). The other two only add terms that cannot be negative.
  - All three flush a subnormal square, as SUMSQ does (A12:A15). The engine keeps the square on those four cells, the
    case's only visibly-off cells; they are owned by **T-3969** (new, sonnet, functions-math).
- **Definitions:** `tools/src/rustsheet_tools/accuracy/defs/sumx.py` (decision 0228). Each is the exact sum of the
  products, or of x^2-y^2, x^2+y^2 and (x-y)^2, of the literals' doubles. Domain errors: a running product, square, term
  or running sum past the double range (x87 running values, reusing `product.running_product`, `product.squares` and
  `aggregate.running_total`), and arrays of different lengths (`#VALUE!` or `#N/A`). Flush as above (T-1007 for
  SUMPRODUCT). No snap. Unit tests: `tools/tests/test_accuracy_sumx.py`, against the probe and the values in numbers.md
  and coercion.md.
- **Tables:** `s1` for each function, 120 points each, filled from excel-graph on 2026-11-03. `rustsheet-accuracy verify`
  finds 0 differences on all four. `rustsheet-eval accuracy` went from 11898 points in 81 tables to 12378 points in 85
  tables, with 0 failing and 0 table defects. All 480 new points are within both Excel's 15 digits and the math. Missing
  tables went from 209 to 205 (−4). No point fails, so there is no `known-failures.tsv` line.
- **Excel against the math (tables):** every Excel error is a math domain error, and every math domain error is an Excel error:
  - SUMPRODUCT: 2 `#NUM!` and 2 `#VALUE!`;
  - SUMX2MY2: 5 `#NUM!` and 4 `#N/A`;
  - SUMX2PY2: 4 `#NUM!` and 6 `#N/A`;
  - SUMXMY2: 7 `#NUM!` and 3 `#N/A`.

  The widest gaps are 5.4e-16, 1.5e-15, 3.6e-16 and 5.2e-16 relative. No table point is an R3 candidate.
- **Math sidecars:** I wrote the SUMPRODUCT and SUMX cells into the probes and probes-numeric sidecars: 306 cells in 17
  sidecars, 9 of them new files. Every other cell is kept as it is on main, so this PR has none of the solver-load churn
  that T-3676 had to revert. `rustsheet-accuracy math --check --suite probes --suite probes-numeric` reports `current`.
  Of the 306 cells, 287 agree with their oracle cell (the same error kind, or within 1e-9 relative). The other 19 are
  Excel's rounding, not definition gaps:
  - probes-numeric/trig-odd-even-sumxmy2-sweep W48:W55 and W60:W67 (SUMX2MY2 with x and y near 1E+4 to 1E+8):
    Excel rounds each square to a double before it subtracts, so the cancellation shows the rounding. **R3 candidates:**
    W54 and W55 (1.3e-4 relative) and W62 and W63 (3.0e-3), where Excel's relative error exceeds G = 1e-4. They are left
    unmarked, as T-3668's COT point was: the engine matches Excel, decision 0188 R3 needs an owner ruling that does not
    exist, and the cells pass on Excel's side.
  - probes/t3048-snap-conditional-sums D10 and E10 and t3677 A17: SUMPRODUCT over 0.1, 0.2, -0.3 (and the same with a
    later 5 or 0). Excel's left-to-right double sum leaves 5.55E-17, while the exact sum of the doubles is 2.78E-17. That
    is a relative error of 1.0 against a residue of the rounding, so these are R3 candidates by the letter of the rule,
    also left unmarked for the same reasons. The engine matches Excel.
- Eval delta: none in the gated suites (no engine change). The new probe case adds 4 visibly-off cells (A12:A15), owned by T-3969.
