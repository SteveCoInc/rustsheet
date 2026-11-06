---
id: T-2766
title: "The ERROR.TYPE/CELL mark of the paired functions passes through LET, IF, CHOOSE, IFERROR, a LAMBDA call and INDEX (probes/t2576, 8 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The rest of probes/t2576-error-type-cell-paired passes: a number from ERROR.TYPE or `CELL("col"|"row",..)` keeps its
mark through LET, IF, CHOOSE, IFERROR, a LAMBDA call and INDEX, and `ERROR.TYPE({#REF!})` carries it per element, as in Excel.

## Context
Follow-up of T-2576, which marks the number only when the argument is the ERROR.TYPE or CELL call itself (parentheses
aside; `is_marked_number`, now in crates/rustsheet-engine/src/functions/util.rs, `TODO(T-2766)`). Rule and
evidence: statistics.md "ERROR.TYPE and CELL results are #NUM! in the paired functions" (excel-graph, 2026-10-26).
Known failures left in eval/baseline/probes.txt for probes/t2576-error-type-cell-paired:
- Calc!D1 `FTEST({1;2;3},LET(x,ERROR.TYPE(#REF!),x))`, D2 `IF(TRUE,..)`, D3 `CHOOSE(1,..)`, D5 `IFERROR(..,0)`,
  D19 `LAMBDA(x,x)(..)`, D20 `INDEX(..,1)`, D24 `ERROR.TYPE({#REF!})`: Excel #NUM!, ours #DIV/0!.
- Calc!E16 `SUMXMY2(ERROR.TYPE(#REF!),5)`: Excel #NUM!, ours 1 (SUMXMY2 reads through `sumx_arg`, not
  `fn_get_two_matrices`; SUMX2MY2 and SUMX2PY2 are unprobed).
- Calc!E34 `LINEST(ERROR.TYPE(#REF!))`: Excel #VALUE! where `LINEST(2)` is 0 (gen-m9 8d3588d8df0812df A186), ours a fit,
  and its spill F34.
- Calc!D27 `FTEST({1;2;3},SINGLE(ERROR.TYPE(#REF!)))`: fixed by T-2938 (typed `SINGLE(x)` is `@x`).
- probes/t2577-indirect-ref-array (T-2577; an OFFSET or INDIRECT lifted into an array of references carries the same mark,
  statistics.md "An array of references is #NUM! in the paired functions"): Calc!A64
  `F.TEST(LET(x,INDIRECT("B1",{1}),x),{-1,0,1})` Excel #NUM!, ours #VALUE! (`Model::evaluate_sample_arg` follows only a
  direct OFFSET or INDIRECT, `TODO(T-2766)`); Calc!A148 `SUMXMY2(INDIRECT("B1",{1}),4)` Excel #NUM!, ours #VALUE!.
- Also probe, for an array of references (T-2577), the first argument of RSQ and STEYX, T.TEST's second sample, and
  COVARIANCE.S's and COVAR's second argument, which the engine marks by analogy.
- Also probe the UNVERIFIED positions statistics.md lists (the second argument of COVAR, COVARIANCE.S and the regression
  family; a marked first argument against an error element of the second).
- IRR's flows carry the same mark as `#VALUE!` (T-2728, financial.md "IRR's error order", `fn_irr` through
  `is_marked_number`, now in functions/util.rs): probes/t2728-irr-guess-ref-mark A53 `IRR(IF(TRUE,CELL("row",..)))`, A55
  CHOOSE, A57 INDEX, A59 IFERROR, A61 a LAMBDA call are `#VALUE!` in Excel, ours `#NUM!`; they are in the sidecar but left
  out of the case's `check` list, so add them back to it when the mark follows the wrappers.
- Gap triage 2026-11-01h: an IRR cell with the LET wrapper is checked, and so visibly off. Its only open owner was
  blocked T-2928 (IRR's fallback digits, which names the case for other cells), so this task now owns it. The formula
  is `IRR(LET(x,CELL("row",D2),x))`: Excel `#VALUE!`, ours `#NUM!` (financial.md "IRR's guess range" lists it).
  - probes/irr-guess-edges-2.case.json (1): P!A37
Carrying the mark through bindings needs a design (a value flag, not syntax); split if bigger than S.

## Acceptance criteria
- [x] The cells above leave eval/baseline/probes.txt (`make eval-baseline`), with no new known-failure lines.
- [x] statistics.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Design: the mark is a flag beside the evaluation, not in `CalcResult` and not a syntax walk
  (crates/rustsheet-engine/src/number_mark.rs). Each `evaluate_node_in_context` call gets a serial; `NumberMark`
  keeps the latest call whose value carries the mark (serial + node address, constant memory). ERROR.TYPE and
  `CELL("col"|"row")` make it; LET, IF, CHOOSE, IFERROR/IFNA (of a number), INDEX, a LAMBDA called in place and a called
  name hand it on (`pass_number_mark` at the point each reads the argument it returns); any other call that receives a
  handed mark clears it at its end, so a function reached by a bypass path never looks marked. `bind_argument` records
  whether a LET value or LAMBDA argument is marked and `VariableStack` keeps the marked ids (and treats a name bound to an
  array of references holding a reference as marked, T-2577). Readers ask with `Model::evaluate_marked`; the old
  syntactic `is_marked_number` (functions/util.rs) is gone.
- Consumers: F.TEST/paired functions (`evaluate_sample_arg`), IRR, SUMXMY2/SUMX2MY2/SUMX2PY2 (either argument `#NUM!`
  after the other's error value and the count check), the regression family's second argument (`SecondMark::NoNumber`:
  cells with no number; CHITEST keeps reading the values, unprobed), LINEST/LOGEST/TREND/GROWTH known_y (`#VALUE!`).
- Probe probes/t2766-mark-positions (30 cells, excel-graph, predictions committed before the fill): confirmed the
  analogy positions of T-2577; found the regression second argument is no-number (not the number), SUMX order, IFNA and a
  called LET name keep the mark, IFS/SWITCH drop it. Recorded in statistics.md; all 30 pass.
- The five t2728-irr-guess-ref-mark wrapper cells (P!A53:A61, odd rows) are back in its `check` list and pass.
- Eval delta: probes known failures 7525 -> 7508 (fixed 17: t2576 D1, D2, D3, D5, D19, D20, D24, E16, E34, F34; t2577
  A64, A148, T64, T148, U64, U148; irr-guess-edges-2 P!A37), no regressions, no new known-failure lines; new case
  t2766-mark-positions 30/30.
- Unit tests: test/statistical/test_number_mark.rs, test_fn_irr_xirr_fits.rs `irr_rejects_a_marked_number_through_its_wrappers`.
