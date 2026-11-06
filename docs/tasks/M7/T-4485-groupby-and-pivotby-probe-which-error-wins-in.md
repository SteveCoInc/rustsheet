---
id: T-4485
title: "GROUPBY and PIVOTBY: probe which error wins in a 2-D array of functions (and against the 2-D #VALUE!); fix T-4449's citation nits"
milestone: M7
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know which error GROUPBY and PIVOTBY return when a 2-D `function` array holds errors: reading order, column-major or
evaluation order, and whether the 2-D `#VALUE!` or the explicit error wins. Then state it in docs/excel-semantics/arrays.md
and match it.

## Context
- From the T-4449 review (excel-fidelity MINOR, PR #3498). probes/t4449-groupby-function-array-edges G!C3:C6 evidence
  "first error wins" only on 1-D arrays (`HSTACK(NA(),1/0)` is `#N/A`, `HSTACK(1/0,NA())` is `#DIV/0!`). Those cells
  cannot tell reading order from column-major or evaluation order. arrays.md "GROUPBY and PIVOTBY" (Function-array
  edges) and the `function_grid` doc comment
  (crates/rustsheet-engine/src/functions/lookup_and_reference/group_functions.rs) say "the first in reading order".
- The engine's `function_grid` returns the first error its recursion over the HSTACK/VSTACK arguments meets, before the
  2-D check in `function_argument`.
- Probe (excel-probe skill, excel-graph, tag M7), on the Data sheet of probes/groupby-pivotby-functions, GROUPBY and
  PIVOTBY, each read as `ROWS&"x"&COLUMNS`, `ERROR.TYPE` and `ARRAYTOTEXT` as in t4449:
  - `VSTACK(HSTACK(_xleta.SUM,1/0),HSTACK(NA(),_xleta.MAX))` and `HSTACK(VSTACK(_xleta.SUM,1/0),VSTACK(NA(),_xleta.MAX))`;
  - `VSTACK(HSTACK(NA(),_xleta.SUM),HSTACK(_xleta.MAX,1/0))`;
  - a ragged 2-D array with one error: `HSTACK(VSTACK(_xleta.SUM,_xleta.MAX),NA())`.
- Citation nit: crates/rustsheet-engine/src/test/test_groupby_function_array_edges.rs, test
  `lambda_without_parameters_is_value_error`. Its comment cites G!C9:C12 and G!C33:C36, but its second formula is G!C23.
- Design nit: the line of arrays.md's "Unprobed (T-1325)" sentence that joins the CORREL sentence runs past the
  wrapped width.

## Acceptance criteria
- [ ] The probe is committed with excel-graph values, and every cell passes (or each miss is owned).
- [ ] arrays.md and the `function_grid` doc comment state the evidenced order for 1-D and 2-D arrays.
- [ ] The two nits above are fixed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Anything else about the `function` argument (T-4449 covered it).

## Outcome
Probe probes/t4485-groupby-function-array-error-order (excel-graph, 10 formulas x 3 reads) shows an explicit error beats
the 2-D `#VALUE!` and the first error is the first in argument order (depth first), not reading or column-major order.
The engine already matched (31 of 31 cells), so no engine change; arrays.md and the `function_grid` doc comment state it.
Citation nit (test comment) and the over-long arrays.md line fixed. Eval delta: +31 passing cells in a new case, no
one baseline line for the new case (passing cells recorded), no regressions.
