---
id: T-4199
title: "DECIMAL: probe the shapes T-1198 left unprobed (an error radix with over-long text, control characters and blanks before the digits, a blank or empty-text cell) and match"
milestone: M5
lane: functions-math
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
DECIMAL gives Excel's result on the argument shapes that docs/excel-semantics/engineering.md "DECIMAL of unreadable
text" lists as "Not probed". The engine guesses on each of them today.

## Context
- Gap triage 2026-11-04, opus pass (`make eval` on main 28588aa2). Every visibly-off cell has an open owner
  (`scripts/exit_owners.py --all-off`), and the evidence floor and catalog coverage are met. So this task comes from an
  engine guess recorded as unprobed, not from a failing cell. DECIMAL is an M5 function (catalog prior 0.1).
  No open task names it.
- The entry (VERIFIED on probes/decimal-unreadable, 84 cells, T-1198) ends: "Not probed: form feed and other control
  characters at the start; an error radix with over-long text; an empty-cell reference or an array as the text;
  blanks after radix-16 leading zeros."
- Ours (`fn_decimal` / `decimal_from_text`, crates/rustsheet-engine/src/functions/math_and_trigonometry/decimal_text.rs;
  values from a release build of 28588aa2):
  - An error radix is returned before the 255-unit length check: `DECIMAL(REPT("1",300),NA())` is `#N/A`. Excel
    checks the length before the radix range and before a non-numeric radix (`DECIMAL(REPT("1",256),37)` is
    `#VALUE!`), so for an error radix either order is plausible.
  - Only space and tab are skipped at the start. Form feed, vertical tab, CR, NBSP and other C0 controls are
    `#NUM!` (LF is already evidenced `#NUM!`).
  - Radix 16 strips leading zeros, then one `x`, and does not skip blanks after them: `"00 1F"` and `"0x 1F"` are
    `#NUM!`. `"0x 1F"` is evidenced; `"00 1F"`, `"0 0x1F"` and `"00"&CHAR(9)&"1F"` are not.
  - A blank cell and a formula's `""` read as empty text, which is 0. Typed `""` is evidenced as 0, the cell forms
    are not.
  - A multi-cell range or an array as the text lifts, one result per element (probes/t2198-m5-lift-argument-kinds
    covers the lift itself). What is unprobed is an array holding an error or an over-long text beside a valid radix.
- Probe, in a new probes case (excel-probe skill, excel-graph, tag M5). Wrap error results in `ERROR.TYPE`.
  1. Over-long text (256 and 300 units) against radix `NA()`, `1/0`, `"x"`, `1` and `37`, and the same text with a
     literal and a cell-held error radix.
  2. `CHAR(n)&"1F"` in radix 16 and `CHAR(n)&"11"` in radix 2 for n = 9, 10, 11, 12, 13, 28 to 31 and 160, and
     `" "&CHAR(12)&"11"`.
  3. Radix 16: `"00 1F"`, `"0 0x1F"`, `"00"&CHAR(9)&"1F"`, `"x 1F"`, `" 0x1F"` and `"0x1F "`.
  4. A blank cell, a formula `""` cell, a `="  "` cell and a cell holding the number 101 (read through its text),
     each in radix 2 and 36.
  5. `DECIMAL({"11",#N/A,"2"},2)`, `DECIMAL(B1:B3,2)` over the same values, and `DECIMAL({"11";"<300 ones>"},{2,NA()})`.
- Fix in `fn_decimal` / `decimal_from_text`, which are the one definition (CLAUDE.md directive 3). If Excel's skip set
  is a known character class, look for an existing whitespace helper before writing one: `rg "trim_start_matches|is_excel_space" crates/rustsheet-engine/src`.

## Acceptance criteria
- [x] A new probes case (excel-probe skill, excel-graph, canary-verified, tag M5) holds items 1 to 5 above, with a
      sidecar.
- [x] engineering.md "DECIMAL of unreadable text" states the rule for each item, cites the new cells, and drops or
      narrows its "Not probed" line.
- [x] The engine matches every new cell.
- [x] probes/decimal-unreadable, probes/t2198-m5-lift-argument-kinds and every gen-m5 DECIMAL cell still pass. No new
      lines in eval/baseline/*.txt (`make eval-baseline` reports 0 regressions).
- [x] `make check` green; reviewers approve.

## Out of scope
- BASE and the engineering base conversions (BIN2DEC, HEX2DEC and the others). Their argument kinds are T-3124's
  and T-3214's.

## Outcome
- New probe probes/t4199-decimal-unprobed-shapes (80 formulas, 87 scored cells, excel-graph 2026-11-04) covers
  items 1 to 5. The array constant `{2,NA()}` is not a valid Excel formula (the workbook is refused), so item 5
  builds its arrays with `CHOOSE`.
- One engine change: `fn_decimal` checks the 255-unit length before it reads the radix, so over-long text is
  `#VALUE!` against an error radix too (Excel: `DECIMAL(REPT("1",256),NA())` is `#VALUE!`; ours was `#N/A`).
  Every other shape already matched: the skip set is exactly space and tab, radix 16 skips blanks, zeros, one
  `x` in that order and nothing after, blank and empty-text cells are 0, and arrays lift element by element.
- engineering.md "DECIMAL of unreadable text" states each rule with the new cells; its "Not probed" line now
  names only CHAR(1) to CHAR(8) and CHAR(14) to CHAR(27).
- Eval delta: probes +1 case, 87 cells, all passing; decimal-unreadable (84) and t2198-m5-lift-argument-kinds
  (104) still pass; no baseline lines added.
