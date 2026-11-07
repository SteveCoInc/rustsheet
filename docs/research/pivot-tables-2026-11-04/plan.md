# Pivot tables (and PivotCharts): research and program plan, 2026-11-04

Owner request: a spike into Excel pivot tables, and the engine side of PivotCharts. It covers deep research
(Excel's public documentation, ECMA-376 and its Microsoft extensions, LibreOffice, mog, gridcraft and other
engines, and live runs of the Excel oracle) and a plan for three things: **enumerating** every pivot feature,
**implementing** them, and **proving** each one matches Excel. This note is for review. The tasks it proposes
(§9) are filed only after the owner agrees. Nothing here changes behaviour.

Companion notes in this folder (data, citations, file:line references):
- `excel-docs.md`: the feature catalog from Microsoft's documentation, ECMA-376 §18.10, [MS-XLSX], Office.js and Graph.
- `libreoffice.md`: LibreOffice's DataPilot source (MPL-2.0, read as data, commit b1653ce).
- `oss-engines.md`: mog, Calipers, gridcraft, EPPlus, ClosedXML, POI and others, plus public Excel-saved
  pivot corpora.
- `oracle-spike.md`: the live excel-graph experiments of 2026-11-04.

## 1. Bottom line

1. **The oracle can render any pivot we can describe.** On 2026-11-04 Excel for the web opened workbooks
   whose `pivotTableDefinition` parts were written by hand: tabular form, two row fields, subtotals, three
   data fields, "% of grand total", a calculated field, a Top-N filter, a descending sort and numeric grouping.
   It laid each one out itself and saved its own `rowItems`/`colItems` and group items. It also re-renders a
   `refreshOnLoad` pivot when it opens the file (answers T-2327). Validation therefore needs no Office.js and
   no desktop Excel. The spec case gives the source data and the pivot definition, and Excel draws the
   pivot. See `oracle-spike.md`.
2. **Nobody has done this to Excel parity.** mog (Apache-2.0) has the broadest open-source engine: refresh,
   three layouts, 14 show-values-as modes, grouping and calculated fields. It has 624 tests, none of them
   checked against Excel, and Excel-saved files already contradict its group labels (`< 10` against Excel's
   `<10`, `Q1` against `Qtr1`). gridcraft and EPPlus make claims that also go unchecked, and they disagree
   with each other (Rank is dense in gridcraft, competition-style in mog and sequential in EPPlus).
   LibreOffice throws away Excel's rendered cells on import and re-renders in its own way. Every engine is a
   source of hypotheses, not of evidence.
3. **The surface is large but closed.** The OOXML schema (`CT_PivotTableDefinition`, `CT_PivotCacheDefinition`,
   their children and the x14/x15 extensions) is a finite list. Enumerating it mechanically, with one catalog
   row per element and attribute, guarantees we cannot miss a feature that affects the file. Office.js's enums
   (aggregations, show-as modes, filter conditions, layout types) cross-check the list from the API side.
4. **About 150 public Excel-saved workbooks with roughly 290 pivots** carry Excel's rendered cells as cached
   values (LibreOffice 64 files, ClosedXML 38, Calipers 13, POI 5, corpus-bench 4). They serve as free
   regression evidence for real-world shapes. The oracle covers what they lack: rank, difference, running
   total, calculated items and blank rows appear in none of them.
5. **Decided (decision 0274, after review):** a new milestone, M16 "Pivot tables", in Phase 1, worked in the
   existing lanes; its tasks are under docs/tasks/M16. The proposal below said a `pivot` lane; the decision
   kept existing lanes so routine allowlists stay unchanged. The original proposal: M16 "Pivot tables", worked in a `pivot` lane parallel to Phase 1. It needs a
   decision record, since decision 0027 orders milestones. It is staged PV0–PV5 (§8), and every stage is gated
   by oracle-backed suites and per-feature coverage rows in the eval report.

## 2. Where we are

SPEC-0015 (T-2081–T-2086, T-2401, T-2404, T-2662, all done) gives us:
- a read-only `PivotCache`/`PivotTable` model (`crates/rustsheet-engine/src/pivot.rs`);
- xlsx import with directive-5 limits (`crates/rustsheet-xlsx/src/import/pivots.rs`);
- GETPIVOTDATA over cache records (`functions/lookup_and_reference/getpivotdata.rs`, `pivot_index.rs`);
- structural edits;
- a case builder that writes one compact pivot with up to one row field, one column field and one data field
  (`tools/src/rustsheet_tools/pivot.py`).

