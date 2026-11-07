---
id: T-4992
title: "Assign owners to the unowned M16 catalog rows listed by the exit row"
milestone: M16
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: [T-4952]
---

## Goal
Every remaining in-scope row of eval/catalog/pivot-features.tsv that the M16 exit row lists as unowned either gets an
open M16 task in its `owner` column or is ruled out of scope by a decision, so the row's owner criterion
(`owners_met`) can read met.

## Context
- T-4952 made the M16 row (report.json `milestone_exit`, `pivot.stages[].unowned`; report.md's lines "PVn remaining rows
  with no open M16 owner") measure decision 0274 item 6's owner criterion. The rule (docs/architecture/eval-harness.md
  "Milestone exit"): a row is owned when its hand-filled `owner` column names an open M16 task; a done, missing or
  other-milestone task owns nothing; an `out` row is its scope decision's and never remains.
- At T-4952's run (2026-11-07) 487 remaining rows had no owner: PV1 358 of 360, PV2 8 of 8, PV3 78 of 78, PV5 43 of
  43. Only PV-1081 and PV-1113 were assigned (T-4915 names them as the attributes it renders).
- Candidates that need a ruling, not a bulk fill: T-4922 (missingCaption `-`, PV-1220?), T-4919 (classic layout,
  PV-1212?), T-4188 (slicers and timelines as pivot filters, the PV5 rows?), the PV1-PV3 implementation tasks of the
  plan (docs/research/pivot-tables-2026-11-04/plan.md §8). A row whose evidence is below the floor may need a probe
  task rather than an implementation task.
- Assign an owner only where the task clearly covers the row; file the missing owner tasks (one per coherent group
  of rows, e.g. per family and stage) and name them in the rows.

## Acceptance criteria
- [x] Each row the exit row lists as unowned has an open M16 `owner` that covers it, or a decision rules it out
      (a `RULES` change in tools/src/rustsheet_tools/pivot_catalog.py citing it).
- [x] `make eval` shows `owners met` for every stage on the M16 row, or the remaining unowned ids with why.
- [x] Local gates green: `scripts/tasks.py validate`, `make hygiene typos`, `scripts/check_governance.py`, `make eval`
      (no code changed; CI runs the rest of `make check`).
- [x] Reviewers approve.

## Out of scope
- Implementing the rows.
- Changing the ownership rule (T-4952).

## Outcome
Done 2026-11-07. The M16 exit row listed 487 remaining rows with no open owner (PV1 358, PV2 8, PV3 78, PV5 43;
`make eval` at e8a012e1, the same list as T-4952's run). Each now has an open M16 `owner` in
eval/catalog/pivot-features.tsv; no other column changed (`cut -f1-10` identical) and no row was ruled out of scope.

- 69 rows to 19 existing open M16 tasks whose goal or text names the row's attribute or behaviour (PV1 47, PV2 3,
  PV3 12, PV5 7):
  - T-4506 (1): PV-0441. Basis: its goal is sources given as defined names.
  - T-4633 (2): PV-1029, PV-1079. Basis: its title and goal name `pageField@item` and `item@h`.
  - T-4636 (4): PV-0029, PV-0030, PV-1092, PV-1093. Basis: its title names the `count` and `countA` custom subtotals.
  - T-4640 (1): PV-0361. Basis: it probes the field names refresh writes (`field_names`).
  - T-4649 (6): PV-1129, PV-1097, PV-1194, PV-1232, PV-0032, PV-0033. Basis: its context names `subtotalTop`, `defaultSubtotal`, `rowGrandTotals`, `colGrandTotals` and the `t="default"` and `t="grand"` lines.
  - T-4650 (10): PV-1197, PV-1224, PV-1201, PV-1018, PV-1020, PV-1017, PV-1022, PV-1225, PV-1227, PV-1211. Basis: its title and context name `compactData`, `outlineData`, Values on rows, `firstDataCol`, `firstHeaderRow`, `rowPageCount`/`colPageCount`, `pageOverThenDown`, `pageWrap` and `grandTotalCaption`.
  - T-4654 (14): PV-0340, PV-0341, PV-0342, PV-0343, PV-0344, PV-0345, PV-0346, PV-0347, PV-0350, PV-0351, PV-0352, PV-0353, PV-0228, PV-0229. Basis: it probes the `sharedItems` flags (`SharedItemsFlags`: the contains flags, `minValue`/`maxValue`) and names `minDate`, `maxDate`, `autoStart` and `autoEnd`.
  - T-4657 (1): PV-1120. Basis: its title names `showAll` beyond one field.
  - T-4660 (2): PV-1091, PV-1116. Basis: its goal names a field's `compact` and `outline` varied apart.
  - T-4717 (2): PV-0895, PV-0362. Basis: it probes a data field's `numFmtId` and a field's or source's number format on item labels.
  - T-4764 (1): PV-1282. Basis: its goal names `x14:pivotField@fillDownLabels`.
  - T-4785 (2): PV-1212, PV-1128. Basis: its goal names `gridDropZones` and `pivotField@subtotalCaption`.
  - T-4802 (10): PV-0022, PV-0026, PV-0121, PV-0122, PV-0123, PV-0124, PV-0125, PV-0126, PV-0236, PV-0237. Basis: it names date-level (years, months) groups, `discretePr`, `groupItems`, `par` and `base`, and what Excel saves for a group.
  - T-4914 (1): PV-1200. Basis: its goal names `dataCaption`.
  - T-4917 (1): PV-0249. Basis: its goal is the date-time items a refresh writes.
  - T-4922 (2): PV-1220, PV-1245. Basis: its goal names `showMissing` and `missingCaption`.
  - T-4962 (7): PV-1441, PV-1442, PV-1429, PV-1432, PV-1433, PV-1534, PV-1535. Basis: it names one slicer cache driving two pivots (`pivotTables`), unlisted and unselected slicer items, a slicer's `x`, and the timeline `selection`.
  - T-4968 (1): PV-0894. Basis: its goal names a data field saved without a `name`.
  - T-4969 (1): PV-1112. Basis: its goal names `multipleItemSelectionAllowed`.
