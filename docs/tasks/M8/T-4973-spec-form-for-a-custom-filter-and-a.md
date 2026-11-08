---
id: T-4973
title: "Spec form for a custom filter and a display-text values list (T-4965 follow-up)"
milestone: M8
lane: engine
status: in-progress
priority: P2
size: M
tier: sonnet
depends_on: []
---

## Goal
The eval spec `"criteria"` form can write a custom filter and a display-text values list, so the T-4895 and T-4939 uploads become eval cases.

## Context
T-4965 left these filters as research sidecars (docs/research/t4895/uploads, docs/research/t4939/uploads) because spec_tables.py and the eval adapter (add_table in crates/rustsheet-eval/src/engine.rs) cannot express them.

## Acceptance criteria
- [ ] spec_tables.py and the adapter accept a custom filter and a display-text values list.
- [ ] The uploads are probes cases with their oracle values.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New filter semantics; other operators stay unprobed.