The engine never renders a pivot: rendered cells are whatever the file holds. xlsx export drops pivots. Open
items: T-2085 (GETPIVOTDATA unknowns), T-2327 (re-render on load, now answered by the spike) and T-2402.
PIVOTBY/GROUPBY (SPEC-0006) are functions and share only aggregation semantics with pivot tables.

**Update 2026-11-06.** While this plan waited to merge, the M6 GETPIVOTDATA track probed and implemented
numeric and single-level date grouping. That covered import, record placement, Excel's labels and
GETPIVOTDATA (T-4225–T-4324; web.md "Grouped pivot fields import as their group items"). It also added a
compact one-field engine layout for grouped spec pivots (`pivot/layout.rs`). T-4170 and T-4171 were
rescoped to what is left, and the layout and spec tasks build on that code. The pivot spec is SPEC-0032:
the number 0030 was taken.

## 3. What the feature space is (summary; full catalog in `excel-docs.md`)

| Family | Features (abridged) | Affects cell values? |
|---|---|---|
| Source and cache | worksheet range, Table, named range, external/OLAP/Data Model (out of scope); item typing (text, number, date, boolean, error, blank); case-insensitive item identity; cache item order; `refreshOnLoad`; `saveData`; missing-item retention (`missingItemsLimit`); shared caches | yes |
| Aggregation | Sum, Count, Average, Max, Min, Product, Count Numbers, StdDev, StdDevp, Var, Varp (Distinct Count needs the Data Model); text, blank and error handling; summation order | yes |
| Show values as | % of grand/column/row total, % of, % of parent row/column/total, difference from, % difference from, running total in, % running total in, rank smallest→largest and largest→smallest, index (base field; base item, previous or next) | yes |
| Layout | compact, outline and tabular (per pivot and per field); subtotals top or bottom, on or off, or custom (several functions); grand totals for rows and columns; repeat item labels; blank row after each item; the Values (Σ) field on rows or columns at any position; page fields (layout over/down, fields per column); captions (`Row Labels`, `Grand Total`, `<item> Total`, `(blank)`, `(All)`, `(Multiple Items)`, custom ones); empty-cell and error captions; location growth and collision | yes: positions and texts |
| Sorting | label ascending/descending, data source order, manual item order, sort by a value field (and by a column item), custom lists | yes: positions |
| Filtering | page fields (single or multiple selection), manual hidden items, label filters (begins with, contains, between…), value filters, date filters (dynamic: this month, Q1…), Top/Bottom N by items, percent or sum; "show items with no data"; evaluation order of several filters; totals of filtered items | yes |
| Grouping | numeric ranges (start, end, interval; labels like `<10`, `10-14`, `>90`), dates (seconds…years, days with an interval; Excel 2016+ auto grouping makes Years/Quarters/Months fields), manual text groups (`Group1`) | yes |
| Calculated | calculated fields (a formula over field sums, applied at totals as well); calculated items (a formula over items of one field; solve order; restrictions) | yes |
| Formatting | number format per data field (changes displayed text), styles, conditional formats, column widths | text only, or not observable |
| Lifecycle | refresh, preserve formatting, defer layout update, structural edits around the pivot, a refresh blocked by non-empty cells (`#SPILL!`-like on the web; spike observation) | yes |
| Interaction | drill-down (Show Details makes a new sheet), Show Report Filter Pages, expand/collapse (`sd="0"` hides children) | yes |
| GETPIVOTDATA | every shape above, plus generated references | yes |
| Slicers and timelines | filter state shared across pivots of one cache | yes, as filters |
| PivotCharts | a chart bound to a pivot (`c:pivotSource`), its series and multi-level categories, field buttons, `c:pivotFmts` | chart caches only (§7) |

## 4. How we enumerate the features (completeness method)

A single machine-readable **pivot feature catalog**, `eval/catalog/pivot-features.tsv`, plays the role
`eval/catalog/excel-functions.tsv` plays for functions. It has one row per feature. Columns:
- `id` (`PV-0001`…) and `family`;
- `name`;
- `ooxml`: the element or attribute path and its default, e.g. `pivotTableDefinition/@outline=1`,
  `dataField/@showDataAs`, `x14:dataField/@pivotShowAs`;
