---
id: T-4234
title: "Test that cmp_text_pairwise agrees with cmp_text on ASCII texts, the invariant behind UNIQUE's hash path (T-4200 review)"
milestone: M7
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A test pins the invariant UNIQUE's hash fast path relies on: on texts that are all ASCII, Excel's pairwise
comparison (`cmp_text_pairwise`) agrees with the total `cmp_text`, so a later collation change cannot quietly make
`first_equal_lines` and `coalesce_equal_lines` group ASCII data differently.

## Context
- Follow-up of T-4200 (excel-fidelity review, MINOR). `unique_of` (crates/rustsheet-engine/src/functions/
  spill_functions/unique.rs) keeps the hash buckets when every text of its data is ASCII and uses
  `coalesce_equal_lines` (functions/util/pairwise_order.rs) otherwise, on the claim that ASCII texts compare
  transitively (docs/excel-semantics/coercion.md "Not a total order", ours note). The reviewer checked it by reading
  `cmp_prefix` and the ASCII tiebreak weights; nothing tests it.
- Suggested test (test/test_collation.rs): over every pair of a set of ASCII texts that exercises prefixes, case,
  hyphen, apostrophe and controls (all 1- and 2-character strings over a representative alphabet, plus prefixed
  longer ones), `cmp_text_pairwise(a, b) == cmp_text(a, b)`; and, on a shuffled column of them,
  `first_equal_lines` and `coalesce_equal_lines` give the same groups.

## Acceptance criteria
- [ ] The test exists and passes; it fails if `cmp_text_pairwise` stops agreeing with `cmp_text` on ASCII.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Non-ASCII texts (T-4200's merge handles them).

## Outcome
Two tests: `t4234_ascii_pairwise_agrees_with_total_order` (test_collation.rs; every pair of ~400 ASCII texts: prefixes, case, hyphen, apostrophe, space, controls) and `t4234_ascii_unique_hash_and_pairwise_group_alike` (unique.rs; a shuffled column with duplicates gives the same groups from `first_equal_lines` and `coalesce_equal_lines`). `test_collation` is now `pub(crate)` so unique.rs shares the text set. No behaviour change; eval delta none.
