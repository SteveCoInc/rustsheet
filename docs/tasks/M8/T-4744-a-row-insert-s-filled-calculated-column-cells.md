---
id: T-4744
title: "A row insert's filled calculated-column cells take Excel's style (s=1 in the cached workbook) (T-4692 review follow-up)"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The cells a row insert fills with a calculated column's formula (T-4692) get the style Excel gives them, and a probe
scores it (number format or another observable property of `s="1"`).

## Context
- Found by T-4692's excel-fidelity review (PR #3784). In the cached workbooks of probes/table-edits/t4692-* (eval/.cache/oracle/excel-graph/<input_sha256>.xlsx), every
  filled cell is saved `<c r="C3" s="1">` while the cells the typed fill wrote have no `s`; ours keeps the inserted
  cell's own style (`Model::fill_inserted_data_rows`, calculated_columns.rs). xlsx-storage.md, "A calculated column is
  saved as `calculatedColumnFormula` plus one formula per row", records it as unscored.
- First find what xf 1 is in those workbooks (styles.xml: a number format, the table's dataDxf, or nothing visible).
  If nothing observable differs (CELL("format"), a number format, TEXT of the value), close this as moot with a note.

## Acceptance criteria
- [x] What `s="1"` is, from the cached styles.xml, recorded in xlsx-storage.md.
- [x] If observable: a probe cell scores it on excel-graph, and ours matches it. (n/a: not observable, see Outcome)
- [ ] `make check` green; reviewers approve.

## Out of scope
- The style of cells a typed fill writes (T-2957).

## Outcome
Moot: nothing observable. Refilled probes/table-edits/t4692-insert-row-inside on excel-graph (2026-11-07; values
unchanged, sidecar not committed) and read its cached workbook: the filled C3 is `s="1"`, and cellXfs 1 is xf 0
(`numFmtId="0"`, font/fill/border 0) plus `applyNumberFormat="1"`; the Calc column's `dataDxfId="0"` is a General
`numFmt`. No CELL("format"), number format or TEXT of the value can tell it from xf 0, so no probe cell can score it
and the engine is unchanged. xlsx-storage.md records the xf and marks the mismatch moot. Eval delta: none (docs only).