- `affects`: `values`, `layout`, `text` or `none`;
- `web`: whether the oracle renders it (`yes`, `no`, `unknown`), answered by the PV0 capability matrix;
- `scope`: `in`, `later`, `out`, or `deviation`, with the decision that set it;
- `stage`: PV1–PV5;
- `status`;
- `evidence`: the cases and the corpus files that cover it.

It is built from three independent sources and reconciled. A source that names a feature the others lack is a
finding.
1. **The schema, mechanically.** A script walks the ECMA-376 transitional XSD (sml-pivotTable and
   sml-pivotTableShared) and the published [MS-XLSX] extension schemas (x14 pivotTableDefinition, pivotField,
   dataField, pivotCacheDefinition, slicers; x15 timelines and pivotTableUISettings; xpdl/x16). It emits every
   element and attribute with its default. Each one gets a catalog row, or an explicit `none` row with the
   reason (for example `@applyFontFormats`: formatting only). This step is what makes the list complete. The
   Open XML SDK class references are the readable form of the same schema.
2. **The API surface.** Every enum value of Office.js `Excel.AggregationFunction`, `ShowAsCalculation`,
   `PivotFilterType`, `LabelFilterCondition`, `ValueFilterCondition`, `DateFilterCondition`, `PivotLayoutType`
   and `SubtotalLocationType`, and every `PivotLayout` and `PivotField` property.
3. **The user-facing documentation.** support.microsoft.com PivotTable articles and the Excel UI's PivotTable
   Options dialog, tab by tab, which `excel-docs.md` lists with URLs.

Each catalog row then gets **edge pools**, as functions have (decision 0094). These are the data shapes that
must be crossed with it: blanks, errors, text in value fields, mixed-type and case-variant items, dates with
times, negative and zero values, single records, empty intersections, ties, and very many items.

## 5. How we implement it (architecture sketch; the real design is a spec, PV0)

The pivot becomes a pure function from (definition, cache) to a **result**. Rendering it into the grid, and
reading it back with GETPIVOTDATA, are separate consumers. This follows the shape mog and LibreOffice share,
but Excel's own intermediate form is the contract.

```
source range ──refresh──▶ PivotCache (typed, interned items, Excel item order; exists today, extended)
                           │
definition (full model) ───┤
                           ▼
 1. record filter   page fields, hidden items, slicer state
 2. grouping        numeric/date/manual groups → derived item ids per record
 3. accumulate      result tree: (row tuple × column tuple × data field) accumulators,
                    plus subtotal and grand-total nodes, custom subtotal functions
 4. calculated      calculated items (per field, solve order); calculated fields (formula over sums,
                    evaluated by the engine's own parser/evaluator in a no-I/O sandbox, directive 3)
 5. auto-show/sort  label/value/date filters, Top-N, sort by label/value/manual; totals of what stays
 6. show values as  three passes (leaves, subtotals, grand total), Excel's base-item rules
 7. layout          emit Excel's own rowItems/colItems (<i t= r= i=><x v=/></i>) and location,
                    then cells: compact/outline/tabular, captions, page area, blank rows
                    ▼
PivotResult ──▶ grid writer (pivot-owned cells; a collision is an error state, not an overwrite)
          ──▶ GETPIVOTDATA (reads the result, not the cells; the SPEC-0015 rule kept)
          ──▶ xlsx export (definition + cache + records + rowItems/colItems)
          ──▶ PivotChart data view (§7)
```

Key choices to settle in the spec:
- **`rowItems`/`colItems` as the intermediate contract.** Excel writes them on every save, and every
  Excel-saved corpus file carries them, so the layout can be checked structurally before any cell. They are
  compact, they show exactly where an engine goes wrong, and our export writes them anyway.
- **When a pivot renders.**
  - Classic Excel never refreshes a pivot on a source edit. Excel's 2025 **PivotTable Auto Refresh**
    (Insider, per data source, on by default for new pivots on local data) changes that, and how it is
    stored in the file is undocumented. The baseline is: refresh on load with `refreshOnLoad`, on an
    explicit API call, and at creation. Auto Refresh is a PV4 item once a probe shows its file flag and
    shows whether Excel for the web honours it.
  - On load without `refreshOnLoad` we keep Excel's saved cells, as today. That way an Excel-saved file stays
    bit-exact even where our renderer is incomplete.
  - Rendered cells are constants in the grid, so formulas that read them need no new dependency machinery.
