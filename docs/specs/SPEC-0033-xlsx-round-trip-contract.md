# SPEC-0033 — The xlsx round-trip contract

- **Status:** draft (ready to implement in T-4172; the normalization list waits on its probe)
- **Milestone / lane:** M15 / research (the harness lives in crates/rustsheet-eval)
- **Tasks:** T-4167 (this spec), T-4172 (suite kind, differ, validator), T-4173 (corpus suites), T-4179 (XSD),
  T-4180 (Excel goldens), T-4183 (SPEC-0034, the source-preserving export; T-4562 fills `SaveReport`), T-4166 (RQ5 probe),
  RQ2 probe (filed with this spec)

## Problem
RustSheet's only round-trip check is `compare.rs::test_load_and_saving`. It loads, saves, reloads and compares cell
values, on 248 local workbooks. It does not look at any part, element or attribute.

The 2026-11-04 census (docs/research/xlsx-round-trip-2026-11-04.md §2) loaded and re-saved 2,028 corpus workbooks.
Every feature the model does not hold was lost in 100% of the files that had it: data validation in 130 files,
page setup in 755, comments in 106, charts in 72, VBA in 9, every unknown `extLst`. Cell content changed in 0.26% of
cells, which is several bugs (T-4140 to T-4157).

Nothing measures any of this, so nothing stops it from getting worse, and M15 has no definition of "kept". This spec
defines what a round trip must preserve, how a pair of files is compared, and how a suite scores and ratchets the
result. Every later M15 task builds on that one definition (CLAUDE.md directive 3).

## Goals / non-goals
Goals:
- **Scenarios.** Name the operations a round trip covers: R0 (no-op), R1 (content edits, with a fixed standard edit
  set) and R2 (structural edits).
- **Equivalence.** Define two levels of equivalence between two packages. Semantic (S) is the gate. Package (P) is a
  structural XML diff and is the diagnostic.
- **Fidelity items.** Define the unit a suite scores, its identity key, and the verdicts it can get.
- **Expected-delta rule.** Define the rule that makes "only the change I made" checkable offline, with no Excel.
- **Package validator.** Define its checks.
- **SaveReport.** Define the API that tells a caller what a save dropped and why.
- **Excel as arbiter.** Say how Excel goldens enter, and how a normalization earns its place: only with evidence
  that Excel itself rewrites the thing on every save.

Non-goals:
- How preservation works (ownership matrix, closure rule, fragments, patching). That is SPEC-0034 (T-4183), with stage 5 in T-4567. This spec only
  says how to tell whether preservation worked.
- XSD validation rules (T-4179) and the oracle round-trip flow (T-4180). They plug into the verdicts defined here.
- R2's row and column edits. R2 is defined here so its items have keys. Its sheet-level edits (rename, add, delete,
  move) are scored from T-5005; row and column edits wait for M4's structural-edit API.
- `.xlsb`, `.xls` and Strict OOXML packages. Strict is listed as a known gap; a Strict input is scored only for
  crashes.

## Excel behavior to match
Excel is the spec here as everywhere (directive 1). The question is never "are the bytes equal". It is "does our file
differ from what Excel would produce for the same operation, in a way a user or Excel could notice".

- **X1. Excel rewrites much of a package on every save.** Evidence:
  - The excel-graph oracle's downloads rewrite `calcId` to 191028 and docProps/app to "Microsoft Excel Online"
    (docs/architecture/oracle.md, "Proof that it is Excel calculating").
  - Calipers' desktop goldens rewrite calcChain, printer settings and docProps timestamps
    (`internal/compare/compare.go` normalizations, read 2026-11-04).

  Which items Excel rewrites on **every** save is RQ2. The rewrites Excel makes only sometimes are not
  normalizations: they are what the goldens of X3 measure.
- **X2. Excel trims, renumbers and reorders where the schema lets it.** Its own saves renumber relationship ids,
  shared-string indexes and style indexes, and reorder unordered collections (the census input files show this
  across Excel builds).
- **X3. Excel's own load and save is the golden where we have it.**
  - Calipers' `roundtrip/` cases have desktop-Excel re-saves (`golden.xlsx` of `init.xlsx`).
  - T-4180 adds excel-graph R0 and R1 goldens.
  - What Excel for the web drops that desktop Excel keeps is RQ1 (T-4180).
