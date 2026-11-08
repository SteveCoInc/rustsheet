---
id: T-5083
title: "Refill the custom-upper probe (T-4965 fill read COLUMNS 3) and re-decide the case-only custom equal rule"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The custom-upper upload (custom filter with operands A and C over cells a and c, delete_column C) has a valid Excel fill, and `CustomFilter::keeps_text`'s case-only rule is decided from it.

## Context
The T-4965 fill of docs/research/t4895/uploads/custom-upper read F!A6 COLUMNS(Sales) = 3, so the delete did not run and its rows 2, 4, 6 may be a no-op. The rule in docs/excel-semantics/numbers.md (marked UNPROVEN by T-4973) and `CustomFilter::keeps_text` (table_types.rs) rest on it. T-4973 therefore added no case for it.

## Acceptance criteria
- [ ] At least two fills of custom-upper (no `operator` attribute, as uploads.py writes it) read COLUMNS 2.
- [ ] A probes case with its oracle values; the numbers.md entry and `keeps_text` follow the evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