- **One definition per concept.**
  - Aggregates reuse the SPEC-0009 walker or the GETPIVOTDATA folds, whichever Excel's evidence matches; a
    probe decides plain sum or another order. Variance reuses `excel_variance`.
  - Formula evaluation for calculated fields and items reuses the parser.
  - Number-format display reuses the formatter.
- **Directive 5.** Caps on items per field, result-tree nodes, output cells (the sheet grid), calculated-item
  recursion and solve iterations. Work is counted against the formula work budget (SPEC-0024). Every cap
  carries a `DEVIATION` entry.
- **The model grows from read-only to full.**
  - Every attribute in the catalog with `affects != none` becomes typed.
  - The others are kept verbatim for round trip.
  - mog's idea of "promote or preserve, with a reason" covers anything we cannot yet compute: such a pivot
    keeps its saved cells and is exported untouched.

## 6. How we prove it (validation)

Six layers, from narrowest to broadest. Each reports per catalog row, so "implemented" always means "N oracle
cells pass".

1. **Oracle pivot suite, `pivots`, hand-written probes.**
   - A spec case gives the source cells and a **definition JSON** that maps one-to-one onto the OOXML. The
     builder grows from one compact shape to the full model: it writes a consistent cache, records and axis
     `<items>`, which is the minimum Excel for the web accepts (spike run 2), and leaves the target area empty.
   - Excel renders the pivot.
   - The sidecar gains a `pivots` section holding every cell of Excel's final pivot area and page area, its
     `location`, `rowItems`, `colItems`, group items and displayed texts. Today sidecars hold formula cells
     only.
   - The engine refreshes on load and is scored cell for cell (values, then text) and structurally. One probe
     case per catalog row plus its edge pools; the excel-probe skill applies unchanged.
2. **Generated suite, `gen-pivot`.**
   - A covering-array generator (pairwise, 3-wise for the dimensions that interact: layout × subtotals ×
     Values position × filters × show-as) over the catalog's dimensions.
   - The data come from the edge pools: types, blanks, errors, case variants, dates, ties and item counts.
   - One pivot per sheet, many sheets per workbook, to keep oracle round trips low.
   - Seeded and reproducible, as gen-m* are. Target: about 20,000 scored pivot cells for PV1, growing by stage.
