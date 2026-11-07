---
id: T-4848
title: "One 15-significant-digit rounding: settle the tie rule per use and route every caller through one helper"
milestone: M1
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Every place that rounds a double to 15 significant digits goes through one helper, with the tie rule each use needs decided by Excel evidence.

## Context
Four implementations, three tie rules: `to_excel_precision` / `to_precision_str` (number_format.rs, `format!("{:.*e}")`, ties to even) behind `cmp_excel_precision` (`=`, COUNTIF, CF Equal), `to_excel_precision_str` (cell text, formula literals) and EXACT (functions/text/common.rs); `significant_15(value, tie_away)` (number_format.rs) with tie-toward-zero in `number_to_text` (`&`, CONCAT, coercion) and tie-away in `cut_to_15_digits` (decimal_round.rs, ROUND family); and `round_sig` (user_model/sequence_detector.rs, `log10`/`powf` float arithmetic). Inferred: 1000000000000015 reads "1000000000000010" through `&""` but `to_excel_precision_str` gives "1000000000000020".

Found by the T-4856 duplicate-concept audit (one definition per concept, CLAUDE.md directive 3). Excel decides each rule (directive 1): where the existing variants disagree and no probe in `eval/` or `docs/excel-semantics/` settles which one Excel follows, probe it first (`excel-probe` skill) and record the finding.

## Acceptance criteria
- [x] One rounding helper (tie rule as a parameter if Excel needs more than one); the others are deleted.
- [x] Probe cases at exact 16th-digit ties for display, `&`, `=` and EXACT pass against oracle values; any rule Excel really splits is recorded in docs/excel-semantics/numbers.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other duplicate concepts of the audit (T-4847 to T-4855 each own one).

## Outcome
- Probe `probes/t4848-sixteenth-digit-ties` (98 cells, excel-graph, 2026-11-07) shows Excel splitting the
  rule. Comparisons (`=`, `<`, COUNTIF) cut an exact 16th-digit tie away from zero: `1E+15+25` equals `1E+15+30`, and
  `1E+14+2.5` equals `1E+14+3`. Text (`&`, `TEXT(,"0.0")`, EXACT) cuts toward zero. MATCH's exact mode matches neither
  neighbour. The finding is in numbers.md, "A 16th-digit tie splits by use".
- `decimal_round::cut_to_15_digits(value, tie_away)` is now the one 15-digit cut:
  - `cmp_excel_precision` calls it with ties away (it used `format!("{:.14e}")` ties to even, which got 10 of the
    probe's comparison cells wrong);
  - EXACT calls it with ties toward zero (6 cells were wrong);
  - so do `to_excel_precision_str` (cell text, printed literals), the formatter's scientific mantissa and fraction
    cuts, the IM* part pre-cut, POWER's odd-root test, SUBSTITUTE's instance cut, pivot number groups and the
    sequence detector's `round_sig` (its `log10`/`powf` arithmetic is gone).
- `to_excel_precision` is deleted. `to_precision(x, n)` stays only for its variable-precision callers (n = 16 in
  engineering misc, a computed n in `round_shown_place`). `number_to_text` keeps `significant_15`, the digit cut
  under `cut_to_15_digits`.
- UNVERIFIED tie directions (unreachable or unprobed), listed in the entry: CF Equal, the scientific mantissa,
  fractions, IM* parts and pivot group quotients.
- Unit tests: `test_sixteenth_digit_ties` (3 tests).
- Eval: probes gain 1 case, which passes 98/98 values and 7/7 texts. Every suite has 0 regressions; known failures
  are unchanged.
