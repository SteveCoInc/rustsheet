---
id: T-4569
title: "Spec: arrays whose elements mix values and references (SPEC-0014 amendment), for XLOOKUP's range if_not_found under a lifted lookup_value"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A SPEC-0014 amendment (or a new spec) defines how the engine carries an array whose elements are a mix of values
and references, and what each reader sees, so that T-4570 can implement Excel's XLOOKUP reference element.

## Context
- lookup.md "XLOOKUP's if_not_found and ADDRESS's sheet_text over an array" (T-4543) and its T-4563 follow-up
  (probes/t4563-xlookup-not-found-ref-element, excel-graph, 2026-11-06).
  `XLOOKUP({2;9},A1:A3,B1:B3,range)` is 2x1: element 1 the value 20, element 2 the range itself, a real reference.
- Readers:
  - The grid shows the reference element as `#VALUE!`, and a lifted operator (`+0`, `&""`, `*{1;1}`) reads it as
    `#VALUE!`, even for a one-cell range.
  - INDEX hands it back as a reference: ISREF TRUE, TYPE 64 (two cells) or 1 (one cell), AREAS 1, CELL("address")
    names its cell, SUM 7, and it spills.
  - Aggregates: SUM over the whole result walks it (`SUM(..F1:F2)` 23 = 20 + 3 + 4; `SUM(..E1)` 25). COUNT is 1,
    COUNTA 2, ISERROR FALSE/TRUE, ISTEXT FALSE/FALSE, TYPE 64, ISREF FALSE.
- Today: SPEC-0014's `RefGrid` holds only references and errors (`reference_operators/areas.rs`). SPEC-0018's nested
  cell (`nested_array.rs`) shows `#VALUE!` and INDEX returns its array or value, but it is not a reference (ISREF,
  CELL, AREAS) and an aggregate reads only its top-left.
- Questions the spec must settle:
  - the representation: extend `RefElement` with a value variant, or give `ArrayNode` a reference variant;
  - every reader class above, and how readers not probed fall back;
  - the bounds a reference element adds to the walk budgets (CLAUDE.md directive 5).

## Acceptance criteria
- [ ] A spec (docs/specs/) or a SPEC-0014 amendment covers the representation, each reader class above with its
      evidence, the unprobed readers' fallback, and the bounds.
- [ ] T-4570's acceptance criteria name the spec.
- [ ] Reviewers approve.

## Out of scope
- The implementation (T-4570).
