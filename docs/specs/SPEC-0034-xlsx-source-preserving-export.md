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
- it then adds back each source part the model does not hold, under the ownership matrix (D5.1) and the closure rule
  (D5.2), written as D5.3 says;
- inside the parts it regenerates, it re-emits the source's unmodelled elements in their schema slots (D5.4);
- the invalidation table (D5.5) says what an edit makes stale, which is dropped and reported;
- every step is bounded (D5.6).

Nothing here touches the engine. Fragments and kept parts are read from the retained source bytes at save time,
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
SaveReport all look it up (directive 3).

| Policy | Meaning | Relationship types (last path segment; the namespace is ECMA-376's or that of the Microsoft extension that defines it) |
|---|---|---|
| `Modelled` | Written from the model; the source part is never copied | `officeDocument`, `worksheet`, `styles`, `sharedStrings`, `theme`, `sheetMetadata`, the rich-value parts (`rdRichValue`, `rdRichValueStructure`, `rdRichValueTypes`), `table`, `pivotCacheDefinition`, `pivotCacheRecords`, `pivotTable`, `externalLink`, `hyperlink` (External), `core-properties`, `extended-properties` |
| `Replay` | Copied byte for byte while its owner (the package, the workbook or a sheet) exists | `customXml` and its `customXmlProps`, `custom-properties`, package `thumbnail`, `connections`, `printerSettings`, `person` (threaded-comment people), web extensions (`webextension`, `webextensiontaskpanes`) |
| `ReplayIfClean` | Copied byte for byte only if no edit invalidated what it is anchored to (D5.5); otherwise dropped and reported | sheet-anchored: `drawing` (with its `chart`, `image`, `chartUserShapes`, chart style and colour parts), `vmlDrawing`, `comments`, `threadedComment`, `ctrlProp`, `slicer`, `timeline`, `queryTable`; workbook-scoped but range-bound: `slicerCache`, `timelineCache` |
| `Active` | Copied byte for byte into `.xlsm`/`.xltm`/`.xlam` only, never interpreted (decision 0275 D5); otherwise `ActiveContentInPlainFormat` | `vbaProject`; ActiveX `control` and its `activeXControlBinary`; `oleObject` and `package` embeddings |
| `Drop` | Never written; `Policy` | `calcChain`; digital signatures (`digital-signature/origin` and its signature parts); `revisionHeaders` and `usernames` |
| unknown | Any relationship type not in the table | `Replay` if the closure rule (D5.2) holds for it; otherwise `Drop` with `ClosureReachesRegenerated` |

Notes:
- The unknown row is the answer to features Excel has not shipped yet. A new part kind that hangs off the package, the
  workbook or a sheet, and points into nothing the export regenerates, survives on day one.
- `comments`, `slicerCache` and `timelineCache` are read by the importer (legacy notes; slicer filters for pivots), but
  the export does not write them, so they are `ReplayIfClean`. The model's view of each is compared, pristine against
  live (D2), and a difference drops the part (`StaleAfterStructuralEdit` on its sheet, or `NotPreserved` until it is
  written from the model). That rule moves a row to `Modelled` as soon as the export writes the part; T-4950 does this
  for pivot-bound drawings.
- Chartsheets, dialogsheets and macrosheets are sheets: their `<sheet>` entry is regenerated `workbook.xml`, which
  the model does not hold. They stay `NotPreserved` in stage 5, a known gap reported per part.
- The theme is `Modelled`, as the export writes the model's theme. Where the importer reads only part of it, the
  loss belongs to the theme row of stage 4, not here.

**D5.2. Closure rule.** The closure of a root part R is every part reachable from R by following `.rels` with
`TargetMode` internal. External targets are kept as written. They are never followed, and never resolved against the
file system. A root is kept only if:
- no part in its closure is `Modelled`, `Drop`, or the root of another sheet's parts;
- the closure is within the limits of D5.6.

Otherwise the root and its whole closure are dropped with `ClosureReachesRegenerated` (or `LimitExceeded`). A part
reached from two kept roots (one image under two drawings) is written once. A part reached from a kept root and a
dropped root is kept, since it is still referenced. Consequences:
- A chart whose rels point only at its own style, colour and embedding parts is kept with its drawing.
- A drawing whose closure reaches a `pivotTable` is dropped until T-4950 writes it from the model.
- A part a kept root reaches only through text (a chart series formula naming a sheet) is not in the closure. Text
  references are D5.5's concern.

**D5.3. Writing kept parts.**
- Kept parts keep their source names, unless a name collides with a part the export writes; the export owns
  `xl/worksheets/sheetN.xml`, `xl/tables/tableN.xml` and the others it numbers. A colliding part is renamed to the next
  free name of the same pattern, and the `.rels` that target it are rewritten. Within a kept part's closure, `.rels`
  are copied byte for byte, except for targets renamed this way.
- The owner's `.rels` (`_rels/.rels`, `xl/_rels/workbook.xml.rels`, `xl/worksheets/_rels/sheetN.xml.rels`) is never
  replayed (research §4 lessons). The export writes its own relationships, then one per kept root with a fresh id
  (`rId<n>` after the export's highest), and remaps the root's attach point to it (D5.4).
- `[Content_Types].xml` is re-derived, as today. Each kept part gets the `Override` the source had for it, or the
  source's `Default` for its extension. Nothing is copied from the source file wholesale.
- A kept root that needs an attach element in a regenerated part (`<drawing r:id>`, `<legacyDrawing r:id>`,
  `<controls>`, the `pageSetup` `r:id` of `printerSettings`, a workbook `<ext>` holding slicer caches) is kept only if
  that element is a fragment D5.4 keeps. Otherwise the root is dropped with `UnmappableRelationship`. Kept parts that no
  element references, such as `customXml`, are attached by their relationship alone.

**D5.4. Fragment slots.** The regenerating writers of `workbook.xml`, `sheetN.xml` and `styles.xml` each declare the
child elements, `ext` URIs and attributes they write: one owned set per part, next to the writer. Everything else
the source part holds there is a fragment. It is read from the source part at save time and re-emitted at its
schema position.

- **Positions.** A fragment's slot is its element's place in the schema sequence:
  - `CT_Worksheet` (ECMA-376 Part 1 §18.3.1.99): sheetPr, dimension, sheetViews, sheetFormatPr, cols, sheetData,
    sheetCalcPr, sheetProtection, protectedRanges, scenarios, autoFilter, sortState, dataConsolidate,
    customSheetViews, mergeCells, phoneticPr, conditionalFormatting, dataValidations, hyperlinks, printOptions,
    pageMargins, pageSetup, headerFooter, rowBreaks, colBreaks, customProperties, cellWatches, ignoredErrors,
    smartTags, drawing, legacyDrawing, legacyDrawingHF, drawingHF, picture, oleObjects, controls, webPublishItems,
    tableParts, extLst;
  - `CT_Workbook` (§18.2.27) and `CT_Stylesheet` (§18.8.39), likewise.

  An element the writer owns is never a fragment, even when the writer writes it differently. Partly modelled elements
  (`sheetViews`, `cols`, `calcPr`) are stage 4's to model, not stage 5's to splice.
- **Owned elements with unknown content.** Inside elements the writer owns, only these are fragments:
  - unknown attributes on the root and on a closed list of owned elements, declared next to the writer (`sheetView`,
    `col`, `calcPr` to start);
  - `<ext>` entries whose `uri` the writer does not write. The export writes the x14 conditional-format and data-bar
    URIs, for example.

  Each fragment carries the namespace declarations it uses. The writer rebuilds `mc:Ignorable` from the prefixes it
  emits.
- **`mc:AlternateContent`.** It is a fragment like any other, kept whole or dropped whole, never half-read.
- **`x15ac:absPath`.** It is kept as Excel keeps it (decision 0275 D6). `SaveOptions::strip_abs_path`, false by
  default, drops it.
- **Classification.** Each fragment is classified when it is read, with one classifier for all three parts:
  - **relationship-bound**: holds an `r:id` (or `r:embed`, `r:link`, `r:pict`). It is remapped through D5.3, or
    dropped with `UnmappableRelationship`.
  - **range-bound**: holds an A1 range or formula text. Known elements are listed: `dataValidations`, `autoFilter`,
    `sortState`, `protectedRanges`, `scenarios`, `customSheetViews`, `rowBreaks`, `colBreaks`, `ignoredErrors`,
    `cellWatches`, `dataConsolidate`, and ext entries holding `xm:sqref` or `xm:f` (x14 data validation, sparklines).
    For unknown elements and ext entries, it is assumed when any attribute or text matches the A1 or R1C1 reference
    grammar.
  - **sheet-name-bound**: holds formula text that names a sheet (a `xm:f`, a `formula1`).
  - **index-bound**: holds a style index (`dxfId`, `s`). It is kept only while the style section it indexes is
    prefix-stable against the pristine table (D3.3).
  - **inert**: none of the above.

**D5.5. Invalidation table.** Triggers come from the pristine-against-live diff (D2), a closed set defined next to
the matrix. Each drop carries SPEC-0033 D7's reason:

| Change | Dropped | Reason |
|---|---|---|
| Cell content (R1) | nothing. Cached values in kept parts (chart caches, external-link caches) stay as the source had them (SPEC-0033 RQ3) | — |
| A modelled feature of sheet S edited (merges, conditional formats, tables, hyperlinks, column or row properties) | range-bound and index-bound fragments of S that overlap the edited range; `ReplayIfClean` parts anchored to an edited table (slicers, query tables) | `StaleAfterStructuralEdit { sheet: S }` |
| Sheet S renamed | sheet-name-bound fragments and `ReplayIfClean` parts, workbook-wide, whose text names S | `StaleAfterStructuralEdit { sheet: S }` |
| Sheet S deleted | every fragment of S; every kept root attached to S, with its closure (D5.2); everything else whose text names S | `OwnerDeleted` |
| Sheet added or moved | nothing. Kept content addresses sheets by name or relationship, not by position. A fragment that holds a sheet index (`localSheetId`, `activeTab`) is index-bound and dropped. | `StaleAfterStructuralEdit` |
| Defined name added, edited or removed | nothing: names are modelled, and fragments that name them are text | — |
| Style table not prefix-stable | index-bound fragments of the section that moved | `StaleAfterStructuralEdit` |
| *(M4)* Rows or columns inserted or deleted on S | range-bound fragments of S; `ReplayIfClean` parts anchored to S (drawings, notes, controls, slicers), until the shifter of stage 5c rewrites them | `StaleAfterStructuralEdit { sheet: S }` |

Never kept stale (decision 0275 D4). A kept item still in the output after its trigger fired is a SaveReport bug
(SPEC-0033 D7.3).

**D5.6. Limits (directive 5).**

| Limit | Value | Over it |
|---|---|---|
| Kept bytes | within the source's `max_uncompressed_bytes`, which the load already charged | — |
| Kept parts | the export's parts plus the kept ones ≤ `max_zip_entries` | the remaining roots are dropped |
| Closure depth | 16 hops (GridCraft's value) | `LimitExceeded { limit: "closure depth" }` |
| Closure size | 1,000 parts per root | `LimitExceeded { limit: "closure size" }` |
| Fragment element depth | 256 (`MAX_DEPTH`, the patcher's) | `LimitExceeded { limit: "fragment depth" }` |
| Fragments per part | 10,000 | `LimitExceeded { limit: "fragments" }` |
| Fragment bytes per part | the part's own size (a fragment is a slice of it) | — |

A limit drops what it covers. It never fails the save and never writes a half-kept closure. The fragment reader is the
patcher's bounded quick-xml scan: no DOM, and no recursion beyond the depth limit.

**D5.7. Reporting and measurement.**
- Every drop is a SaveReport entry (SPEC-0033 D7, T-4562), so stage 5 needs T-4562 first. Entries are keyed as
  SPEC-0033 D2 keys items: a part by its part key, a fragment by its element key.
- The R2 scenarios of SPEC-0033 are what measure this stage. The sheet-level ones (rename, add, delete, move) need no
  M4 and are filed with it. Rows and columns follow M4.
- Each implementation task carries an Excel re-save probe (T-4890's tooling): Excel for the web opens the
  regenerated-and-kept saves, and the saves keep the parts and fragments.

**D5.8. Excel behaviour to verify.** All of this is UNVERIFIED until the probes run:
- Excel opens a package whose kept parts sit under renamed names;
- it accepts the rebuilt `mc:Ignorable`;
- it accepts fragments emitted in schema order next to regenerated siblings;
- it keeps an `x15ac:absPath` it did not write.

Each finding goes into docs/excel-semantics/xlsx-storage.md.

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
