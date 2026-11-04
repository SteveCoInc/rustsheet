---
id: T-3975
title: "CONVERT with a binary and a decimal prefix: Gibyte to zbyte (t3861-convert-prefix-sweep A52)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
CONVERT between a binary-prefixed and a decimal-prefixed information unit matches Excel to the last bit.

## Context
T-3861 applies both prefixes as one factor, the nearest double to their combined power of ten times
the binary prefixes' exact ratio (docs/excel-semantics/engineering.md "CONVERT constants and evaluation
order"). One fresh-sweep cell mixes the two kinds and misses by 1 ulp:
probes-numeric/t3861-convert-prefix-sweep Calc!A52 `CONVERT(4.73E-10,"Gibyte","zbyte")` is
5.0787988275200007E+20 in Excel, 5.07879882752E+20 here (× 1E21 × 2^30). `* 2^30 * (1/1E-21)` (the engine's
form before T-3861) gives Excel's value. The sweep's other three mixed cells pass. One cell is not enough to
choose a rule: probe the mixed pairs (every binary prefix against a spread of decimal prefixes, both
directions, bit and byte, a few values) on excel-graph first.
Code: the prefix factor at the end of `convert_of` in crates/rustsheet-engine/src/functions/engineering/convert.rs.

## Acceptance criteria
- [x] A probe of mixed binary/decimal prefix pairs is oracle-filled, and the rule it shows is in engineering.md.
- [x] A52 and the probe's cells pass, and no CONVERT cell of any suite changes from pass to fail.
- [x] `make check` green; reviewers approve.

## Out of scope
- Light-year and parsec factor digits (T-2047).

## Outcome
- Probe probes/convert-mixed-prefix (672 cells, excel-graph): every binary prefix against the 21
  decimal prefixes, both directions, bit and byte, seeded random values and 1, 3, 0.1, 7.5E-6.
  Rule (engineering.md "CONVERT constants and evaluation order"): with a binary prefix on either
  unit the prefix factor is `(P_from × B_from) / (P_to × B_to)`, each unit's own constant, so a
  decimal target prefix enters as the rounded reciprocal of its constant. The pure-decimal rule
  (T-3861) is unchanged.
- Structural model (reviewer-excel-fidelity check 7 (c)), chosen from 7 candidate orders scored on
  the 679 mixed cells (the probe plus the sweeps' 7). Four candidates fit all 679. They agree on
  every mixed pair, because a binary factor is a power of two. Hold-out by cell hash (30%, SHA-256
  of case and cell, the T-3861 split; it was computed after the first full scoring, so the guard
  that really comes after the freeze is the fresh sweep below), shown as bit-exact/≤4 ulp/15 digits:
  hold-out 204/204/204 of 204, train 475/475/475 of 475; correctly rounded 151/204/200 and
  338/475/474; the pure-decimal rule 184/204/202 and 415/475/475.
- Fresh sweep after the freeze: probes-numeric/t3975-convert-mixed-prefix-sweep, 150 seeded calls
  (seed 39750), values ±1E-250..1E250. Model 150/150 bit-exact; correctly rounded 108, the
  pure-decimal rule 125. All 150 Excel values are inside Gate A's envelope against exact rational
  math (Fraction), so none is outside.
- Not a fit of kind (a): no constant was chosen, so no `FITTED` tag.
- Eval delta: probes +672 passing cells in a new case (baseline line added; 0 regressions in
  every gate suite). probes-numeric t3861-convert-prefix-sweep 142 → 143 of 150 (A52), holdout 148
  → 148, new sweep 150/150. No CONVERT cell changes from pass to fail.
