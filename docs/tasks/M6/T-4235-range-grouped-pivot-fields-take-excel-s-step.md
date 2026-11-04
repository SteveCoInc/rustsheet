---
id: T-4235
title: "Range-grouped pivot fields take Excel's step rule, and the eval adapter registers grouped spec pivots"
milestone: M6
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A pivot field grouped by numeric ranges puts each value in the group Excel puts it in, and GETPIVOTDATA on such a field
answers as Excel does, so every cell of probes/t4228-pivot-range-groups passes.

## Context
- Follow-up of T-4228, whose excel-graph probe established the rule (docs/excel-semantics/web.md "Grouped pivot fields
  import as their group items", the numeric-range bullet): below `startNum` the first item; past `endNum` the last; a
  value equal to `endNum` the last inner group; otherwise step `floor(q)` with q = (v - start) / interval rounded to 15
  significant digits, and a step past the last inner group goes to the last (`>end`) item, not clamped. A number item in
  GETPIVOTDATA picks the group whose lower bound equals it (the `>end` item for `endNum`; a number inside the `<start` group, -0.5, is `#REF!`: one point, Calc C38), a
  text item the label exactly (no padding trimmed, case-folded), and an omitted item or `"(blank)"` is `#REF!`.
- The importer's rule today (crates/rustsheet-xlsx/src/import/pivots/groups.rs `group_index`) floors the unrounded
  quotient, so 0.3, 0.6 and 0.7 by 0.1 land one step low, and it does not special-case `endNum`. GETPIVOTDATA treats a
  grouped field's items as text, so number items are `#REF!`.
- The eval engine adapter does not register a spec pivot with a `group` (crates/rustsheet-eval/src/spec_pivots.rs): it
  needs the spec pivot to reach the engine with its grouping (share `apply_groups`, do not copy it), and the rendered
  cells Excel lays out (the builder renders none for a grouped pivot).
- Owns the relabel-and-reorder quirk (web.md, pivot 3): a `<groupItems>` list of the wrong length is not refused;
  Excel relabels the groups with its own labels and reorders them, while the importer keeps the file's labels.
- Owns every visibly-off cell of probes/t4228-pivot-range-groups.
- Rounding q "to 15 significant digits" is not fixed by pivot 5: pivots 1 and 5 rule out an absolute tolerance on q
  (4e-13 below 123 rounds up, 8e-13 does not), but a relative tolerance of about 3.3e-15, or rounding v as well as q,
  fits too. Treat the rule as a structural model (reviewer-excel-fidelity check 7) and test the near-boundary points;
  reuse the engine's existing 15-digit rounding helper (`rg` for it) rather than a new one.

## Acceptance criteria
- [x] `group_index` follows the rule above, with unit tests on the probe's points.
- [x] GETPIVOTDATA number items on a range-grouped field match the probe's C30:C47 and A35:A36.
- [x] probes/t4228-pivot-range-groups passes (the eval adapter registers grouped spec pivots); baseline updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- Date groups (T-4236).

## Outcome
- One rule for numeric range groups, `NumberGroups` (crates/rustsheet-engine/src/pivot/number_groups.rs): below
  `startNum` the first item, past `endNum` the last, `endNum` the last inner group, otherwise step
  `floor(to_excel_precision(q, 15))` (the engine's existing 15-digit helper), a step past the last inner group `>end`.
  `group_by_numbers` (same file) groups a cache field by it, keeps the grouping on `CacheField::number_groups`, and
  relabels a `<groupItems>` list of the wrong length with Excel's labels, end groups first (pivot 3). The importer's
  `group_field` and `Txn::add_pivot_table` both call it; the importer's `group_index` keeps only dates.
- GETPIVOTDATA: a number item on a range-grouped field picks the group whose lower bound it is (`lower_bound_index`,
  `endNum` is `>end`); a group with no record is `#REF!` on such a field (an empty item of an ungrouped field keeps its
  answer, unprobed: T-4248).
- Eval adapter: a grouped spec pivot is registered (`PivotTableInput::group`, `PivotRangeGroup`; a breaking change to
  the `doc(hidden)` test-support input, every constructor updated), and the facade writes the layout Excel shows from
  the engine's `row_layout` (crates/rustsheet-engine/src/pivot/layout.rs, with the aggregation GETPIVOTDATA shares);
  the unexpected-cell check bounds the layout by `max_layout_rows`. SPEC-0015 "Spec cases in the eval adapter" records
  why this one shape is laid out in Rust.
- Structural model (reviewer-excel-fidelity check 7 (c)): candidates were (A) round q to 15 digits, (B) a relative
  tolerance of 3.3e-15 on q, (C) round the value to 15 digits, then q; the correctly rounded candidate is the plain
  `floor((v - start) / interval)` the importer used before (P). A was chosen on the T-4228 points; the split below was
  stated after that choice (by value: hold-out when `sha256(repr(v)) % 10 < 3`, margin 10 points), so it shows the
  guard rather than gating the choice. Per record of probes/t4228-pivot-range-groups (55 values, the group Excel put
  each in, decoded from the rendered power-of-two sums), exact group:

  | candidate | train (42) | hold-out (13) | fresh sweep (105) |
  |---|---|---|---|
  | A (chosen) | 42 | 13 | 105 |
  | B | 41 | 13 | 81 |
  | C | 42 | 13 | 99 |
  | P (correctly rounded) | 29 | 9 | 87 |

  A beats P on hold-out (13 against 9) and its hold-out rate equals its train rate (100%). Fresh sweep (R4.2): after
  the freeze, probes/t4235-pivot-range-rounding was generated (generator and seed 42350 in the PR description) and
  oracle-filled by excel-graph (2026-11-04): 81 points at relative offsets 1e-16 to 3e-14 below inner boundaries of four
  groupings (by 0.1, 7, 0.001 and 0.25) and 24 log-uniform over the ranges, 25 of them where A, B and C disagree. Gate A
  envelope: not applicable (no number is computed; group membership is exact or not), and the case's 242 cells pass
  bit-exact.
- Eval delta: probes known failures -122 (all of probes/t4228-pivot-range-groups, now 168/168); new case
  probes/t4235-pivot-range-rounding 242/242. No regressions.
- Follow-ups: T-4248 (GETPIVOTDATA on an empty item of an ungrouped field), T-4249 (numeric group points past the
  evidence: other wrong item counts, a partial last group, GETPIVOTDATA numbers past the end).

## Landing notes
- PR #3238, head 89b2827 (after two `make catch-up` merges of main). CI was green on 89b2827, and all three reviewers
  (security, design, excel-fidelity) approved 13a3230, the last code head before the catch-ups. The PR was enqueued,
  then removed by the merge queue at 2026-11-04T22:57Z with reason MERGE_CONFLICT.
- Cause: T-4236 (#3239, main 7d66a77a7) merged date groups into the same places. It conflicts in
  `docs/excel-semantics/web.md` (the "Grouped pivot fields" entry), `tools/src/rustsheet_tools/pivot.py` (module
  docstring) and `eval/baseline/probes.txt`. It also adds `by: Option<String>` to `PivotGroupSpec`
  (crates/rustsheet-eval/src/case.rs).
- Next step: merge `origin/main` in by hand, as a merge commit, not `make catch-up`, which only handles baseline-only
  conflicts.
  - Keep both docstrings in pivot.py: T-4236's date-group text, with its last line changed to say the engine adapter has
    the engine lay out numeric groups (T-4235).
  - Merge the two web.md entries.
  - In crates/rustsheet-eval/src/spec_pivots.rs `pivot_input`, map only a numeric group (`by` absent or `range`) to
    `PivotRangeGroup`. Leave a date-grouped spec pivot unregistered, as before, so its cells stay as T-4236 baselined
    them. Also limit the grouped-layout box in spill_extent.rs to numeric groups.
  - Then run `make eval-baseline` (regressions must be 0) and `make check-fast`. Re-run excel-fidelity and design on the
    resulting diff, since the adapter change is code. Then enqueue.
- Do not repeat: `make catch-up` refuses this conflict (it is not baseline-only).
