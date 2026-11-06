# SPEC-0032 — The pivot engine: model, PivotResult, layout contract, refresh and grid ownership

- **Status:** draft (accepted on merge of T-4121)
- **Milestone / lane:** M16 / engine (decision 0274)
- **Tasks:** T-4121 (this spec).
  - Governs engine and API tasks T-4139, T-4141, T-4142, T-4147, T-4149, T-4151, T-4152, T-4154, T-4163,
    T-4165, T-4168, T-4169, T-4171, T-4174, T-4175, T-4176, T-4178, T-4182, T-4184, T-4186, T-4187,
    T-4188 and T-4506.
  - Tests: T-4132, T-4133, T-4155, T-4160, T-4177.
  - Probes it waits on: T-2327, T-4130, T-4135, T-4161, T-4164, T-4170, T-4185, T-4505, T-4582, T-4583,
    T-4584, T-4585, T-4586.

## Problem
The engine keeps a loaded workbook's pivots read-only (SPEC-0015). It answers GETPIVOTDATA from the cache
and never refreshes, renders or exports a pivot.

The one exception is a compact layout of one row field and one data field, used for grouped spec pivots
(`pivot/layout.rs`, T-4235). Its numeric and date group rules live in `pivot/number_groups.rs` and
`pivot/date_groups.rs`.

M16 asks for Excel's whole pivot table:
- refresh from the source;
- the three layout forms, subtotals, filters, sorting, show-values-as, grouping, calculated fields and
  items;
- export;
- the data a PivotChart plots.

T-4124 scores every rendered cell against Excel. Today every pivot-area cell of the oracle-rendered
probes fails (eval/baseline/probes.txt): 241 in t4123 and t4128, 54 in t4134, 120 in t4137 and 188
in t4138.

## Goals / non-goals
Goals:
1. **One pipeline.** A pivot is a pure function from (definition, cache) to a `PivotResult`. Rendering,
   GETPIVOTDATA, export and PivotChart data all read the result, and nothing else recomputes a pivot.
2. **Excel's own intermediate form as the contract.** The result's layout is Excel's `rowItems`/`colItems`
   encoding and `location`, which the sidecars record (T-4123). Layout bugs are therefore found
   structurally before cell values (T-4124 `pivot_layout`).
3. **Refresh semantics as Excel's:**
   - a pivot refreshes on load when its cache is `refreshOnLoad`, on an explicit API call, and when it is
     created;
   - by design default, pending probe T-4586: never on a source edit (Auto Refresh only as T-4182
     evidences), and a pivot loaded without `refreshOnLoad` keeps the cells the file saved. No probe yet
     shows what Excel does in either case, because every rendered probe so far is `refreshOnLoad`.
4. **Bounded work and memory on untrusted pivots** (directive 5), with every cap a `DEVIATION` entry.

Non-goals (decision 0274):
- OLAP, the Data Model and DAX;
- external and consolidation sources;
- styles and conditional formats beyond preservation;
- UI-only commands (Show Report Filter Pages, field-drop auto grouping);
- drawing charts.

## Excel behavior to match
The evidence lives in docs/excel-semantics/pivots.md and web.md (GETPIVOTDATA and groups). Highlights the
design depends on:
- **Refresh:** a `refreshOnLoad` cache is rebuilt from the source on open, from the recalculated
  values (probes/t4128-pivot-spike R: `DataF!C2` `=1000` cached as 0, and the pivot shows 1004). Whether
  Excel also overwrites rendered constants already in the area is open (T-2327).
- **Item identity and order** (probes/t4134-pivot-items):
  - text items are case-insensitive; numbers compare exactly; text is never a number. Which casing shows
    when the cache and the source disagree, and whether date items with and without a time part are one
    item, are open (T-4582);
  - a field keeps its listed item order. New items go after the listed ones, not ascending, though
    source order versus a descending sort is open (T-4582). A listed item with no records is left out
    without renumbering;
  - number items render as numbers; booleans, errors and blanks render as text (`TRUE`, `#N/A`,
    `(blank)`).
