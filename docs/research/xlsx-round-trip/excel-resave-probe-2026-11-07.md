# Excel for the web re-save probe of the patch tier (T-4890, 2026-11-07)

Does Excel open what the source-preserving export saves, and does its own save keep it? This is the first Excel
evidence on SPEC-0034's patched packages. It is narrower than T-4180, which scores the suites against Excel goldens.

## 1. Method
1. **Our saves.** Run `rustsheet-eval roundtrip --save-dir DIR` on main at 0f7c07b86, after T-4623, with all four
   round-trip suites. Of what it writes, keep:
   - for the 29 fixtures: R0, R1/E6 and R1/all;
   - for a fixed sample of 24 corpus cases: R0 and R1/all. The sample is every n-th case in sorted order, 8 each
     from calipers, corpus-bench and poi.
2. **Excel's saves.** Run `rustsheet-oracle resave -b excel-graph` on each set of our saves, and on the sources
   themselves. Excel for the web, through Microsoft Graph, opens each file, recalculates it (the oracle sets
   `fullCalcOnLoad` and makes a scratch edit so that the session persists) and saves it. The oracle reports a file
   Graph cannot open as `refused`, with Excel's error.
3. **Comparison.** `rustsheet-eval package-diff` compares Excel's save of each R0 save with Excel's save of its
   source. Both sides carry Excel's own normalisation and recalculation, so what is left is what Excel kept
   differently from us. The script ignores the noise every SharePoint save writes: ids, dates, `AppVersion`, and the
   customXml parts it adds and numbers in no fixed order.
4. **Patched or regenerated.** A save counts as patched when `xl/workbook.xml` is the source's, byte for byte.
   Otherwise it counts as regenerated.

Reproduce with `docs/research/xlsx-round-trip/resave_probe.py DIR target/release/rustsheet-eval`. It prints the
table below, the hint tally of the patched saves, and each R0 save that differs after Excel's save.
- The corpus `.xlsm` in the sample (poi suffix-generator) is out: Graph refuses its source too
  (`MetroFileFormatMismatch`, a limit of the Graph workbook API).
- calipers rich_text R0 needed T-4910's stdout workaround.

## 2. Results
| scenario | patched: opened | patched: refused | regenerated: opened | regenerated: refused |
|---|---|---|---|---|
| R0 | 52 | 0 | 0 | 0 |
| R1/E6 (new number format, T-4623) | 29 | 0 | 0 | 0 |
| R1/all | 23 | 0 | 20 | 9 |

**Patched saves.** Excel opened every one of the 104.
- Of the 52 R0 saves, 51 differ from Excel's save of the source in nothing beyond the noise above. The other one,
  calipers pv26_2_002, has a calcChain too large for the differ (more than 10^6 elements).
- Every R1/E6 save keeps its new `0.000` format through Excel's save.
- What the patched saves covered:
  - 79 rewrote a worksheet;
  - 52 appended style records (T-4623);
  - 23 appended shared strings;
  - 15 copied a calcChain unchanged;
  - 6 added the metadata parts;
  - 2 hold a cell outside its row's `spans` (corpus-bench 48354 and 55817, R1/all);
  - 38 grew a sheet's `<dimension>`.

**Regenerated saves.** All 9 refusals are regenerated saves of R1/all. Each failed with `FileCorruptTryRepair`
("We're sorry, but something went wrong with this file").
- R1/all falls back to regeneration when E5 needs a shared-string part the patch tier does not write yet (T-4831),
  or when a shared formula is touched (T-4832).
- Three causes were isolated by uploading variants (§3). They cover all 9 refusals:
  - 7 are the fixtures external-link-*, with missing externalLink parts. Six of them are also confounded by the empty
    creation date.
  - 1 is fixtures valueless-cell-forms, with an empty creation date.
  - 1 is poi link-external-workbook-a, with an empty `cellStyles`.
- The variants are under the probe's `var/`, `var2/` and `var3/` directories, each with its own `resave.json`.

## 3. Refusals
- **Empty creation date** (fixtures valueless-cell-forms). Its source has an empty `docProps/core.xml`, and the
  regenerated save writes `<dcterms:created xsi:type="dcterms:W3CDTF"></dcterms:created>`.
  - The R1/E5 save (788ba5e3…a9) is refused.
  - Each one-part variant is still refused: metadata.xml removed, theme removed, the source's workbook.xml, the
    source's styles.xml, and `<t></t>` written as `<t/>`.
  - Removing `docProps/core.xml` makes it open (ff7146f3…4a). Removing only the empty `created` element also makes
    it open (0120e27f…74).
  - Fix: T-4909.
- **Empty `cellStyles`** (poi link-external-workbook-a). This save has no `[n]` reference and its core.xml has a real
  `created` date. Its `xl/styles.xml` holds `<cellStyles count="0"></cellStyles>`, which no saved R1/all save has.
  - The R1/all save (814c986a…b6) is refused.
  - With the element removed (5d941486…10), it opens.
  - With the Normal cell style added (73e6065f…0a), it opens.
  - Fix: T-4909.
- **External links** (the 7 fixtures external-link-*). The regenerated save keeps `[1]Sheet!A1` formulas but drops
  `xl/externalLinks/*` and `<externalReferences>`. Every one of these R1/all saves also has the empty `created`
  element, so the rule was isolated on external-link-cached-text R1/E5:
  - c0f37b44…20 is refused.
  - Without core.xml (10d86a14…65) it is still refused.
  - Without only the empty `created` (9b892ce5…5d) it is still refused.
  - Positive control: with the source's externalLink part, its relationship, its override and `<externalReferences>`
    added back (299381fb…e5), it opens.
  - Negative control: valueless-cell-forms R1/E5 without the empty `created` opens, and the same package with one
    `[1]Uses!$A$1` formula added (dbeb46ad…c6) is refused.
  - Their R0 and R1/E6 saves are patched, keep the parts, and open.
  - Fix: the existing T-4143.

## 4. What this settles
Every hint in docs/excel-semantics/xlsx-storage.md "A patched package keeps the source's hints" that these saves
exercised is accepted by Excel for the web. That covers:
- a grown `<dimension>` and new rows below the used range (E4 to E6);
- stale row `spans`;
- a calcChain copied unchanged;
- added metadata parts;
- appended styles and shared strings.

Not exercised, so still unverified:
- a dropped calcChain (no standard edit adds or removes a formula cell);
- rows emptied by clearing;
- desktop Excel (M14).