- **X4. Formula spelling.** Whether Excel reads the number and range spellings our export writes the same as its own
  is RQ5 (T-4166). Until then, a changed spelling counts as a `changed` item, not a normalization.
- **X5. Leading whitespace of a stored formula.** Excel drops it on import (docs/excel-semantics/syntax.md, T-2784,
  T-2949). It is therefore not an S difference. It stays visible in P.

Findings of RQ1, RQ2 and RQ5 go into docs/excel-semantics/xlsx-storage.md, section "Round trips", which this spec's
first implementation task creates.

## Design

### D1. Scenarios
| Id | Operation | Scored from |
|---|---|---|
| R0 | load, recalculate, save | T-4172 |
| R1/En | load, recalculate, apply edit En of the standard set, recalculate, save | T-4172 |
| R1/all | load, recalculate, apply E1–E6 together, recalculate, save | T-4172 |
| R2/rename, R2/add, R2/delete, R2/move | load, recalculate, rename, add, delete or move a sheet, save (D5, "R2's sheet edits") | T-5005 |
| R2/… (rows, columns) | load, recalculate, insert or delete rows or columns, save | M4 |

R0 recalculates as R1 does, so cached values that differ only because of engine parity cancel out in the
expected-delta rule (D5). They are scored by the function suites, not here. A run uses the harness's fixed clock, so
docProps and `NOW()` are reproducible.

**The standard edit set.** Each edit acts on the first visible worksheet. Cells are visited in row-major order inside
the sheet's stored dimension. An edit with no target (for example, no numeric constant) is skipped and reported as
`n/a` for that case.

As implemented (T-4514):
- The edits act on the first sheet in tab order, because the facade does not expose visibility yet.
- The dimension is the model's used range.
- Each edit is its own transaction, and `R1/all` applies them one after another.
- E1 and E2 skip spill and array members, which hold an anchor's result and no input of their own. A write the
  engine refuses because of its target moves E1 and E2 on to their next candidate (64 at most). Any other edit
  with no valid target is `n/a`. The CLI line counts refused writes and `n/a` edits.
- `Δ(E)` keeps two cell sets. Written cells explain their input, formula, computed flag and value, and their style
  only under E6. Value-changed cells explain their value only. Appended style records and the style counts are
  explained only under E6, and the shared-string counts only under E5. `dimension@ref` is explained only on the
  edited sheet.
- An R0 item excuses the same R1 item only up to its weight, so a larger `<truncated>` tail is still a violation.
  Validator items also match on their detail.
- A scenario that fails to run is one `scenario error` item. The case's R0 items still count.
- `Δ(E)` is the union of the edits' change sets: the cells they wrote and the cells whose values changed.

| Id | Edit |
|---|---|
| E1 | The first numeric constant cell gets its value plus 1 |
| E2 | The first formula cell becomes `=1+1` |
| E3 | The first empty cell inside the dimension gets the number 42 |
| E4 | Column A, one row below the dimension, gets the number 42 |
| E5 | E4's row, column B, gets the text `rustsheet-roundtrip-E5`, a string not in the shared-string table |
| E6 | E4's row, column C, gets the number 0.5 with number format `0.000`, a style the workbook may not have |

E5 and E6 force an append to the shared-string and style tables. That is how index stability (SPEC-0034 D3.3) gets measured.

### D2. Fidelity items
A **fidelity item** is one unit of the source package's content, with a stable key that does not depend on
renumbering. The comparison (D3, D4) finds each item of the source in the output and gives it a verdict.

| Kind | Key | Compared as |
|---|---|---|
| `cell` | sheet name, A1 | value and type, formula text, resolved style properties (D3.4) |
| `sheet` | sheet name | state, order, kind (worksheet, chartsheet, macro sheet, dialog sheet), sheetPr (tab colour, outline, codeName), sheet views, sheetFormatPr, and every other sheet setting except `dimension` (which N9 derives) |
| `name` | scope, name | formula text, hidden, comment |
| `feature` | owner (workbook, sheet name, table name), feature, ordinal or natural key (D3.2) | the feature's canonical form: one record per merge range, CF rule, DV rule, hyperlink, note, threaded comment, table, autoFilter, page setup, header/footer, protection, outline, and so on |
| `part` | owner chain from the package root by relationship type, plus the ordinal among its siblings of the same type (never the part name) | presence, content type, relationship closure, bytes or XML (D4) |
| `element` | owner part key, element path from the root with namespace URIs, ordinal among same-named siblings; `ext` elements keyed by `uri` | presence and XML (D4) |
| `attribute` | owner element key, attribute namespace URI and local name | value after the defaults table (D4.3) |

