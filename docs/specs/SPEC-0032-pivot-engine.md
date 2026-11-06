# SPEC-0032 — The pivot engine: model, PivotResult, layout contract, refresh and grid ownership

- **Status:** draft (accepted on merge of T-4121)
- **Milestone / lane:** M16 / engine (decision 0274)
- **Tasks:** T-4121 (this spec); governs T-4139, T-4141, T-4142, T-4147, T-4149, T-4151, T-4152, T-4154,
  T-4163, T-4165, T-4168, T-4169, T-4171, T-4174, T-4175, T-4176, T-4178, T-4184, T-4186, T-4506

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
   - never on a source edit (Auto Refresh only as T-4182 evidences);
   - a pivot loaded without `refreshOnLoad` keeps the cells the file saved.
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
- **Refresh:** a `refreshOnLoad` cache is rebuilt from the source on open
  (probes/t4128-pivot-spike R).
- **Item identity and order** (probes/t4134-pivot-items):
  - text items are case-insensitive; numbers compare exactly; text is never a number. Which casing shows
    when the cache and the source disagree is open (T-4582);
  - a field keeps its listed item order. New items go after the listed ones, not ascending, though
    source order versus a descending sort is open (T-4582). A listed item with no records is left out
    without renumbering;
  - number items render as numbers; booleans, errors and blanks render as text (`TRUE`, `#N/A`,
    `(blank)`).
- **Layout** (probes/t4123-pivot-capture, t4137-pivot-captions, t4138-pivot-geometry):
  - geometry per form: location, `firstHeaderRow`/`firstDataRow`/`firstDataCol`;
  - in tabular form, repeated outer labels are blank and subtotal lines read `<item> Total`, or
    `<item> <Function>` for custom subtotals;
  - the `Values` caption sits above the first data column;
  - the first row header is `Row Labels` while a non-axis field is compact; which fields decide is
    open (T-4583);
  - the caption attributes apply, and so do `missingCaption` and `errorCaption` (under Sum);
  - page fields are laid out with `pageWrap`; the sidecar capture window is confirmed by T-4584.
- **Values:**
  - a calculated field applies to the sums at every level (t4128 K);
  - a Top-N filter changes the subtotals and grand totals (t4128 K);
  - an empty intersection is a blank cell.
- **Grid:** a layout that would meet a foreign cell collapses to `#SPILL!` at the anchor, with the
  location saved as that one cell (t4128 S).

Unknowns are owned by probe tasks: T-4582, T-4583 and T-4584 (above), T-4135 (aggregates over text,
blank and error), T-4161 (show-values-as),
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
`fn refresh(cache: &mut PivotCache, source: &dyn SourceCells) -> Result<(), PivotCacheError>` rebuilds
the cache through the existing `CacheBuilder`, the one builder (directive 3):
- field names come from the header row; duplicate names are renamed as Excel does, which no probe has
  shown yet, so T-4141 probes it before implementing it;
- items follow the identity rules above, existing items keep their order and new ones are appended;
- the `sharedItems` flags are recomputed;
- groups are rebuilt by `number_groups`/`date_groups` and, for manual groups, by T-4171.

A Table or a defined name resolves to its range at refresh (T-4506). `SourceCells` reads values only:
refresh does no I/O and never evaluates a formula itself. It reads the values the recalculation already
produced, so a refresh is ordered after the calculation it depends on.

### PivotResult (T-4142, T-4163, T-4165, T-4174, T-4175)
`fn compute(table: &PivotTable, cache: &PivotCache, budget: &mut WorkBudget) -> Result<PivotResult, PivotError>`
runs in fixed stages. The order is Excel's where a probe shows one (Top-N totals: t4128 K), and LibreOffice's
otherwise as a hypothesis to probe:
1. **Record filter:** page field selections, hidden items (`h`), slicer state (T-4188).
2. **Groups:** each record's group item per grouped field (precomputed in the cache).
3. **Accumulate:** a result tree keyed by tuples of item indices. Each (row tuple, column tuple, data
   field) holds an accumulator, and so do subtotal and grand-total nodes. Aggregates reuse the
   GETPIVOTDATA folds and `excel_variance` (SPEC-0015), so a cell and a GETPIVOTDATA call can never
   disagree.
4. **Calculated items** (T-4175, solve order) and **calculated fields** (T-4174). Calculated fields are
   evaluated over the sums by the engine's parser and evaluator, in a sandbox with no references, names
   or volatile functions and under the work budget.
