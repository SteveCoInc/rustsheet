---
id: T-4198
title: "TEXTSPLIT with a one-cell error reference as a delimiter (B1 and B1:B1 disagree in ours) and an array pad_with against an error element of a delimiter list: probe and match"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
TEXTSPLIT gives Excel's result when a delimiter argument is a reference to one error cell, in every spelling of that
reference. It also gives Excel's result when an array pad_with meets an error element of a delimiter list. Both are
recorded as unprobed in docs/excel-semantics/text.md "TEXTSPLIT: argument errors first, an all-empty result".

## Context
- Gap triage 2026-11-04, opus pass (`make eval` on main 28588aa2). Every visibly-off cell has an open owner
  (`scripts/exit_owners.py --all-off`: 3 cells, all owned by deferred tasks), and the functions-text lane has no ready
  opus task. So this task comes from an engine guess that text.md lists as unprobed, not from a failing cell.
- Ours is inconsistent today. With B1 = `=NA()` and B2 = `=1/0` (release build of 28588aa2):
  `TEXTSPLIT("abc",B1,,1/0)` is `#N/A`, but `TEXTSPLIT("abc",B1:B1,,1/0)` is `#DIV/0!`. In `fn_textsplit`
  (crates/rustsheet-engine/src/functions/text/textsplit.rs) a one-cell reference evaluates to a scalar error, which
  sets `delimiter_error`: an argument error, the first one after the text's. A one-cell range evaluates to a 1x1 array
  that holds the error, which is an error element of a list and comes after ignore_empty. Excel normally reads `B1` and
  `B1:B1` alike, so at least one of the two is probably wrong. Note that text.md's "Unprobed" sentence says ours gives
  `#DIV/0!` for the `B1` form, which is no longer true. Fix the sentence along with the rule.
- Precedent for one-cell error references: arrays.md "A one-cell error reference is a 1x1 array in TOCOL, TOROW and
  FILTER (T-3497)". There Excel reads a one-cell error reference as a 1x1 array holding the error, not as an error
  argument, while a literal error stays an argument error. TEXTSPLIT may or may not follow it. Only Excel decides.
- Existing evidence that must keep passing: probes/textsplit-error-order A1-A18 (excel-graph, 2026-10-23). There
  A10 `TEXTSPLIT("abc",B1,,1/0)` has B1 blank and gives `#DIV/0!`, A2 a literal `NA()` delimiter gives `#N/A`, and A3
  `{",",#N/A}` gives `#DIV/0!`. Also probes/textsplit-empty-delimiter, textsplit-empty-text, t2581-textsplit-textbefore
  and the gen-m7 TEXTSPLIT cells that text.md cites.
- Probe, in a new probes case (excel-probe skill, excel-graph, tag M7). Read each result through `ERROR.TYPE` or
  `ARRAYTOTEXT(..,1)`, as textsplit-error-order does. Put `#N/A` and `#DIV/0!` in error cells.
  1. col_delimiter as a one-cell error reference in each form: `B1`, `B1:B1`, `$B$1`, `Calc!B1`, `INDEX(B1:B2,1)`,
     `LET(d,B1,TEXTSPLIT("abc",d,,1/0))`, and a defined name for B1. Pair each with ignore_empty `1/0` (to separate
     the argument error from the list element error), with ignore_empty `"x"`, and with valid later arguments.
  2. The same for row_delimiter (`TEXTSPLIT("abc",",",B1,1/0)`, `B1:B1`), and with an error text argument
     (`TEXTSPLIT(B2,B1)`), to confirm that the text's error still comes first.
  3. A one-cell reference to a text cell holding `,` (control: it splits), and a two-cell range B1:B2 of errors.
  4. An array pad_with against an error element of a delimiter list: `TEXTSPLIT("a,b;c",{",",#N/A},";",,,{1,2})`,
     `TEXTSPLIT("abc",{",",#N/A},,,,{1,2})`, and the same with the error list given as a range (`B1:C1` with C1 `,`).
- Fix in `fn_textsplit` / `split_delimiters` (or `Model::delimiter_list` in functions/text/lifted.rs, if TEXTBEFORE and
  TEXTAFTER share the rule; probe their delimiter in the `B1` and `B1:B1` forms too, two cells each, so a shared helper
  does not change them unprobed). Search for the T-3497 helpers (`one_cell_error_ref_as_array` and
  `Model::is_one_cell_variable` in functions/spill_functions.rs) before writing a new one. Reuse them if the rule is the
  same (CLAUDE.md directive 3).

## Acceptance criteria
- [ ] A new probes case (excel-probe skill, excel-graph, canary-verified, tag M7) holds items 1 to 4 above, with a
      sidecar.
- [ ] text.md "TEXTSPLIT: argument errors first, an all-empty result" states the rule for one-cell error references
      and for the pad_with/list-element order, cites the new cells, and drops or narrows its "Unprobed" sentence.
- [ ] The engine matches every new cell. `TEXTSPLIT("abc",B1,,1/0)` and `TEXTSPLIT("abc",B1:B1,,1/0)` give the same
      result whenever Excel's do, and any difference is backed by a cited cell.
- [ ] probes/textsplit-error-order, textsplit-empty-delimiter, textsplit-empty-text, t2581-textsplit-textbefore and
      every gen-m7 TEXTSPLIT cell still pass. No new lines in eval/baseline/*.txt (`make eval-baseline` reports 0
      regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- TEXTBEFORE/TEXTAFTER/TEXTSPLIT argument kinds (an error text, empty text in ignore_empty or match_mode, pad_with
  kinds): T-2876 (research). Leave those cells to it.
- The lift of TEXTSPLIT over a text array, which is already evidenced.
