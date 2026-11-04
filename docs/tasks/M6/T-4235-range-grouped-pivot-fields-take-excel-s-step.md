---
id: T-4235
title: "Range-grouped pivot fields take Excel's step rule, and the eval adapter registers grouped spec pivots"
milestone: M6
lane: engine
status: todo
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
- [ ] `group_index` follows the rule above, with unit tests on the probe's points.
- [ ] GETPIVOTDATA number items on a range-grouped field match the probe's C30:C47 and A35:A36.
- [ ] probes/t4228-pivot-range-groups passes (the eval adapter registers grouped spec pivots); baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Date groups (T-4236).