- **Layout** (probes/t4123-pivot-capture, t4137-pivot-captions, t4138-pivot-geometry):
  - geometry per form: location, `firstHeaderRow`/`firstDataRow`/`firstDataCol`;
  - in tabular form, repeated outer labels are blank and subtotal lines read `<item> Total`, or
    `<item> <Function>` for custom subtotals;
  - with the Values field on columns, the `Values` caption sits above the first data column. With one
    data field and no column field there is no Values row (t4137 K5). With Values on rows, the grand
    total reads `Total <data field>` (t4138 G6);
  - in tabular form the first row header is the field's name only when every pivotField has
    `compact="0"`, and `Row Labels` otherwise (t4583 R1–R6). Column and page fields are T-4609's. An
    outer row field left at the default moves its subtotals onto the item line (t4583 R1). Compact
    pivots left at the default show `Row Labels` (t4123 C, t4128 K, t4138);
  - `rowHeaderCaption`, `grandTotalCaption` and `dataCaption` apply (`colHeaderCaption` replaces `Column Labels` in compact
    form, t4583 C1), and so do `missingCaption` and `errorCaption` (under Sum);
  - page fields are laid out with `pageWrap`; the sidecar capture window is confirmed by T-4584.
- **Values:**
  - a calculated field applies to the sums at every level (t4128 K);
  - a Top-N filter changes the subtotals and grand totals (t4128 K);
  - an empty intersection is a blank cell, but GETPIVOTDATA answers 0 for it under Sum, Average, Count,
    Max and Min (web.md, T-2404).
- **Grid:** a layout that would meet a foreign cell collapses to `#SPILL!` at the anchor, with the
  location saved as that one cell (t4128 S).

Unknowns are owned by probe tasks: T-4582, T-4583, T-4584 and T-4586 (above), T-2327 (rendered constants
under a refreshOnLoad pivot), T-4585 (field names), T-4135 (aggregates over text,
blank and error, now evidenced in pivots.md; the summation order of totals is T-4589's), T-4161
(show-values-as),
T-4164 (filters and sort), T-4170 (manual and nested groups), T-4505 (calculated fields and items),
T-4130 (capability matrix 2). An engine task implements a rule only once its probe has landed (directive
1).

## Design

### Model (T-4139)
`crates/rustsheet-engine/src/pivot.rs` keeps `PivotCache` and `PivotTable` and grows them, typed, by every
catalog row (T-4125) whose `affects` is not `none`:
- per pivot: form flags (`compact`, `outline`, `outlineData`, `compactData`), `rowGrandTotals`,
  `colGrandTotals`, `dataOnRows`, `dataPosition`, the caption attributes, `showMissing`/`missingCaption`,
  `showError`/`errorCaption`, `pageWrap`, `pageOverThenDown`, the filters (T-4165) and the calculated
  items (T-4175);
- per field: `compact`, `outline`, `subtotalTop`, `defaultSubtotal`, the custom subtotal set, `sortType`,
  `showAll`, `insertBlankRow`, the items with `h`, `sd`, `s` and `t`, and `numFmtId` (T-4169);
- per data field: `subtotal`, `showDataAs`, `baseField`/`baseItem`, `numFmtId`.

Everything else in the parts is kept verbatim (an `Opaque` list of the definition's unread elements and
attributes) for export (T-4176). The SPEC-0015 limits stay, and the importer fills both forms. A model
change is bitcode-encoded like the rest (no stored format depends on it).

### Refresh (T-4141, T-4171, T-4506)
`pivot/refresh.rs` holds
`fn refresh(cache: &PivotCache, source: &dyn SourceCells, workbook_values_left: usize) -> Result<PivotCache, PivotCacheError>`.
It builds a new cache through the existing `CacheBuilder`, the one builder (directive 3), passing it the
workbook's remaining record-value budget as the importer does. The caller swaps the new cache in only on
success (see the limits). Its rules:
- field names come from the header row, and duplicate or blank names follow Excel's rule (probe T-4585,
  which T-4141 depends on);
- items follow the identity rules above, existing items keep their order and new ones are appended;
- the `sharedItems` flags are recomputed;
- groups are rebuilt by `number_groups`/`date_groups` and, for manual groups, by T-4171.

A Table or a defined name resolves to its range at refresh (T-4506). `SourceCells` is a trait in
`pivot/refresh.rs`, implemented by `Model` over the evaluated cell values. It reads values only, so a
refresh does no I/O and never evaluates a formula. Because it reads what the recalculation produced, a
refresh is ordered after that calculation.

