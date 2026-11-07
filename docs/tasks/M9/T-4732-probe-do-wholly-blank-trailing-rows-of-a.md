---
id: T-4732
title: "Probe: do wholly blank trailing rows of a D-function database count as records (DCOUNTA with omitted field)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A committed excel-graph probe settles whether wholly blank rows at the end of a D-function database are records.

## Context
- docs/excel-semantics/criteria.md "Database edge-pool smoke check (T-2249)": `DCOUNTA(Db!A1:D12,,Db!K1:K2)` was 8 (not
  11) in a scratch suite that was deleted; no committed cell shows it. db-front-end-sweep (T-3650) cannot tell, because
  every function but DCOUNT/DCOUNTA with an omitted field ignores all-blank records.
- Only DCOUNT and DCOUNTA with an omitted field count records, so the probe must use them over a database with trailing
  blank rows and a criteria block that matches every record (a header over a blank row).

## Acceptance criteria
- [ ] A probes case with `DCOUNTA(db,,crit)` and `DCOUNT(db,,crit)` over a database with 0, 1 and 3 trailing blank rows,
      oracle values committed (`rustsheet-oracle`, excel-graph).
- [ ] The criteria.md T-2249 bullet cites the cells and drops its UNVERIFIED mark; the engine follows if it differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other bullets of the T-2249 entry (settled by T-4688).
