---
id: T-4569
title: "Spec: arrays whose elements mix values and references (SPEC-0014 amendment), for XLOOKUP's range if_not_found under a lifted lookup_value"
milestone: M6
lane: functions-lookup
status: done
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
  - One-argument INDEX keeps it wrapped: `INDEX(..,2)` is 1x1 `#VALUE!`, and its (1,1) is the range, spilling
    "x", "y" (t4563 A25, V25:V26). `INDEX(..,2,1)` hands the reference straight back.
  - Aggregates: SUM over the whole result walks it (`SUM(..F1:F2)` 23 = 20 + 3 + 4; `SUM(..E1)` 25). COUNT is 1,
    COUNTA 2, MAX 20 (A113, uninformative: F1:F2 are below 20), ISERROR FALSE/TRUE, ISTEXT FALSE/FALSE,
    TYPE 64, ISREF FALSE.
- Today: SPEC-0014's `RefGrid` holds only references and errors (`reference_operators/areas.rs`). SPEC-0018's nested
  cell (`nested_array.rs`) shows `#VALUE!` and INDEX returns its array or value, but it is not a reference (ISREF,
  CELL, AREAS) and an aggregate reads only its top-left.
- Questions the spec must settle:
  - the representation: extend `RefElement` with a value variant, or give `ArrayNode` a reference variant;
  - every reader class above, and how readers not probed fall back;
  - MAX and MIN over the result: probe them with an if_not_found range holding values above the hits (and below,
    for MIN) before the spec rules on them, predictions first;
  - the bounds a reference element adds to the walk budgets (CLAUDE.md directive 5).

## Acceptance criteria
- [x] A spec (docs/specs/) or a SPEC-0014 amendment covers the representation, each reader class above with its
      evidence, the unprobed readers' fallback, and the bounds.
- [x] T-4570's acceptance criteria name the spec.
- [x] Reviewers approve.

## Out of scope
- The implementation (T-4570).

## Outcome
- Probe `probes/t4569-xlookup-ref-element-max-min` (excel-graph, 2026-11-06), the formulas filled in three rounds. The
  predictions for each round were written before its fill (docs/research/t4569/predictions.md).
  - Every aggregate reads the reference element's top-left cell: MAX 50 not 60, SUMSQ 401, AVERAGE 35. This also
    corrects lookup.md's T-4563 line: `SUM(..F1:F2)` = 23 is 20 + F1, not 20 + 3 + 4, which is 27. So SPEC-0014
    rule 4 holds unchanged for the reference element.
  - COUNT, COUNTA and SUMPRODUCT follow rule 5.
  - LARGE, SMALL and MEDIAN are `#N/A` over references on another sheet, and `#VALUE!` over references on the
    formula's own sheet. Round 3 reproduced t2065's VERIFIED `#VALUE!` on its own sheet in the same fill. This was
    found in review round 1, and the spec keeps today's same-sheet behaviour.
  - Text and TRUE hits are read as array elements (`TopLeftElements`).
  - Per-element callers give `#VALUE!` (N: the value) for the value element and the range call for the
    reference element.
- Spec: SPEC-0014 amendment "Mixed grids, value elements beside references":
  - Representation: `RefElement::Value(ArrayNode)`, boxed if it breaks the 32-byte assertion.
  - Producer: the driver keeps a scalar value. XLOOKUP gets a reference form only for a lifted lookup_value with a
    range if_not_found. A grid always holds at least one reference.
  - Readers: one arm in each reader class. Unprobed readers fall back to the value view.
  - Bounds: no new budget.
- lookup.md entry extended. T-4570 now names the spec in its Context and acceptance criteria, and owns the
  t4569 visibly-off cells (Calc A1:A8, A11:A15, A17, A20:A24, A26, A31:A32, A35, A40:A44, A50:A53, A56:A60, A63,
  B20:B24, B26, B31:B32, B34, C20:C23, C31:C32, D20:D28, D31:D34; Local E4:E5).
- Eval delta: +1 probe case, 102 cells scored. The 34 that match are the value-view and COUNT/COUNTA cells, and the
  per-element cells already right, plus the same-sheet t2065 forms (Local E1:E3). The other 68 are new known failures owned by T-4570. No regressions.