**Who refreshes, and when (T-4151).**
- The importer only records `refreshOnLoad` on the cache, since import cannot calculate.
- The facade's open path refreshes every marked cache after the first recalculation. It then
  recalculates what depends on the pivots (GETPIVOTDATA, and formulas that read pivot cells).
- `Txn` refreshes a pivot when it creates the pivot, and when asked to.
- Auto Refresh is added only as T-4182 evidences it.

### PivotResult (T-4142, T-4163, T-4165, T-4174, T-4175)
`fn compute(table: &PivotTable, cache: &PivotCache, budget: &mut AreaBudget) -> Result<PivotResult, PivotComputeError>`
runs in fixed stages. The order is Excel's where a probe shows one (Top-N totals: t4128 K), and LibreOffice's
otherwise as a hypothesis to probe:
1. **Record filter:** page field selections, hidden items (`h`), slicer state (T-4188).
2. **Groups:** each record's group item per grouped field (precomputed in the cache).
3. **Accumulate:** a result tree keyed by tuples of item indices. Each (row tuple, column tuple, data
   field) holds an accumulator, and so do subtotal and grand-total nodes. Aggregates reuse the
   GETPIVOTDATA folds and `excel_variance` (SPEC-0015), so a cell and GETPIVOTDATA read one accumulator.
   Mixed values follow pivots.md "Aggregates over text, blanks, booleans and errors" (T-4135). The
   order in which a total accumulates its records is Excel's own: it is neither record order nor the
   sum of the children (pivots.md "Summation order...", T-4589).
   Where Excel shows them differently, the difference is a rule on the reading side: an empty
   intersection renders blank but GETPIVOTDATA answers 0 (web.md); a caption substitute renders as text.
4. **Calculated items** (T-4175, solve order) and **calculated fields** (T-4174). Calculated fields are
   evaluated over the sums by the engine's parser and evaluator, in a sandbox with no references, names
   or volatile functions and under the work budget.
5. **Label, value and date filters, Top-N, then sorting** (T-4165). Totals cover what stays: verified for
   Top-N (t4128 K). For the other filters and for hidden items (stage 1), this is T-4164's to probe.
6. **Show values as** (T-4163), in three passes: leaves, subtotals, grand total.
7. **Layout lines:** Excel's `rowItems`/`colItems` (`t`, `r`, `i`, `x`) for the form, subtotal placement,
   grand totals, Values position and blank rows (T-4147, T-4168).

`PivotResult` holds the lines, the cell value of each (row line, column line, data field), the page-area
items and the location size. It is derived state and never encoded.

It lives in `Evaluation` (evaluation.rs) as `pivot_results`, beside `pivot_indexes`, keyed like them.
It is built on demand by render and by GETPIVOTDATA. It is dropped at every point where `pivot_indexes`
is cleared today: `displace_pivots` on structural edits, source renames, sheet deletion and sheet undo
(sheet_undo.rs restores the `PivotTable`, never a result). It is also dropped when a refresh swaps the
cache.

### Rendering and the grid (T-4149, T-4151, T-4169)
`fn render(result, table) -> Vec<(CellRef, PivotCell)>` turns the lines into cells:
- captions, labels with the T-4134 item rendering, values, totals;
- the page area;
- the empty and error captions;
- number formats as styles (T-4169).

`pivot/layout.rs`'s compact one-field layout becomes the T-4149 renderer's first case, its tests kept;
there is no second layout path.

The grid writer owns a pivot's area:
- a refresh clears the old area and writes the new one as constants, so formulas that read them need no
  new dependency kind;
- a layout that would meet a cell the pivot does not own writes `#SPILL!` at the anchor and sets the
  location to that cell;
- structural edits keep T-2086 and T-2401's rules, and implement nothing new before T-4185's probe.

Every refresh dirties GETPIVOTDATA cells as SPEC-0015 requires.

### GETPIVOTDATA (T-4154)
Reads the `PivotResult` for every PV1 shape (several data fields, page fields, hidden items, nested axes),
with SPEC-0015's rules and T-2085's answers. The SPEC-0015 index is rebuilt by T-4154 as a lookup table
over the result's nodes, built once per result. It keeps its role as the fast lookup, but folds nothing
itself, so a cell and a GETPIVOTDATA call never disagree on the accumulated value (the reading-side rules above
still apply), and no second computation remains.