3. **Excel-saved corpora.** The suite `pivot-corpus` fetches files and never vendors them (LibreOffice
   MPL-2.0, ClosedXML MIT, Calipers, POI and corpus-bench; EPPlus's Polyform-NC files are excluded). Three
   checks:
   - (a) import and GETPIVOTDATA;
   - (b) re-render from the saved cache must equal Excel's saved cells and `rowItems`/`colItems`, but only
     when the file is fresh, meaning its records reproduce its cells; stale files are reported, not scored;
   - (c) a full refresh from the source range, when `refreshOnLoad` is set.

   T-4906 implements (b) and (c): a pivot is fresh when its saved records give every value Excel saved at the
   cells its own `rowItems`/`colItems` place them, so freshness does not depend on the renderer; a fresh pivot's
   saved area is scored cell by cell and its engine `rowItems`/`colItems` compared; (c) is compared where the
   saved cells are a render of the records the source still holds (eval-harness.md "Pivot freshness").
4. **Round trip through Excel.**
   - An engine-created or engine-edited pivot is exported, and the oracle opens it. A refusal
     (`OpenWorkbookBlockedWorkbook`) or a repair is a failure.
   - Excel's re-render must equal ours, and its rewritten `rowItems` must equal ours.
   - This is also how export is validated; ClosedXML's list of crash invariants is the checklist.
5. **Metamorphic and property tests (no oracle, every CI run).**
   - Refresh is idempotent.
   - For every value cell, GETPIVOTDATA with that cell's tuple returns the cell.
   - A Sum subtotal equals the sum of its children, where Excel's own evidence says so.
   - Permuting the source rows changes only order-dependent outputs (first-seen casing).
   - A PIVOTBY formula equals the pivot for the shapes both support.
   - Import → export → import is a fixed point.
6. **Robustness.** A fuzz target over pivot import and render (hostile `rowItems`, indices, groups, formulas,
   huge item counts). No panics and bounded work, as directive 5 requires.

**Gate and exit.** Each stage's suites join the CI eval gate (baseline ratchet) when the stage opens. A stage
exits when:
- every catalog row in that stage has at least 30 oracle cells, counted per row by edge pool;
- the `pivots`, `gen-pivot` and fresh-corpus cells are 100% not visibly off (decision 0083's rule), apart
  from listed deviations;
- every remaining row has a task, or an `out` scope with its decision.

The eval report gains a "Pivot features" table with one row per catalog row (cells, pass rate, evidence
depth), as the function catalog has.

## 7. PivotCharts: what the engine owes them

RustSheet has no charts and will not draw any. A PivotChart adds four obligations on top of the pivot
itself:
1. **Its data.**
   - A PivotChart plots the pivot's visible result. Each row-axis tuple becomes a category, multi-level when
     several row fields are nested.
   - Each column-axis tuple times each data field becomes a series.
   - Subtotals and grand totals are left out.
   - Filters, hidden items and collapsed (`sd="0"`) items apply.
   - Show-as values are plotted as shown.
   - The engine exposes this as a `PivotChartData` view derived from `PivotResult`. It is cheap once the
     result exists.
2. **The file.**
   - The chart part keeps `c:pivotSource` (`[Book.xlsx]Sheet!PivotTable1`, `fmtId`), `c:pivotFmts` and the
     c14/c15 pivot options.
   - Its `c:numCache`/`c:strCache`/`c:multiLvlStrCache` must be rewritten from `PivotChartData` on export, or
     Excel and other readers show stale numbers.
   - This needs chart-part preservation in the xlsx crate, part of M11's unknown-part round trip.
3. **Evidence.** If Excel for the web rewrites the chart caches when it re-renders the pivot, which is a PV0
   probe, the caches become an oracle for (1). POI's `WithChartSheet.xlsx` and LibreOffice's
   `forum-mso-de-104083.xlsx` already hold Excel-written caches.
4. **Constraints.**
   - Field buttons and chart-driven filtering are UI; we only need the filter state they produce, which is
     the pivot's.
   - Chart types a PivotChart cannot use (scatter, bubble, stock) matter only to a create API.

So PivotCharts are stage PV5. They are small once PV1–PV3 exist, and their only hard dependency is chart-part
round trip.

## 8. Stages

| Stage | Scope | Exit evidence |
|---|---|---|
| **PV0 Foundations** | decision (M16, lanes, scope ruling, PivotChart ambition); SPEC-0032 (model, result, layout contract, refresh semantics, limits); feature catalog (schema walk + API + docs); oracle capability matrix (which catalog rows the web opens and renders; desktop-only ones go to M14); builder generalization; sidecar `pivots` capture; harness scoring of pivot areas; corpus fetcher; the spike findings committed as probes | catalog merged; `pivots` suite with ≥ 1 filled probe per family; capability matrix in `docs/excel-semantics/pivots.md` |
| **PV1 Refresh and core render** | refresh from a worksheet range or Table into the cache with Excel's item typing, identity and order; the 11 aggregates over records with text, blank and error rules; compact/outline/tabular; default subtotals top/bottom; grand totals; several data fields and the Values field on either axis; page fields (single/multi); manual hidden items; label sort; captions, `(blank)`, empty/error captions; location growth, page-area offsets and collisions; grid writer; refresh on load and by API; GETPIVOTDATA over every PV1 shape (closes T-2085) | `pivots` + `gen-pivot` (core) 100%; corpus files with only PV1 features re-render exactly |
| **PV2 Analysis** | all 14 show-values-as modes; sort by value; label/value/date filters; Top/Bottom N; filter order and totals of filtered items; custom subtotal functions; show items with no data; repeat labels; blank rows; per-field layout mixes; data-field number formats (displayed text) | the same, per row |
| **PV3 Grouping and calculation** | numeric, date and manual groups, with Excel's labels; shared group fields; calculated fields; calculated items with solve order and Excel's restrictions; errors they produce | the same |
| **PV4 Lifecycle and I/O** | xlsx export of everything above (validated by the Excel round trip); create/edit/refresh API and the Python binding; structural edits (T-2402); shared caches; Table and named-range sources, dynamic ranges; drill-down as an API (records behind a cell) | round-trip suite 100% |
| **PV5 PivotCharts, slicers, timelines** | `PivotChartData`; chart-part round trip and cache rewrite; slicer and timeline state imported as filters and exported | chart-cache evidence, where the probe shows it exists |
| **Out of scope** (proposed, by decision) | OLAP and the Data Model / Power Pivot (DAX, Distinct Count), external connections, multiple consolidation ranges (revisit), styles and conditional formats beyond preservation, Show Report Filter Pages (a UI command), expand/collapse UI (its `sd` state is in PV2) | |

The order is deliberate. PV1's layout engine and result tree carry everything after them. Several later
features change where the cells go (filters, sort, blank rows), so the layout must be right first.

## 9. First tasks proposed (filed after review)

Filed 2026-11-04 as T-4120–T-4188 under docs/tasks/M16 (decision 0274): every stage's tasks, not only these nine.

All are size S or M, as CLAUDE.md requires; anything bigger becomes a spec first.
1. **Decision:** M16 "Pivot tables", a `pivot` lane parallel to Phase 1, the scope ruling of §8, and the
   PivotChart ambition (data view and round trip, no drawing). Research lane, S.
2. **Commit the spike as probes:** the re-render on load (closes T-2327); a calculated field on sums;
   Top-N totals; label sort; numeric group labels; the collision that gives `#SPILL!`. Research lane, S, each
   needing builder support from task 4.
3. **SPEC-0032**, the pivot engine: model, `PivotResult`, `rowItems` contract, refresh semantics, grid
   ownership, limits, export. Pivot lane, M.
4. **Builder v2:** a definition JSON one-to-one with OOXML; a consistent cache, records and items; an empty
   target area; validation that refuses what Excel would refuse (ClosedXML's invariants). Harness lane, M.
5. **Sidecar `pivots` capture and pivot-area scoring** (snapshot reads non-formula cells inside Excel's saved
   pivot locations, plus `rowItems`/`colItems`). Harness lane, M.
6. **Feature catalog v1:** the schema walker, the API and docs reconciliation, the TSV, and its report
   table. Harness and research lanes, M.
7. **Oracle capability matrix:** one minimal probe per catalog row; which ones Excel for the web refuses,
   repairs or renders. Research lane, M, split by family.
8. **Corpus suite `pivot-corpus`:** fetch the LibreOffice, ClosedXML, Calipers, POI and corpus-bench pivot
   workbooks, classify each as fresh or stale, and score import and GETPIVOTDATA now (render scoring later).
   Harness lane, M.
9. **First PV1 probes, ahead of the renderer:** cache item order and identity (case, type order, booleans,
   errors, dates with times); the 11 aggregates over text, blank and error data; Excel's sum order; captions.
   Research lane, S each.

## 10. Risks and open questions

- **Web against desktop.** The oracle is Excel for the web. Rendering runs in the same calculation engine,
  but some features may be refused or read-only on the web (OLAP, possibly calculated items and some
  filters). The capability matrix (task 7) settles this. Desktop-only rows go to M14 (decision 0039) and
  never block M16.
- **The collision rule.** The web showed `#SPILL!` in the anchor cell when the new layout would overwrite
  other cells. This is Excel's documented 2025 "PivotTable #SPILL!" behaviour (Beta 2508+): the pivot
  collapses to one cell, which affects GETPIVOTDATA and PivotCharts. Older desktop builds prompt instead.
  The engine follows the evidenced current behaviour. Probes still need to show what GETPIVOTDATA returns
  against a spilled pivot, and when the pivot recovers.
- **Insider-era drift.** Auto Refresh and pivot `#SPILL!` arrived in 2025. The oracle is current Excel for
  the web, so goldens follow it (directive 1). Behaviour that is still rolling out is recorded with its
  date, so a later change shows up as a sidecar conflict and not as silent drift.
- **What the web can reach.** Graph's pivot API is list, get, refresh and refreshAll; it cannot create or
  edit pivots. Excel for the web's UI cannot create groups or calculated fields, but it renders them when
  they are authored in the file (spike runs 5 and 6). Calculated items and the Excel 2010+ show-as modes
  (stored in x14 extensions) are the first rows for the capability matrix.
- **Phase ordering.** Decision 0027 puts Phase 1 first. A parallel lane takes worker capacity from Phase 1's
  exit, so the owner decides the share.
- **Size.** This is the largest feature RustSheet has taken on: an estimated 30–50 tasks across PV0–PV5. The
  feature catalog and per-row coverage are what keep it honest, as the function catalog does for functions.
- **Licences.** LibreOffice (MPL-2.0) and EPPlus (Polyform-NC) are ideas only; no code is ported. Corpora
  are fetched, never vendored; EPPlus's files are not used at all.
