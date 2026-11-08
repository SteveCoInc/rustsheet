# SPEC-0034 — Source-preserving xlsx export

- **Status:** draft (stages 1–3 implemented; stage 5, D5, specified, T-4567)
- **Milestone / lane:** M15 / research (the code lives in crates/rustsheet-xlsx and crates/rustsheet)
- **Tasks:** T-4183 (this spec); the implementation tasks are filed with it (see Rollout)

## Problem
The export rebuilds every part from the engine's model (`export/mod.rs`). Anything the model does not hold is lost,
and much of what it holds is rewritten differently: the theme from a template, docProps, window geometry,
`fileVersion`, page setup, `sheetFormatPr`. The round-trip suite measures it (SPEC-0033, T-4172, T-4509, T-4513):
`roundtrip-fixtures` keeps 82% of its 18,796 fidelity items. The 2026-11-04 census found the same over 2,028 corpus
workbooks: every unmodelled feature lost in 100% of the files that have it.

Modelling every feature takes years and never covers features Excel has not shipped yet. The owner's goal is 99% of
fidelity items kept (2026-11-06). Every project that gets close keeps the source and changes only what changed:
- umya writes untouched sheets back byte for byte;
- Formualizer patches cell values in place;
- POI and EPPlus edit a retained DOM.

This spec makes that the export's default, as decision 0275's D3 chose.

## Goals / non-goals
Goals:
- **No edit, no change.** A workbook loaded and saved with no edit and no changed cached value is written back with
  every part byte-identical to the source.
- **Content edits patch only what they touch.** A cell edit, or a recalculation that changed cached values, rewrites
  only those `<c>` elements, appends new shared strings and styles, and copies every other byte, part and relationship
  unchanged. Unknown parts, unknown elements, `extLst`, `mc:AlternateContent` and features Excel ships later all
  survive.
- **Fallback.** Anything the patcher does not handle (a structural edit, a sheet added or removed, a defined name
  edited) falls back to today's regenerating export. It never writes a half-patched package. The fallback stays
  exactly as lossy as today, and stage 5 shrinks it.
- **Directive 5.** Retention is bounded by the load limits already in force, and the patcher streams with depth and
  size limits.

Non-goals:
- Structural edits that preserve range-bound content (row and column insert or delete). They need M4's reference
  shifting (stage 5c). Stage 5 (D5) keeps everything else a regenerated save can keep.
- Workbooks created from scratch, which have no source and keep the regenerating export.
- Byte-identical zip containers. Entry order, compression level and timestamps may differ: SPEC-0033 D4.6 ignores
  them. Part *contents* are what is preserved.

## Excel behavior to match
- **E1.** Excel's own save rewrites much of a package (SPEC-0033 X1). This export does not try to imitate Excel's
  rewrite: it keeps the source as it was. SPEC-0033 compares against the source, and against Excel's save only through
  the evidence-gated normalization list.
- **E2. calcChain.** Excel rebuilds a missing calcChain on load, and it may repair a stale one. A patched package
  keeps the source's calcChain while the set of formula cells is unchanged. When a formula cell is added or removed,
  the patcher drops calcChain, with its relationship and content-type override. That is UNVERIFIED for repair-free
  opening of a stale chain; T-4180's oracle flow checks it.
- **E3. Cached values and `fullCalcOnLoad`.** A patched package keeps the source's `calcPr`. Whether Excel trusts our
  recalculated cached values is RQ3 (SPEC-0033); until T-4180 answers it, `calcPr` is left as the source had it.

## Design

### D1. Retaining the source, and who does what
- `rustsheet::Workbook::from_xlsx_bytes` (and `open`, which reads into bytes first) keeps the source bytes as an
  `Arc<[u8]>` beside the model. It also keeps the import options it loaded them with: locale, time zone, language
  and `ImportLimits`, the inputs of `rustsheet_xlsx::import::load_from_xlsx_bytes_with_limits`.
- `Workbook::new` has neither.
- A workbook loaded with a password keeps the decrypted package, moved into zeroizing storage that is wiped when
  the workbook is dropped. It is never copied into plain memory, and its `Debug` shows only its length. The save
  writes it unencrypted, as today (SPEC-0031 non-goal): a verbatim save of an encrypted file is its plaintext.

Layering. The facade owns retention and builds the pristine model. `rustsheet-xlsx` owns the diff and the
patcher, and never depends on the facade. The entry point is:

```rust
/// Writes `model`, preserving `source` where SPEC-0034 allows: verbatim, patched or regenerated.
pub fn save_preserving<W: Write + Seek>(
    model: &Model,
    pristine: &Model,
    source: &[u8],
    writer: W,
) -> Result<(W, SaveTier), XlsxError>;
```