A `feature` item exists only for features the feature model of T-4172 knows. Content it does not know is still
scored, as `part`, `element` and `attribute` items. **No source content goes unscored.**

To stop one lost element from yielding hundreds of items, a lost element absorbs its descendants and their
attributes, and is reported once with its descendant count.

**Verdicts:**

| Verdict | Meaning | Gate |
|---|---|---|
| `kept` | equal in the output (S for modelled kinds, P for the rest) | pass |
| `normalized` | differs only in a way the normalization list N (D4.4) allows | pass |
| `changed` | present in the output, not equal | fail |
| `lost` | absent from the output | fail |
| `dropped` | absent, and the save's `SaveReport` (D7) names it with a reason that the case's scenario allows (D7.2) | pass, counted in its own column |
| `stale` | kept although an R2 edit made it stale (a range-bound fragment whose range moved) | fail |
| `added` | in the output, not the source, and not allowed by the expected delta (D5) | fail |
| `n/a` | the scenario does not apply | not scored |

`dropped` is to fidelity what an accepted deviation is to eval cells (decision 0036). It is visible, counted apart,
never hidden.

### D3. Semantic equivalence (S, the gate)
Both packages are read into one **canonical feature model**: the importer's typed model plus the feature records of
D2. S equality is equality of those models, with these rules.

1. **Cells.**
   - A value compares by type and bits. Text compares exactly.
   - Formula text compares as the file stores it, read by the snapshot reader, not after the importer's
     parse-and-print: printing both sides with the engine's own printer would hide a printer bug (T-4140 was found
     this way). Two rules make the stored text canonical as Excel stores it:
     - X5's leading spaces, TABs and LFs are dropped;
     - an `_xlfn.` or `_xlfn._xlws.` prefix is dropped where the function's stored form has none (T-0172: Excel
       re-stores such a prefix without it).

     A prefix the function needs, or one the export adds wrongly (T-0417), still compares (T-4513).
   - An empty-string cell is not a blank cell (T-4148).
2. **Collections the schema leaves unordered** compare as multisets by natural key:
   - merge ranges by ref;
   - hyperlinks by ref;
   - DV and CF rules by sqref plus priority;
   - defined names by scope and name.

   Ordered ones (sheets, rows, CF priority, table columns) compare in order.
3. **Indexes are resolved, never compared.** That covers shared strings, cellXfs, dxfs, fonts, fills, borders,
   numFmt ids, theme colour indexes and the indexed palette, rIds, and table and pivot ids that only cross-reference.
4. **A style compares as its resolved properties:**
   - the font record;
   - the fill (pattern, fg, bg, gradient stops);
   - the border sides;
   - alignment;
   - protection;
   - the number-format **code** (not its id; the built-in ids map to their en-US codes);
   - quotePrefix.

   A theme colour compares as theme index plus tint. The tint compares to 1e-6 relative, because Excel writes
   rounded tints (T-0419, X2).
5. **Defined-name and feature formulas** (CF, DV, names, table totals) compare as cell formulas do (rule 1).

### D4. Package equivalence (P, the diagnostic)
P equality is a structural diff of every part pair matched by the `part` key of D2.

1. **XML parts** are compared as namespace-aware trees:
   - names by namespace URI and local name, never by prefix;
   - namespace declarations themselves are ignored;
   - `mc:Ignorable` compares as a set of namespace URIs.
2. **Child order.** Children compare in order, except at the paths the schema leaves unordered: rule D3.2's
   collections plus `extLst` children by `uri`. The differ keeps that list as data, with a schema citation per path.
3. **The attribute-defaults table.** An absent attribute equals its schema default (`indexed="0"`, `hidden="0"`,
   `customFormat="0"` and so on). The differ keeps the table as data, each entry citing the ECMA-376 or MS-OI29500
   default.
