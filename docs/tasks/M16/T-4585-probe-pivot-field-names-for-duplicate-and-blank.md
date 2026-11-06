---
id: T-4585
title: "Probe pivot field names for duplicate and blank source headers"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
docs/excel-semantics/pivots.md states, with a filled probe, how Excel names the pivot cache fields of a source whose header row has duplicate names (`Sales`, `Sales`), blank header cells, or names differing only in case.

## Context
SPEC-0032 Refresh and T-4141 rebuild a cache from its source, and field names come from the header row. No probe shows Excel's renaming yet. Build a source whose cache the builder leaves for Excel to rebuild (refreshOnLoad), fill it with excel-graph, and read the saved `cacheField` names from the sidecar's `pivots` section (`cache_fields`). Context: MS-OI29500 18.10.1.3 (cacheField names are unique and case-insensitive).

## Acceptance criteria
- [ ] A filled probe covers duplicate, blank and case-variant header names. pivots.md records each saved name, or the refusal, if Excel refuses the file.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The engine rename (T-4141).
