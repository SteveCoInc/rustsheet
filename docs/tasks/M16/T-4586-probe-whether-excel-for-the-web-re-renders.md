---
id: T-4586
title: "Probe whether Excel for the web re-renders a pivot without refreshOnLoad on open, and after a source edit"
milestone: M16
lane: research
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
docs/excel-semantics/pivots.md states, with filled probes, two things about a pivot without `refreshOnLoad`: whether Excel for the web re-renders it on open, and whether it re-renders a pivot after a source edit with no refresh.

## Context
SPEC-0032 Goal 3 assumes that a pivot without `refreshOnLoad` keeps the cells the file saved, and that a source edit does not re-render. Every rendered probe so far (t4123, t4128, t4134, t4137, t4138) is `refreshOnLoad` (pivot_definition.py), so neither is evidenced (Excel-fidelity review of T-4121). Cases:
- (a) a pivot with `refreshOnLoad=0` whose saved cells and `rowItems` disagree with its cache;
- (b) the same pivot with no `rowItems`;
- (c) a `refreshOnLoad` pivot whose source cell a `check` formula reads after the oracle edits the source without a refresh (Graph range write, then a read of the pivot area).

The builder needs a `refreshOnLoad` switch. T-4182 (Auto Refresh) builds on (c).

## Acceptance criteria
- [x] Filled probes for (a), (b) and (c); pivots.md entries for each; SPEC-0032 Goal 3 updated if Excel differs.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine behaviour (T-4151).

## Outcome
probes/t4586-pivot-refresh-on-load (excel-graph, 2026-11-06) answers all three; docs/excel-semantics/pivots.md
"Pivots without refreshOnLoad keep their saved cells; a source edit does not re-render" (VERIFIED):
- (a) `refreshOnLoad` off, saved cells East 99 / Grand Total 99 and `rowItems` listing East only, over a source
  whose cache says East 5, West 20 and whose refresh would say East 15: Excel keeps the saved cells and saves the
  `rowItems`, location and the cache's field flags and shared items back unchanged (the snapshot
  does not record cache records).
- (b) the same with no `rowItems`/`colItems`: the same saved cells; Excel adds no layout lines.
- (c) a `refreshOnLoad` pivot renders East 5 on open; after a Graph range write of the source (B2 = 100, read
  back as 100) and a FullRebuild calculate it still shows East 5 and total 25: no re-render without a refresh.

This matches SPEC-0032 Goal 3's design default, now cited as evidence there. The case builder gains
`refresh_on_load`, `area` and the raw `rowItems`/`colItems` slots (pivot_definition.py, pytest); the oracle's
existing `entered` step is the source edit, so no new oracle feature. The sheet C pivot cells and Q!A3:F3 are
visibly off until the adapter renders defined pivots (owner T-4152). Not probed: Auto Refresh (T-4182) and a
source edit on a non-refreshing pivot.