### Export (T-4176, T-4187)
Writes the typed model plus the verbatim rest, the cache and records, and the result's `rowItems`/`colItems`
and location. The ClosedXML invariants (oss-engines.md §4) are the checklist, and the round-trip suite
(T-4177) is the proof. Pivot parts move from the `Replay` class of the M15 ownership matrix (T-4183, T-4567; decision 0275) to
modelled.

### PivotChart data (T-4186)
`PivotChartData { categories: Vec<Vec<CellValue>>, series: Vec<Series> }` is derived from the
`PivotResult`:
- one multi-level category per row line that is not a subtotal or a grand total;
- one series per (column line, data field);
- values as shown, after filters and collapse.

Export rewrites the chart part's caches from it (T-4187).

### API (T-4178, T-4184)
The facade keeps today's names and makes them public:
- `Txn::add_pivot_table(&PivotTableInput)`, today doc-hidden, becomes public. `PivotTableInput` grows into
  the one input type, mirroring the case-builder-v2 `definition` schema, so the harness and the API share
  one shape.
- New: `Txn::set_pivot_table(sheet, name, &PivotTableInput)` and `Txn::refresh_pivot_table(sheet, name)`.
- New: `Workbook::pivot_table(sheet, name) -> Option<PivotView>`.

The engine keeps pivot names unique per sheet, which is its own rule and has not been probed against
Excel, so every call takes the sheet and the name. Each `Txn` change returns
the `ChangeSet` of the cells it wrote and cleared (api-design.md rule 2). The grid writer records them
as it renders.

Errors:
- The engine's `PivotComputeError` (`Limit`, `Source`, ...) folds into the facade's existing
  `edit::PivotError` as new variants.
- `EditError::Pivot` is re-documented as public API (T-4178).
- A pivot created through the API that hits a limit is not added: the `Txn` fails, and nothing is
  written.

Drill-down is `PivotView::records(cell)`, which returns an iterator of record indices (T-4184). The
Python binding mirrors all of this (T-4178). `pivot_places` stays doc-hidden harness support.

### Untrusted-input limits
Every pivot part comes from an untrusted file (directive 5). The import limits of SPEC-0015 stay (`MAX_RECORDS`,
`MAX_ITEMS`, `MAX_FIELDS`, and `MAX_RECORD_VALUES` over cache record values). They bound the cache, not the
work done on it. The new limits are below, and each is a `DEVIATION` entry (decision 0014) that the task
introducing it adds:
- **Work.** `compute` runs as a budgeted entry point, as a formula run does. It draws work units from
  SPEC-0024's `AreaBudget`, opened by `Model::with_area_budget` with a pivot cap `MAX_PIVOT_WORK`, and on
  load from the recalculation's `RecalcBudget` pool (SPEC-0028). There is no second budget type
  (SPEC-0024 rejected a separate `WorkBudget`). Every stage charges it before doing the work:
  - each record per stage;
  - each result node created;
  - each filter test;
  - each sort comparison;
  - each calculated-field or calculated-item evaluation;
  - each layout line.

  Sorts are O(n log n) over precomputed keys, and a comparator never formats or re-evaluates. Running out
  stops `compute` at once with `PivotComputeError::Limit`.
