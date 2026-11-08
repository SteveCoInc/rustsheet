---
id: T-5070
title: "Probe XMATCH/XLOOKUP binary modes on the accent-after-shared-prefix pairs (pairwise vs total collation)"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence settles whether XMATCH/XLOOKUP's binary modes (search_mode 2/-2) compare two texts with the pairwise collation (`cmp_text_pairwise`, Excel's `<`/`=`) or the total one (`cmp_text`).

## Context
T-4847 (PR #4062) made `cmp_searched` (functions/lookup_and_reference/lookup_table.rs) use `cmp_text_pairwise`, as the linear lookups do. Its probe probes/t4847-xmatch-binary-text-collation has no cell where the pairwise and total forms differ (the fidelity reviewer's MINOR). The two differ on accents after shared leading units (docs/excel-semantics/coercion.md "Shared leading units are skipped before the sum", probes/t3181-collation-prefix-callers). Add binary-mode cells over such pairs (sorted data and lookup values) and record the result in docs/excel-semantics/lookup.md's T-4847 entry.

## Acceptance criteria
- [ ] A probe case with at least 10 binary-mode cells where the two collation forms predict different results is oracle-filled, and the engine matches it (or a follow-up owns each off cell).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
