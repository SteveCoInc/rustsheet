---
id: T-4447
title: "SORT's fractional and near-integer sort_order and sort_index: probe whether Excel floors them with SORTBY's near-integer snap, and match (ours truncates)"
milestone: M7
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
SORT reads a fractional or near-integer `sort_order` and `sort_index` as Excel does, so that `SORT(A1:A3,1,-0.5)`,
`SORT(A1:A3,1,1-2^-23)`, `SORT(A1:A3,1,-1-2^-23)` and `SORT(A1:B3,2-2^-23)` sort (or fail) as Excel's do. The rule is
probed first, then implemented, and recorded in docs/excel-semantics/arrays.md.

## Context
- docs/excel-semantics/arrays.md "SORTBY with a single-value by_array or a fractional sort_order" (T-2310, T-2370,
  probes/t2310-sortby-single-by-array H97-H165, probes/t2370-sortby-mixed-by-array-snap) evidences SORTBY: a sort_order
  is its floor after the near-integer snap (`near_integer_whole`, crates/rustsheet-engine/src/functions/util/near_integer.rs):
  -0.5 and -0.9999 sort descending, 1.5 ascending, 0.5 and -1.5 `#VALUE!`, `1-2^-23` ascending, `1-2^-21` `#VALUE!`,
  `-1-2^-23` descending, `-1-2^-22` `#VALUE!`. The same entry ends: "SORT's own sort_order still truncates (unprobed for
  fractions)".
- Ours: `Model::sort_by_keys` (crates/rustsheet-engine/src/functions/spill_functions.rs) casts with `order as i32` and
  `index as i64`, both truncations toward zero, before `validate_sort_order` and the `index < 1` check. So if SORT follows
  SORTBY, ours is wrong at least for a negative fractional order (-0.5: ours 0, `#VALUE!`; SORTBY's rule -1, descending)
  and for an order or index just below an integer (`1-2^-23`: ours 0, `#VALUE!`; the snap gives 1). A positive
  fraction (`SORT(..,0.5)`, `SORT(..,2.7,..)`) gives the same result under both, which is why the existing gen-m7 and
  probes cells with a 0.5 or 2.7 sort_index pass and say nothing here.
- Write one probe case, probes/t4447-sort-fractional-keys (excel-probe skill, excel-graph, tag M7), over A1:B3 = (3,c),
  (1,a), (2,b), each result read through `TEXTJOIN(",",,..)` and `ERROR.TYPE` so the order and the error are both scored:
  - sort_order: -0.0001, -0.5, -0.9999, -1.5, 0.0001, 0.5, 0.9999, 1.5, 1.9999, `1-2^-21`, `1-2^-22`, `1-2^-23`,
    `1-2^-24`, `-1-2^-22`, `-1-2^-23`, `-1-2^-24`, `2-2^-22`, `"-0.5"`, and the same values as one-element
    `{..}` arrays and as a two-key list `{-0.5,1}` with sort_index `{1,2}`;
  - sort_index: 0.9999, `1-2^-23`, `1-2^-21`, 1.5, 2.5 (past the 2 columns), `2-2^-23`, `2+2^-22`, -0.5, and a two-key
    list `{1.5,2.0000001}`;
  - by_col TRUE with the same sort_index fractions over the transposed data (`TRANSPOSE(A1:B3)`).

## Acceptance criteria
- [ ] probes/t4447-sort-fractional-keys is committed with oracle values (excel-graph) for every formula listed above.
- [ ] Every cell of it passes (`make eval-show CASE=probes/t4447-sort-fractional-keys` shows no failure), with the
      reading shared with SORTBY's (`near_integer_whole` or the existing SORTBY path), not a second copy of the snap; if
      Excel's SORT turns out to truncate, the probe's cells pass unchanged and the entry says so.
- [ ] Every gen-m7 and probes cell that passes on main still passes (the eval gate shows 0 regressions).
- [ ] docs/excel-semantics/arrays.md: the "SORTBY with a single-value by_array or a fractional sort_order" entry (or a
      new SORT entry it links) states SORT's rule for sort_order and sort_index with the probe's cells, and the
      "unprobed for fractions" sentence is gone.
- [ ] A unit test in the SORT tests pins one negative fractional sort_order and one just-below-an-integer sort_index.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORT.sort_order and SORT.by_col given empty text, an error or a multi-cell range (T-2874).
- SORTBY's sort_order and by_array rules (already evidenced, T-2310, T-2370).
- Lifted by_col elements (T-2366, T-3497).
