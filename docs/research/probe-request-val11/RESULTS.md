# Probe request val11: results (2026-11-13)

Everything Graph could run unattended, through excel-graph (Excel for the web). The Script Lab and by-hand items
(sections 5 and 6, val10's section 2 and 3, and the by-hand halves of items 1, 3, 4, 6, 7, 25 and 26) were **not run**:
this oracle has no Office.js or browser. They stay open for the oracle owner.

## Section 3: excel-graph fills

- `probes`: 86 of the 88 pending cases were filled (85 have their sidecar beside the case; see the parked one below) of val9/val10/val11 have sidecars (the 48 M4 `table-edits` cases among them).
- `probes-numeric`: `t5524-chisq-df1-complement-fresh` has a sidecar and a math sidecar.
- Refused by Excel (an answer, `PROBE-REQUEST.md`): `table-edits/t5123-table-delete-rows-all-data-side-by-side`
  (delete rows 4:7) and `table-edits/t5358-no-header-delete-rows-top-straddle` (delete rows 2:4), both 409
  `InsertDeleteConflict` ("This won't work because it would move cells in a table on your worksheet"). They stay
  pending.
- `t5107-search-fresh-tails` was filled, but its Excel values are parked at `docs/research/probe-request-val9/t5107-search-fresh-tails.oracle.json`, not beside the case. One of its points runs past the math generator's 600 s limit (`PointTimeoutError`, T-3680), and the `eval-math-check` gate will not pass a filled case without math values. Move the sidecar back once the definition has a deterministic budget.
- Typed entries refused by Excel, recorded in the sidecar as REFUSED: `typed-formula-length-utf16-and-open-call`
  Sheet1!A6, A16, A21 and A26.
- Visibly-off cells have owners: T-4579, T-4894, T-4994, T-2574, T-5107, T-4922, T-4924, T-4925 (amended) and
  T-90011 to T-90020 (new).

## val9 accuracy batches (LINEST-R42.md)

The four `r4-T-2574` batches were copied into `eval/accuracy/tables/` for the fill and taken out again, as asked. Their
sidecars are committed beside the batches: `t2574-accuracy/{LINEST,TREND,LOGEST,GROWTH}/r4-T-2574.oracle.json`.

## Item 2: chart-save

All 22 through Graph, see `chart-save-probe/RESULTS.md`. In short: parts already in Excel 16's form come back byte for
byte, series ids kept (pie and doughnut gain data-point ids). Parts not in Excel's form are rewritten.
`123233_charts.xlsx` is refused (501 FileCorruptTryRepair).

## Item 27: sheet-defaults render

`sheet-defaults-probe/refs/sheet-defaults.render.json` and `.pdf` (12 sheets, 186 cells).

## Section 4 and val10/val9 runs by path

Each file's output is beside it: `<file>.graph.json` (values), `<file>.graph.xlsx` (the recalculated workbook the tool
keeps) and `<file>.graph.err` (stderr: the sheet ops, or the refusal). The shift probes ran with
`tools-case-ops.patch` applied, and it was reverted after. The four `*-records` resaves are in
`shift-probes-2026-11-11/records/` with `resave.json`.

There are two kinds of refusal:

- **Excel refused.** These are answers: `FileCorruptTryRepair` at `createSession`, or a 4xx on a sheet op.
- **Our tool refused.** Excel did not answer these:
  - The canary failed. Precision as displayed: all three `pad-*` cases, but not the control. With the option set,
    Excel for the web rounded the canary's `=SQRT(2)` to 1.4142135623731, its displayed 15 digits, so the option
    applies on the web, and to every sheet. Since the tool refuses such values, item 4's numbers need Script Lab
    (its `cases.js`) or a recalc that omits the canary.
  - The canary could not be embedded. Three missing-parts packages: one has a dangling sheet relationship, and two
    use `SHEETS()`, whose answer the canary sheet would change. Running them needs a canary-free session.