- The facade's `to_xlsx_bytes` re-imports `source` with the stored options into `pristine` and calls it.
- With no source it calls `save_xlsx_to_writer` as today.
- `save_xlsx_to_writer` stays the regenerating export, and the Structural tier calls it.

**Memory (directive 5).** The pristine model makes a save's peak about twice the model's size, bounded by the same
`ImportLimits` the load already passed. The retained bytes are at most `max_uncompressed_bytes`. A caller that cannot
afford twice the model should save a workbook created with `Workbook::new`, which has no source, and keep today's
cost.

### D2. What changed: a save-time diff
At save, the export re-imports the retained source into a second `Model` (the pristine model) and compares it with
the live model. This costs one more import per save, which is the price of not threading dirty flags through every
engine mutation. Dirty tracking can replace it later as an optimisation behind the same interface.

The comparison classifies the change:

| Class | Condition | Export |
|---|---|---|
| **Unchanged** | every sheet's cells compare equal: input (formula text or constant), style index and cached value; and the workbook structure compares equal (D2.1) | the source bytes, verbatim |
| **Content** | structure equal; some cells differ | patch (D3) |
| **Structural** | structure differs | regenerate (today's export), with a `SaveReport` entry naming the reason |

**D2.1. Structure** is everything the patcher does not rewrite:
- the sheet list (names, order, state);
- defined names;
- merges, hyperlinks, conditional formats, data tables, tables;
- column and row properties (width, height, hidden, style);
- frozen panes and views;
- the style table *except appended records* (D3.3);
- the theme.

Comparison is of the typed model, which both sides share, so there is one definition of each field (directive 3). An
engine change that alters how a feature is modelled cannot make the diff disagree with itself.

### D3. The patcher
**D3.1. Package.**
- Copy every zip entry of the source unchanged, except the parts D3.2–D3.4 rewrite, with their contents replaced.
- Relationships and `[Content_Types].xml` are copied as they are, except for two edits (T-4561):
  - the calcChain removal of E2;
  - the metadata parts D3.4 adds, each with one workbook relationship and one `<Override>`.

  Both are splices of whole elements found by the parser. A stale override or link for an added part, a child
  written with start and end tags, or a calcChain relationship pointing elsewhere makes the patcher fall back.
- No other part is added.

**D3.2. Worksheets.** A sheet with changed cells is rewritten by a streaming pass over its XML:
- every event is copied through unchanged;
- a `<c>` whose reference is in the sheet's changed set is replaced by the cell the regenerating export would write
  for it (`export::worksheets`' cell writer; one definition);
- a changed cell with no `<c>` is inserted in column order into its `<row>`, and a missing `<row>` in row order;
- a cell that became empty, with the default style, is removed.

Row attributes (`spans`, height, style) are kept. `<dimension>` is grown to cover a new cell outside it, written as
the regenerating export writes it. It is never shrunk (T-4560): a range larger than the cells is what Excel keeps for
formatted rows, and Excel's tolerance of a stale range is UNVERIFIED (docs/excel-semantics/xlsx-storage.md, "A
patched package keeps the source's hints").

The pass is bounded (directive 5):
- `ImportLimits`' byte budget, as the importer's read was;
- element depth 256;
- the source's row and cell order. A sheet whose rows or cells are out of order, or whose `<c>` lacks `r`, makes the
  patcher fall back to regeneration rather than guess.

**D3.3. Index stability.** The patch keeps every existing index:
- **Shared strings:** a changed cell whose text already exists in the source table uses its index. New text is
  appended at the end of `sharedStrings.xml`, and `count` and `uniqueCount` are updated. If the source has no table and
  a new string is needed, the patcher falls back to regeneration, so that it never adds a part.
- **Styles:** the importer is assumed to keep the source's `cellXfs` order, with the model appending new styles at
  the end. This is an assumption: T-4561 verifies it against `import/styles.rs`. The patcher checks it holds, meaning
  the first *n* records of the live style table equal the pristine one's; otherwise it falls back. New records (`xf`, and the `font`, `fill`, `border` and `numFmt` they need) are appended to
  their collections in `styles.xml`, with the counts updated.
- **Inline strings** in the source stay inline when unchanged. A changed string cell is written as a shared string.

**D3.4. Formula cells.**
- A changed cached value rewrites the cell's `<v>` (and `t`), keeping its `<f>` as stored, `t="shared"` and `si`
  included.
- A changed formula is written as the regenerating export writes it.
- A shared-formula group whose master's formula changed, or whose master was cleared or replaced by a constant,
  breaks: every member is written with its own formula. Otherwise the patch would change cells it was not asked to.
- An edited member leaves the group and is written with its own formula. The other members keep `t="shared"`.
- Dynamic-array anchors and their spill ranges, and rich-value errors (`vm`), follow the regenerating export's rules
  for `cm`, `vm` and `metadata.xml` (T-4561). Which rule applies depends on the source's metadata parts:
  - **None** (`xl/metadata.xml` and the two rich-value parts all absent): the package takes the export's layout.
    Patched cells carry the export's `cm`/`vm`, and once one does, the export's metadata parts are added. A
    source cell that already holds a `vm` with nothing to resolve it loads as a plain error. Under the added
    metadata it would name a rich error, so such a source falls back.
  - **Its own:** a value patch keeps a cell's `vm` only for the same rich-error kind, because the two layouts
    number their blocks differently. Any other `cm`/`vm` change falls back.

**D3.5. calcChain** follows E2.

**docProps.** The Unchanged tier keeps docProps byte-identical, `dcterms:modified` included. The Content tier also
keeps them as the source had them, which on purpose departs from today's export: today's export stamps
`dcterms:modified` from the workbook clock (`Workbook::set_clock`, SPEC-0008). Keeping the source's time keeps the
part byte-identical until SPEC-0033 N2's evidence (T-4503) says whether Excel restamps it. If it does, the Content
tier restamps only that element.

### D4. SaveReport
SPEC-0033 D7's `SaveReport` gains the tier:

```rust
/// How a save wrote the package (SPEC-0034).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SaveTier {
    /// Every part is the source's, byte for byte.
    Verbatim,
    /// The source with these sheets' cells patched (and shared strings and styles appended).
    Patched { sheets: Vec<String> },
    /// Rebuilt from the model, for this reason (a structural change, a fallback trigger, no source).
    Regenerated { reason: String },
}
// SaveReport { tier: SaveTier, dropped: Vec<Dropped> }
```

The drops are:
- `Policy` for calcChain;
- `NotPreserved` for each unmodelled source part a regeneration loses.

This replaces SPEC-0033 D7.1's plan of a coarse report in T-4183's first stage: the report ships with T-4562.
Until then the round-trip suite scores every missing item `lost`, which is the same score.

### D5. Stage 5: preserving under regeneration (T-4567)
Stages 1–3 keep the source whenever the patcher can. A save that falls back to regeneration still writes only what the
model holds:
- every structural change (D2.1);
- every fallback reason of D3;
- every workbook created from scratch with a source attached.

That is the "stays exactly as lossy as today" of the goals. Stage 5 shrinks that loss without M4. Range-bound content
under row and column edits waits for M4.

**Shape.** The Regenerated tier with a source becomes "regenerate, then keep":
- the export writes every part it models, as today;
- it then adds back each source part the model does not hold, under the ownership matrix (D5.1), the coupling groups
  and the closure rule (D5.2), written as D5.3 says;
- inside the parts it regenerates, it re-emits the source's unmodelled elements in their schema slots (D5.4);
- the invalidation table (D5.5) says what an edit makes stale, which is dropped and reported;
- every step is bounded (D5.6).

Nothing here touches the engine. Kept parts and fragments are read from the retained source bytes at save time,
beside the pristine re-import of D2. The model and `import/` gain no preservation fields: one definition of "what the
source had" (directive 3), and no memory cost for workbooks that never save.

**Provisional.** Rows marked *(M4)* depend on the structural-edit API M4 will ship:
- row and column insert and delete;
- `RefMap`, Excel-exact reference adjustment, docs/milestones.md M4.

Today those edits exist only behind the `harness_row_insert` feature (decisions 0076, 0098). The rows are written
against the planned API and are re-checked when M4's spec lands. Everything else is implementable now.

**D5.1. Ownership matrix.** One table in `rustsheet-xlsx` (`export/ownership.rs`) maps each relationship type to a
policy. It is keyed by relationship type, then by content type for parts reached without a typed relationship. Part
names vary between writers, so they are never keys. Each row is defined once, and the reader, the writer and the
SaveReport all look it up (directive 3). The relationship-type names below are the last path segment. The
namespace is ECMA-376's, or that of the Microsoft extension that defines the type.

| Policy | Meaning | Relationship types |
|---|---|---|
| `Modelled` | Written from the model; the source part is never copied | `officeDocument`, `worksheet`, `styles`, `sharedStrings`, `theme`, `sheetMetadata`, `rdRichValue`, `rdRichValueStructure`, `rdRichValueTypes`, `table`, `pivotCacheDefinition`, `pivotCacheRecords`, `pivotTable`, `externalLink`, `hyperlink` (External), `core-properties`, `extended-properties` |
| `Replay` | Copied byte for byte while the package or the workbook exists | `customXml` and its `customXmlProps`, `custom-properties`, package `thumbnail`, `connections`, `person`, `webextension`, `webextensiontaskpanes` |
| `ReplayIfClean` | Copied byte for byte unless an edit invalidated what it is anchored to (D5.5); otherwise dropped and reported. Attach-bound rows also need their attach element (D5.3). | Sheet-anchored: `drawing` (and the `chart`, `image`, `chartUserShapes`, chart style and colour parts under it), `vmlDrawing`, `comments`, `threadedComment`, `ctrlProp`, `control` (ActiveX) and its `activeXControlBinary`, `oleObject`, `package` (embeddings), `image` from a sheet (`<picture>` background), `printerSettings`, `slicer`, `timeline`, `queryTable`. Workbook-scoped but bound to sheets or tables: `slicerCache`, `timelineCache`. |
| `Active` | Copied byte for byte only into a macro-enabled file (`.xlsm`, `.xltm`, `.xlam`); never interpreted (decision 0275 D5). Otherwise dropped with `ActiveContentInPlainFormat` | `vbaProject` and its `vbaProjectSignature` |
| `Drop` | Never written; `Policy` | `calcChain`; digital signatures (`digital-signature/origin` and its parts); `revisionHeaders`, `usernames`; the unmodelled members of a `Modelled` family (`richValueRel`, `rdArray`, `rdSupportingPropertyBag`, `rdSupportingPropertyBagStructure`, `rdRichValueWebImage`, `richStyles`): next to a regenerated `rdRichValue` they would be orphans |
| unknown, package- or workbook-reached | Any type not in the table, hanging off the package or `workbook.xml` | `Replay` if the closure rule (D5.2) holds; otherwise `Drop` with `ClosureReachesRegenerated` |
| unknown, sheet-reached | Any type not in the table, hanging off a sheet | `ReplayIfClean`, anchored to that sheet, and dropped on any D5.5 trigger for it. Example: `namedSheetView` (2019), whose filters carry ranges. |

Notes:
- **New Excel features.** The unknown rows are the answer to features Excel has not shipped yet. A new part kind
  survives on day one unless it points into what the export regenerates, or a trigger fires on its sheet.
- **Active content needs more than its part.** `vbaProject` is kept only when all of these hold:
  - the output's main content type is the macro-enabled one (`sheet.macroEnabled.main+xml`, or the template or add-in
    form);
  - the `codeName` attributes of `workbookPr` and of each sheet's `sheetPr` are kept (D5.4).

  The export writes no macro-enabled package today, so T-5010 adds that path.
- **ActiveX and OLE are not `Active`.** Excel keeps ActiveX controls and OLE embeddings in a plain `.xlsx`:
  poi/45540_form_Footer.xlsx holds 40 `control` parts, and poi/WithEmbeded.xlsx holds an `oleObject` and a `package`.
  Only VBA is barred there.
- **Read but not exported.** `comments`, `slicerCache` and `timelineCache` are read by the importer (legacy notes,
  slicer filters for pivots) but not written by the export, so they are `ReplayIfClean`. The model's view of each is
  compared, pristine against live (D2), and a difference is a D5.5 trigger. A row moves to `Modelled` once the export
  writes the part, as T-4950 does for pivot-bound drawings.
- **Connections.** Parts tie to `connections` by `connectionId` attribute, not by relationship: `queryTable`s and
  modelled `pivotCacheDefinition`s. The importer does not follow those ids, so they are not in any closure. The
  connections part is kept verbatim, so its ids keep their values. A modelled pivot cache whose regenerated XML has
  lost its `connectionId` simply no longer names one, and nothing dangles.
- **Chartsheets, dialogsheets and macrosheets** are sheets. Their `<sheet>` entry is in the regenerated `workbook.xml`,
  which the model does not hold, so they stay `NotPreserved` in stage 5: a known gap, reported per part.
- **The theme** is `Modelled`, as the export writes the model's theme. Where the importer reads only part of it, the
  loss belongs to stage 4's theme item, not here.

**D5.2. Coupling groups and the closure rule.**
- **Closure.** The closure of a root part R is every part reachable from R by following `.rels` with `TargetMode`
  internal. External targets are kept as written. They are never followed, and never resolved against the file system.
- **Coupling groups.** Excel ties some sibling parts of a sheet together by shape and id references that no
  relationship records. Each group is kept or dropped as one unit, and so is the union of the groups' closures:
  - **Legacy drawing group:** the `vmlDrawing` of `<legacyDrawing>` (T-4999 takes every `vmlDrawing` of the sheet,
    the header and footer one of `<legacyDrawingHF>` included, rather than read the sheet XML to tell them apart: an
    over-coupling that only drops more together), `comments`, `threadedComment`, every `ctrlProp`
    with the `<controls>` element, the ActiveX `control` parts, the `oleObject`/`package` parts with `<oleObjects>`, and
    the `drawing` holding their `a14:compatExt` shapes. One VML part holds both note shapes and control shapes
    (corpus-bench 17-35). A `<control shapeId>` names a VML `_x0000_s<n>`, and the drawing's `compatExt spid` names the
    same shape. A `threadedComment` needs its legacy `comments` and the VML (calipers officejs/comment_add).
  - **Slicer group:** each `slicer` part, the drawing holding its `sle:slicer` graphic frame, the sheet's
    `x14:slicerList` ext, and its `slicerCache`. The same holds for `timeline`, `x15:timelineRefs` and `timelineCache`.
- **Kept, or dropped.** A root, or a coupling group, is kept only if:
  - no part in its closure is `Modelled` or `Drop`;
  - no chart in its closure holds `c:pivotSource`. A pivot chart is bound to its pivot table by that text, not by a
    relationship (excel-saved t4187-pivot-chart-caches has no chart `.rels`). T-4950 writes those drawings from the
    model instead;
  - it is within the limits of D5.6.

  Otherwise the root, or the group, is dropped with `ClosureReachesRegenerated`, or with the D5.6 reason.
- **Shared parts.** A part reached from more than one kept root is written once. That holds even across sheets, as
  with one image under two sheets' drawings. A part reached from a kept root and from a dropped root, for whatever
  reason (`OwnerDeleted` included), is kept, because something kept still references it.
- **Text references.** A part that a kept root reaches only through text (a chart series formula naming a sheet) is
  not in the closure. Text references are D5.5's concern.

**D5.3. Writing kept parts.**
- **Names.** Kept parts keep their source names, unless a name collides with a part the export writes. The export
  owns `xl/worksheets/sheetN.xml`, `xl/tables/tableN.xml` and the other parts it numbers. A colliding part is renamed
  to the next free name of the same pattern. The `.rels` that target it are rewritten, and its `Override` follows the
  new name. Within a kept closure, `.rels` are otherwise copied byte for byte.
- **Owner relationships.** The owner's `.rels` (`_rels/.rels`, `xl/_rels/workbook.xml.rels`,
  `xl/worksheets/_rels/sheetN.xml.rels`) is never replayed (research §4 lessons). The export writes its own
  relationships, then one per kept root with a fresh id (`rId<n>` after the export's highest). It then remaps every
  reference to the old id inside the kept fragments (D5.4).
- **Content types.** `[Content_Types].xml` is re-derived, as today. Each kept part gets the `Override` the source had
  for it, under its written name, or the source's `Default` for its extension. Nothing is copied from the source file
  wholesale.
- **Attach elements.** An attach-bound root needs an element in a regenerated part that references it:
  - `<drawing r:id>`, `<legacyDrawing r:id>`, `<legacyDrawingHF r:id>`, `<picture r:id>`;
  - `<controls>`, `<oleObjects>`;
  - the `pageSetup r:id` of `printerSettings`;
  - a workbook `<ext>` holding slicer or timeline caches.

  Such a root is kept only if that element is a fragment D5.4 keeps. Otherwise it is dropped with
  `UnmappableRelationship`. Roots no element references, such as `customXml` or `connections`, are attached by their
  relationship alone.

**D5.4. Fragment slots.** The regenerating writers of `workbook.xml`, `sheetN.xml` and `styles.xml` each declare the
child elements, `ext` URIs and attributes they write. That is one owned set per part, kept next to its writer.
Everything else the source part holds there is a fragment. Fragments are read from the source part at save time and
re-emitted.

- **Positions.** A fragment's slot is its element's place in the schema sequence:
  - `CT_Worksheet` (ECMA-376 Part 1 §18.3.1.99): sheetPr, dimension, sheetViews, sheetFormatPr, cols, sheetData,
    sheetCalcPr, sheetProtection, protectedRanges, scenarios, autoFilter, sortState, dataConsolidate,
    customSheetViews, mergeCells, phoneticPr, conditionalFormatting, dataValidations, hyperlinks, printOptions,
    pageMargins, pageSetup, headerFooter, rowBreaks, colBreaks, customProperties, cellWatches, ignoredErrors,
    smartTags, drawing, legacyDrawing, legacyDrawingHF, drawingHF, picture, oleObjects, controls, webPublishItems,
    tableParts, extLst.
  - `CT_Workbook` (§18.2.27): fileVersion, fileSharing, workbookPr, workbookProtection, bookViews, sheets,
    functionGroups, externalReferences, definedNames, calcPr, oleSize, customWorkbookViews, pivotCaches, smartTagPr,
    smartTagTypes, webPublishing, fileRecoveryPr, webPublishObjects, extLst.
  - `CT_Stylesheet` (§18.8.39): numFmts, fonts, fills, borders, cellStyleXfs, cellXfs, cellStyles, dxfs, tableStyles,
    colors, extLst.
  - An `mc:AlternateContent` block takes the slot of the element its `mc:Choice` wraps. Excel wraps `controls` and
    `oleObjects` this way, and puts `x15ac:absPath` between `workbookPr` and `bookViews`.
  - An element with no slot, such as `xr:revisionPtr` or one in an unknown namespace, goes right after its nearest
    preceding known sibling in the source.

  An element the writer owns is never a fragment, even when the writer writes it differently. Partly modelled elements
  (`sheetViews`, `cols`, `calcPr`) are stage 4's to model, not stage 5's to splice.
- **Owned elements with unknown content.** Inside elements the writer owns, only two things are fragments:
  - unknown attributes, on the root and on a closed list of owned elements declared next to the writer. The list
    starts with `workbookPr` (its `codeName` is needed by kept VBA), `sheetView`, `col` and `calcPr`;
  - `<ext>` entries whose `uri` the writer does not write. The export writes the x14 conditional-format and data-bar
    URIs, for example.
- **Namespaces.** Each fragment carries the namespace declarations it uses: the prefixes of its element and attribute
  names, and also those named in `mc:Requires`, `mc:Ignorable`, `mc:ProcessContent` and `mc:MustUnderstand` values.
  `Requires="x14"` relies on the root's `xmlns:x14`. The writer rebuilds the root's `mc:Ignorable` from the prefixes it
  emits.
- **`mc:AlternateContent`.** It is kept whole or dropped whole, never half-read. Its relationship ids, in the
  `mc:Choice` and the `mc:Fallback` alike, are rewritten through D5.3's remap. If any of them cannot be mapped, the
  whole block drops with `UnmappableRelationship`.
- **`x15ac:absPath`.** The source's value is kept (decision 0275 D6, which reverses the research doc §5's "dropped
  deliberately"). `SaveOptions::strip_abs_path`, false by default, drops it. What Excel itself writes on re-save is
  probed (D5.8).
- **Classification.** One classifier serves all three parts, and a fragment may fall into several classes:
  - **Relationship-bound.** Holds an `r:id`, `r:embed`, `r:link` or `r:pict`. It is remapped (D5.3), or dropped with
    `UnmappableRelationship`.
  - **Range-bound.** Holds an A1 range or formula text. Known elements: `dataValidations`, `autoFilter`, `sortState`,
    `protectedRanges`, `scenarios`, `customSheetViews`, `rowBreaks`, `colBreaks`, `ignoredErrors`, `cellWatches`,
    `smartTags` (`cellSmartTag r`), `webPublishItems` (`sourceRef`), `dataConsolidate` (`dataRef ref`), the `controlPr`
    and `objectPr` anchors, and ext entries holding `xm:sqref` or `xm:f` (x14 data validation, sparklines). Kept
    `ctrlProp` parts are range-bound too (`fmlaLink`, `fmlaRange`, `linkedCell`, `listFillRange`). For unknown elements
    and ext entries, range-binding is assumed when any attribute or text matches the A1 or R1C1 reference grammar.
  - **Sheet-name-bound.** Holds text that names a sheet: formula text, `dataRef sheet`, or `fmlaLink`.
  - **Id-bound.** Holds an id or index into something the export numbers:
    - a style index (`dxfId`, `s`), against the styles section it indexes;
    - a table `id` (`x15:tableSlicerCache tableId`; the export numbers tables from 1, `export/tables.rs`);
    - a `pivotCacheId`;
    - a sheet's `tabId` (its `sheetId`) or `localSheetId` (its position), which are compared separately;
    - a `shapeId`;
    - an external-link index `[n]` in kept formula text;
    - a custom-view `guid` that pairs `customSheetViews` with `customWorkbookViews`.

    Such a fragment is kept only while the export writes the same id for the same target. For styles, that means
    D3.3's prefix test. For tables, pivot caches, sheets and external links, the export's numbering is compared with
    the source's. The export writes the model's `sheetId`, which the importer reads from the source. Otherwise the
    fragment is dropped with `StaleAfterEdit` (D5.5). Remapping ids instead of dropping is left to later work.
  - **Inert.** None of the above.

**D5.5. Invalidation.** The triggers form a closed set, each detected from the pristine-against-live diff (D2):
- `CellContent(S)`;
- `FeatureEdit(S, range)` (merges, conditional formats, tables, hyperlinks, column or row properties);
- `NotesEdit(S)`;
- `PivotOrTableEdit(id)`;
- `StyleTableMoved(section)`;
- `SheetRenamed(S)`, `SheetDeleted(S)`, `SheetAdded`, `SheetMoved(S)`;
- `DefinedNameEdit`;
- *(M4)* `RowsOrColumns(S)`.

The set is defined once, next to the matrix. Each drop carries a SPEC-0033 D7 reason with a `sheet` (none for
workbook-wide content):

| Trigger | Dropped | Reason |
|---|---|---|
| `CellContent(S)` | nothing. Cached values in kept parts (chart caches, external-link caches) stay as the source had them (SPEC-0033 RQ3) | — |
| `FeatureEdit(S, range)` | range-bound fragments of S overlapping the range | `StaleAfterEdit { sheet: S }` |
| `NotesEdit(S)` (the model's notes on S differ from the pristine ones) | S's legacy drawing group (D5.2), whole | `StaleAfterEdit { sheet: S }` |
| `PivotOrTableEdit(id)` (a modelled pivot or table that a slicer, timeline or query table is bound to) | that slicer or timeline group, or that query table | `StaleAfterEdit { sheet: the slicer's sheet }` |
| `StyleTableMoved(section)` (the live table is not a prefix-stable extension, D3.3) | id-bound fragments indexing that section | `StaleAfterEdit { sheet: none }` |
| `SheetRenamed(S)` | sheet-name-bound fragments and kept parts, workbook-wide, whose text names S | `StaleAfterStructuralEdit { sheet: S }` |
| `SheetDeleted(S)` | every fragment of S; every root and coupling group attached to S, with its closure, except parts something kept still references (D5.2); everything else whose text names S; id-bound fragments that hold S's `localSheetId` or `tabId` | `OwnerDeleted` |
| `SheetAdded`, `SheetMoved(S)` | content addressing sheets by name or relationship is unaffected. Id-bound fragments holding a sheet position (`localSheetId`, `activeTab`) are dropped when the position changed | `StaleAfterStructuralEdit { sheet: S }` (the added or moved sheet) |
| `DefinedNameEdit` | a slicer or timeline group whose cache `name` no longer has its matching workbook defined name (corpus-bench 118-8 holds `<definedName name="Slicer_Customer">#N/A</definedName>` per cache). Otherwise nothing: names are modelled, and fragments that name them are text that resolves at load | `StaleAfterEdit { sheet: the slicer's sheet }` |
| *(M4)* `RowsOrColumns(S)` | range-bound fragments of S; every `ReplayIfClean` root and coupling group anchored to S, until stage 5c's shifter rewrites them | `StaleAfterStructuralEdit { sheet: S }` |

- **Never kept stale** (decision 0275 D4). A kept item still in the output after its trigger fired is a SaveReport bug
  (SPEC-0033 D7.3).
- **`StaleAfterEdit`** is new in SPEC-0033 D7 (this task). It scores `dropped` when the run's edits explain it: R1's
  `Δ(E)` or an R2 edit. `StaleAfterStructuralEdit` stays R2's.
- **Sheet-name matching** reuses the engine's sheet-name quoting (`quote_name`, and `unquote_sheet_prefix`, made
  public for it). It is not a second definition.
- **Kept `Replay` parts are not scanned** for sheet names. They are package- and workbook-scoped by construction, so
  no sheet edit invalidates them. `customXml` naming a sheet stays as written, as Excel does.

**D5.6. Limits (directive 5).** Each limit drops what it covers, never fails the save, and never writes a half-kept
closure or group. Roots and groups are added in source part-name order, and a limit drops every remaining one from the
point it is hit. That makes the result deterministic.

| Limit | Value | Over it |
|---|---|---|
| Output bytes (regenerated parts, kept parts and fragments) | `max_uncompressed_bytes` | `LimitExceeded { limit: "output bytes" }` |
| Output parts | `max_zip_entries` | `LimitExceeded { limit: "zip entries" }` |
| Closure depth | 16 hops (GridCraft's value) | `LimitExceeded { limit: "closure depth" }` |
| Closure size | 1,000 parts per root or group | `LimitExceeded { limit: "closure size" }` |
| Fragment element depth | 256 (`MAX_DEPTH`, the patcher's) | `LimitExceeded { limit: "fragment depth" }` |
| Fragments per part | 10,000 | `LimitExceeded { limit: "fragments" }` |
| Emitted fragment bytes per part, namespace declarations included | twice the source part's size | `LimitExceeded { limit: "fragment bytes" }` |
| Text scans (sheet-name and range matching in D5.5) | one linear pass per trigger over each kept part or fragment, at most 16 MiB per part. A larger one is treated as matching. | dropped with the trigger's reason |

The fragment reader is the patcher's bounded quick-xml scan: no DOM, and no recursion beyond the depth limit.

Implementation notes (T-4999):
- The decisions read the source and the saved package within the load's `ImportLimits`. Each read of a package (its
  relationships, then the probes of its parts) has its own budget, so a save reads at most a small constant number of
  times `max_uncompressed_bytes`.
- Each part is probed once, at most 1 MiB of it.
- Each root's closure is computed once. All units together visit at most 10^6 closure parts, past which every remaining
  unit is `LimitExceeded { limit: "closure work" }`.
- A part a probe cannot read makes its unit `LimitExceeded { limit: "part read" }`.
- The regenerated tier is written to memory before it is copied to the caller's writer, so its own inventory can be
  read. That is a peak of about twice the output, which the model it is written from already bounds.

**D5.7. Reporting and measurement.**
- **Report.** Every drop is a SaveReport entry (SPEC-0033 D7, T-4562), so stage 5 needs T-4562 first. Entries are
  keyed as SPEC-0033 D2 keys items: a part by its part key, a fragment by its element key.
- **R2 scenarios.** These are what measure the stage. The sheet-level ones (rename, add, delete, move) need no M4 and
  are scored from T-5005 (SPEC-0033 D5, "R2's sheet edits"). Rows and columns follow M4.
- **Probes.** Each implementation task carries an Excel for the web re-save probe (T-4890's tooling). It shows the
  regenerated-and-kept saves open and keep their parts and fragments.
- **Desktop Excel.** Excel for the web cannot vouch for VBA, ActiveX or printer-settings retention (SPEC-0033 RQ1).
  Those also need a desktop Excel check (M14) before their semantics entries are VERIFIED.

**D5.8. Excel behaviour to verify.** Each item stays UNVERIFIED until its probe runs, and each finding goes into
docs/excel-semantics/xlsx-storage.md:
- **Parts and names:**
  - kept parts under renamed names;
  - an orphan `printerSettings` with no `pageSetup r:id`.
- **Placement:**
  - a rebuilt `mc:Ignorable`;
  - fragments emitted next to regenerated siblings;
  - an `mc:AlternateContent` block or an element with no schema slot placed by D5.4's rules.
- **Groups and ids:**
  - a legacy drawing group kept whole, against one with a member missing;
  - a `threadedComment` without its legacy comments, which must never be written, as the probe confirms;
  - a slicer cache whose `tableId` or `pivotCacheId` no longer matches;
  - a kept slicer cache whose defined name was removed.
- **Content types:**
  - ActiveX and OLE kept in a `.xlsx`;
  - VBA kept with and without its `codeName` attributes.
- **absPath.** What Excel writes for `x15ac:absPath` on its own re-save, on the web and on desktop.

## Alternatives considered
- **Dirty flags through the engine.** These are cheaper at save, but every mutation path (edits, recalculation,
  spills, undo) must set them correctly, and one missed path silently writes stale bytes. The save-time diff can't
  miss a change. Dirty flags can replace it once the diff exists to test them against.
- **DOM retention (POI, EPPlus).** Rejected in decision 0275: memory, and a typed model fighting a DOM.
- **Regenerate plus preserved fragments only (GridCraft, Mog).** These still rewrite every byte of every regenerated
  part, so any attribute nobody thought to capture is lost. They are the stage 5 answer for structural edits, not the
  default.

## Test plan
- **Round-trip suites (SPEC-0033).**
  - R0 on `roundtrip-fixtures` and the corpus suites: every case whose recalculation reproduces its cached values is
    `Verbatim`, and every part is kept.
  - The kept-item rate is the M15 metric. Target ≥ 99% (owner goal).
- **R1 (T-4514).** The expected-delta rule holds for the standard edits, with the patched tier.
- **Unit tests.**
  - Each fallback trigger (out-of-order rows, missing `r`, a non-prefix style table, a structural edit) regenerates.
  - Shared-formula breaking, inline strings, a new string with no table, and a new style appended.
  - A `<c>` removed by clearing a cell.
- **Hostile inputs.** Depth, size, and duplicate cell references in a row make the patcher fall back, never panic.

## Rollout
1. **Retain the source and add the verbatim tier** (D1, D2 Unchanged/Content/Structural classification, `Verbatim`
   only; everything else regenerates). The fixtures suite should jump for every case whose recalculation reproduces its
   cached values.
2. **Sheet patcher for changed cached values and inputs** (D3.1, D3.2, D3.4), with the shared-strings append (D3.3).
3. **The styles append** (D3.3) and the `metadata.xml` and calcChain rules (D3.4, D3.5, E2).
4. **SaveReport tiers** (D4), wired into the round-trip `dropped` verdict.
5. **Stage 5** (D5). T-4567 specified it and filed its tasks, listed in the T-4567 task file's Outcome. Stage 5a
   (everything but row and column edits) needs no M4. Stage 5b is the row and column rows of D5.5, after M4. Stage 5c
   is the shifter that keeps range-bound content under those edits instead of dropping it (research §8 items 28 and
   29).
