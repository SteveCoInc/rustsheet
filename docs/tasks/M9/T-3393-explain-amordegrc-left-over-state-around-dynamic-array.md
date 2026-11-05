---
id: T-3393
title: "Explain AMORDEGRC left-over state around dynamic-array formulas (probes -ag, -ah)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's behaviour for AMORDEGRC's left-over state around dynamic-array formulas is explained and the engine matches it.

## Context
T-2844 left two known failures: probes/amordegrc-fraction-state-ag (a fraction above `=SEQUENCE(1,2)` above the 485
charge is 0 in Excel, 485 ours; emptying `amor_left_over` at a dynamic-array formula's place in `run_excel_order` did not
change it, so how the pass reaches a spilling anchor needs finding) and -ah (`=SEQUENCE(1,2)+AMORDEGRC(..485..)` in A2
above an AMORLINC in A3 leaves 485 for A1, ours 132; -ai, the same formula with `*0` and no A3, leaves 0). See
financial.md "AMORDEGRC below period 1".

- probes/amordegrc-fraction-state-ag.case.json and probes/amordegrc-fraction-state-ah.case.json (Sheet1!A1 of -ah:
  Excel 485, ours 132)

## Acceptance criteria
- [x] A probe of an error element in a lifted call (`{1,-1,0.5}`) is added. More probes tell where a dynamic-array formula (and the charges inside it) sits in the order; -ag and -ah pass.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- **Explained:** a dynamic-array formula is not special. Certain function calls empty the left-over value, at the
  point Excel evaluates them and after they have read their arguments. Among them are SEQUENCE, SORT, UNIQUE,
  FILTER, XLOOKUP, the array-shaping functions, TEXTJOIN, CONCAT, MMULT, MINVERSE, LINEST, MODE.MULT and a called
  LAMBDA. TRANSPOSE, `={1,2}`, MDETERM, LET, INDEX, MATCH, SUMPRODUCT, ROWS and RAND keep it. Within a formula the
  calls run left to right. The full lists are in financial.md "AMORDEGRC below period 1".
- **Probes:** 70 new excel-graph workbooks, probes/amordegrc-fraction-state-aj to -da (2026-11-05). They include the
  lifted error element `{1,-1,0.5}` (-as, -at: 776, `#NUM!`, 776, which ours already gave).
- **-ai does not reproduce:** it was 0 on 2026-10-30. On 2026-11-05 its byte-identical copy -av is 485, and so is a
  refill of -ai, which is not committed. -ai keeps its 10-30 value. Its A1 is the one allowed regression
  (decision 0284), owned by T-3610, like gen-m9 A76.
- **Engine:** `Function::empties_amor_left_over` (kinds.rs) lists the calls.
  - `Model::empty_amor_left_over` empties the value after each of those calls. This happens in `evaluate_function`
    and in the reference-form paths of XLOOKUP, TAKE, DROP and VSTACK, and after a called LAMBDA (`call_prepared`).
  - The anchors run before the Excel-order chain. Each anchor's run now records the value it left
    (`Evaluation::anchor_amor`), and `run_excel_order` puts that value back at the anchor's place.
- **Eval delta:** probes known failures 4662 -> 4662. 2 are fixed (-ag, -ah). There is 1 new-coverage failure and
  1 allowed regression.
  - The new failure is -cc A2, `SUM(LINEST({1,2,3},{1,2,4}))`, 4 ulps off and not visibly off. It is a LINEST
    last-digit cell.
  - The regression is -ai A1 (decision 0284).
  - Every other suite is unchanged.
- **Follow-up:** T-4289 asks whether the same calls are what empties WEEKNUM's stale state.
