---
id: T-3910
title: "Fix the T-3889 probe citations in pivotby.rs and arrays.md"
milestone: M7
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3889]
---

## Goal
The T-3889 evidence citations name the right probe cells.

## Context
- Follow-up of T-3889 (PR #2947), from its Excel-fidelity review (MINOR findings). Evidence: probes/t3889-pivotby-option-order
  (49 questions in P!B1:B49, oracle sidecar beside it).
- `crates/rustsheet-engine/src/functions/lookup_and_reference/pivotby.rs` (the comment above the `evaluate_options` call in
  `Model::pivotby_args`) and the doc comment of `blank_options_are_refused_before_a_later_error` in
  `crates/rustsheet-engine/src/test/test_fn_pivotby.rs` cite "P!B1-B46". The blank-option rows B47-B49 belong too (B1-B49).
- docs/excel-semantics/arrays.md, entry "PIVOTBY in the gen-m7 arg-pairs batch (T-3841)", the bullet that ends "Owner
  T-3889.":
  - It labels the probe "order9 Bn" without saying what that is; cite `P!Bn` of probes/t3889-pivotby-option-order instead.
  - B25 is listed twice: it is the `=1/0` row only, and B26 is the `NA()` row.
  - It leaves out B48-B49 (a blank row_sort_order next to a lifted `{0,1}` col_total_depth, with and without a later `NA()`, both `#VALUE!` first).
  - It cites B36, B40 and B43 for "short values are `#VALUE!` alone", but none of those rows has short values. Cite the
    older evidence the code already names (probes/groupby-pivotby-followup G!A81, probes/pivotby-followup P!A301) and
    keep B46 for short values losing to a later error.

## Acceptance criteria
- [ ] The three places above cite the cells that hold the evidence, and nothing else changes (comments and docs only).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any behavior change.
