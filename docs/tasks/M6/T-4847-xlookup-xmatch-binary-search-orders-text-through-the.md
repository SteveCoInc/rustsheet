---
id: T-4847
title: "XLOOKUP/XMATCH binary search orders text through the shared collation, not upper-cased code points"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Binary-mode XLOOKUP and XMATCH (search_mode ±2) order text the way Excel's collation does, the same order SORT, MATCH and the linear XMATCH modes use.

## Context
`cmp_searched` (functions/lookup_and_reference/lookup_table.rs) falls back to `impl Ord for CalcResult` (calc_result.rs), which compares `to_uppercase()` code points. Every other text ordering goes through the collation: `compare_values` → `cmp_text`, `compare_values_pairwise` → `cmp_text_pairwise` (functions/util.rs), `sort_key_cmp` (functions/spill_functions.rs). Data sorted by SORT can then be binary-searched in another order (hyphens, accents, punctuation vs letters, "ß" vs "SS"). Callers: XLOOKUP `binary_hit` (functions/xlookup.rs), XMATCH (functions/lookup_and_reference/xmatch.rs). Also: `CellKey::Str(s.to_uppercase())` (spill_functions.rs) is a third text-equality key whose comment names a removed `array_nodes_equal`. T-3132 and T-1028 changed only the number side of the Ord.

Found by the T-4856 duplicate-concept audit (one definition per concept, CLAUDE.md directive 3). Excel decides each rule (directive 1): where the existing variants disagree and no probe in `eval/` or `docs/excel-semantics/` settles which one Excel follows, probe it first (`excel-probe` skill) and record the finding.

## Acceptance criteria
- [ ] Binary XMATCH/XLOOKUP text comparison calls the collation helper; `Ord for CalcResult` no longer has its own text rule (or is removed).
- [ ] Probe cases (accented, hyphenated, ß) for search_mode 2/-2 pass against oracle values.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other duplicate concepts of the audit (T-4847 to T-4855 each own one).
