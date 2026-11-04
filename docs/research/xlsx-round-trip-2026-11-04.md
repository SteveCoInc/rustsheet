# xlsx round-trip fidelity: research and proposed plan (2026-11-04)

Owner request: load a workbook, make a small edit (add a cell, change a number), save it, and
get back a file that differs from the original only in ways we expect. Nothing we do not
understand should be silently dropped, including features Excel ships after us. The owner also
asked for a survey of how other projects handle this (mog, gridcraft and others), a strong plan
with test data, and an answer on whether a later-phase milestone already covers it. This
document is a proposal for owner review. No tasks are filed yet, and nothing here takes priority
over exiting Phase 1.

Scripts and raw census: `docs/research/xlsx-round-trip/`.

---

## 1. Short answers

**Is there a milestone for this?** Not really. M11 (Phase 1, "Real-world fidelity") has one
scope bullet, "xlsx round-trip preservation of unknown parts" (`docs/milestones.md`). It has no
exit criterion, no spec, no task and no eval suite, and M11's exit is measured only on
corpus-bench formula values. M10 has a similar orphan bullet, "Style storage and round-trip".

`docs/architecture/overview.md` says `rustsheet-xlsx` does "unknown-part passthrough". **That is
not true.** No passthrough exists at any level.

Proposal: a new milestone, **M15 "xlsx round-trip fidelity"**, placed in Phase 2. A decision
record would move the two orphan bullets out of M10 and M11, so they never block Phase 1 exit.
Details in §7.

**How good is the round trip today?** For cell content it is good. For everything else it is
poor.

- Over 2,028 workbooks (POI, Calipers, SpreadsheetBench, local fixtures) and 9.0M cells, an
  IO-only load+save changes **about 23,600 cells (0.26%) in 94 files**.
- Almost all of them come from two bugs: external-workbook references (about 18,000 cells) and
  dropped empty-string cells (about 3,700 cells). Those and the smaller bugs are listed in §2.2.
- At the package level we **drop every feature we do not model, in 100% of the files that have
  it**: charts, drawings, images, comments, threaded comments, data validation, autoFilter,
  page setup, headers and footers, sheet and workbook protection, VBA, pivots, external links,
  custom properties, customXml, chartsheets (the whole sheet disappears), rich text, and every
  unknown `extLst`. See §2.1.

**What do the best projects do?** They keep a typed model for what they understand. Everything
else is carried as verbatim bytes that belong to an owner, with a relationship-closure rule, ID
remapping and explicit invalidation when an edit makes those bytes stale (Mog, GridCraft). The
cheapest high-fidelity trick is to write untouched parts back byte-for-byte (umya) and to patch
only the changed cells in place (Formualizer). Testing uses Excel's own re-save as the golden
(Calipers). See §3.

**The plan in one line:** first define what "equivalent" means and measure it on every corpus
(stage 0). Then build a preservation layer, so that unknown parts and elements survive (stages
1–2). Then make the files we write index-stable, so untouched sheets are written back verbatim
and a content edit patches only the cells it changes (stage 3). Then model the high-frequency
features (stage 4). Then make structural edits safe (stage 5, with M4). Every step is gated by a
round-trip suite whose known losses only shrink, like the eval baselines.

---

## 2. Where we are (measured 2026-11-04, main @ ee79bfa4)

### 2.1 Package and feature losses

Method: `load_from_xlsx` → `save_to_xlsx`, no edit. The table counts the files in which a feature
is present in the original and lost or reduced in the output. Full table:
`docs/research/xlsx-round-trip/census-2026-11-04.md`.

| Feature (files that have it) | lost in |
|---|---|
| sheetFormatPr (1,981), page margins (1,979), fileVersion (1,817), workbook `extLst` (1,481; mostly `calcFeatures`, which Excel uses to decide whether to trust cached values), styles `extLst` (1,664) | all |
| Page setup (755), header/footer (522), print options (87), page breaks (54), printer settings (361) | all |
| sheetPr: tab colour (79), outline settings (171), codeName (VBA linkage) | all |
| Custom document properties (523), customXml (38), web extensions (321), thumbnail (31) | all |
| Data validation (130; x14 ext 16) | all |
| Sheet autoFilter outside tables (98) | all |
| Sheet protection (33), protected ranges (8), workbook protection (26), cell `protection` in styles (97) | all |
| Legacy comments/notes (106), threaded comments (13), VML (122) | all |
| Drawings (154), charts (72), images (67), OLE objects (6), ActiveX and form controls (9) | all |
| Pivot tables and caches (26), slicers (2), query tables and connections (7) | all |
| External link parts and `externalReferences` (70) | all; formulas are also rewritten, §2.2 |
| VBA project (9 .xlsm) | all; the file is not even marked macro-enabled |
| Chartsheets (9), macro sheets (2) | all; **the sheet itself disappears** |
| Rich text runs in shared strings (75), phonetic properties (381) | all; flattened to plain text |
| Indexed colour palette (191), custom table styles (43) | all |
| Unknown worksheet `extLst` (76) | 69 |
| Conditional formatting (182) | 0 lost by element count (content not compared) |
| Theme (1,961) | replaced by the bundled template, which keeps colours and drops font and format schemes (audit) |

