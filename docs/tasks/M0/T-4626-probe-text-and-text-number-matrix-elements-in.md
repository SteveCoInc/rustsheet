---
id: T-4626
title: "Probe text and text-number matrix elements in MDETERM, MINVERSE and MMULT"
milestone: M0
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
It is established what Excel returns for a text or text-number element of an MDETERM, MINVERSE or MMULT matrix,
whether written in an array literal or held in a range cell. Each result is recorded with oracle evidence in
`docs/excel-semantics/arrays.md`.

## Context
- T-4025 (review, fidelity minor) gives the Gate A math sidecars no value for a text matrix element
  (`eval_math.matrix_rows`), because only part of this is evidenced:
  - a direct text argument is `#VALUE!`: `MDETERM("5")` and `MDETERM("x")` (probes/stats-evidence-floor-t1623
    S!Q113, Q119);
  - blanks and booleans are `#VALUE!` (arrays.md "MDETERM / MINVERSE: a blank cell in the matrix is #VALUE!",
    probes/matrix-boolean-elements G1-G13);
  - coercion.md says "MDETERM, MINVERSE and MMULT are `#VALUE!` for a text cell in a matrix", but cites no probe
    cell for it.
- Unprobed: a text-number literal element (`MDETERM({"5"})`, `MDETERM({1,"2";3,4})`), a non-numeric text literal
  element (`MDETERM({1,"a";3,4})`), a text-number cell and a non-numeric text cell in a MINVERSE or MMULT range
  (`INDEX(MINVERSE(A1:B2),1,1)` with A1 = `"1"` typed as text, `MMULT(A1:B1,C1:C2)` with a text cell), and a
  one-cell text range (`MDETERM(A1)` with A1 = `"5"`).
- Use the `excel-probe` skill and add a new probes case for these cells.

## Acceptance criteria
- [x] A probes case with the cells above (each function, literal and range forms, text-number and non-numeric
      text) is oracle-filled.
- [x] arrays.md records the results with the case and cell ids, and the coercion.md line cites them or is
      corrected.
- [x] If Excel reads a text number as a number anywhere, a follow-up task is filed for `eval_math.matrix_rows`
      and the engine.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine and math-sidecar changes (follow-ups if the probe shows they are needed).

## Outcome
- New probe probes/t4626-matrix-text-elements (excel-graph 2026-11-06, 28 cells). MDETERM, MINVERSE and MMULT give
  `#VALUE!` for every text or text-number element: array literals (`{1,"2";3,4}`, `{1,"a";3,4}`, `{"5"}`), text cells
  in a range, a one-cell text range, a direct text argument, and either MMULT argument (M1-M26).
- The error is one scalar `#VALUE!`, not per-element errors: `SUM(IFERROR(MINVERSE(B1:C2),100))` is 100, and so is a
  2x2 MMULT product whose text element touches only one column (M16, M17, M27). M28 is an all-number control (-1).
- Excel never reads a text number as a number here, so there is no follow-up. `eval_math.matrix_rows` keeps refusing
  text like blanks and booleans (all are `#VALUE!` in Excel, which the math does not model); only its docstring now
  cites the evidence.
- Recorded in arrays.md ("MDETERM / MINVERSE: a blank cell in the matrix is #VALUE!"). The coercion.md line now cites
  the probe.
- Eval delta: +28 probe cells, all pass (the engine already matches). No engine change.
