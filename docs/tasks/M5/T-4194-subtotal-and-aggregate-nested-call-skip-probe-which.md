---
id: T-4194
title: "SUBTOTAL and AGGREGATE nested-call skip: probe which formulas count as calling SUBTOTAL (defined name, untaken IF branch, uncalled LAMBDA, unused LET binding, text) and match it"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
SUBTOTAL, and AGGREGATE under options 0-3, skip exactly the cells Excel skips as "containing a subtotal". This
includes formulas where SUBTOTAL or AGGREGATE appears only through a defined name, a branch that is never taken, a
LAMBDA that is never called, an unused LET binding or a text argument, and the spilled cells of a dynamic-array
formula that calls one.

## Context
- Gap triage 2026-11-04, opus pass (`make eval` on main 5941bb95). Every visibly-off cell has an open owner, and the
  functions-math lane has no ready task. docs/excel-semantics/numbers.md "SUBTOTAL and AGGREGATE over hidden and
  filtered rows" (probes/subtotal-hidden-filtered, excel-graph, T-1036) evidences the skip for direct calls:
  `=1+SUBTOTAL(9,A1:A2)`, `=IF(TRUE,SUBTOTAL(..))`, `=SUM(SUBTOTAL(..))`, `=LET(x,SUBTOTAL(..),x)` and
  `=AGGREGATE(9,0,A1:A2)` count as 0, `AGGREGATE(9,4,..)` reads them. The entry then says: "whether a SUBTOTAL name
  inside a string, a defined name, an untaken IF branch, an uncalled LAMBDA body or an unused LET binding counts is
  unprobed (the engine counts any call in the tree)." SUBTOTAL is M5 (catalog `prior` 2), so a wrong total here is
  visible in ordinary workbooks.
- Ours: `Model::cell_formula_is_one_of` (crates/rustsheet-engine/src/functions/subtotal.rs) asks
  `Node::contains_function` (crates/rustsheet-engine/src/expressions/parser/contains_function.rs) on the cell's parsed
  formula. That is a syntax walk. So an untaken `IF(FALSE,SUBTOTAL(..),5)`, an unused `LET(x,SUBTOTAL(..),5)` and an
  uncalled `LAMBDA(x,SUBTOTAL(9,x))` are skipped. A defined name whose body calls SUBTOTAL is read, because the walk
  does not enter names. `INDIRECT`/`"SUBTOTAL"` text is read. A spilled cell of `=MAP(..,LAMBDA(r,SUBTOTAL(9,r)))`
  has no formula of its own, so ours reads it, while its anchor is skipped. Excel may flag by syntax (as ours does) or
  by what was evaluated. The probe decides, and the spill case decides whether the flag is per formula or per cell.
- Probe (one sheet; a column of these formulas under `=SUBTOTAL(9,B1:B12)`, `=SUBTOTAL(109,..)`,
  `=AGGREGATE(9,0,..)` and `=AGGREGATE(9,4,..)`; the excel-probe skill). Use one cell per form so each is told apart
  by a power-of-two value, as subtotal-hidden-filtered does:
  - `=IF(FALSE,SUBTOTAL(9,X1),4)`;
  - `=LET(x,SUBTOTAL(9,X1),8)`;
  - `=LET(f,LAMBDA(r,SUBTOTAL(9,r)),16)`;
  - `=IFERROR(16,SUBTOTAL(9,X1))`;
  - a defined name `st` = `SUBTOTAL(9,Sheet1!X1)` used as `=st+0`;
  - a defined name `k` = `32` used as `=k`, as a control;
  - `="SUBTOTAL"&""` and `=LEN("SUBTOTAL(9,A1)")`;
  - `=INDIRECT("X1")` where X1 holds a SUBTOTAL formula, which shows whether the flag is the referenced cell's or the
    formula's;
  - `=SUBTOTAL(9,X1)*0+64` as a control;
  - a 3-row spill `=MAP(X1:X3,LAMBDA(r,SUBTOTAL(9,r)))`, with the total over the anchor and over a spilled cell.
- Related open tasks, not duplicated: T-1195 (perf: cache the contains-SUBTOTAL flag per parsed formula; if the rule
  becomes evaluation-based, update T-1195's Context in the same PR), T-4193 (AGGREGATE scalars and error order, same
  module), T-2902 (Python facade hidden rows).

## Acceptance criteria
- [ ] A probes case (excel-probe skill, excel-graph, canary-verified, tag M5) holds the forms above with defined names
      in the case file.
- [ ] numbers.md states the rule with the new cells and drops the "unprobed" sentence (or narrows it).
- [ ] The engine matches every new cell. The predicate stays one helper shared by SUBTOTAL and AGGREGATE (directive 3).
      If Excel's flag is per cell (spilled cells skipped), the PR records how the spill child finds its anchor's flag.
- [ ] probes/subtotal-hidden-filtered and every aggregate-* probe still pass. Eval known failures only shrink
      (`make eval-baseline`, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Hidden columns, worksheet-level autoFilter and manual hides inside a filtered table (numbers.md lists them as
  unprobed; they need workbook edits the probe format does not drive), the per-formula cache (T-1195), and
  AGGREGATE's scalar rules (T-4193).