| File | Result |
|---|---|
| `probe-request-val11/cfix-loadvals-value-whitespace/value-ws-booleans.xlsx` | refused: OracleRefusedError: FileCorruptTryRepair: We're sorry, but something went wrong with this file. |
| `probe-request-val11/cfix-loadvals-value-whitespace/value-ws-date.xlsx` | values (`.graph.json`) |
| `probe-request-val11/cfix-loadvals-value-whitespace/value-ws-errors.xlsx` | values (`.graph.json`) |
| `probe-request-val11/cfix-loadvals-value-whitespace/value-ws-number-forms.xlsx` | refused: OracleRefusedError: FileCorruptTryRepair: We're sorry, but something went wrong with this file. |
| `probe-request-val11/cfix-loadvals-value-whitespace/value-ws-numbers.xlsx` | values (`.graph.json`) |
| `probe-request-val11/cfix-loadvals-value-whitespace/value-ws-only-b.xlsx` | values (`.graph.json`) |
| `probe-request-val11/cfix-loadvals-value-whitespace/value-ws-only-e.xlsx` | values (`.graph.json`) |
| `probe-request-val11/cfix-loadvals-value-whitespace/value-ws-only-n.xlsx` | values (`.graph.json`) |
| `probe-request-val11/cfix-loadvals-value-whitespace/value-ws-shared-index.xlsx` | refused: OracleRefusedError: FileCorruptTryRepair: We're sorry, but something went wrong with this file. |
| `probe-request-val11/linked-name-legacy-probe/linked-name-legacy.xlsx` | values (`.graph.json`) |
| `probe-request-val11/cse-row-column-area-probe/cse-row-column-area.xlsx` | values (`.graph.json`) |
| `probe-request-val11/external-link-own-names/external-link-own-names.xlsx` | values (`.graph.json`) |
| `probe-request-val10/cfix-missing-parts-probe/package-comments-part-missing.xlsx` | values (`.graph.json`) |
| `probe-request-val10/cfix-missing-parts-probe/package-name-scoped-to-no-sheet.xlsx` | refused: OracleRefusedError: FileCorruptTryRepair: We're sorry, but something went wrong with this file. |
| `probe-request-val10/cfix-missing-parts-probe/package-no-styles-part.xlsx` | values (`.graph.json`) |
| `probe-request-val10/cfix-missing-parts-probe/package-row-style-negative.xlsx` | values (`.graph.json`) |
| `probe-request-val10/cfix-missing-parts-probe/package-row-style-past-cellxfs.xlsx` | refused: OracleRefusedError: FileCorruptTryRepair: We're sorry, but something went wrong with this file. |
| `probe-request-val10/cfix-missing-parts-probe/package-sheet-empty-rid.xlsx` | refused: OracleError: refusing to recalculate: xl/worksheets/sheet2.xml uses _xlfn.SHEETS(), whose answer the canary sheet would change |
| `probe-request-val10/cfix-missing-parts-probe/package-sheet-part-missing.xlsx` | refused: OracleError: cannot embed the canary: xl/_rels/workbook.xml.rels points at xl/worksheets/sheet9.xml, which is not in the package |
| `probe-request-val10/cfix-missing-parts-probe/package-sheet-without-relationship.xlsx` | refused: OracleError: refusing to recalculate: xl/worksheets/sheet2.xml uses _xlfn.SHEETS(), whose answer the canary sheet would change |
| `probe-request-val10/cfix-missing-parts-probe/package-styles-dangling-relationship.xlsx` | refused: OracleRefusedError: FileCorruptTryRepair: We're sorry, but something went wrong with this file. |
| `probe-request-val10/cfix-missing-parts-probe/package-styles-relocated.xlsx` | values (`.graph.json`) |
| `probe-request-val10/cfix-missing-parts-probe/package-table-part-missing.xlsx` | refused: OracleRefusedError: FileCorruptTryRepair: We're sorry, but something went wrong with this file. |
| `probe-request-val11/table-style-info-probe/table-control-opposite.xlsx` | values (`.graph.json`) |
| `probe-request-val11/table-style-info-probe/table-control.xlsx` | values (`.graph.json`) |
| `probe-request-val11/table-style-info-probe/table-no-column-stripes.xlsx` | values (`.graph.json`) |
| `probe-request-val11/table-style-info-probe/table-no-first-column.xlsx` | values (`.graph.json`) |
| `probe-request-val11/table-style-info-probe/table-no-flags.xlsx` | values (`.graph.json`) |
| `probe-request-val11/table-style-info-probe/table-no-last-column.xlsx` | values (`.graph.json`) |
| `probe-request-val11/table-style-info-probe/table-no-row-stripes.xlsx` | values (`.graph.json`) |
| `probe-request-val11/table-style-info-probe/table-no-style-info.xlsx` | values (`.graph.json`) |
| `probe-request-val11/table-style-info-probe/table-style-defined.xlsx` | values (`.graph.json`) |
| `probe-request-val11/table-style-info-probe/table-style-undefined.xlsx` | values (`.graph.json`) |
| `probe-request-val11/font-flag-off-probe/font-flag-empty.xlsx` | values (`.graph.json`) |
| `probe-request-val11/font-flag-off-probe/font-flag-padded.xlsx` | values (`.graph.json`) |
| `probe-request-val11/font-flag-off-probe/font-flag-upper-case.xlsx` | values (`.graph.json`) |
| `probe-request-val11/font-flag-off-probe/font-flag-word.xlsx` | values (`.graph.json`) |
| `probe-request-val11/font-flag-off-probe/font-flags.xlsx` | values (`.graph.json`) |
| `probe-request-val11/malformed-records-probe/col-overlap-control.xlsx` | values (`.graph.json`) |
| `probe-request-val11/malformed-records-probe/hyperlink-reversed-control.xlsx` | values (`.graph.json`) |
| `probe-request-val11/malformed-records-probe/row-repeated-control.xlsx` | values (`.graph.json`) |
| `probe-request-val11/malformed-records-probe/col-overlap.xlsx` | values (`.graph.json`) |
| `probe-request-val11/malformed-records-probe/hyperlink-reversed.xlsx` | values (`.graph.json`) |
| `probe-request-val11/malformed-records-probe/row-repeated.xlsx` | values (`.graph.json`) |
| `probe-request-val11/import-load-probe/merges-overlap.xlsx` | refused: OracleRefusedError: FileCorruptTryRepair: We're sorry, but something went wrong with this file. |
| `probe-request-val11/import-load-probe/numfmt-col-row.xlsx` | values (`.graph.json`) |
| `probe-request-val11/import-load-probe/sizes-default-custom.xlsx` | values (`.graph.json`) |
| `probe-request-val11/import-load-probe/sizes-default-plain.xlsx` | values (`.graph.json`) |
| `probe-request-val11/formula-string-value-probe/formula-string-value-out-of-range.xlsx` | refused: OracleRefusedError: FileCorruptTryRepair: We're sorry, but something went wrong with this file. |
| `probe-request-val11/formula-string-value-probe/formula-string-value.xlsx` | values (`.graph.json`) |
| `probe-request-val11/transition-evaluation-probe/transition-evaluation-control.xlsx` | values (`.graph.json`) |
| `probe-request-val11/transition-evaluation-probe/transition-evaluation.xlsx` | values (`.graph.json`) |
| `probe-request-val11/precision-as-displayed-probe/pad-table-control.xlsx` | values (`.graph.json`) |
| `probe-request-val11/precision-as-displayed-probe/pad-table.xlsx` | refused: OracleError: canary failed, so this workbook was not recalculated; refusing its values: |
| `probe-request-val11/precision-as-displayed-probe/pad-unrounded-fullcalc.xlsx` | refused: OracleError: canary failed, so this workbook was not recalculated; refusing its values: |
| `probe-request-val11/precision-as-displayed-probe/pad-unrounded.xlsx` | refused: OracleError: canary failed, so this workbook was not recalculated; refusing its values: |
| `probe-request-val11/chart-series-name-probe/series-control.xlsx` | values (`.graph.json`) |
| `probe-request-val11/chart-series-name-probe/series-bare-name.xlsx` | values (`.graph.json`) |
| `probe-request-val11/chart-series-name-probe/series-prefixed-workbook-name.xlsx` | values (`.graph.json`) |
| `probe-request-val11/chart-series-name-probe/series-sheet-name.xlsx` | values (`.graph.json`) |
| `probe-request-val11/chart-series-name-probe/series-workbook-name-values.xlsx` | values (`.graph.json`) |
| `probe-request-val11/chart-series-name-probe/series-workbook-name.xlsx` | values (`.graph.json`) |
| `probe-request-val11/undeclared-entity-probe/entity-control.xlsx` | values (`.graph.json`) |
| `probe-request-val11/undeclared-entity-probe/entity-shared-strings.xlsx` | refused: OracleRefusedError: FileCorruptTryRepair: We're sorry, but something went wrong with this file. |
| `probe-request-val11/undeclared-entity-probe/entity-styles.xlsx` | refused: OracleRefusedError: FileCorruptTryRepair: We're sorry, but something went wrong with this file. |
| `probe-request-val11/undeclared-entity-probe/entity-theme.xlsx` | refused: OracleRefusedError: FileCorruptTryRepair: We're sorry, but something went wrong with this file. |
| `probe-request-val11/undeclared-entity-probe/entity-app.xlsx` | refused: OracleRefusedError: FileCorruptTryRepair: We're sorry, but something went wrong with this file. |
| `probe-request-val11/undeclared-entity-probe/entity-core.xlsx` | refused: OracleRefusedError: FileCorruptTryRepair: We're sorry, but something went wrong with this file. |
| `structural-probes-2026-11-11/engine-refused/t5123-insert-column-last-column-nonblank.case.json` | refused: GraphRequestError: POST worksheets('S')/range(address='A:A')/insert -> 400: {"error":{"code":"NonBlankCellOffSheet","message":"Microsoft Exc |
| `structural-probes-2026-11-11/engine-refused/t5123-insert-row-last-row-nonblank.case.json` | refused: GraphRequestError: POST worksheets('S')/range(address='1:1')/insert -> 400: {"error":{"code":"NonBlankCellOffSheet","message":"Microsoft Exc |
| `structural-probes-2026-11-11/engine-refused/t5358-table-delete-rows-around-table.case.json` | values (`.graph.json`) |
| `structural-probes-2026-11-11/engine-refused/t5358-table-delete-rows-header-and-data.case.json` | refused: GraphRequestError: POST worksheets('S')/range(address='3:5')/delete -> 409: {"error":{"code":"InsertDeleteConflict","message":"This won't wo |
| `structural-probes-2026-11-11/engine-refused/t5358-table-delete-rows-whole-table.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-delete-left-cells.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-delete-left-lines-spans-names.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-delete-left-on-sheet-and-formats.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-delete-left-ranges.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-delete-left-records.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-delete-up-cells.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-delete-up-cuts-spill.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-delete-up-lines-spans-names.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-delete-up-on-sheet-and-formats.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-delete-up-ranges.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-delete-up-records.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-delete-up-table-in-part.case.json` | refused: GraphRequestError: POST worksheets('S')/range(address='C4:D5')/delete -> 409: {"error":{"code":"InsertDeleteConflict","message":"This won't  |
| `shift-probes-2026-11-11/cases/t5465-shift-delete-up-table-whole.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-down-blank-off-the-grid.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-down-cells.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-down-cuts-spill.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-down-into-spill.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-down-lines-spans-names.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-down-off-the-grid.case.json` | refused: GraphRequestError: POST worksheets('S')/range(address='C4:D5')/insert -> 400: {"error":{"code":"NonBlankCellOffSheet","message":"Microsoft E |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-down-on-sheet-and-formats.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-down-ranges.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-down-records.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-down-table-in-part.case.json` | refused: GraphRequestError: POST worksheets('S')/range(address='C4:D5')/insert -> 409: {"error":{"code":"InsertDeleteConflict","message":"This won't  |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-down-table-whole.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-right-cells.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-right-cuts-spill.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-right-lines-spans-names.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-right-off-the-grid.case.json` | refused: GraphRequestError: POST worksheets('S')/range(address='C4:D5')/insert -> 400: {"error":{"code":"NonBlankCellOffSheet","message":"Microsoft E |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-right-on-sheet-and-formats.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-right-ranges.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-right-records.case.json` | values (`.graph.json`) |
| `shift-probes-2026-11-11/cases/t5465-shift-insert-right-table-in-part.case.json` | refused: GraphRequestError: POST worksheets('S')/range(address='C4:D5')/insert -> 409: {"error":{"code":"InsertDeleteConflict","message":"This won't  |
| `probe-request-val9/stress-t5447/t5447-typed-value-readers.case.json` | values (`.graph.json`) |
