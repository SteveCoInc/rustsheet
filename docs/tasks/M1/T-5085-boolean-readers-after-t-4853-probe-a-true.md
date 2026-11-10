---
id: T-5085
title: "Boolean readers after T-4853: probe a TRUE-named sheet in a 3-D span and an Excel-opened xsd:boolean fixture, route the remaining xlsx flags through xsd_boolean"
milestone: M1
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4853]
---

## Goal
Every TRUE/FALSE reader left after T-4853 rests on evidence, and the xlsx flags it missed share its helper.

## Context
These are the reviewers' MINOR findings on PR #4080 (T-4853), deferred because every reviewer had approved.
- **Parse side effect (excel-fidelity).** `name_needs_quoting` now quotes a sheet named TRUE or FALSE, and the lexer's
  `consume_sheet_span` uses it. So an unquoted 3-D span `=TRUE:Sheet2!A1` or `=Sheet1:false!A1` no longer reads as a
  span, while the single-sheet `TRUE!A1` still parses. How Excel parses those, and the FORMULATEXT of a quoted
  `'TRUE:Sheet2'!A1`, are unprobed. Probe them (excel-probe skill) and record the result in references.md.
- **xlsx leniency (excel-fidelity).** `xsd_boolean` (crates/rustsheet-xlsx/src/import/util.rs) reads the four
  xsd:boolean literals exactly. `parse_bool_with_default` trims and lower-cases first. Excel's own reading of `true`,
  `TRUE` and padded values is unprobed (xlsx-storage.md "Boolean attributes and boolean cells (T-4853)", UNVERIFIED).
  Confirm with an Excel-opened fixture, for example `showGridLines="true"`, `cm="TRUE"`, `date1904="true"`.
- **Missed sites (design).** conditional_formatting.rs (`custom`, `stopIfTrue`, `percent`, `bottom`; around lines 262,
  356, 451, 541) and styles.rs `wrapText` (around line 556) still compare `== Some("1")`. Route them through
  `xsd_boolean`, or through `parse_bool_with_default` where a style attribute's lenient read applies. Five `<f>` flag
  arms in sheet_data.rs repeat `xsd_boolean(&value) == Some(true)`; a small helper could share it.
- **Docs (excel-fidelity).** xlsx-storage.md's "That covers ..." list sits after the lenient-read sentence, so it reads
  as if `date1904` and `iterate` were lenient; they are strict, and `sheetView` `tabSelected`/`showGridLines` are
  lenient. Reorder it. T-4853's Outcome says roundtrip-poi improved over 4 cases; it was 5 cases and 14 items
  (`npe.xlsx` R1 1397 -> 1392 too), and it omits that `xsd_boolean` became exact in cafed4601.
- **Out-of-crate reader (design).** crates/rustsheet-eval/src/xlsx_snapshot.rs (around line 872) reads `t="b"` as
  `trim() == "1"` or `true` in any case, which is laxer than `cell_boolean`. Say why it differs, or align it.

## Acceptance criteria
- [ ] The 3-D span and FORMULATEXT cells are probed and recorded; the lexer and printer follow the evidence.
- [ ] An Excel-opened fixture settles the lenient attribute read, or the UNVERIFIED stays with a reason.
- [ ] The listed conditional_formatting.rs and styles.rs flags read through the shared helper; behavior is shown unchanged
      or improved by the roundtrip and corpus gates.
- [ ] xlsx-storage.md and T-4853's Outcome are corrected; typed input reads its boolean through `boolean_text`.
- [ ] `make check` green; reviewers approve.

- **Typed input (design).** model.rs `reenter_printed_input` still reads a typed boolean with
  `to_lowercase().parse::<bool>()`. It accepts the same set as `boolean_text`; switch it so one helper remains.

## Out of scope
- The currency-list builder and the stale `value_needs_quoting` references (T-5082).