- **Result nodes.** `MAX_PIVOT_NODES` (10,000,000, the same order as `MAX_RECORD_VALUES`, since a node
  costs about one record value's memory) bounds the result tree's nodes, counted as they are
  created in stages 3, 4 and 7. That count includes subtotal and grand-total nodes and the nodes `showAll`
  or calculated items add without records. Axis depth is bounded by the fields on an axis.
- **Cells.** `MAX_PIVOT_CELLS` (4,000,000, four columns of a full 1,048,576-row sheet, more than any
  pivot of `MAX_RECORDS` records renders in tabular form with a few data fields) is checked in stage 7 from the line counts (row lines ×
  column lines × data fields, plus the page area), in saturating arithmetic. The check runs before any
  cell is allocated, not at render.
- **Calculated formulas.** A calculated field or item is parsed under the parser's depth limit. It is
  accepted only if it uses an allow-list of pure scalar functions and its own pivot's field and item
  names. It is rejected at definition time if it contains:
  - references, names, Tables or external links;
  - volatile functions, or functions that reach the host (`INDIRECT`, `OFFSET`, `WEBSERVICE`, `RTD`,
    `CALL`, `HYPERLINK`, `GETPIVOTDATA`);
  - `LET` or `LAMBDA`;
  - array-producing functions.

  Each evaluation runs under SPEC-0024's per-formula budget and charges the pivot run's work units.
  Calculated-item references are resolved with an explicit stack (no native recursion), with cycle
  detection and a depth of 64.
- **Workbook total.** The caps above are per pivot. A workbook can hold `MAX_PIVOTS` pivots, each with
  `refreshOnLoad`, so load shares one workbook-wide work pool (the `RecalcBudget`) and cell total, as
  `CacheBuilder` shares `workbook_values_left`. Once the shared total runs out, later pivots are not refreshed.
- **Failure is atomic.** A refresh builds a new cache and a new `PivotResult` and swaps them in only on
  success. Past any limit, or on an unresolvable source (an external sheet, a connection, or an
  unresolved name or Table, which never fall back to a file or network read), the following stay as
  they were: the cache, the result, the cells and GETPIVOTDATA's answers. Those answers may then be
  stale, but they are consistent with the cells. On import this is a warning, never a load failure.
  The API returns `edit::PivotError::Limit` or `Source` through `EditError::Pivot`. Excel would render
  the pivot, which is the `DEVIATION`.
- **Grid.** The area a refresh clears is the area this pivot wrote at its last render, clamped to the
  sheet. It is never the file's `location` as given, because a foreign cell under a stale location must
  not be cleared.
- **Other bounds.**
  - `PivotView::records` returns an iterator bounded by `MAX_RECORDS`.
  - The `Opaque` verbatim store counts toward the import limits.
- **No panics.** Item indexing, tuple building, `u32` index narrowing (by the cap argument the cache
  builder uses) and show-values-as division all return `PivotError` or an Excel error value. The fuzz
  target (T-4133) extended to compute and render is an acceptance criterion of T-4142 and T-4147.

## Alternatives considered
- **Render from a query engine** (mog's GROUP BY/rollup). It hides Excel's line encoding, so layout bugs
  show up only as wrong cells. Rejected; the line contract is cheaper to check.
- **Re-render on import, as LibreOffice does.** That would lose Excel-saved cells that are already
  right, if Excel itself keeps them (T-4586). Rejected for now: re-render only on `refreshOnLoad`, or on
  request. Revisit if T-4586 shows that Excel re-renders.
- **Separate code paths per form.** The forms differ only in line-to-cell placement. One renderer with
  per-field form flags.

## Test plan
- **Probes:** the `pivots` probes (t4123, t4128, t4134, t4137, t4138 and the later probe tasks) score
  every pivot cell and layout through T-4124.
- **Generated:** `gen-pivot` (T-4132, T-4160) adds covering-array coverage.
- **Corpora:** fresh pivot-corpus files (T-4131) re-render exactly.
- **Metamorphic** (T-4155):
  - refresh is idempotent;
  - GETPIVOTDATA of each non-empty value cell's tuple equals the cell, when no missing or error caption
    is set; empty intersections answer 0 instead; `#SPILL!`-collapsed pivots stay out until T-4130;
  - permuting source rows changes only order-dependent output;
  - import, export, import is a fixed point.
- **Unit tests per stage**, and the fuzz target (T-4133) extended to compute and render.
- **Benchmark:** 100k records with 1k GETPIVOTDATA cells, and a 10k-line render, each at most 50% slower
  than the stage before it (decision 0081).

## Rollout
1. T-4139: model and import.
2. T-4141: refresh.
3. T-4142: result tree.
4. T-4147: layout lines.
5. T-4149: cells.
6. T-4151: grid and lifecycle (its refresh defaults follow T-4586).
7. T-4152: the adapter renders through the engine, so the probes' pivot cells start passing.
8. T-4154: GETPIVOTDATA over the result.
9. PV2 and PV3 stages (T-4163, T-4165, T-4168, T-4169, T-4171, T-4174, T-4175), each after its probe
   (T-4161, T-4164, T-4170, T-4505, T-4582, T-4583, T-4584).
10. Auto Refresh (T-4182), once its evidence lands.
11. Export and API (T-4176–T-4178, T-4184), then PivotChart data and I/O (T-4186, T-4187) and slicers
    (T-4188).

Until T-4152 lands, spec-case pivots with a `definition` stay unregistered and their cells known failures.
