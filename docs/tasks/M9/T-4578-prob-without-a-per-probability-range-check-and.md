---
id: T-4578
title: "PROB without a per-probability range check and with Excel's sum tolerance; CHISQ.TEST and Z.TEST overflow errors (T-4098 probe)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
PROB, CHISQ.TEST and Z.TEST give Excel's results on the cells T-4098's probe and tables found off: PROB checks only the
sum of the probabilities, with Excel's tolerance; CHISQ.TEST is `#NUM!` for an overflowing squared difference; Z.TEST
answers Excel's `#NUM!` and 0.5 where z overflows with sigma given (probe A48, A49, one cell each; with sigma omitted
it is unprobed and out of scope).

## Context
- statistics.md "CHISQ.TEST, Z.TEST and PROB: Gate A domains (T-4098)" states the rules and the evidence.
- PROB (`crates/rustsheet-engine/src/functions/statistical/prob.rs`): the engine is `#NUM!` for any probability outside
  [0, 1], where Excel checks only the sum (`PROB({1,2},{1.5,-0.5},1)` is 1.5), and accepts a sum within 1E-14 of 1,
  where Excel refuses 1E-15 below and 1E-14 above but accepts 1E-15 above. Cells: probes/t4098-tests-prob-domains A3,
  A4, A5, A32, A33; accuracy/PROB/s1 A113, A125 (known-failures.tsv lines owned here).
- CHISQ.TEST (`statistical/chisq.rs`): a squared difference that overflows gives p 0 in the engine, `#NUM!` in Excel:
  probes/t4098-tests-prob-domains A44, A45.
- Z.TEST (`statistical/z_test.rs`): with sigma 1E-300, x = -1E+300 is `#NUM!` in Excel (engine 0) and x = 1E+300 is 0.5
  (engine 1): probes/t4098-tests-prob-domains A48, A49.
- The exact cut of PROB's tolerance is not pinned: the cells fit "the double sum cut to 15 significant digits is 1".
  A probe between the probed amounts (for example 3E-15 and 6E-15 above 1, 2E-16 and 6E-16 below) can settle it first.

## Acceptance criteria
- [x] probes/t4098-tests-prob-domains A3, A4, A5, A32, A33, A44, A45, A48 and A49 pass.
- [x] accuracy/PROB/s1 A113 and A125 pass, and their known-failures.tsv lines are removed.
- [ ] The semantics entry's "Ours" line is updated; `make check` green; reviewers approve.

## Out of scope
- CHISQ.TEST's df for equal counts of different shapes (T-2650).

## Outcome
- PROB (`prob.rs`) checks only the sum of the probabilities (no per-probability [0, 1] check), and the sum passes when
  `decimal_round::cut_to_15_digits(sum) == 1`, the existing 15-significant-digit helper (FLOOR, PERCENTRANK), in place
  of the fork's `|sum - 1| <= 1E-14`.
- Tolerance probe `probes/t4578-prob-sum-tolerance` (24 cells, excel-graph 2026-11-06): predictions under "the double
  sum rounded to 15 significant digits is 1" were committed in the case's description (commit "add PROB sum-tolerance probe with predictions") before the
  fill; all 24 matched. The discriminating pairs: `{0.5,0.500000000000005}` (sum 4.88E-15 above 1) is a number but
  `{1,5E-15}` (5.11E-15 above) `#NUM!`; `{0.5,0.5}-{0,5E-16}` (4.44E-16 below) a number but `{1,-5E-16}` (5.55E-16
  below) `#NUM!`. No fixed or ulp-count tolerance splits both pairs. The rule is structural and reuses an existing
  helper; no constant was tuned to cells (not a fit).
- CHISQ.TEST/CHITEST (`chisq.rs`): an infinite squared difference is `#NUM!`. Z.TEST/ZTEST (`z_test.rs`): with sigma
  given, an infinite z is `#NUM!` when positive, 0.5 when negative; sigma omitted is unchanged (unprobed).
- statistics.md "CHISQ.TEST, Z.TEST and PROB: Gate A domains (T-4098)": the tolerance rule is now evidenced, "Ours"
  updated. Unit tests in `test/statistical/test_fn_{prob_data_order,chisq_test,z_test}.rs`.
- Eval delta: probes known failures 8399 -> 8390 (t4098 A3, A4, A5, A32, A33, A44, A45, A48, A49 fixed), new case
  t4578 24/24 bit-exact, 0 regressions; Gate A: accuracy/PROB/s1 A113, A125 pass, their known-failures.tsv lines
  removed, 0 failing.
- Left as is: Gate A's PROB definition (`tools/src/rustsheet_tools/accuracy/defs/significance.py`) still gives no math
  value to sums between the old probed bands; it could now code the 15-digit rule.