- The brief's candidates: T-4922 owns PV-1220 (and PV-1245). T-4919 names `gridDropZones` too, but only for pivots with
  column fields and no row field; PV-1212 went to T-4785, which covers classic layout in general and stays open after
  T-4919. T-4188 is done, so it owns nothing: its PV5 rows went to T-4962 and the new PV5 probe tasks.
- 418 rows to 15 new tasks, one per family, stage and kind (PV1 311, PV2 5, PV3 66, PV5 36). The engine tasks own rows
  with failing cells; every one of those cells is a gen-pivot cell, whose causes the T-4132 owner tasks own cell by cell.
  The research tasks own rows below the 30-cell floor with no failing cell (probe, then implement):
  - T-5019 (engine, 34 rows): PV1 layout catalog rows pass: the gen-pivot cells left on axes, fields, items, location and form
  - T-5020 (engine, 17 rows): PV1 aggregation catalog rows pass: the gen-pivot cells left on the eleven functions and dataFields
  - T-5021 (engine, 19 rows): PV1 custom subtotal catalog rows pass: the gen-pivot cells left on avg to varP subtotals
  - T-5022 (engine, 13 rows): PV1 cache catalog rows pass: the gen-pivot cells left on cache fields, shared items and records
  - T-5023 (engine, 6 rows): PV1 source-refresh catalog rows pass: the gen-pivot cells left on worksheet sources and refreshOnLoad
  - T-5024 (engine, 5 rows): PV1 caption catalog rows pass: header captions, errorCaption, showError and the pivot name
  - T-5025 (engine, 5 rows): PV2 page-field catalog rows pass: the gen-pivot cells left on pageFields, pageField@fld and @hier
  - T-5026 (research, 13 rows): Probe pivot rowItems and colItems line attributes to the evidence floor (i@i, i@r, i@t, x, item@t)
  - T-5027 (research, 52 rows): Probe pivot cache item values in shared items and records to the evidence floor (v, u, x, count, longText)
  - T-5028 (research, 6 rows): Probe pivot cache refresh switches to the evidence floor (enableRefresh, missingItemsLimit, saveData, x15)
  - T-5029 (research, 3 rows): Probe pivot caption switches to the evidence floor (cacheField@caption, showHeaders, showMultipleLabel)
  - T-5030 (research, 20 rows): Probe pivot range and quarter groups and group-item values to the evidence floor (rangePr, groupItems)
  - T-5031 (research, 16 rows): Probe pivot slicer caches and slicers to the evidence floor (slicerCacheDefinition, x14:slicer)
  - T-5032 (research, 15 rows): Probe pivot timeline caches and timelines to the evidence floor (timelineCacheDefinition, x15:timeline)
  - T-5033 (research, 194 rows): Propose a scope decision for the pivot catalog's OLAP-only cache item attributes and Table slicer caches
- T-5033 owns the 194 rows that look out of scope until a decision rules on them: the OLAP formatting and
  member-property attributes of cache items, records and group items, tuples, `mpMap`, `mappingCount`,
  `propertyName`, `sqlType`, and `x15:tableSlicerCache`.
- Owners verdict after the change (`make eval`): PV1 owners met (0 of 360 remaining unowned), PV2 met (0 of 8), PV3
  met (0 of 78), PV5 met (0 of 43). report.md: "M16 evidence not met, owners met". `make phase1-exit`'s M16 line:
  "not met: gen-pivot not met; pivot-corpus-lo not met; pivot-corpus-closedxml not met; probes not met; evidence not
  met (PV1 139/373, PV2 211/211, PV3 13/91, PV5 13/56 at the floor); owners met; layout not met; corpus render not met".
- Review (fidelity): the 18 cache-level `@f` rows (PV-0132 … PV-0770, `groupItems/*@f`, `sharedItems/*@f`,
  `pivotCacheRecords/r/*@f`) first went to T-4175 on the reading that `f` flags a calculated item; no evidence ties
  these cache attributes to calculated items (only the pivotField item's `f="1"`, pivots.md), and a calculated item has
  no records, so they moved to the cache-item probe T-5027. T-4956's two slicer rows (PV-1431 `i@nd`, PV-1514
  `slicerCacheHideItemsWithNoData@count`) moved to the slicer probe T-5031, since T-4956 only records them.
- 194 of the owned rows (T-5033's) wait on a scope ruling rather than an implementing task: "owners met" there means a
  decision is pending. Those rows still count against evidence and visibly-off, so the milestone cannot exit on them;
  T-5033 is P1 so the ruling does not sit.
- Rebase: T-4699 (insertBlankRow, item@sd, blank lines) closed on main meanwhile, and `tasks.py validate` refused
  its three rows (PV-0028, PV-1083, PV-1106) as owned by a done task; they moved to the PV1 layout owner T-5019, which
  re-checks them once T-4699's work is scored.
