---
id: T-5034
title: "Probe pivot named sources at their edges: a ref beside a name, Table totals rows and header-less Tables, sheet-scoped, relative and non-range names"
milestone: M16
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
docs/excel-semantics/pivots.md "A pivot over a defined name or a Table reads it again at the refresh" states, with
filled probes, what Excel for the web refreshes a `refreshOnLoad` pivot from in the named-source cases T-4506 left
UNVERIFIED, and the engine follows (or a follow-up task is filed).

## Context
T-4506 (probes/t4506-pivot-named-sources) shows Excel reads a static name, an OFFSET or INDEX name and a Table
again at the refresh. `Model::pivot_source_area` (crates/rustsheet-engine/src/pivot/refresh.rs) takes its own
reading, unprobed, for:
- a `worksheetSource` with both `ref`/`sheet` and `name`: the engine reads the range;
- a Table with a totals row: the engine leaves the totals row out; a Table without a header row: no refresh;
- a sheet-scoped name (`Sheet!Name` in the attribute, or a bare name that only a sheet defines): no refresh;
- a name whose body is relative (`=Data!A1:B6` without `$`): the engine reads it from the first sheet's A1;
- a name that is not one range of one sheet (a value, a union, a name that refers to itself, an error): no
  refresh, the saved cells kept;
- a name or Table that changes between the save and the load (removed, renamed): no refresh.
Also capture what Excel saves back as the cache's `worksheetSource` (the snapshot's `pivots` section does not
hold it today).

## Acceptance criteria
- [ ] Filled probes for each bullet; a pivots.md entry replacing the UNVERIFIED reading; an engine follow-up
  task where Excel differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- External and connection sources.
