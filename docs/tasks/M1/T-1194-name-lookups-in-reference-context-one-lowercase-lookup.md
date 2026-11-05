---
id: T-1194
title: "Name lookups in reference context: one lowercase lookup per use, and look through formula-bodied names in is_multi_area"
milestone: M1
lane: perf
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A defined name used in reference context is looked up once per use, and a name whose formula body cannot stand for several areas (`=0.5`, `=OFFSET(...)`) keeps the allocation-free intersection path, with the same results.

## Context
Performance review of T-1065 (MINOR, deferred). `Model::is_multi_area` (reference_operators.rs) is
true for every name with a `Formula` body, through `Model::defined_name_formula` (defined_names.rs),
which lowercases the name and does a HashMap lookup each time. A name in reference context is now
looked up up to three times (`is_multi_area` in the caller, `defined_name_reference`, then
`evaluate_node_with_reference` for a body that is not multi-area), each with its own lowercase
allocation; COUNT/COUNTA arguments, `bind_argument`, ROWS/COLUMNS, FREQUENCY and every intersection
call `is_multi_area`. Options: store the lowercased key in the `DefinedNameKind` node at parse time,
or resolve once and pass the `Arc<Node>` down. Also look through the body one level in
`is_multi_area` (names inside the body counted as multi-area without looking, so recursion stays
bounded), so `dyn A1` with `dyn` = `=OFFSET(...)` does not build an area list.

## Acceptance criteria
- [ ] No change in eval results; the test_multi_area_arguments.rs pair-path test covers a formula-bodied scalar name.
- [ ] At most one name-key allocation per name use in reference context (shown by the code path, or a counting test).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The value path's lookup in model.rs (same pattern; may share the fix).

## Handoff (resumed)
Findings by a sonnet run (no code changed):
- The `is_multi_area` half is already in place: `may_be_multi_area(node, look_through_names)` in
  `reference_operators/mod.rs` opens a name body once and counts names inside it as multi-area,
  so `=0.5` / `=OFFSET(...)` bodies keep the pair path. Only a test for a formula-bodied scalar
  name in `test_multi_area_arguments.rs` may be missing (check around line 638).
- The remaining half needs a design choice: `parsed_defined_names` is keyed
  `(Option<u32>, String)`, so every `parsed_defined_name` call allocates a lowercase `String`
  (defined_names.rs ~l.230). Options: store the lowercased key in `DefinedNameS` (44 match sites
  of `Node::DefinedNameKind`, parser + serialization), or resolve once and pass an `Arc<Node>`
  down through `defined_name_reference` / `evaluate_node_with_reference` / the `is_multi_area`
  callers (count.rs, frequency.rs, let.rs, map_reduce.rs, cast.rs, util.rs).
- Next step: pick one, add a counting test, run the perf reviewer-free `make check-fast`.
