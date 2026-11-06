---
id: T-4543
title: "XLOOKUP's if_not_found and ADDRESS's sheet_text over an array or range: probe and match, and retire lookup.md's stale UNVERIFIED list"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
XLOOKUP with an array or multi-cell `if_not_found` and ADDRESS with an array or multi-cell `sheet_text` give Excel's
values and shapes, on excel-graph evidence. docs/excel-semantics/lookup.md's "Lookup and
reference function argument modes" no longer lists as UNVERIFIED the arguments that other probes have covered since.

## Context
- Gap triage 2026-11-06 (`make eval` on main 25779368). Every visibly-off cell has an open owner (`make unowned-off
  ALL=1` lists none), and no lookup or criteria function has an attributed visibly-off cell, so the work left in this
  lane is the semantics lookup.md still leaves open. lookup.md, "Lookup and reference function argument modes", says:
  "UNVERIFIED (not probed): OFFSET's cols, height and width, XLOOKUP/XMATCH search_mode, XLOOKUP's if_not_found,
  ADDRESS's sheet_text, INDIRECT's a1 and TRIMRANGE's trim_cols".
- Oracle-filled cases that pass today have since probed most of that list:
  - OFFSET cols: `OFFSET(A1,0,{0,1})` in probes/reference-lift-elements and reference-lift-consumers-b1c;
  - OFFSET height and width: `OFFSET(A1,0,0,{..})` in probes/t2065-lift-small, reference-lift-consumers-b2 and
    t2816-lift-sumsq; `OFFSET(A1,0,0,1,{..})` in probes/reference-lift-consumers;
  - INDIRECT a1: `INDIRECT("B1",{1;0})` and its siblings in probes/t2577-indirect-ref-array;
  - TRIMRANGE trim_cols: `TRIMRANGE(A1:B3,0,{0;1})` in probes/reference-lift-pass-through-b;
  - XLOOKUP and XMATCH search_mode: lookup.md's own later entries (probes/xlookup-array-search-mode-indirect, and the
    T-1958 XMATCH entry).
  Check each one against its oracle cells before you cite it. What remains unprobed is XLOOKUP's if_not_found and
  ADDRESS's sheet_text.
- What the engine does today (a scratch run on main 25779368 with A1:A3 = 1, 2, 3, B1:B3 = 10, 20, 30 and
  D1:D2 = "x", "y"):
  - `XLOOKUP(9,A1:A3,B1:B3,{"p";"q"})` (a miss) returns the whole 2x1 array (`ROWS` is 2 and the second element is
    "q"). `XLOOKUP(2,A1:A3,B1:B3,{"p";"q"})` (a hit) is one cell (`ROWS` is 1). So if_not_found is taken whole, not
    lifted. The question is whether Excel lifts it (a 2x1 of 20, 20 on a hit) or returns it whole. The only Excel cell
    with an array if_not_found is a gen case with an array lookup_value:
    `XLOOKUP({2;9},Data!A1:A4,Data!B1:B4,{"x";"y"})`.
  - `ADDRESS(1,1,1,TRUE,{"S1";"S2"})` is 2x1, and element 2 of `ADDRESS(1,1,1,TRUE,D1:D2)` is "y!$A$1". Every argument
    lifts (`rustsheet-eval lift-kinds` gives ADDRESS `Lift` five times), but no Excel cell backs the lift of
    sheet_text.
- Probe (excel-probe skill, excel-graph, tag M6) on the data above. Read each multi-cell result through ROWS, COLUMNS
  and INDEX of each element, and each error result through ERROR.TYPE:
  - XLOOKUP if_not_found: a 2x1 and a 1x2 array literal and a 2x1 range, each on a miss and on a hit; a 2x2 array;
    the 2x1 array beside an array lookup_value of the same length and of a different one
    (`XLOOKUP({2;9},..,{"p";"q"})`, `XLOOKUP({2;9;7},..,{"p";"q"})`); an if_not_found array that holds an error
    element; a 2-D return_array (a row return) with a 2x1 if_not_found on a miss.
  - ADDRESS sheet_text: a 2x1 array literal and a 2x1 range; each beside a lifted row_num of the same length and of a
    different one; an error element (`{"S1";#N/A}`), a number element and an empty-text element.
- Commit the engine's values as predictions before the fill (docs/research/t4543/). The rule is structural and chosen
  on these cells, so this is not a fit.

## Acceptance criteria
- [x] probes/t4543-xlookup-not-found-address-sheet-lift (excel-graph, canary-verified) covers every form in Context,
      and its predictions were committed before the fill.
- [x] Every cell of the case passes. Any cell that does not is recorded in lookup.md and owned by a task filed with
      `make new-task` and named in the Outcome. A wrong type, error, value or shape gets `campaign: semantics` and a
      `## Visible-parity stage` section.
- [x] In lookup.md, the "UNVERIFIED (not probed)" sentence quoted in Context names only what is still unprobed, and
      each argument it drops cites the case and cells that cover it.
- [x] No new known failures in any baseline; `make check` green; reviewers approve.

## Out of scope
- XMATCH with an error lookup_value beside lifted modes (T-4544).
- The error order inside a CHOOSECOLS/CHOOSEROWS index list (T-2372) and the M6 lookup argument kinds (T-2865).
- INDIRECT's sheet-prefix spellings (T-4545) and its a1 read from a cell (T-4538).
- TRANSPOSE of an error (T-3966).

## Outcome
- Probe probes/t4543-xlookup-not-found-address-sheet-lift (28 questions, 252 cells, excel-graph, canary-verified,
  2026-11-06), tagged M6. The engine's values were committed first (docs/research/t4543/predictions.md); the rule is
  structural, read off these cells, so it is not a fit.
- Result: 250 of 252 cells pass as predicted.
  - XLOOKUP's if_not_found is not lifted. A scalar miss returns it whole (array, 1x2, 2x2, range), a hit ignores it.
    Under an array lookup_value every miss takes its first element.
  - ADDRESS's sheet_text lifts like the other arguments.
  - The two misses, W45:W46: under an array lookup_value Excel keeps a range if_not_found as a reference element. The
    grid shows `#VALUE!`, and INDEX hands back the range and spills "x", "y". Ours is `#VALUE!` and a blank. Owner:
    T-4563 (opus, `campaign: semantics`, with a Visible-parity stage). It is not fixed here: the carrier
    (SPEC-0018's nested cell or a reference element) changes unprobed readers (ISREF, lifted reads, aggregates), and
    T-4563 probes those first.
- lookup.md: a new entry, "XLOOKUP's if_not_found and ADDRESS's sheet_text over an array". The "UNVERIFIED (not
  probed)" sentence now names only the CHOOSECOLS/CHOOSEROWS error order. Each argument it drops cites the case and
  cells that cover it, checked against their oracle cells.
- Eval delta: +1 case (252 cells: 250 pass, 2 new-case known failures owned by T-4563). No change to any existing case;
  0 regressions. "No new known failures" holds for the existing cases. The new case's two cells are new coverage, not
  regressions.