4. **The normalization list N.** An item may be normalized only if it is on this list **and** its evidence row in
   docs/excel-semantics/xlsx-storage.md "Round trips" shows Excel rewriting it on every save. Until then a
   candidate's differences count as `changed` and are tagged `candidate-N<k>`, so the report shows what the probe
   would settle. Candidates (RQ2):

   | Id | Candidate |
   |---|---|
   | N1 | `xl/calcChain.xml` and its relationship and override |
   | N2 | `docProps/core.xml` `dcterms:modified`, `cp:lastModifiedBy` |
   | N3 | `docProps/app.xml` `Application`, `AppVersion`, `DocSecurity`, `TitlesOfParts` and `HeadingPairs` (derived from the sheet list) |
   | N4 | `workbook.xml` `calcPr@calcId` |
   | N5 | `fileVersion@lastEdited`, `@lowestEdited`, `@rupBuild`, `@appName` |
   | N6 | `xr:revisionPtr` and every `xr:uid` / `xr2:uid` / `xr3:uid` |
   | N7 | `x14ac:dyDescent` |
   | N8 | `row@spans` |
   | N9 | `dimension@ref`, recomputed from the cells |
   | N10 | `sst@count` and `sst@uniqueCount`, and the other derived counts: `cellXfs@count`, `fonts@count` and the rest |
   | N11 | `bookViews/workbookView` window geometry (`xWindow`, `yWindow`, `windowWidth`, `windowHeight`) |
   | N12 | `mc:AlternateContent` with a `x15ac:absPath` choice (decision 0275 D6 keeps it; normalized only if Excel rewrites it to the saving path) |

5. **Binary and other non-XML parts** compare as bytes.
6. **Zip-level details are ignored:** entry order, timestamps, compression, extra fields.
7. **Whitespace.** Inner whitespace of text content is significant. Whitespace-only text between elements of
   element-only content is not.

### D5. The expected-delta rule (R1, R2)
Let `P(a, b)` be the multiset of non-`kept`, non-`normalized` items of D4 between packages `a` and `b`, and let
`src` be the source. For an R1 run with edits E:

`P(src, out_R1) − P(src, out_R0) ⊆ Δ(E)`

`Δ(E)` is exactly:
1. the cells E wrote;
2. the cells whose cached value after R1's recalculation differs from their cached value after R0's (the engine
   reports this set from the two runs, not by diffing);
3. the shared-string and style records E5 and E6 appended, which reach `P` only as `added` records, because indexes
   are resolved;
4. the derived attributes of N9 and N10, whether or not N has been evidenced, because E itself changes them;
5. nothing else.

Any other item is a **delta violation**, scored as the item's verdict (`added`, `lost`, `changed`). The rule holds
however lossy R0 is. That makes it measurable today and gives the owner's "only the change I made" guarantee
directly. A delta violation always fails, even when the same item is in the R0 baseline.

As implemented (T-4514, decision 0336): violations take the case's baseline like any other item, with scenario `R1/E<n>` or
`R1/all`, so a new violation is a regression and a fixed one is a fix. They cannot fail outright while E6 makes every
save regenerate (T-4623). Once the patch tier writes new styles, the remaining known violations can be burnt down,
and the rule can become absolute.

**R2's sheet edits** (T-5005, SPEC-0034 D5.7). R2 holds the same rule, `P(src, out_R2) − P(src, out_R0) ⊆ Δ(E)`,
with each edit's own `Δ(E)`. Each edit acts on the first sheet in tab order, as R1's do:
- **The edits.**
  - `R2/rename` renames it `rustsheet-roundtrip-R2`, and `R2/add` adds a sheet of that name at the end. A case
    whose workbook already has a sheet of that name is `n/a`.
  - `R2/delete` deletes the first sheet, and `R2/move` moves it to the end. Both are `n/a` for a workbook of one
    sheet.
- **A renamed sheet** keeps its identity. The output's part keys and semantic facts are keyed back under the
  source's name before they are matched, so only what the rename itself rewrote can differ.
- **`Δ(E)`** holds:
  - the cells whose input the edit rewrote (a formula naming a renamed or deleted sheet; the engine does not report
    these, so the harness compares every formula before and after) and the cells whose cached value it changed;
  - the workbook part's sheet list and book views, and each sheet view's `tabSelected` after a delete or move;
  - a defined name rewritten to name the renamed sheet, or to `#REF!` by a delete, and, under a delete or move, a
    sheet-scoped name, whose `localSheetId` key counts tab positions;
  - what the extended properties list inside `TitlesOfParts` and `HeadingPairs`;
  - an added sheet's facts and parts, a deleted sheet's facts, and the `order` facts a delete or move shifts.
- **A deleted sheet's parts** are not in `Δ(E)`. They pass only as `dropped`, when the report names them
  `OwnerDeleted` (D7.2), which SPEC-0034 stage 5 adds (T-5004). `StaleAfterStructuralEdit` passes for a part of the
  edited sheet when it names that sheet, by its source or its new name.
