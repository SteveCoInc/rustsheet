---
id: T-4849
title: "Conditional-format cellIs, text and duplicate rules compare through the formula comparison helpers"
milestone: M10
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Conditional-format rules compare values the way the matching formulas do (15-digit equality and ordering, Excel text matching, the duplicate key).

## Context
conditional_formatting.rs: cellIs uses `cmp_excel_precision` only for Equal/NotEqual and raw float `>`/`<=`/Between for the rest (0.1+0.2 vs 0.3: "equal" matches, "<=" does not, while the formula `<=` gives TRUE); `apply_cf_text` uses Unicode `to_lowercase` + `contains` with no wildcards (SEARCH uses `fold_case` + `WildcardPattern`); `cell_value_key` for duplicate values keys numbers by exact `format!("{n}")` and text by `to_lowercase`; the date bound uses `parse_finite_number` rather than `parse_excel_number`.

Found by the T-4856 duplicate-concept audit (one definition per concept, CLAUDE.md directive 3). Excel decides each rule (directive 1): where the existing variants disagree and no probe in `eval/` or `docs/excel-semantics/` settles which one Excel follows, probe it first (`excel-probe` skill) and record the finding.

## Acceptance criteria
- [ ] cellIs, text-contains/begins/ends and duplicate/unique rules call the shared comparison, fold and key helpers.
- [ ] Probe cases for each difference above pass against oracle values (CF evaluation evidence).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other duplicate concepts of the audit (T-4847 to T-4855 each own one).