Lower-level losses found by the audit (file:line in the audit notes):
- Fonts: underline variants and vertAlign.
- Fills: pattern types and gradients; every fill is written as `solid`.
- Alignment: indent, rotation, shrinkToFit.
- Rows and columns: outline level and collapsed.
- Sheet views: non-frozen split panes, zoom, topLeftCell, rightToLeft. The workbook window is
  hard-coded to 800×600.
- calcPr: calcMode, fullCalcOnLoad.
- Defined names: hidden and comment attributes (T-3301).
- Notes: author.
- Shared formulas: every formula is written out expanded.
- docProps: title and subject.

### 2.2 Cell-level differences (IO only, no recalculation)

After load → evaluate → save, 261 of the 2,028 files have some cell that differs. Re-run with
`NOEVAL` (load → save, no recalculation), 94 files still differ, in these classes. Each is a real
bug or a missing normalization. **None has a task yet.** The 15,900 "formula cell without a
cached value" cells are an artifact of skipping recalculation and are not counted in the 23,600
above.

| Class | Example | Files | Severity |
|---|---|---|---|
| External-workbook references are printed as `'[1]Sheet'!A1` and the externalLink parts are dropped | `VLOOKUP(C2,[1]Sort!$B:$C,2,FALSE)` → `VLOOKUP(C2,'[1]Sort'!$B:$C,2,FALSE)`, 16,998 cells in corpus-bench 59932 | 60+ | high: the saved formulas point at a link that no longer exists |
| External structured reference becomes `#REF!` | `[1]!Table3[[#This Row],[Resource Name]]` → `#REF!` (cb 33094) | 1 | high |
| A shared-string index with trailing whitespace is read as 0 | POI bug69769: `<v>2 </v>` loads as "Konto" instead of "Mustermann". The snapshot reader trims, the importer does not. | 1 | high: wrong value on import, not only on export |
| Empty-string cells are dropped | `t="s"` pointing at `""` becomes blank (POI bug66675, 3,604 cells). ISBLANK and COUNTA change. | 5 | medium |
| A CR in a shared string comes back as a literal `_x000d_` | POI 60289 | 1 | medium |
| Leading whitespace in a formula is lost | `= FG141` → `=FG141` (Calipers pv26) | 4 | low; Excel keeps it (SPEC-0023) |
| Number literals are reprinted | `9.99E+307` → `9.99e307`, `0.000000000001` → `1e-12` | 6 | low, but needs an Excel probe: does Excel accept the lowercase, unpadded form in a file? |
| `_xlfn.` is added to GAMMALN, ISO.CEILING, ECMA.CEILING | | 3 | known: T-0417 |
| Formula cells without a cached value are written with a value | `<v>0</v>` | 36 | low; recalculation normally hides it |
| Reversed ranges are normalized | `O2:O1` → `O1:O2` | 1 | probably benign; Excel normalizes on entry, probe it |

The cell values that differ only after recalculation are engine parity, which the function
lanes already track, not IO.

Also found:
- `import/tables.rs:85` and `:154` read `dataDxfId` from `headerRowDxfId`, a bug inherited from
  IronCalc.
- `poi/style-alternate-content.xlsx`, a legitimate file, fails to load.

### 2.3 What tests exist

- `compare.rs::test_load_and_saving` runs load → save → load over 248 local workbooks and compares
  **cell values only**.
- About 15 targeted round-trip tests cover data tables, iterate, the date system, dynamic
  arrays, rich errors, names, hyperlinks, merges and conditional formatting.
- No test looks at a part, element or attribute. No suite measures fidelity.

---

## 3. What other projects do

Sources were read in shallow clones (mog, calipers, gridcraft, IronCalc, umya-spreadsheet,
formualizer, xlsx-populate, exceljs). The others are covered from their documentation.

