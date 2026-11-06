---
id: T-4666
title: "Probe ISREF of a LET or LAMBDA variable bound to a union (incl. negated and empty-intersection operands) and _xleta.COUNT with SCAN/REDUCE initial-value forms"
milestone: M7
lane: functions-logic
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence (excel-graph) for two inferences that T-4475 (#3708) left unprobed: ISREF of a LET variable or LAMBDA
parameter bound directly to a union, and the stored eta form `_xleta.COUNT`/`_xleta.PRODUCT` with SCAN's and REDUCE's
initial-value forms. The engine is matched to every cell.

## Context
- T-4475 made ISREF TRUE for any variable bound to a union (`Model::is_union_variable`, functions/logical/let.rs;
  functions/information/mod.rs). Only SCAN's and REDUCE's accumulator is evidenced
  (probes/t4475-scan-reduce-reference-initial F9, G9). docs/excel-semantics/references.md (the union entry) calls the
  direct forms an inference. Reviewer finding (excel-fidelity, #3708): `ISREF((-A1,B1))` and `ISREF((A1 B2,C1))` are
  FALSE (probes/isref-union-operands). If such a union ever binds as `Binding::Areas`, the new branch would say TRUE.
- T-4475's first fill wrote bare `COUNT`/`PRODUCT` (unknown names in a file, `#VALUE!`), so `_xleta.COUNT` with these
  initial values is unprobed (arrays.md "SCAN and REDUCE keep errors in the LAMBDA"; gen-m7/2c5ea078f25443a2 A1767 is
  the one cell, `REDUCE(Data!C7,Data!E12,_xleta.COUNT)` = 1).
- Also from that review: arrays.md's pointer to "Functions passed by name" names xlsx-storage.md, but that section is
  in arrays.md (xlsx-storage.md:205 says a bare name is `#NAME?`). Fix the cross-reference.
- One probe case, probes/t4666-union-variable-isref (excel-probe skill, tag M7), Sheet1 A1 TRUE, A2 the text "1",
  A3 2.5, B1 1, C1 2:
  - `LET(x,(A1,B1),ISREF(x))`, `LAMBDA(x,ISREF(x))((A1,B1))`, `LET(x,(-A1,B1),ISREF(x))`, `LET(x,(A1 B1,C1),ISREF(x))`,
    `LET(x,IF(TRUE,(A1,B1)),ISREF(x))`;
  - `REDUCE(IF(TRUE,A2),A3,_xleta.COUNT)`, `REDUCE(XLOOKUP(2.5,A1:A3,A1:A3),A3,_xleta.COUNT)`,
    `REDUCE((A2,A3),A3,_xleta.COUNT)`, `REDUCE(IFERROR(A2,0),A3,_xleta.COUNT)`, `REDUCE(A1,A1,_xleta.PRODUCT)`,
    `SCAN(IF(TRUE,A2),A3,_xleta.COUNT)`. Write the `_xleta.` prefix in the case so the file stores the eta form.

## Acceptance criteria
- [ ] probes/t4666-union-variable-isref is committed with excel-graph values for every formula above.
- [ ] Every cell passes, or each cell that does not is owned by a filed task.
- [ ] references.md's union entry cites the cells instead of "an inference, not yet probed". arrays.md records the
      eta results, and its "Functions passed by name" pointer is corrected.
- [ ] `make check` green; reviewers approve.

## Out of scope
- REDUCE's returned-reference accumulator (T-4478, done) and the `a:v` parameter ranges (T-4494).
