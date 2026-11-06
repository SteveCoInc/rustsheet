# SPEC-0034 — Source-preserving xlsx export

- **Status:** draft (stages 1–3 ready to implement)
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
- Structural edits that preserve (row and column insert or delete, sheet moves). They need M4's reference shifting and
  the ownership matrix of stage 5.
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
- Relationships and `[Content_Types].xml` are copied as they are, except for the calcChain removal of E2.
- No part is added, so no relationship is added.

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
- Dynamic-array anchors and their spill ranges follow the regenerating export's rules for `cm` and `metadata.xml`.
  If the source had no `metadata.xml` and one is needed, the patcher falls back.

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

### D5. Stage 5: preserving under regeneration
For structural edits, the ownership matrix, closure rule, fragment slots, invalidation table and preservation limits
of decision 0275 apply. T-4183 was filed to specify them. This spec chose instead to make preservation the default
through the source and the patcher, which needs none of them, and moved them to stage 5. Its spec is task
T-4567, which depends on M4's reference shifting for range-bound content. SPEC-0033 D7's
`ClosureReachesRegenerated` and the `Drop` policy are defined there.

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
5. **Stage 5** (D5), after M4: T-4567 specifies it, then its own tasks.