5. **Label, value and date filters, Top-N, then sorting** (T-4165). Totals cover what stays.
6. **Show values as** (T-4163), in three passes: leaves, subtotals, grand total.
7. **Layout lines:** Excel's `rowItems`/`colItems` (`t`, `r`, `i`, `x`) for the form, subtotal placement,
   grand totals, Values position and blank rows (T-4147, T-4168).

`PivotResult` holds the lines, the cell value of each (row line, column line, data field), the page-area
items and the location size. It is derived state, dropped whenever the pivot or its cache changes, and
never encoded.

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
- structural edits keep T-2086 and T-2401's rules.

Every refresh dirties GETPIVOTDATA cells as SPEC-0015 requires.

### GETPIVOTDATA (T-4154)
Reads the `PivotResult`, not the cache index, for every PV1 shape (several data fields, page fields,
hidden items, nested axes), with SPEC-0015's rules and T-2085's answers. The SPEC-0015 index stays as the
fast path, used only where the result is not built.

### Export (T-4176, T-4187)
Writes the typed model plus the verbatim rest, the cache and records, and the result's `rowItems`/`colItems`
and location. The ClosedXML invariants (oss-engines.md §4) are the checklist, and the round-trip suite
(T-4177) is the proof. Pivot parts move from M15's `Replay` class to modelled (decision 0275).

### PivotChart data (T-4186)
`PivotChartData { categories: Vec<Vec<CellValue>>, series: Vec<Series> }` is derived from the
`PivotResult`:
- one multi-level category per row line that is not a subtotal or a grand total;
- one series per (column line, data field);
- values as shown, after filters and collapse.

Export rewrites the chart part's caches from it (T-4187).

### API (T-4178, T-4184)
The facade gains `Txn::create_pivot(PivotDefinition)`, `Txn::set_pivot(..)`, `Txn::refresh_pivot(name)`
and `Workbook::pivot(name) -> Option<PivotView>`, plus drill-down
(`PivotView::records(cell) -> Vec<RowIndex>`). `PivotDefinition` mirrors the case-builder-v2
`definition` schema, so the harness and the API share one shape.

The doc-hidden `add_pivot_table` and `pivot_places` stay as harness support until T-4152 moves the eval
adapter to the public API.

### Untrusted-input limits
- The import limits of SPEC-0015 stay.
- The result's nodes are bounded by `MAX_RECORD_VALUES`, and the rendered cells by the sheet grid and a
  `MAX_PIVOT_CELLS` of 4,000,000.
- Calculated formulas are bounded by the formula work budget (SPEC-0024) and 64 levels of calculated-item
  references; solve iterations are bounded.
- Past a cap the pivot is not re-rendered: it keeps the cells the file saved, and the refresh returns
  `PivotError::Limit` (the API's `EditError::Pivot`).

Each cap is a `DEVIATION` entry (decision 0014) added by the task that introduces it.

## Alternatives considered
- **Render from a query engine** (mog's GROUP BY/rollup). It hides Excel's line encoding, so layout bugs
  show up only as wrong cells. Rejected; the line contract is cheaper to check.
- **Re-render on import, as LibreOffice does.** That loses Excel-saved cells that are already right.
  Rejected: re-render only on `refreshOnLoad`, or on request.
- **Separate code paths per form.** The forms differ only in line-to-cell placement. One renderer with
  per-field form flags.

## Test plan
- **Probes:** the `pivots` probes (t4123, t4128, t4134, t4137, t4138 and the later probe tasks) score
  every pivot cell and layout through T-4124.
- **Generated:** `gen-pivot` (T-4132, T-4160) adds covering-array coverage.
- **Corpora:** fresh pivot-corpus files (T-4131) re-render exactly.
- **Metamorphic** (T-4155):
  - refresh is idempotent;
  - GETPIVOTDATA of each value cell's tuple equals the cell;
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
6. T-4151: grid and lifecycle.
7. T-4152: the adapter renders through the engine, so the probes' pivot cells start passing.
8. T-4154: GETPIVOTDATA over the result.
9. PV2 and PV3 stages (T-4163, T-4165, T-4168, T-4169, T-4171, T-4174, T-4175), each after its probe.
10. Export and API (T-4176–T-4178, T-4184), then PivotChart data and I/O (T-4186, T-4187) and slicers
    (T-4188).

Until T-4152 lands, spec-case pivots with a `definition` stay unregistered and their cells known failures.
