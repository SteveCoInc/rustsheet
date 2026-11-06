---
id: T-4685
title: "TOCOL and TOROW with both ignore and scan_by_column errors: probe and match; settle two stale arrays.md UNVERIFIED notes"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel cells show which error TOCOL and TOROW return when `ignore` and `scan_by_column` are both errors. The engine
matches them, and arrays.md records the rule. Two arrays.md notes still say UNVERIFIED although committed Excel
cells answer them. They are rewritten to cite those cells.

## Context
- Gap triage 2026-11-06 (evening steward run, `make eval` on agent/steward/2026-11-06). Every failing functions-lookup
  cell of the run has an open owner, and no lookup task was ready, so the lane's UNVERIFIED notes are the gap left.
  TOCOL and TOROW are M7 (catalog prior 0.3).
- The rule that is still open: arrays.md "TOROW, TOCOL, SORT and TEXTJOIN lifted arguments (gen-m7)", in the "Ours"
  paragraph, says "an error ignore or scan_by_column comes before an invalid one, in argument order (T-2313; an error
  ignore beside an error scan_by_column is unprobed)". The T-3497 entry ("TOCOL, TOROW and SORT: which bad argument
  wins") covers a text or out-of-range ignore against an error scan_by_column, never two errors. No suite has the form
  (searched 2026-11-06).
- What the engine does today (scratch evaluation, A1:A3 = 1,2,3, G8 `=NA()`, G9 `=1/0`). It returns the first error
  in argument order, for literals and cells alike: `ERROR.TYPE(TOCOL(A1:A3,NA(),1/0))` is 7, `TOCOL(A1:A3,1/0,NA())`
  2, `TOROW(A1:A3,1/0,NA())` 2, `TOROW(A1:A3,G8,G9)` 7, `TOCOL(A1:A3,G9,G8)` 2, and `TOROW(A1:A3,#NULL!,#NAME?)` 1.
  With a one-cell error reference as the array (a 1x1 array, arrays.md T-3497 entry), `TOCOL(G9,G8,1/0)` is 7.
- Stale note 1, same "Ours" paragraph: "an error array is every lifted element's result (TOCOL's too, by analogy with
  TOROW's evidence: UNVERIFIED, T-2365 probes `ROWS(TOCOL(1/0,0,{0;1}))`)". T-2365 is done. Its probe,
  probes/t2365-wrap-expand-tocol-error-lift S!G10 and S!G11 (`ROWS(TOCOL(1/0,0,{0;1}))`, `ROWS(TOCOL(1/0,{0;1}))`), has
  excel-graph values, and the WRAPCOLS/EXPAND entry of arrays.md already quotes them as 2. Point the sentence at those
  cells and drop "UNVERIFIED". Check them first with `make eval-show CASE=probes/t2365-wrap-expand-tocol-error-lift`.
- Stale note 2: the paragraph above "## A one-cell error reference in the other array functions (T-3667)" says "The
  direct form `VSTACK("12",Data!G9)` is not probed yet. Ours: `#DIV/0!`, as `fn_vstack` returns any error argument" and
  "Status: VERIFIED (excel-graph) for the SCAN cell; the direct forms UNVERIFIED". The T-3667 entry right below it
  records `ROWS(VSTACK("12",A1))` = 2 and `INDEX(VSTACK("12",A1),2)` = `#DIV/0!` (probes/t3765-stack-error-reference,
  excel-graph, 2026-11-02). Since T-3765 the engine stacks the reference as a 1x1 array (`ROWS(VSTACK("12",G9))` is 2 in
  a scratch run). Rewrite the "Ours" and "Status" lines to match, citing the t3765 cells.
- Code: `fn_tocol` and `fn_torow` in crates/rustsheet-engine/src/functions/lookup_and_reference/tocol_torow.rs.
- Probe design: one case, the excel-probe skill (excel-graph, tags M7, lookup). Wrap each call as
  `ERROR.TYPE(<call>)` so each is one cell. Cover both orders (NA() then 1/0, and 1/0 then NA()), literal errors,
  error cells, and a `#NULL!`/`#NAME?` pair, for TOCOL and TOROW, and with a one-cell error reference as the array. Add
  a lifted range ignore whose elements are an error and a number against an error scan_by_column, plus one passing
  control per function.

## Acceptance criteria
- [ ] probes/t4685-tocol-torow-two-error-arguments.case.json is committed with excel-graph oracle values and holds
      every form listed in Context.
- [ ] Every probe cell passes, or each cell that does not is listed in a new functions-lookup task (`campaign:
      semantics` with a `## Visible-parity stage` section) filed with `make new-task` that names the case and its
      cells. Those new known failures are recorded with `make eval-baseline`.
- [ ] arrays.md states the two-error rule and cites the probe cells. The two stale notes no longer say UNVERIFIED
      and cite probes/t2365-wrap-expand-tocol-error-lift S!G10:G11 and probes/t3765-stack-error-reference.
      `rg -n 'not probed yet|T-2365 probes' docs/excel-semantics/arrays.md` prints nothing.
- [ ] Any engine change has a unit test in tocol_torow.rs that pins the probed result.
- [ ] `make check-fast` green; no new known-failure line in eval/baseline/ other than those owned as above; reviewers
      approve.

## Out of scope
- The TOCOL/TOROW argument kinds of T-2873 (empty text, an error or a range as ignore or scan_by_column on its own).
- SORT and SORTBY argument order (T-4684).