- **Measurement.** A sheet edit leaves the verbatim and patch tiers, so each R2 save is regenerated. Its violations
  are therefore mostly what a regenerated save loses: they are the measure of SPEC-0034 stage 5, and shrink as it
  lands. They take the baseline as new coverage, as R1's did (decisions 0336, 0345).

**Byte stability**, which is M15 exit criterion 8, is the stronger form scored with R1. A part that `Δ(E)` does not
touch must be byte-identical to the source part. It is measured from T-4172 and gated only once SPEC-0034's verbatim
tier lands. Until then it is reported.

### D6. The package validator
Every output of every scenario is checked. Any failure is a `package-invalid` verdict on the case, which always
fails.

| Id | Check |
|---|---|
| V1 | `[Content_Types].xml` gives every part a content type (Default by extension or Override), and no Override names a missing part |
| V2 | Every internal relationship target exists. External targets (`TargetMode="External"`) are not followed |
| V3 | Relationship ids are unique within each `.rels` |
| V4 | Every `r:*` attribute (namespace `…/officeDocument/2006/relationships`) in every XML part resolves in that part's `.rels` |
| V5 | No two part names are equal under OPC equivalence (ASCII case-insensitive, `\` as `/`; the same rule as `PartNames`) |
| V6 | `workbook.xml` sheets have unique ids and unique names (case-insensitive, at most 31 UTF-16 units, none of `: \ / ? * [ ]`), and each `r:id` points at a sheet-kind part |
| V7 | `sharedStrings` indexes, style indexes and dxf ids used by cells and features are in range |
| V8 | The package opens with our own importer, under default limits, with no error (the reload) |

XSD validation (T-4179) adds V9 under the same verdict. "Excel for the web refused to open it" (T-4180) is a separate
case verdict, `excel-refused`. It always fails, and is recorded with Excel's error text.

### D7. SaveReport
Callers, and the suite's `dropped` verdict, need to know what a save left out on purpose. The facade gains:

```rust
/// What a save left out of the package, and why (SPEC-0033 D7).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct SaveReport {
    /// One entry per dropped item, in package order.
    pub dropped: Vec<Dropped>,
}

/// One item a save did not write.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Dropped {
    /// The source part's name as its zip entry spells it (`xl/calcChain.xml`); the suite resolves
    /// it to the part's fidelity-item key (D2), so a writer's renaming never changes a report.
    /// Element-level entries come with SPEC-0034 stage 5's fragments.
    pub item: String,
    /// Why it was not written.
    pub reason: DropReason,
}

