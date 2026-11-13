# Probe request val11 (2026-11-12): every open probe request, in one change

This branch is **val10 plus one commit**. val10 (`research/probe-request-val10`, f1ae335ca) was never opened as a PR,
so this one change carries every open request: val10's items (see `docs/research/probe-request-val10/README.md`,
unchanged), the updates to some of them since, and everything new. Merge this instead of val10. #4127 (val9) can be
closed: val10, and so this change, already carries it.

**Excel for the web only.** Every item runs through excel-graph, through Office.js (Script Lab) in Excel for the web,
or by hand in the browser. One item needs the web on a Windows machine (marked). Nothing needs desktop Excel.

The change only adds or updates docs and pending probe cases. No `.github/` file, no Rust or Python source, and no
baseline is touched. New probe cases have no `.oracle.json` sidecar yet, so the gates count them as pending. A
fixture that would be scored against placeholder values sits under `docs/`, to run by path (L59).

**Paths.** Each folder below is the lane's request as written. Where its README says `docs/research/<name>/`, or "beside
this request", read `docs/research/probe-request-val11/<name>/`. Generators import `../probe_package.py`, which is in
this directory. Public third-party workbooks are not carried: `fetch_public_inputs.py` downloads them from their
repositories at pinned revisions and checks each sha256 (four probes; their READMEs say so at the top).

**Return.** For excel-graph fills: the new `*.oracle.json` sidecars, committed beside their cases. For runs by path:
the JSON (or `.err`) and the recalculated workbook the tool keeps. For Script Lab and by-hand items: a `RESULTS.md`
beside the item's README with the JSON lines, notes, screenshots and downloaded copies its README asks for. A refusal
or "the web has no such command" is an answer: please record it.

Still open from val10: the first M4 fill (val10 item 2, 53 cases), which several lanes wait on.

## 1. Asked first: charts

| # | Probe | Decides (lane, task) | How | Time | Return |
|---|---|---|---|---|---|
| 1 | `chart-insert-probe/` | charts lane **P-CH-8 (asked first)**, P-CH-9, P-CH-3: what the web makes when a chart is inserted (radar, bubble, stock, surface, pie of pie, 3-D types, histograms, secondary axis, Recommended Charts) | Script Lab: `cases.js`, one RUN in a new blank workbook (27 charts, 12 data sheets); then by hand on that workbook | ~25 min | pictures of the two histograms and each Insert gallery, one radar from the ribbon, the Recommended Charts pane for 4 (up to 12) selections, one downloaded copy |
| 2 | `chart-save-probe/` | charts lane P-CH-1: does a save by the web give back an untouched chart part byte for byte | **Graph, no person**: 22 workbooks, one PATCH each, via `docs/research/chart-selection-probe/record_graph.py`; or by hand on the 5 marked † | Graph unattended; by hand ~10 min | one saved copy per input; `python3 -I compare.py inputs <saved>` prints the table for RESULTS.md. Inputs: `python3 -I ../fetch_public_inputs.py chart-save-probe inputs` |

## 2. Rulings that wait on Excel

| # | Probe | Decides (lane, task) | How | Time | Return |
|---|---|---|---|---|---|
| 3 | `circular-references-probe/` | sev-2 P559284750, SPEC-0002 Design 14 (approved; every rule naming a cell is provisional until this answers) | by hand, 11 cases C1-C11, a new workbook each | ~30 min | which cells the warning, the status bar and Formulas > Error Checking > Circular References name |
| 4 | `precision-as-displayed-probe/` | decision 0462 (P559132978): work starts when it answers | 4 files through the oracle tool, then Script Lab: 16 cases, one button per fresh workbook | ~40 min | the 4 JSON outputs, the 16 JSON lines |
| 5 | `transition-evaluation-probe/` | decision 0463 (P559132976) | oracle tool, the control first, then the case | ~5 min | both JSON / `.err` and the recalculated workbooks |
| 6 | `udf-cached-value-probe/` | engine-c; the coordinator's ruling on cached values of cells Excel cannot calculate (corpus rows #11, #15) | by hand or Office.js, stateful: open, read, edit an unrelated cell, edit a precedent, Calculate Workbook | ~10 min | the cell values at each step |
| 7 | `chart-series-name-probe/` | design, P561102112, for the I-13 chart payload; **wanted by 2026-11-15** | one oracle-tool command per file (6), then Script Lab READ / CHANGE / SET, with a download after each | ~30 min | the JSON outputs, 13 JSON lines, the downloads |
| 8 | `eval/suites/probes/cases/data-table-reads-a-table.case.json` | engine T-5644's fix (P561645177); SPEC-0051 depends | excel-graph fill (section 3) | unattended | sidecar |

