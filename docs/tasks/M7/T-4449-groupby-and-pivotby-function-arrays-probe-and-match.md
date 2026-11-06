---
id: T-4449
title: "GROUPBY and PIVOTBY function arrays: probe and match a ragged HSTACK/VSTACK of functions, several error elements and a parameterless LAMBDA"
milestone: M7
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
GROUPBY and PIVOTBY give Excel's result for the `function` argument shapes T-1325 left unprobed: a ragged array of
functions built with HSTACK and VSTACK, an array of functions holding two or more error elements, and a LAMBDA with no
parameters (`LAMBDA(1)`), alone and inside a function array. The rules are probed first, then implemented, and
recorded in docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY".

## Context
- docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY" (the function-array bullets, probes/groupby-pivotby-functions,
  -functions2, -functions3, excel-graph) evidences row and column arrays of functions, the 3-over-2 `#VALUE!`, and one
  error element as the result (`HSTACK(SUM,NA())` is `#N/A`: functions G!A781, functions2 G!A701). It then lists, as
  "Unprobed (T-1325)": mismatched heights inside a function array (the engine pads with `#N/A`, so the result is
  `#N/A`), which of several error elements wins (the engine: the first evaluated), and a LAMBDA without parameters (the
  engine: `#VALUE!`). T-1325 is done and no open task owns these.
- Ours: `Model::function_elements` and `Model::function_grid` (crates/rustsheet-engine/src/functions/lookup_and_reference/group_functions.rs);
  the grid's doc comment says the `#N/A` padding is unprobed. A LAMBDA's argument count is decided where the functions
  are called (the "A LAMBDA gets one argument when it needs at most one" bullet, `LAMBDA([x],1)` evidenced, a
  zero-parameter LAMBDA not).
- Write one probe case, probes/t4449-groupby-function-array-edges (excel-probe skill, excel-graph, tag M7), on the
  Data sheet of probes/groupby-pivotby-functions (keys A2:A11, values D2:E11), each result read through
  `ROWS(r)&"x"&COLUMNS(r)`, `ERROR.TYPE(r)` and `ARRAYTOTEXT(r)` so the shape, the error and the values are all scored:
  - ragged: `HSTACK(VSTACK(_xleta.SUM,_xleta.MAX),_xleta.MIN)` and `VSTACK(HSTACK(_xleta.SUM,_xleta.MAX),_xleta.MIN)`
    over one value column and over two (D2:E11), GROUPBY and PIVOTBY (column field C2:C11);
  - several errors: `HSTACK(NA(),1/0)`, `HSTACK(1/0,NA())`, `VSTACK(_xleta.SUM,NA(),1/0)`, `HSTACK(_xleta.SUM,#REF!,NA())`,
    and an error beside a non-function value (`HSTACK(NA(),5)`, `HSTACK(5,NA())`);
  - zero parameters: `LAMBDA(1)`, `LAMBDA(SUM(D2:D3))`, `HSTACK(_xleta.SUM,LAMBDA(1))`, and `LET(f,LAMBDA(1),f)` as the
    function, over one and two value columns.

## Acceptance criteria
- [ ] probes/t4449-groupby-function-array-edges is committed with oracle values (excel-graph) for every formula above.
- [ ] Every cell of it passes (`make eval-show CASE=probes/t4449-groupby-function-array-edges`).
- [ ] The eval gate shows 0 regressions; every probes/groupby-pivotby-functions* and gen-m7 cell that passes on main still passes.
- [ ] docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY": the three items leave the "Unprobed (T-1325)" sentence and
      are stated as rules citing the new cells; the `function_grid` doc comment cites them instead of "unprobed".
- [ ] A unit test pins one ragged function array and one zero-parameter LAMBDA result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PIVOTBY subtotal placement and intransitive runs (T-4388, T-4401, T-4414, T-4428): do not change pivot_placement.rs
  or pivotby.rs's subtotal code here.
- Eta names with zero minimum arguments other than NOW (ROW, COLUMN, SHEET; T-4450)
  and nested GROUPBY cells (T-2609, T-2686).