/// Why a save dropped an item.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DropReason {
    /// The workbook carries no preserved copy of it (today: everything the model does not hold).
    NotPreserved,
    /// The preservation policy never writes it (calcChain, digital signatures; the `Drop` policy of T-4567).
    Policy,
    /// A structural edit on this sheet made range-bound content stale (decision 0275 D4).
    StaleAfterStructuralEdit { sheet: String },
    /// A content edit (a modelled feature, the notes, a pivot or table, the style table) made kept content
    /// stale, on this sheet or workbook-wide (`None`) (SPEC-0034 D5.5).
    StaleAfterEdit { sheet: Option<String> },
    /// Its owner (a sheet, a table) was deleted.
    OwnerDeleted,
    /// Its relationship closure reaches a part the export regenerates (the closure rule of T-4567, SPEC-0034 stage 5).
    ClosureReachesRegenerated,
    /// Keeping it would exceed a preservation limit.
    LimitExceeded { limit: String },
    /// One of its relationship ids could not be remapped.
    UnmappableRelationship,
    /// Active content (VBA, ActiveX) cannot go into a file that is not macro-enabled (decision 0275 D5).
    ActiveContentInPlainFormat,
}
```

1. **API.**
   - `Workbook::save_xlsx_with_report(&self, path, opts) -> Result<SaveReport, SaveError>` and
     `to_xlsx_bytes_with_report(&self, opts) -> Result<(Vec<u8>, SaveReport), SaveError>` are added.
   - `save_xlsx` and `to_xlsx_bytes` keep their signatures and discard the report.
   - The Python binding returns the report as a list of `(item, reason)` tuples from
     `save_xlsx(..., report=True)`.
   - Until stage 5, the report is filled coarsely (T-4562): one `NotPreserved` entry per source part of a
     relationship type the export does not model, and calcChain as `Policy`. A part of a modelled type the export
     fails to write (a dropped pivot cache) is not reported until T-4999 compares the output's inventory.
     Element-level entries come with fragments.
2. **What a `dropped` verdict accepts.** An item in the report is `dropped` (pass) only for these reasons:
   - `Policy` or `ActiveContentInPlainFormat`, in any scenario;
   - `StaleAfterStructuralEdit` or `OwnerDeleted`, in an R2 run whose edit explains it;
   - `StaleAfterEdit`, in an R1 or R2 run whose edits explain it (R1's `Δ(E)`, D5).

   `NotPreserved`, `ClosureReachesRegenerated`, `LimitExceeded` and `UnmappableRelationship` remain `lost`. The
   report says why, but that is not a reason to pass. A `LimitExceeded` drop of a hostile-set case passes.
3. A reported item that is in fact still in the output is a `SaveReport` bug, scored `changed`.

### D8. The `roundtrip` suite kind
Round-trip suites live under `eval/roundtrip/<name>/suite.toml`, outside `eval/suites/` (as Gate A's tables do), so
no cell report or milestone measure counts them (T-4172).

```toml
name = "roundtrip-calipers"
kind = "roundtrip"
root = "../../.cache/suites/calipers"
include = ["roundtrip/*/init.xlsx", "pv_goldens/**/golden.xlsx"]
scenarios = ["R0", "R1/E1", "R1/E2", "R1/E3", "R1/E4", "R1/E5", "R1/E6", "R1/all"]
# Optional Excel golden, by path relative to the case: `golden` compares S(ours, Excel's) for R0.
golden = { R0 = "golden.xlsx" }
exceptions = "exceptions.toml"
gate = false
baseline = "cells"   # fidelity items take the item slot of the ordinary cells-mode baseline
milestone = ["M15"]
```

- **Discovery and fetch.** These reuse the `xlsx` kind's machinery (`include`, `exclude`, `[fetch]`, licence and
  origin keys). There is one case discoverer, not two (directive 3).
- **Baseline.** `eval/baseline/<suite>.txt` is the ordinary cells-mode file, keyed by case and scenario,
  `<case>#<scenario>` (decision 0345, T-4961): `C` gives the item count of a case's scenario, and each
  `F` line holds a failing item of it, `<scenario> <kind> <item key> <verdict>`. The ratchet is the usual one, per key, so a new scenario
  is a new key and cannot turn an existing scenario's loss into new coverage. A line may disappear (fixed). A new
  failing item is a regression unless the key is new (new coverage). `update-baseline` refuses regressions.
  `--allow-regressions` needs a decision record, as everywhere.
  A corpus suite (Calipers, POI, corpus-bench) uses `baseline = "cases"` instead. Its item lines would run to
  tens of megabytes (about 550,000 on the first run, T-4173). Each failing case is one `F` line,
  `* <n> <digest>`:
  - n is the case's failing items, each weighted as the kept rate weighs it and at least 1;
  - the digest is a hash of the set of their keys.

  `roundtrip::diff` sets the rules:
  - A line whose n fell is a fix.
  - A line whose n held or rose is a regression. This includes a loss traded for a fix of the same weight.
  - A fix that outweighs a new loss passes. The cells-mode fixtures suite still names every item, and
    `roundtrip.json` lists them for any run.
- **Round-trip regressions beyond the ratchet**, in both modes:
  - A case whose checked count fell is a regression. The count includes what the output let the scorer compare, so
    an unreadable output compares one item.
  - A case that newly did not run (`!Status`) is a regression, whatever its count.
  - Updating a `cases` suite refuses to drop a case, because a missing case means the cache was fetched under an
    older include.
- **Kept rate.** The CLI line and the report give each suite's kept rate: the source items compared, less those the
  failing items cover, over the items compared. A failing item covers:
  - a lost element: its whole subtree;
  - a lost XML part: all of its elements;
  - a `<truncated>` item: every difference it hides;
  - any other change or loss: 1 item;
  - `added` items and validator failures: no source items. The baseline still gates them.

  A case that did not run is reported as not scored, never as kept. M15's goal is stated as this rate.
