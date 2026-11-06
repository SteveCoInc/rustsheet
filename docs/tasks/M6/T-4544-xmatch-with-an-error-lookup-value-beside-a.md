---
id: T-4544
title: "XMATCH with an error lookup_value beside a lifted match_mode or search_mode, and an error-cell search_mode against each bad match_mode: probe and match"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
XMATCH's error order matches Excel, on excel-graph evidence, in the forms docs/excel-semantics/lookup.md has inferred
by analogy: an error lookup_value beside a lifted match_mode or search_mode, the first element of an array mode, and
an error search_mode against each kind of bad match_mode. lookup.md cites the cells for each, and the two entries that
call these forms UNVERIFIED are VERIFIED or corrected.

## Context
- Gap triage 2026-11-06 (`make eval` on main 25779368). No lookup function has an open visibly-off cell, so the work
  left in this lane is the rules lookup.md takes on trust:
  - lookup.md, the T-1958 XMATCH search_mode bullet: "The first array-mode elements are unprobed, as is an error
    lookup_value next to a lifted search_mode (`XMATCH(NA(),A1:A3,0,{1;-1})`, implemented as a 2x1 of `#N/A` by
    analogy with MATCH's match_type, UNVERIFIED)."
  - lookup.md, "gen-m6 arg-pairs batch (T-3496)": "Status: UNVERIFIED (probe pending; cells are oracle values, the
    general rule is not probed)". The rule there is that with search_mode a `#DIV/0!` cell, an out-of-range numeric
    match_mode (7, -2.5, "12") gives `#DIV/0!`, while non-numeric text in match_mode still wins with `#VALUE!`.
    Implemented by T-3542 and T-3667 from the cells of four gen-m6 cases only.
- T-2746 probed the XLOOKUP form of the first question (probes/t2746-xlookup-error-lookup-array-modes: an error
  lookup_value beats an invalid mode element). T-2637 (research, todo) covers the MATCH form. Nothing covers XMATCH.
- What the engine does today (a scratch run on main 25779368 with A1:A3 = 1, 2, 3, D5 `=NA()` and D6 `=1/0`):
  `XMATCH(NA(),A1:A3,0,{1;-1})` is 2x1 with element 1 `#N/A`; `XMATCH(2,A1:A3,9,D5)` is `#N/A`;
  `XMATCH(2,A1:A3,D6,D5)` is `#DIV/0!`; `XMATCH(2,A1:A3,"x",D5)` is `#VALUE!`; `XMATCH(2,A1:A3,9,NA())` is `#N/A`;
  `XMATCH(2,A1:A3,9,1/0)` and `XMATCH(2,A1:A3,0,D6)` are `#DIV/0!`.
- Probe (excel-probe skill, excel-graph, tag M6). Read every result through ERROR.TYPE, and each multi-cell result
  through ROWS and INDEX of each element:
  - an error lookup_value (`NA()`, `1/0`, a cell holding `#N/A`) with match_mode `{0;9}`, `{9;0}`, `{9;8}` and with
    search_mode `{1;-1}`, `{1;3}`, `{3;1}`, `{3;5}`; then both modes as arrays at once; then a 2-cell range of modes in
    place of each array;
  - the first element of each array mode with a valid lookup_value (`INDEX(XMATCH(2,A1:A3,{0;1}),1,1)`,
    `INDEX(XMATCH(2,A1:A3,0,{1;-1}),1,1)`, `INDEX(XMATCH(2,A1:A3,0,{3;1}),1,1)`);
  - search_mode an error (a literal `#N/A`, `NA()`, `1/0`, a cell holding `#N/A` or `#DIV/0!`) against match_mode
    valid (0), out of range (9, -2.5), numeric text ("12", "1"), non-numeric text ("x"), empty text, TRUE, a blank
    cell and an error of another kind (so it shows which of two errors wins).
- Commit the engine's values as predictions before the fill (docs/research/t4544/). The rule is chosen on these cells.
  Give it a small hold-out as well: a second set of mode pairs, generated, with its predictions frozen before its fill
  (decision 0188 R4).
- Code: `fn_xmatch` (crates/rustsheet-engine/src/functions/lookup_and_reference/xmatch.rs) and the mode readers it
  shares with XLOOKUP (crates/rustsheet-engine/src/functions/xlookup.rs). If both functions turn out to read their
  modes in one order, read it through one helper (directive 3).

## Acceptance criteria
- [ ] probes/t4544-xmatch-error-modes and its hold-out case (excel-graph, canary-verified) cover every form in
      Context. Each case's predictions were committed before its fill.
- [ ] Every cell of both cases passes. Any cell that does not is recorded in lookup.md and owned by a task filed with
      `make new-task` (`campaign: semantics` with a `## Visible-parity stage` section) and named in the Outcome.
- [ ] lookup.md: both UNVERIFIED passages quoted in Context are replaced by the evidenced rule, with the cells cited.
- [ ] The cells of probes/lookup-argument-modes, t2746-xlookup-error-lookup-array-modes and the four gen-m6 cases of
      the T-3496 entry still pass. No new known failures in any baseline; `make check` green; reviewers approve.

## Out of scope
- MATCH's error lookup_value against a lifted match_type (T-2637).
- XLOOKUP's if_not_found lift (T-4543).
- Near-integer modes (T-2656, done).
