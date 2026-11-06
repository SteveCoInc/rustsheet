---
id: T-4123
title: "Sidecars capture Excel's rendered pivot areas and rowItems/colItems"
milestone: M16
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
An oracle sidecar records, for every pivot Excel saved, its location, every cell of the location and of the page-field area (constants included), rowItems/colItems, group items and displayed texts, so the engine's render can be scored.

## Context
Sidecars hold formula cells only today (probes/pivot-rendered-cells reads the pivot through `=P!A3` formulas). Excel writes the pivot area as constants and rewrites the pivot parts on save (oracle-spike.md run 1, run 4). Values come from the snapshot reader (one parser, full precision). Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] `rustsheet-eval snapshot` emits a `pivots` section; `rustsheet-oracle fill` stores it in `*.oracle.json` (format documented in eval-harness.md).
- [x] A filled probe shows Excel's cells for a compact and a tabular pivot.
- [x] Snapshot limits (XML-bomb, part size) apply; a malformed pivot part is skipped, never a panic.
- [x] `make check` green; reviewers approve.

## Out of scope
- Scoring the captured cells (task `scoring`).

## Outcome
- **Snapshot:** `rustsheet-eval snapshot` emits `pivots`
  (`crates/rustsheet-eval/src/xlsx_snapshot/pivots.rs`). Per pivot: name, sheet, `<location>` and its
  header-row and page counts, Excel's `rowItems`/`colItems` lines, and the cache fields with their
  `<groupItems>`. Parts go through the size-capped reader, lists stop at the engine's pivot limits,
  and a malformed or location-less pivot is skipped (unit tests).
- **Fill:** `rustsheet-oracle fill` adds a sidecar `pivots` section with every cell of each pivot's
  area, blanks as `empty` (`snapshot.py` `pivot_areas`, at most 100,000 cells).
  `OracleFile::pivots` reads it, and T-4124 scores it.
- **Probe:** `probes/t4123-pivot-capture` (excel-graph, 2026-11-06) holds Excel's compact pivot
  (C!A3:E7, 3 row lines and 4 column lines) and tabular pivot (T!A3:E12, 8 and 3 lines), with
  full-precision percentages.
- **Docs:** docs/architecture/eval-harness.md documents the sidecar format and the builder-v2
  `definition` schema, the follow-up the #3516 review deferred.
