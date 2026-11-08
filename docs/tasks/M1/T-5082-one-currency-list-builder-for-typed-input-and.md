---
id: T-5082
title: "One currency-list builder for typed input and coercion; drop stale value_needs_quoting references (T-4851 follow-up)"
milestone: M1
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4851]
---

## Goal
The engine builds the "`$` plus the locale's own symbol" currency list in one place, and no doc names the removed
`value_needs_quoting`.

## Context
These are the design reviewer's MINOR findings on PR #4075 (T-4851), deferred because every reviewer had approved.
- model.rs `is_typed_negative_currency_percent_fraction` builds `["$", locale symbol]` by hand. T-4851 added
  `Model::typed_currencies` (cast.rs) for exactly this list.
- cast.rs `coercion_currencies` (`$`, `€` and the locale's symbol) and `typed_currencies` differ only by the euro sign.
  One private builder should own the locale-symbol dedup.
- docs/excel-semantics/numbers.md (about line 1200) and docs/specs/SPEC-0003-newer-error-kinds.md (about line 118)
  still name utils.rs `value_needs_quoting`. numbers.md also says it shares `parse_excel_number`. Since T-4851 it is
  `Model::text_needs_quote_prefix` on `parse_typed_number`.

## Acceptance criteria
- [ ] `is_typed_negative_currency_percent_fraction` uses `typed_currencies`; `coercion_currencies` is built from the
      same helper; behavior unchanged (the typed-input and coercion probes stay green).
- [ ] numbers.md and SPEC-0003 name `Model::text_needs_quote_prefix` and its reader.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to which currencies typed input or coercion reads.
