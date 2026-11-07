---
id: T-4848
title: "One 15-significant-digit rounding: settle the tie rule per use and route every caller through one helper"
milestone: M1
lane: engine
status: in-progress
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
- [ ] One rounding helper (tie rule as a parameter if Excel needs more than one); the others are deleted.
- [ ] Probe cases at exact 16th-digit ties for display, `&`, `=` and EXACT pass against oracle values; any rule Excel really splits is recorded in docs/excel-semantics/numbers.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other duplicate concepts of the audit (T-4847 to T-4855 each own one).
