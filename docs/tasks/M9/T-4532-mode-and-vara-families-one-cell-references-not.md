---
id: T-4532
title: "MODE and VARA families: one-cell references not yet probed ((A1), sheet-qualified, LET-bound, TAKE, OFFSET, INDIRECT, a defined name) to an empty, text or boolean cell: probe and match"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The one-cell reference forms that docs/excel-semantics/statistics.md leaves UNVERIFIED for the
MODE family (MODE, MODE.SNGL, MODE.MULT) and the VARA family (VARA, VARPA, STDEVA, STDEVPA) have
Excel evidence, and the engine matches it.

## Context
- "MODE: a one-cell range reference to an empty cell is #VALUE! (T-2256)": VERIFIED for
  `Data!C11:C11`-style ranges and a blank single-cell reference (gen-m9 775d053de498b8c7 A352,
  a63d1c7d6f96c56d A381, 5cebac1f02bf30f2 A183, 329b177b5d5165ad A172). "UNVERIFIED: a one-cell
  reference returned by INDEX, OFFSET or IF, or a defined name (ours: `#VALUE!` too)." Ours:
  `functions/statistical/mode_functions.rs` (T-2471).
- "VARA family: single-cell reference arguments": VERIFIED for a plain reference and one picked by
  IF, INDEX or XLOOKUP, and array literals (probes/vara-single-ref-picked, 41 cells). "Forms `(A1)`,
  `Sheet1!A1`, LET-bound and TAKE-picked cells were not probed: UNVERIFIED."
- Both entries ask the same question (does a reference reached another way read as a reference
  cell or as a value), so one probe answers both. No open task owns either sentence (T-1291,
  T-1567, T-1617, T-2471 are done).
- Opus: the reference-ness of each form decides between three readings (empty cell skipped, empty
  as `#VALUE!` for MODE, text as 0 for VARA versus a direct value's coercion), and a mismatch may
  touch the shared argument reader in functions/statistical/ rather than one function.
- Use the excel-probe skill (excel-graph oracle).

## Acceptance criteria
- [ ] A committed probe case with an excel-graph sidecar holds MODE, MODE.SNGL, MODE.MULT, VARA,
      VARPA, STDEVA and STDEVPA beside a numeric range that has a mode / a variance, with one extra
      argument in each of these forms: `(B1)`, `Sheet1!B1`, `LET(r,B1,...)`, `TAKE(B1:B3,1)`,
      `OFFSET(B1,0,0)`, `INDIRECT("B1")`, `INDEX(B1:B3,1)`, `IF(TRUE,B1)`, a defined name for B1, and
      a one-cell range `B1:B1`, where B1 is in turn empty, `"x"`, `"2"`, TRUE.
- [ ] Every new cell passes, or each remaining failure is owned by a filed task named in
      statistics.md.
- [ ] statistics.md's MODE and VARA entries replace their UNVERIFIED sentences with the probed
      rules and cite the cells.
- [ ] No new known-failure lines in eval/baseline/probes.txt or eval/baseline/gen-m9.txt other than
      the new case's owned cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The direct-literal VARA forms (`VARA(1,"x")`, probes/stat-argument-modes S!D37-D39).
- Variance digits (the one-pass/two-pass switch).
