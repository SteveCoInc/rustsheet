---
id: T-4533
title: "xlsx rows and cells without an r attribute: probe how Excel places them (after an explicit r, a backwards explicit r, mixed rows) and match the importer"
milestone: M11
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The positional placement of r-less `<row>` and `<c>` elements in an xlsx has Excel evidence, so
docs/excel-semantics/xlsx-storage.md "Cells and rows without r are positional" is VERIFIED (or
rewritten), and `rustsheet-xlsx` places every such cell where Excel does.

## Context
- The entry (T-0181, done): "Status: UNVERIFIED (probe pending; ECMA-376 and POI files only, no
  Excel observation)". The evidence is ECMA-376 Part 1 §18.3.1.73 and §18.3.1.4 and three POI
  files with no formulas (poi/54288.xlsx, poi/59746_NoRowNums.xlsx, poi/InlineString.xlsx), so no
  Excel value has ever been scored on an r-less cell. No open task owns the probe.
- Ours: `start_row` and `start_cell` in crates/rustsheet-xlsx/src/import/sheet_data.rs: an r-less
  row is the row after the previous one (row 1 if first), an r-less cell the column after the
  previous cell of its row (column A if first); past XFD or row 1,048,576 is an import error (a
  DEVIATION entry, directive 5).
- Unsettled shapes: an r-less cell after an explicit `r` that jumps ahead; an explicit `r` that goes
  backwards or repeats a column (does the later cell win, does Excel refuse the file?); an r-less row
  after an explicit row number; an r-less cell in a row whose own `r` is missing; a shared formula
  whose anchor is r-less.
- Opus: the probe needs hand-written sheet XML (`read_parts`/`patch_parts`/`write_parts` in
  tools/src/rustsheet_tools/oracle/package.py, T-0014,
  not spec2xlsx's normal output), cells holding formulas that read the placed values (`=SUM(A1:E5)`,
  `=ADDRESS(ROW(),COLUMN())` beside them), and an oracle run that reports whether Excel opened the
  file at all; a refused file is a result, recorded as such.
- Use the excel-probe skill (excel-graph oracle).

## Acceptance criteria
- [ ] A committed probe case (xlsx built by a committed script, with an excel-graph sidecar) covers
      the shapes above, each read back by formulas on the sheet, or records which files Excel
      refused to open.
- [ ] Every new cell passes, or each remaining failure is owned by a filed task named in
      xlsx-storage.md.
- [ ] xlsx-storage.md's entry states the probed rule, cites the cells, and its status is VERIFIED
      or says which shapes stay unprobed.
- [ ] No new known-failure lines in any eval/baseline/*.txt other than the new case's owned cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The DEVIATION limits past XFD and row 1,048,576 (directive 5).
- Export: the exporter always writes `r`.
