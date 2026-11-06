---
id: T-4640
title: "Probe pivot field names for number, boolean and error headers, multi-character case folds and blank-versus-suffix collisions"
milestone: M16
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
pivots.md "Pivot field names for duplicate and blank source headers" settles its remaining UNVERIFIED points, and `field_names` (crates/rustsheet-engine/src/pivot/refresh.rs) follows them.

## Context
T-4627 (probes/t4627-pivot-field-name-collisions) showed that a taken name takes the first free `<header><k>` from 2, that blank headers are named after every text header (`X`, `=""`, `Field4` save as `X`, `Field42`, `Field4`), and that `ß` and `SS` are one name (`SS2`) though separate items. Still open:
- a number, boolean or error header (`=1`, `=TRUE`, `=NA()` and a typed `1`): `field_names` refuses them today (`PivotCacheError::Header`);
- other multi-character case folds in names: `ẞ`/`SS`, `ﬁ`/`FI`, `ǆ`/`Ǆ`, `ς`/`σ`, `ı`/`I` (`name_key` folds only `ß`);
- a blank's `Field<n>` meeting a suffixed name from a later duplicate (`Field`, `=""` in column 2, `="Field"`): is the duplicate `Field2` and the blank `Field22`, or the reverse? And `Field4` meeting `field4` typed in lower case.
- carried over from T-4627's code pointers: whether a refresh matches the new fields to the old cache's by name, by case or by position (`refresh_builder` matches by case-folded name), and whether GETPIVOTDATA's item argument `"#N/A"` names an error item (pivot_index.rs).

Use the formula-header trick and the case builder's `cache_names` option (pivot_definition.py) for empty cells and typed duplicates.

## Acceptance criteria
- [ ] Filled probe cases for each point; the pivots.md entry's UNVERIFIED part closed.
- [ ] `field_names` and its unit tests match the evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Item identity (T-4141, T-4612).