- **Exceptions.** `exceptions.toml` holds narrow, per-case tokens in Calipers' style. Each token names the decision
  or task that owns it and the items it excuses (`volatile`, `candidate-N4`, …). A self-test runs every case with
  its exceptions removed and checks that each exception still excuses something. Stale exceptions fail.

  As implemented (T-4514): the manifest's `exceptions` key names the file. Each `[[exception]]` has three fields:
  - `case`, the case id;
  - `item`, one baseline line's item, such as `R1/E6 element /officeDocument[0]/theme[0]:… lost`;
  - `owner`, either `T-<n>` or `decision <nnnn>`.

  Excused items are removed before the baseline and the rates are computed. An exception that excuses nothing fails
  `--check` and `--update-baseline`, and is listed by a report run. No suite has exceptions yet.
- **Scenario rates.** Each scenario family has its own kept rate: `R0`, the `R1/` edits together (their violations
  over the items they compared), and `R0/again` (idempotence). The CLI line and the report give each one a suite
  ran. Every listed scenario reruns every case, so the corpus suites list `R0` and `R1/all`, and the fixtures list
  all nine scenarios.
- **Report.** `make eval` gains a fidelity section:
  - verdict counts by scenario and by kind;
  - the top lost features by number of files, as in the 2026-11-04 census table;
  - the worst cases;
  - validator failures.

  Its JSON is in `report.json` under `roundtrip`, so docs/progress.md can read it later without a parser of its own.
- **Limits** (directive 5). A case runs under the default `ImportLimits` and the existing per-case timeout. The differ
  reads parts through the bounded `Package` reader, never through an unbounded parser, and caps reported items per
  case (the report says how many were cut).

## Alternatives considered
- **Byte equality.** Fails on every file, because Excel itself rewrites much on every save (X1, X2). It says
  nothing about what matters.
- **Cell values only.** This is today's `test_load_and_saving`. It misses 100% of the feature losses the census
  found.
- **Semantic gate only, as in Calipers.** It cannot see content the feature model does not know, which is exactly
  the "new Excel feature" case the owner cares most about. That is why D2 scores unknown content as `element` and
  `attribute` items.
- **Canonical XML (C14N) equality per part.** Too strict on renumbered indexes and ids, too loose on ordering
  semantics, and it has no notion of resolved styles. D4 is C14N's intent with the renumbering and schema knowledge
  added.
- **Normalizing by our judgment of "harmless".** Rejected. The normalization list is evidence-gated (D4.4), as
  every other Excel claim is (directive 1).

## Test plan
The differ is tested before it is trusted:
- **Identity.** `P(src, src)` and `S(src, src)` are empty for every file of every round-trip suite.
- **Invariance.** A crafted copy of a workbook with renumbered rIds, shuffled shared strings (indexes rewritten),
  duplicated and renumbered style records, re-prefixed namespaces, reordered unordered collections, and default
  attributes written out, compares `kept` and `normalized` everywhere.
- **Sensitivity.** Each of these mutations of a workbook is found with the right verdict: one lost part, one lost
  `ext`, one lost attribute, one changed cell value, one changed style property, one added element, one dangling
  `r:id` (V4).
- **Idempotence**, a metamorphic property: saving our own R0 output again gives `P(out_R0, out_R0') = ∅`. Today it
  should already hold. A failure means the export is unstable.
- **Delta rule.** For a workbook without formulas, R1/E1's violations are empty and `Δ(E1)` is one cell. A crafted
  output that also drops an unrelated element is caught as a violation.
- **Validator.** One crafted failing package per V check.
- **Exceptions.** The self-test fails on an exception that excuses nothing.

## Rollout
1. **T-4172.** The suite kind, scenarios R0 and R1, the item model, the S and P differ, the validator, baselines,
   the report section and the tests above, run over the fixtures workbooks. It also creates the "Round trips"
   section of docs/excel-semantics/xlsx-storage.md.
2. **T-4173.** The corpus suites (calipers with `init.xlsx`, poi, corpus-bench, fixtures) and their baselines.
3. **RQ2 probe** (filed with this spec), then **T-4166** (RQ5). These move N items from candidate to evidenced and
   settle X4.
4. **T-4179** (V9, XSD) and **T-4180** (Excel goldens, RQ1, `excel-refused`).
5. **SaveReport.**
   - The coarse `NotPreserved` report is part of T-4562 (SPEC-0034 D4), which also adds the save tier. Until then the suite treats
     every missing item as `lost`, which is the same score.
   - Element-level entries follow fragments.
   - R2's sheet edits are scored from T-5005. Its row and column scoring follows M4.
