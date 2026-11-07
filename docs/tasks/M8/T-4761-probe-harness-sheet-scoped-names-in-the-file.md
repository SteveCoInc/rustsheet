---
id: T-4761
title: "Probe harness: sheet-scoped names in the file, row and column moves, and a defined name's stored text, for defined-name edit probes (T-4747 follow-up)"
milestone: M8
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The probe harness can express the defined-name edit probes T-4747 could not run.

## Context
T-4747 could not probe: a sheet-scoped name defined in the file (the case spec's `names` are workbook-scoped and `api_names`
are added after `sheet_ops`, so a scoped name never sees the edit); row and column moves (`sheet_ops` has only sheet moves);
and the stored text of a rewritten name (the oracle returns values only). Extend the case spec/spec2xlsx/oracle as far as
excel-graph allows, then run the probes and record them in docs/excel-semantics/references.md; whatever only desktop Excel
can answer goes to M14 (decision 0039).

## Acceptance criteria
- [ ] Probe cases for each of the three items, with predictions committed before the fill, and references.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
