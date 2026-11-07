---
id: T-4171
title: "Pivot grouping in refresh and PivotResult, reusing the M6 group rules"
milestone: M16
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4170, T-4141]
---

## Goal
A refreshed pivot builds its groups from the source as Excel does, and PivotResult lays out grouped fields in every form, reusing the M6 rules (`pivot/number_groups.rs`, `pivot/date_groups.rs`, the importer's `import/pivots/groups.rs`) rather than a second implementation; manual groups and nested date levels follow T-4170's evidence.

## Context
Rescoped 2026-11-06. M6 implemented grouping at import (group labels become cache items) and a compact one-field layout for grouped spec pivots (`pivot/layout.rs`, T-4235/T-4246). What is missing is grouping during a refresh from the source, grouped fields in multi-field and tabular/outline layouts, and manual and multi-level groups. SPEC-0032 decides where the shared group code lives (directive 3). Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

Owner of these visibly-off pivot cells (decision 0245; moved from T-4152): probes/t4128-pivot-spike
  G!A3:B7. Cause: the case builder's `group_labels` (`while lo <= end`) writes one bin past
  `endNum`, so the cache's `<fieldGroup>` lists 7 group items and the pivotField 7 `<item x>`
  where Excel's grouping of startNum 45000, endNum 45400, interval 100 has 6. The importer rebuilds
  the 6 groups and refuses `x="6"` (`item index 6 is past the field's items`,
  crates/rustsheet-xlsx/src/import/pivots/table.rs), so the pivot never loads. Excel opens the file,
  rebuilds its groups on refresh and renders its own items (pivots.md "Numeric and date group labels
  are rewritten by Excel"). The builders keep their bin count: changing it changes the built file
  and its oracle input hash.

Owner of these visibly-off pivot cells (decision 0245; T-4170, pivots.md "Manual groups and date levels as Excel for
the web renders them"). Cause: the importer skips a cache whose source field carries Excel's `par`-only
`<fieldGroup par="2"/>` beside a discrete group (`a fieldGroup without groupItems`,
crates/rustsheet-xlsx/src/import/pivots/groups.rs `group_field`), so no pivot of the case loads:
- probes/t4170-pivot-manual-groups MC!A3:B14, MO!A3:C14, MT!A3:C14, MS!A3:C18, MG!A3:B8, MB!A3:B10, MP!A3:B14, Calc!B1:B5,
  Calc!B8:B10, Calc!B12:B14, Calc!B16, Calc!B18

Owner of these visibly-off pivot cells (decision 0245; T-4170). Cause: the load-time refresh reads the source fields
only, so it has no Quarters or Years field (the fields a date group adds) and `remap` fails; each pivot over one keeps
its saved cells, which the builder leaves empty (`plan_pivot_refresh`, crates/rustsheet-engine/src/pivot/grid.rs;
`refresh`, pivot/refresh.rs). The import itself works: DM (Date alone) renders, and GETPIVOTDATA on DX answers:
- probes/t4170-pivot-date-levels DC!A3:B23, DO!A3:D23, DT!A3:D23, DS!A3:D27, DY!A3:B16, DX!A3:F11

## Acceptance criteria
- [ ] The importer accepts a `refreshOnLoad` cache's pivotField `<item x>` entries past its rebuilt groups (the refresh lays the field out again), so probes/t4128-pivot-spike G renders Excel's 6 items.
- [ ] T-4170 probes pass; the M6 grouping probes keep passing; corpus group pivots re-render.
- [ ] No duplicated grouping logic (`scripts/check_duplication.py`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Auto date grouping on field drop (a UI action).
