---
id: T-4191
title: "VLOOKUP and HLOOKUP with an INDEX index that fails: probe whether Excel keeps INDEX's own error or maps it to #REF!, and match it"
milestone: M2
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
VLOOKUP and HLOOKUP give Excel's error, and the right point in their argument order, when the col_index_num or
row_index_num is an `INDEX(...)` (or another reference-returning call) that fails with an error other than `#REF!`.

## Context
- Gap triage 2026-11-04, opus pass (`make eval` on main 5941bb95). Every visibly-off cell has an open owner, and the
  functions-lookup lane has no ready task. This is a rule the engine guesses on an M2 function (catalog `prior` 8.5
  for VLOOKUP, 1.5 for HLOOKUP).
- docs/excel-semantics/lookup.md "VLOOKUP / HLOOKUP search before validating the index": an INDEX index that itself
  fails (`INDEX(E1:E4,9)`, `#REF!`) is `#REF!` in both functions, found or not found (probes/lookup-index-hlookup
  H7-H10, excel-graph, T-0264). The entry then says: "Whether the INDEX error propagates as is or is always mapped to
  `#REF!` is unprobed (an INDEX failing with another error, e.g. `INDEX(E1:E4,"x")`, would tell)." The cells probed
  so far fail with `#REF!` either way, so they cannot separate the two rules.
- Ours: `Model::lookup_index_arg` (crates/rustsheet-engine/src/functions/lookup_and_reference/index_arg.rs) evaluates
  the index with `eval_ref_or_index`. A failed INDEX comes back as a value error and goes through `cast_to_number`
  before the search. So `INDEX(E1:E4,"x")` (`#VALUE!`) or `INDEX(E1:E4,NA())` (`#N/A`) is returned at once, before the
  lookup value is searched. A referenced cell that holds an error is mapped to `#REF!` only after a match
  (`resolve_lookup_index`). Excel may instead treat any failed reference-returning index like the referenced error
  cell (`#REF!` found, `#N/A` not found), or keep INDEX's error. The probe decides.
- Cells to probe, for VLOOKUP and HLOOKUP alike, each with a found and a not-found lookup value (the
  lookup-index-hlookup layout): `INDEX(E1:E4,"x")`, `INDEX(E1:E4,NA())`, `INDEX(E1:E4,1/0)`, `INDEX(E1:E4,-1)`,
  `INDEX(E1:E4,0,2)` (a column past a one-column range), `INDEX(E1:E4,1.5)` against a text cell. Add
  `OFFSET(E1,"x",0)`, `OFFSET(E1,-5,0)`, `INDIRECT("nope")` and `CHOOSE(3,E1,E2)`, so the rule is stated for
  reference-returning calls in general and not for INDEX only. Wrap each error in `ERROR.TYPE` too. Also check the
  order against a bad lookup value: `VLOOKUP(NA(),A1:C3,INDEX(E1:E4,"x"))`.
- Related: T-2865 (lookup argument kinds, research) covers empty text and errors as direct arguments, not a failing
  reference-returning index.

## Acceptance criteria
- [ ] A probes case (excel-probe skill, excel-graph, canary-verified, tag M2) holds the cells above.
- [ ] lookup.md's "VLOOKUP / HLOOKUP search before validating the index" entry states the rule, cites the new cells,
      and drops the "unprobed" sentence.
- [ ] The engine matches every new cell. The change is in `lookup_index_arg` / `resolve_lookup_index`, shared by both
      functions (no per-function copy).
- [ ] probes/lookup-index-hlookup, lookup-index-order, lookup-index-shapes, lookup-pick-index-shapes and
      lookup-argument-edges still pass. Eval known failures only shrink (`make eval-baseline`, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- XLOOKUP and MATCH (their own entries), INDEX's own error rules (evidenced), and lifted multi-cell indexes (T-1426,
  evidenced).