| Project | Pattern | Unknown parts | Unknown elements / extLst | Test method |
|---|---|---|---|---|
| **Mog** (Rust) | Typed model, plus a **package ownership matrix** (`write/package_ownership.rs`). Every part pattern has an owner, its relationship and content types, *dirty-invalidation triggers* and an *opaque policy*. | Classified: inert-opaque (replayed: customXml, printerSettings, chartsheets, webextensions, thumbnail), typed-owned, active-quarantined (VBA, ActiveX), forbidden (signatures, dropped), diagnostics-only (revisions). An opaque part may only point at its own folder family. | Unowned workbook children are replayed in schema order if they have no relationships. `mc:AlternateContent` is dropped. `mc:Ignorable` is rebuilt from the prefixes actually written. Range-bound raw sheet XML (protected ranges, custom views, controls…) is **cleared on any row or column insert or delete**. No global extLst replay. | `xlsx-gate` package-graph validator. Structural XML diff: namespace-aware, unordered paths, attribute-default table. Calipers corpus. |
| **Calipers** (Mog's corpus) | | | | Golden = **desktop Excel's re-save** of `init.xlsx`. The gate is *semantic*: values, types, formulas, fonts, sheets, names, merges, panes, date1904. It ignores ordering, rIds, sst and style indexes, number spelling and tint text. Narrow per-case exceptions are set in `config.json`, with a self-test that a mismatch nobody excused still fails. An optional `--package` diff normalizes calcChain, printerSettings, docProps timestamps and calcId. 83 roundtrip cases, 247 in total; hostile files sit in `_disabled/`. |
| **GridCraft** (Rust) | Typed model plus `Preserved{parts, elements, sheets}` owned by the workbook or a sheet (`model/src/preserved.rs`). | Allow-listed roots, kept only if the **relationship closure** (≤ 16 hops) never reaches a part GridCraft regenerates. Parts are renamed on collision, rIds remapped inside fragments, and a fragment is dropped if an ID cannot be mapped. VBA only goes into .xlsm. | Fixed schema slots: legacyDrawingHF, picture, oleObjects, controls, found even inside AlternateContent. The calcFeatures ext is kept. A VML merge keeps controls and regenerates notes. Unmodelled extLst is lost (admitted). | `gridcraft-cli check`: package validator ("what Excel would repair"), **no relationship type lost**, semantic re-read by category. Run over 4,224 real files. Never checked in Excel. |
| **Formualizer** (Rust) | **Byte-span patching**. Its strict recalc path rewrites only `<v>` and `t=` spans and keeps every other zip member byte-identical, local headers included. | Untouched | Untouched | Value-only edits; refuses ZIP64 and encryption. |
| **umya-spreadsheet** (Rust) | Typed, with **lazy sheets**: a sheet never deserialized is written back byte-for-byte with its relationship closure. | Kept only for untouched sheets | Lost once a sheet is touched | |
| **IronCalc** (our upstream), SheetJS CE, ExcelJS, openpyxl, Univer | Pure regeneration | Lost (openpyxl and SheetJS keep VBA on request) | Lost | Cell-value round trip |
| **Apache POI**, NPOI, EPPlus, Open XML SDK, ClosedXML, xlsx-populate, docx4j | **DOM retention**: the whole infoset is kept and edited in place. | Kept (an OPC package of generic parts) | Kept automatically (XmlBeans infoset, `OpenXmlUnknownElement`) | Unknown content goes stale after row shifts, because nobody updates refs inside extLst. calcChain is dropped or trimmed. |
| **LibreOffice** | **InteropGrabBag**: a hidden property map on objects, which import fills with unhandled markup and export writes back. | Partial | Partial; grab bags leak on copy/paste unless cleared on edit | |

ECMA-376 Part 3 (Markup Compatibility) does not help a saving consumer: the editions that
defined `PreserveElements` and `PreserveAttributes` for markup editors were superseded, and the
current standard has neither. A consumer that saves has to decide for itself what to keep. Every
serious project ends up with the same rule: **keep it only while you can prove it is still
consistent; otherwise drop it and say so.**

Recurring lessons:

1. **calcChain** is always dropped. `calcId` and the `calcFeatures` ext decide whether Excel
   trusts cached values or recalculates on open, so write them deliberately.
2. **Never replay source `.rels` or `[Content_Types].xml` wholesale.** Re-derive them from the
   current owners, and remap `r:id`s inside kept fragments.
3. **Range-bound opaque content goes stale on structural edits**: x14 data validation `xm:sqref`,
   sparklines, control linked cells, query-table ranges. Model it, shift it, or invalidate it;
   never leave it silently stale.
4. **Active content** (VBA, ActiveX) is kept as bytes, never interpreted, and only in
   macro-enabled formats. Signatures are dropped.
5. **Bound everything**: total kept bytes, per-part size, part count, relationship count, closure
   hops, fragment depth.
6. **Test against Excel's own re-save**, not against the input bytes. Excel itself rewrites a
   great deal on save.

---

## 4. The round-trip contract: what "not different in a way we don't expect" means

Excel is the spec here too (directive 1). The question is not "are the bytes equal" but "does
the file differ from what Excel itself would produce for the same operation, in any way a user,
or Excel, could notice?". We define three scenarios and two equivalence levels.

**Scenarios**
- **R0, no-op:** load → save.
- **R1, content edit:** load → set one or more cell values or formulas → recalc → save. This is
  the owner's main use case.
- **R2, structural edit:** insert or delete rows or columns, add, rename or delete a sheet, move
  a range. This depends on M4.

**Equivalence levels**
- **Semantic (S).** Everything we model or preserve compares equal after both files are parsed
  into a canonical feature model. For cells that means value, type, formula text and style
  properties; then sheets, names, merges, CF, DV, comments, tables, views, print setup,
  protection and so on, plus the inventory of preserved parts and fragments by owner and kind.
  This is the gate.
- **Package (P).** A structural XML diff part by part. It is namespace-aware and order-insensitive
  where the schema allows. It uses an attribute-default table, so `indexed="0"` equals the
  attribute being absent. It renumbers rIds, shared-string and style indexes, part names and
  zip metadata. It applies a short, decision-recorded **normalization list**:
  - calcChain;
  - docProps timestamps and `lastModifiedBy`/`Application`/`AppVersion`;
  - `calcId`;
  - `xr:revisionPtr` and `xr:uid`s;
  - `x14ac:dyDescent`;
  - `row@spans`;
  - `fileVersion` build numbers.

  This is the diagnostic, and the ratchet for "nothing lost". Every item on the list must cite
  evidence that Excel itself rewrites or drops it on save.

**The expected-delta rule (R1).** `P(original, ours_edited)` must contain nothing beyond:
1. the edited cells;
2. the cached values of cells whose value changed on recalculation;
3. new shared strings and styles appended;
4. the normalization list.

This can be checked **offline**, without Excel, by diffing against our own R0 output: R1's
package diff minus R0's must be exactly the edit. That gives the owner's "only the change I
made" guarantee directly.

**Excel as arbiter.** For each R0 or R1 case, the golden is what Excel does:
- R0: Excel re-saves the original, as in Calipers.
- R1: Excel applies the same edit through the oracle's range PATCH and saves.

We compare ours with Excel's under S, and report P. The excel-graph oracle already uploads a file,
edits it and downloads Excel's save (`docs/architecture/oracle.md`). Two open questions:
- Excel for the web may itself drop desktop-only features (RQ1).
- A file Excel for the web refuses to open is a corruption signal we can record cheaply. The
  repair dialog itself is desktop only, so that part is M14.

**Opening cleanly.** Every output must pass an offline package validator:
- content types for every part;
- every relationship target exists, and rIds are unique;
- no dangling `r:id` in any part;
- XSD validation of every regenerated part against the ECMA-376 transitional schemas plus the MS
  extension schemas, which catches the element-order bugs Excel "repairs".

---

## 5. Proposed architecture

The design goal: **typed where we understand, verbatim where we don't, never stale, always
bounded.** It combines Mog's ownership discipline, GridCraft's closure rule, umya's clean-part
shortcut and Formualizer's in-place patching. Full DOM retention (POI and EPPlus style) is
rejected. Its memory cost fights directive 5, it fights our typed model, and it still leaves
extension content stale after structural edits.

### Layer 1: source package retained, bounded

On import, keep the source package:
- every zip entry as `Arc<[u8]>`, with its content type and parsed relationships;
- attached to the `Workbook` as an opaque `SourcePackage`.

It is charged against `ImportLimits`, with new limits: kept bytes, part count, relationships,
closure hops (GridCraft's values as the starting point). Over a limit, we keep nothing extra and
emit a diagnostic; the file still loads. The engine never reads these bytes. Only
`rustsheet-xlsx` does.

### Layer 2: ownership matrix (one table, one definition)

One table maps every part pattern and relationship type to an owner and a policy:

| Policy | Meaning | Examples |
|---|---|---|
| `Modelled` | Regenerated from the typed model | workbook, worksheets, styles, sharedStrings, tables, theme (once modelled whole), metadata |
| `Replay` | Written back verbatim while its owner exists and its closure is intact | customXml, custom.xml, webextensions, printerSettings, thumbnail, charts, drawings and media until modelled, chartsheets, pivots until SPEC-0015 grows export, externalLinks |
| `ReplayIfClean` | Verbatim only if no edit touched what it depends on; otherwise dropped with a diagnostic | drawings anchored to cells after a structural edit, pivot caches whose source range was edited (caches carry their own refresh flag, RQ4) |
| `Active` | Verbatim bytes, only into .xlsm/.xltm; never interpreted | vbaProject.bin, ActiveX |
| `Drop` | Never written | calcChain, digital signatures, revisions (with a diagnostic) |
| **unknown** | Default for any relationship type not in the table: `Replay` if its closure reaches no `Modelled` part, otherwise `Drop` plus a diagnostic | whatever Excel ships next |

The last row is the answer to "features we have not heard of yet". A new kind of part Excel adds
(as richData, threadedComments, featurePropertyBag and webextensions each were) survives the round
trip on day one, as long as it hangs off the package or a workbook or sheet relationship and does
not point into parts we regenerate.

### Layer 3: element-level slots inside regenerated parts

When we regenerate `workbook.xml`, `sheetN.xml` and `styles.xml`, the importer captures what the
typed reader does not consume:
- unknown child elements, keyed by their position in the schema sequence (`CT_Worksheet` and
  others);
- unknown `<ext uri>` entries, keyed by URI;
- unknown attributes on the root and on well-known elements;
- the namespace declarations each one needs.

Each fragment is classified on capture:
- **has r:ids**: remapped on write, dropped if one cannot be mapped;
- **range-bound**: contains `sqref`, `ref` or formula text (detected per known element, and
  assumed for unknown exts that contain A1-like attributes);
- **sheet-name-bound**: contains sheet names.

The writer emits fragments in schema order and rebuilds `mc:Ignorable` from the prefixes actually
emitted, as Mog does.

`mc:AlternateContent` blocks are kept whole when their owner is unmodified, and dropped (with a
diagnostic) when it is. `x15ac:absPath` is dropped deliberately, because it leaks the author's
path; this needs a probe.

### Layer 4: invalidation (when kept content becomes stale)

Each kept item carries dirty triggers. These are a closed set, defined once next to the ownership
matrix:

| Trigger | Invalidates |
|---|---|
| Cell content edit (R1) | Nothing structural. Cached values in replayed parts that cache cell values do not update (chart caches, pivot caches, external link caches); we keep them and write `fullCalcOnLoad`, or Excel's equivalent, only if a probe shows Excel needs it (RQ3). |
| Structural edit on sheet S (R2) | Range-bound fragments and `ReplayIfClean` parts anchored to S. They are dropped with a diagnostic until stage 5 teaches the shifter to rewrite them. |
| Sheet rename | Sheet-name-bound fragments workbook-wide |
| Sheet delete | Everything that sheet owns, plus anything whose closure reaches it |
| Style table rebuild | Nothing, once stage 3 makes styles append-only |

Every drop goes into a `SaveReport` (a list of `{owner, item, reason}`), which the facade returns
from `save_xlsx`. Callers, and the round-trip suite, can then tell "we dropped X because you
inserted a row" from a bug.

### Layer 5: index stability, verbatim sheets and cell patching (the small-edit fast path)

To make R1 diffs tiny, keep the source's indexes:
- **shared strings are append-only** (source order kept, new strings appended);
- **cellXfs, dxfs, fonts, fills and borders are append-only** (source order kept);
- defined names, sheets and tables keep their source order and IDs.

With stable indexes, an **untouched sheet can be replayed byte-for-byte**, together with its
relationship closure. A sheet counts as touched if a cell in it was edited, its recalculated
values differ from the cached values, or a structural edit hit it. A **touched sheet with
content-only edits** can be written by a streaming patcher. The patcher copies the source XML
and replaces or inserts only the `<c>` elements whose content changed, keeping every other byte:
`extLst`, views, print setup, unknown elements. Regeneration is the fallback for structural
edits and for files that are not index-stable (for example a broken source sst).

That gives three tiers per part, **verbatim → patched → regenerated**, chosen per part at save
time. All three must pass the same semantic gate, so the tiers can be checked against each
other: regenerating a patched file must be S-equal. That keeps two writers honest.

Why both preservation (layers 1–4) and patching (layer 5)? Patching alone gives near-perfect R1
fidelity but nothing for R2 or for new workbooks. Preservation alone still rewrites every byte
of every sheet: any attribute we never thought to capture is lost, and diffs are noisy. Together
they give byte-identical untouched content and an exact, explainable delta on touched content.

### Safety (directive 5)

- Kept bytes are never parsed by the engine.
- Fragments are parsed only by the bounded XML reader we already use.
- The patcher is a streaming tokenizer with depth and size limits.
- Zip-bomb, rels-cycle and XML-bomb inputs join a hostile set (POI's clusterfuzz files and
  Calipers' `_disabled/` are a start). The new limits get a `DEVIATION`-style entry where they
  refuse what Excel opens.
- VBA is carried, never executed. We take no position on its safety; it is the user's file.

---

## 6. Test data and harness

### 6.1 A `roundtrip` suite kind in `rustsheet-eval`

The harness runs R0 and, per case, a deterministic R1 set:
- set an existing number;
- set an existing formula;
- add a value in an empty cell inside the used range;
- add a value outside the used range;
- add a new string, so the sst grows;
- add a styled cell, so styles grow.

It scores **fidelity items**. One item is one feature instance, for example "sheet2
dataValidation #3", "part xl/charts/chart1.xml" or "ext {05C60535…} on Sheet1". An item passes
if S-equal to the golden and fails if lost, changed or stale-kept. A third bucket holds
`dropped-with-reason`, scored like the accepted-deviation column.

Baselines work like `eval/baseline/` (known lost items, only shrink; `make eval-baseline`
refuses regressions). The report gets a fidelity section beside the gap list: lost items by
feature, by corpus, worst files. Goldens: Excel re-saves where we have them (Calipers), and our
own R0 plus the expected-delta rule everywhere else.

The package validator and the XSD check run on every output and gate at 100%.

### 6.2 Corpora (existing, fetched)

| Corpus | Files | Value for round trip |
|---|---|---|
| Calipers `roundtrip/` + all goldens | 83 / 458 | **Windows-Excel re-save goldens already exist.** The fetch currently takes only `golden.xlsx`; widen it to `init.xlsx` so R0 can be scored as Calipers scores it. Covers autofilter, CF, comments, drawing, pivot, charts, picture, textbox, themes, DV, header/footer, protection, rich text, outlines, RTL. |
| POI | 366 | The most feature-diverse: VBA, chartsheets, macro sheets, OLE, ActiveX, external links, plus the hostile set. |
| corpus-bench (SpreadsheetBench) | 912 | Real-world distribution: 477 with custom properties, 401 page setup, 380 header/footer, 71 autoFilter, 68 DV, 46 external links. |
| Local fixtures + upstream calc tests | 292 | IronCalc-authored and spec-built |

To add, fetched and never vendored, with licences recorded:
- SheetJS `test_files` (Apache-2.0);
- the ClosedXML and Open XML SDK test files (MIT);
- LibreOffice `sc/qa/unit/data/xlsx` (MPL-2.0, fetched);
- openpyxl's test data;
- the FUSE subset of xlsx-corpus-bench, for real-world breadth.

### 6.3 Feature probes (built like our function probes)

**(a) Authored probes, Excel-normalized.** A probe spec gets a `features` block, and spec2xlsx
builds the workbook with XlsxWriter, which can author almost every feature we lose today:
- CF of every type, including x14 data bars and icon sets;
- DV of every type and list source;
- notes, charts of every type, sparklines, images, textboxes and shapes, form buttons, VBA;
- tables with styles, autoFilter, panes, split views, zoom;
- page setup, headers and footers with images;
- protection, outlines, rich strings, defined names, hyperlinks, custom properties.

The oracle then has Excel re-save each probe, R0 and each R1 edit, and the downloaded file is
the golden. One probe per feature × variant, with a small matrix of edits next to and on top of
the feature: edit inside a DV range, edit a CF-referenced cell, edit a chart's source data.
This is the "probe spreadsheets like function compatibility" the owner asked for. Breadth over
functions is explicitly not the aim: one formula-heavy probe is enough.

**(b) Excel-only features.** Some features XlsxWriter cannot author: threaded comments, slicers,
timelines, pivots, linked data types and images in cells, the 2024 checkboxes, Power Query
connections, sensitivity labels, sheet views and named sheet views. These come from:
- existing corpora (Calipers `pivot`, POI);
- Office.js scripts run through the oracle, since Calipers' `officejs/` shows this works;
- a short owner-authored list in desktop Excel (M14 territory if nothing else works).

**(c) Unknown-feature probes (synthetic, no Excel needed).** A generator takes any corpus file
and injects invented content into every slot:
- an unknown part with its own relationship from the package, the workbook and a sheet;
- unknown children at each schema position;
- an unknown `ext` URI in each extLst;
- unknown attributes;
- an unknown namespace under `mc:Ignorable`;
- an `mc:AlternateContent` with an unknown `Requires`.

The expectations come straight from the ownership matrix and the invalidation table: R0 and R1
keep it all, and R2 drops exactly what the table says. This is how we test the promise about
future Excel features without waiting for Excel to ship them.

**(d) Hostile set.** Zip bombs, rels cycles, closure chains longer than the hop limit,
oversized fragments, XML bombs, ZIP64, duplicate part names, case-colliding names. Expected: load
or refuse within limits, never panic, never keep unbounded bytes.

---

## 7. Milestone proposal: M15, xlsx round-trip fidelity (Phase 2)

**Placement.** Phase 2, alongside M3, M4 and M13. It is independent of M3 and M13, and its
stage 5 depends on M4's structural edits. Its tasks stay `blocked` until Phase 1 exits, like the
other Phase 2 milestones. A decision record would:
- move the orphan bullets (M10 "Style storage and round-trip", M11 "unknown parts") into M15;
- correct `docs/architecture/overview.md`.

**Optional carve-out for owner decision.** Allow the stage 0 *measurement* tasks to run in Phase
1 at P3, if a lane is idle. They change no engine behaviour and would give a baseline early.
Default: no, per the owner's instruction.

**Bugs found here.** The bugs in §2.2 affect Phase 1 cell values and formulas on import or export:
- the external reference rewrite;
- the shared-string index with trailing whitespace;
- empty strings dropped;
- `_x000d_`;
- the `dataDxfId` bug.

They would be filed as ordinary M11 tasks, because they are wrong cell content, not fidelity.
Only the owner's go-ahead is needed.

**Proposed exit criteria (measured by a `milestone_exit` M15 row):**
1. **Package validity**: 100% of outputs pass the validator and XSD check, over every corpus and
   scenario.
2. **R0**: over Calipers `roundtrip/` with Excel goldens, 100% of fidelity items S-equal, except
   items owned by an open M14 task. Over every other corpus, ≥ 99.9% of items kept with no
   unexplained loss, and 0 lost parts outside the `Drop` policy.
3. **R1**: the expected-delta rule holds for 100% of corpus files on the standard edit set, and
   S-equality with Excel's own edited save on ≥ 99% of the feature probes.
4. **Unknown-feature probes**: 100% kept under R0 and R1. Under R2, 100% kept or dropped exactly
   as the ownership matrix says, each drop reported.
5. **Modelled features**: DV, comments and notes, autoFilter, page setup and headers/footers,
   protection, sheetPr, rich text, full fills, alignment and fonts, and outlines are modelled
   (not just replayed) and shift correctly under R2 (with M4).
6. **Hostile set**: no panic, bounded memory, `make fuzz` targets for the patcher and the
   fragment parser.
7. **Byte stability**: untouched parts are byte-identical to the source under R1 on 100% of
   index-stable files.

---

## 8. Staged work plan (tasks to be filed after review)

Sizes follow our S/M rule. Each stage has a spec where it is architectural.

**Stage 0: measure (no behaviour change).**
1. SPEC: round-trip contract. S and P equivalence, normalization list with evidence,
   expected-delta rule, fidelity items, SaveReport shape.
2. A `roundtrip` suite kind and fidelity scoring in `rustsheet-eval`, with a structural XML diff
   (namespace-aware, defaults table, unordered paths) and the package validator.
3. Widen the Calipers fetch to `init.xlsx`; add the round-trip suites over Calipers, POI,
   corpus-bench and fixtures with baselines.
4. XSD validation of outputs: vendor or fetch the ECMA-376 transitional and MS extension schemas
   (licence check).
5. An oracle flow for Excel goldens: R0 re-save and R1 edit-then-save, plus "Excel for the web
   refused to open" as a recorded verdict.

**Stage 1: package preservation.**
6. SPEC: ownership matrix, closure rule, limits, invalidation table.
7. Bounded `SourcePackage` retention in the importer.
8. Writer: `Replay`, `Active` and `Drop` policies, rId and part-name remapping, content-type
   derivation. Then chartsheets and macro sheets kept as sheets (in the sheet order and in
   `workbook.xml`).
9. External links kept as parts, with formulas printed back as `[1]Sheet!A1`.
10. Unknown-feature probe generator (6.3c) and the hostile set (6.3d).

**Stage 2: element preservation.**
11. Fragment capture and emission for workbook, worksheet and styles, in schema-slot order;
    `mc:Ignorable` rebuild; AlternateContent policy.
12. Keep `calcFeatures`, `fileVersion`, `workbookPr`, sheetFormatPr and phoneticPr. Decide `calcId`
    and `fullCalcOnLoad` from a probe.
13. Range-bound and sheet-name-bound classification, plus SaveReport.

**Stage 3: index stability and the small-edit fast path.**
14. Append-only shared strings and styles (keep source order).
15. Verbatim replay of untouched sheets.
16. Streaming cell patcher for content-only edits, with cross-check against regeneration.

**Stage 4: model the high-frequency features.** These are ordered by the census frequency
(§2.1). Each one moves from replayed to modelled and gets an Excel-normalized probe set:
17. Page setup, margins, header/footer, print options, page breaks.
18. Data validation (including x14).
19. Notes and threaded comments (VML regeneration à la GridCraft's merge).
20. Sheet autoFilter.
21. Protection (sheet, workbook, cell).
22. sheetPr (tab colour, outline, codeName) and outlines on rows and columns.
23. Rich text and phonetic.
24. Full fills (pattern and gradient), fonts, alignment, indexed palette.
25. Full theme.
26. Views (zoom, split, topLeftCell, RTL).

Charts, drawings, pivots and slicers stay `Replay`/`ReplayIfClean` in M15. Modelling them is its
own future milestone.

**Stage 5: structural edits (with M4).**
27. A reference shifter for modelled range-bound features and for known extension shapes (x14
    DV sqref, sparklines, CF x14).
28. Drawing anchors moved with rows and columns, so drawings can be kept under R2 instead of
    dropped.

The bug fixes from §2.2 are filed separately as M11 tasks, if approved.

---

## 9. Decisions and open questions for the owner

**Decisions:**

**D1.** Create M15 in Phase 2 and move the orphan bullets, as recommended. The alternative keeps
them in M11 and blocks Phase 1 exit.

**D2.** Allow the stage 0 measurement tasks in Phase 1 at P3? Recommended no.

**D3.** Patching tier (layer 5) in scope for M15? Recommended yes: it is what makes "change one
number" produce a one-cell diff.

**D4.** Stale-content policy on structural edits: drop and report (Mog) rather than keep stale
(POI, GridCraft). Recommended drop, because a stale `sqref` silently validates the wrong cells.

**D5.** Keep VBA in .xlsm byte-for-byte. Recommended yes, with the file type preserved.

**D6.** Drop `x15ac:absPath` (it leaks the author's file path) or keep it as Excel does?
Recommended keep (Excel is the spec), with an option to strip it.

**Research questions (probes):**

**RQ1.** What does Excel for the web drop or rewrite on save that desktop keeps (VBA, ActiveX,
printer settings, custom views)? This decides where web goldens can be trusted.

**RQ2.** Which of the normalization list does Excel itself rewrite on every save? Evidence is
needed for each item.

**RQ3.** After we change cell values, does Excel trust our cached values given calcId,
calcFeatures and fullCalcOnLoad? And what does Excel do with stale chart and pivot caches on open?

**RQ4.** Pivot caches with `refreshOnLoad`: is replaying them after a source edit safe?

**RQ5.** Does Excel accept our reprinted number literals (`9.99e307`, `1e-12`) and the reversed
ranges we normalize, in a stored formula?

---

## Appendix: how the measurements were taken

Scripts and method: `docs/research/xlsx-round-trip/README.md`. Raw feature table:
`docs/research/xlsx-round-trip/census-2026-11-04.md`.

Inputs: 2,028 workbooks. 1,999 loaded; the 29 refusals were POI hostile or malformed files plus
`style-alternate-content.xlsx`.

Run: `load_from_xlsx` → (`evaluate`) → `save_to_xlsx`. The cell comparison uses `rustsheet-eval
snapshot` of both files. The IO-only run (`NOEVAL`) separates IO differences from recalculation
differences.

External sources studied:
- github.com/fundamental-research-labs/mog
- github.com/fundamental-research-labs/calipers
- github.com/storytold/gridcraft
- github.com/ironcalc/IronCalc
- github.com/MathNya/umya-spreadsheet
- github.com/psu3d0/formualizer
- github.com/open-xml-templating/xlsx-populate
- github.com/exceljs/exceljs
- the Apache POI, Open XML SDK, openpyxl, SheetJS, EPPlus and docx4j documentation
- LibreOffice InteropGrabBag (FOSDEM 2014)
- ISO/IEC 29500-3 (Markup Compatibility)