## 3. excel-graph fills, no person

The two commands of val10 fill every pending case of val10 and of this change:

```sh
cd tools
uv run rustsheet-oracle fill -b excel-graph --suite probes
uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric
```

| # | Cases (`eval/suites/probes/cases/`) | Decides (lane, task) | Return |
|---|---|---|---|
| 9 | `data-table-reads-a-table` (item 8) | engine T-5644 | sidecar |
| 10 | 12 older cases with no sidecar: `3d-ref-one-sheet-span-row-delete`, `defined-name-body-scope-{file-order,first-sheet-inactive,first-sheet-moved,later-sheet-active,later-sheet}`, `defined-name-relative-wraps`, `plain-reader-of-api-range-name` (P560003598, sev-3, the engine waits), `subtotal-areas-lifted-pick`, `t4924-pivot-no-row-label-column`, `text-result-limit-builders`, `xirr-xnpv-whole-column-against-a-range` | design desk's sweep of cases with no sidecar | sidecars |
| 11 | `typed-percent-currency-display` (6 typed entries) | P561614667, UI lane and coordinator: the displayed text of typed percent and currency entries | sidecar, with the ENTERED lines (each entry's numberFormat) and its `text` |
| 12 | `sheet-names-cjk-punctuation` (18 cells) and five `sheet-names-cjk-punctuation-unquoted-*` cases (one punctuation class each) | engine-c, corpus rows #15-#17 (~21 files, all ERROR today); a refusal is an answer | sidecars, or the REFUSED line |
| 13 | `legacy-lookup-value-range-intersection`, `legacy-and-or-operation-intersection` | cfix-recalc (M2): does a legacy MATCH / VLOOKUP / HLOOKUP / LOOKUP intersect a range lookup_value, and AND / OR / XOR a comparison over a range | sidecars |
| 14 | updated since val10: `implicit-intersection-two-d-range` (now 36 cells, FORMULATEXT rows, whole columns, a named block, an OFFSET block), `legacy-lookup-value-intersection` (review round 2), `eng-impower-modulus-choice`, `t5107-search-fresh` (and `docs/research/t5107-fin-val4/search-fresh-predictions.tsv`) | as in val10 | sidecars of the current files |

## 4. Through the oracle tool, by path, no person

Each README gives the command, in the form
`cd tools && uv run rustsheet-oracle recalc -b excel-graph <dir>/<file> > <dir>/<file>.graph.json 2> <dir>/<file>.graph.err`.
Run a folder's control first where it has one, and stop if Excel refuses the control.

| # | Probe | Decides (lane, task) | Files | Time | Return |
|---|---|---|---|---|---|
| 15 | `cfix-loadvals-value-whitespace/` | cfix-loadvals, CR-322050446 (NBSP, `+`, inner and whitespace-only forms) and T-4711: how Excel reads a cell `<v>` padded with whitespace | 9 workbooks, ~110 cells | unattended | JSON / `.err` each; a refused workbook is an answer |
| 16 | `linked-name-legacy-probe/` | engine-c P559880417 (the withdrawn engine change waits) | 1 workbook, 108 cells, `predictions.tsv` | unattended | JSON |
| 17 | `cse-row-column-area-probe/` | engine-c P561218625: ROW() / COLUMN() with no argument in a CSE array over an area | 1 workbook, 36 cells, `predictions.tsv` | unattended | JSON |
| 18 | `external-link-own-names/` | engine-c P560248216: this workbook's names over a linked workbook's cells | 1 fixture, 38 cells | unattended | JSON (it is under `docs/`, see its README) |
| 19 | `docs/research/probe-request-val10/cfix-missing-parts-probe/` (updated) | cfix-loadref, P560810770, P558955276, CR-322034591: does Excel open a package with a missing or odd part | 6 packages added since val10 (`package-name-scoped-to-no-sheet`, `-sheet-empty-rid`, `-styles-relocated`, `-styles-dangling-relationship`, `-row-style-past-cellxfs`, `-row-style-negative`), README updated | unattended | one session each; a 501 is an answer |
| 20 | `table-style-info-probe/` | P560297170: a table's tableStyleInfo with flags absent | 10 workbooks | unattended | JSON + the recalculated workbook (what Excel wrote back) |
| 21 | `font-flag-off-probe/` | P560140699: `<b val="0"/>` and schema-invalid `val`s | 5 workbooks | unattended | JSON + recalculated workbook |
| 22 | `malformed-records-probe/` | P559416790 | 6 workbooks (3 cases, 3 controls) | unattended | JSON / `.err` + recalculated workbook |
| 23 | `undeclared-entity-probe/` | P558994790 (this copy: 8 files, new sha256s) | 6 through the tool, control first; 2 opened by hand | ~10 min for the 2 | JSON / `.err`; what the web shows for the 2 |
| 24 | `import-load-probe/` | cfix-uiperf, the import-load specs: row heights, overlapping merges, column/row styles | 4 packages | unattended | JSON; P-2's 501 is an answer |
| 25 | `formula-string-value-probe/` | P559389085: a formula's cached value typed as a shared string | 2 workbooks, then 3 steps by hand with one | ~5 min | JSON, then 4 cells before and after Calculate Workbook |
| 26 | `xls-name-as-cell-probe/` | P560018445: names like TAX2024 / USA1 in an `.xls` (322 formulas of the xls sample wait) | by hand: upload 2 `.xls`, open, download each as `.xlsx`, then the tool on each download | ~15 min | every message's text, the 2 downloads, the 2 JSON |
| 27 | `sheet-defaults-probe/` | ui-b, P561741584 (T-5650): column width from `baseColWidth`, default row height | 1 workbook, 12 sheets: `rustsheet-oracle render --probes <dir> --out <dir>/refs --probe sheet-defaults` (tier-1 render recording) | unattended | the `refs/` recording |

## 5. Office.js in Script Lab (Excel for the web)

| # | Probe | Decides (lane, task) | How | Time | Return |
|---|---|---|---|---|---|
| 28 | `pivot-style-render-probe/` | pivot lane: decision 0459's pivot styles, T-5556 label alignment and indent, T-5566 (switches a file leaves out), pivot UI P-5 (merged labels); the UI lane paints against it | open the workbook (15 sheets), `runAll()` | ~10 min | the JSON |
| 29 | `pivot-default-style-probe/` | pivot lane: decision 0459 item 4 (a new pivot takes the workbook's default pivot style) | 3 workbooks: `runAll()`, then download each | ~15 min | 3 JSON + 3 downloads (`pivotTableStyleInfo` is the answer) |
| 30 | `docs/research/workbook-locale-probe/` (**updated**: take this copy) | SPEC-0046 / M17 (T-4810, decision 0443): regional format and display language | one click per run, a fresh workbook each, under one pair of settings: R1, R2, R7, R8 first; R9 (de-CH) added; R3-R6 later | a few min a run | the JSON lines (each carries its settings); one sentence on a separator override answers U6 |
| 31 | `docs/research/probe-request-t5145-recalc/` (**updated**: the script now has buttons; workbooks unchanged) | T-5145 Manual mode, precedents and dependents | one button per fresh workbook (Script Lab cannot type `run("M1")`) | as val10 | JSON lines |
| 32 | `docs/research/probe-request-val10/d-contracts-24-distinct-dedupe-probe/` (**updated**: adds W1-W3, Find's matching) | D-CONTRACTS-24 (I-06 1.1.0 vectors freeze 2026-11-15) | `findAll` / `find` per case, besides val10's calls | ~10 min more | matching cells, in order |
| 33 | `protection-move-probe/` | P560075187, which decides the model lane's sev-3 P559954551 | SETUP click (5 sheets), one cut-and-insert per sheet by hand, READ click, one download | ~10 min | JSON + download; no "Insert cut cells" is an answer |
| 34 | `sheet-name-characters-probe/` | P560852728: which characters a sheet name may hold, and how a formula spells it | 2 Script Lab cases (NAMES; NUL alone) and renames by hand in a third workbook | ~10 min | JSON + 3 downloads |
| 35 | `rule-anchor-probe/` | model lane and engine-b: a rule's formula anchor when its areas are listed out of order (rule_anchor V1, UNVERIFIED) | Office.js, 3 steps | ~10 min | the cells that pass, `sqref` and formula of the download |
| 36 | `one-cell-merge-probe/` | model lane, D-MODEL-62 (d) and I-03: does a merge of one cell make a merge | by hand and Office.js, 3 steps | ~10 min | sentences + the download's `<mergeCells>` |
| 37 | `cf-relative-wrap-probe/` | engine-c, CR-322219666 (B3), P559198894: does a conditional format's relative row past the grid wrap | Office.js (displayed fill of A1:A6) or by eye | ~5 min | the fill of A1:A6 |

## 6. By hand in Excel for the web

| # | Probe | Decides (lane, task) | Time | Return |
|---|---|---|---|---|
| 38 | `data-table-on-the-web-probe/` | SPEC-0051 (what-if data tables, your question of 11-12): formula bar of a result cell, refusal wording, undo after a clear | ~5 min | notes for 6 steps |
| 39 | `table-design-options-probe/` | model lane's Table Design CR (`table_design` "to probe"): Header Row, Total Row, Resize Table | ~20 min | notes for 10 steps |
| 40 | `sort-moves-notes-probe/` | model lane's sort CR (P559273238): do notes, threaded comments and hidden rows move on Data > Sort | ~10 min | notes for 3 steps |
| 41 | `d-model-91-table-filter-probe/` (rewritten for the web after val10 left it out) | model lane, D-MODEL-91: a table's own AutoFilter | ~20 min | sentences + the downloads' `<autoFilter>` / `<sortState>` |
| 42 | `d-model-93-history-sheet-name-probe/` | model lane, D-MODEL-93: a sheet named History | ~10 min | sentences, any message's exact text |
| 43 | `table-header-column-names-probe/` | model lane, for the pivot UI: a table column's name from a header cell that is not plain text | ~10 min | the column names as structured references |
| 44 | `coauthor-selection-probe/` | UI, D-UI-169 (a co-author's selection fades after 0.8 s): does the web keep it drawn | ~15 min, **two accounts** | screenshots at 0, 1, 5, 60 s + 7 short steps |
| 45 | `review-comments-probe/` | UI lane's Review-area specs R1-R5 | 5 parts of 5-10 min, **two of them with two accounts** | notes, screenshots, 2 downloads (`64759.xlsx`: fetch, see its README) |
| 46 | `pivot-ui-probe/` (README replaced 23h41Z: Parts 3 and 4 now name their downloads) | pivot UI lane P-2, P-3, P-4: slicer items with no data, deleting a slicer, typing over a label, slicer/timeline pictures | 4 parts of ~10 min | notes, pictures, 2 downloads |
| 47 | `pivot-ui-probe-2/` (README replaced 23h44Z after review) | pivot UI lane P-7, P-9, P-6, P-1: refresh on edit, a click on a pivot cell, date grouping, other sources | 4 parts of ~10 min | notes, 1 download (`PivotTable-AllSources-input.xlsx`: fetch, see its README) |
| 48 | `web-load-probe/` | import performance lane P-4, P-5: how the web opens a large workbook | ~10 min with a screen recording | the recording / times. **Part 1 only**: Part 2 (a 119 MB synthetic upload) is **held, generator fix pending**; it follows in a later batch |
| 49 | `render-pixels-probe/` | WebGL lane P-RGL-1..3: borders, gridlines and glyphs pixel by pixel | ~20 min, **Windows only** (its README says what to do on a Mac) | 16 PNGs at the screen's own resolution |

## Left out

- `eval/suites/probes-numeric/cases/t3247-poisson-closed-tail.case.json`: on HOLD until its branch merges (it gains
  rows; a filled case cannot be extended).
- Outline A1-A5 (Group, Ungroup, Hide / Show Detail, Clear Outline, SUBTOTAL over collapsed groups): still written for
  desktop Excel only, with no web version yet.
- `defined-name-question-mark`: its review moved it to the change that reads it; not ready.
- `data-table-reads-itself` (the second data-table case): still in review; it follows.
- web-load-probe Part 2 (item 48): held until its generator is fixed; the change carries Part 1 only, and no `generate.py`.
