# Fill probe request: Excel for the web, not recorded results

These 24 workbooks request independent Excel observations for P558996742's missing fill stratum.
Steve, the oracle owner, records them. The coordinator assembles this folder into the oracle request
PR; observations return through the mirror. There is no desktop Excel route in this request.
The Office.js gym is not yet a demonstrated recording route, so the evidence obligation stays
**INCOMPLETE** until actual recordings arrive. Fake-context recorder tests are not Excel evidence.

## Inputs and frozen predictions

- `probes/probes.json`: every source, whole destination (including source), direction, extension
  length, exact AutoFill type, classification to probe, source values/formats/merges and workbook pin.
  Coordinates in `sourceRect`/`targetRect` and in engine plans are zero-based, inclusive.
  `length` counts added rows or columns, not the total destination extent or the number of cells.
- `probes/SHA256SUMS`: SHA-256 pins for all 24 untouched `.xlsx` seeds. Each has one sheet, `Fill`.
  Dates are numeric serials with `yyyy-mm-dd` formatting, using Excel's 1900 date system. Request
  en-US Excel settings, default English custom lists, no user-added custom lists, and fresh copies.
- `frozen-predictions.json`: actual native `rustsheet_wasm::Engine::load_xlsx` followed by
  `Engine::fill_plan` at engine commit `b17511b88b5ddee8c161147bc96453c3c4234e35`. Clock, UTC zone
  and seed are pinned in that file. Inputs load from XLSX so styles and merges are not discarded.
  It contains **23 plans and one partial-merge refusal**, not Excel-computed answers.
- `predict-fill.rs`: the small prediction driver. Plan cell order, `input`, `from`, `numberFormat`
  and `series` are preserved, with no hand-adjustment to what we think Excel should do. A plan is
  not an evaluated final workbook: `from` identifies the formatting source and the plan does not
  contain a merge-topology delta. Request the actual merge results, including refusals, independently.

Do not overwrite the frozen predictions after seeing Excel results. Record discrepancies, including
an unexpected success or refusal, as observations. `seriesKindToProbe` names the hypothesis, not
verified Excel behavior. Formula dependencies outside the fill target are present in the seed file.
The recorder captures the whole target before and after, including unchanged source cells and blanks.

## Which API records each case?

**Graph v1.0 cannot perform any of these fills through a documented workbookRange operation.** Its
complete methods table was checked on 2026-11-10: no `autoFill`, `fillDown` or `fillRight`. Graph can
upload the seeds and read values, formulas and number formats after another actor fills them, but
updating cells to our predicted values is not an independent fill oracle. No undocumented endpoint,
beta capability, UI gesture or runtime behavior is claimed verified.

All rows therefore need the gym's **Office.js `Range.autoFill`**, documented in ExcelApi 1.9. The
recorder also needs ExcelApi 1.13 to observe merges. The destination is always explicit; the preview
null-destination/double-click behavior is not used. Actual outcomes, including copy-vs-series defaults,
date stepping, locale lists, formatting inheritance and merged-source acceptance, are **unverified**
until Steve runs them. The references establish API availability, not any predicted sequence.

| Case | Source → whole target | Direction / added length | Kind / explicit Office.js type | Graph fill |
|---|---|---|---|---|
| `number-single-default` | A1 → A1:A6 | down / 5 rows | lone number / FillDefault | no documented API |
| `number-single-series` | A1 → A1:A6 | down / 5 rows | forced numeric series / FillSeries | no documented API |
| `number-step-down` | A1:A2 → A1:A7 | down / 5 rows | two-number step / FillDefault | no documented API |
| `number-step-up` | A4:A5 → A1:A5 | up / 3 rows | backward step / FillDefault | no documented API |
| `number-step-right` | B2:C2 → B2:H2 | right / 5 columns | horizontal step / FillDefault | no documented API |
| `number-step-left` | E2:F2 → A2:F2 | left / 4 columns | backward horizontal step / FillDefault | no documented API |
| `number-negative-decimal` | A1:A2 → A1:A6 | down / 4 rows | negative fractional step / FillDefault | no documented API |
| `number-copy` | A1:A2 → A1:A7 | down / 5 rows | repeating values / FillCopy | no documented API |
| `date-days-leap` | A1 → A1:A5 | down / 4 rows | cross leap day / FillDays | no documented API |
| `date-weekdays` | A1 → A1:A6 | down / 5 rows | Friday through weekend / FillWeekdays | no documented API |
| `date-month-end` | A1 → A1:A5 | down / 4 rows | January 31 / FillMonths | no documented API |
| `date-years-leap` | A1 → A1:A6 | down / 5 rows | February 29 / FillYears | no documented API |
| `date-two-seeds` | A1:A2 → A1:A6 | down / 4 rows | inferred two-day step / FillDefault | no documented API |
| `weekday-names` | A1:A2 → A1:A8 | down / 6 rows | Mon, Tue custom list / FillDefault | no documented API |
| `text-trailing-padded` | A1:A2 → A1:A6 | down / 4 rows | Item007, Item009 / FillDefault | no documented API |
| `text-trailing-single` | A1 → A1:A5 | down / 4 rows | Part 9 / FillDefault | no documented API |
| `formula-relative-down` | B2 → B2:B6 | down / 4 rows | relative references / FillDefault | no documented API |
| `formula-mixed-down` | B2 → B2:B6 | down / 4 rows | A2, $A2, A$1, $A$1 / FillDefault | no documented API |
| `formula-mixed-right` | B2 → B2:F2 | right / 4 columns | mixed reference axes / FillDefault | no documented API |
| `formula-relative-up` | B5 → B2:B5 | up / 3 rows | reverse formula adjustment / FillDefault | no documented API |
| `formatted-series` | A1:A2 → A1:A6 | down / 4 rows | alternating formats / FillDefault | no documented API |
| `formatted-blank` | A1:A2 → A1:A6 | down / 4 rows | styled numeric/blank pair / FillDefault | no documented API |
| `merged-source-tiled` | A1:B1 → A1:F1 | right / 4 columns | full merged tiles / FillDefault | no documented API |
| `merged-source-partial` | A1:B1 → A1:E1 | right / 3 columns | clipped merged tile or refusal / FillDefault | no documented API |

Engine options map `FillDefault` to its default `mode: series, ctrl: false, series: false`, not to
the force-series option. `FillSeries` sets `series: true`; `FillCopy` selects copy mode. The four
date variants select `dateUnit`. Whether these implementation choices match Excel is what we ask.

## Recording procedure for Steve

1. Verify `SHA256SUMS` before upload; record the same pin in the result. Open a fresh uploaded copy
   for each case. Do not resave a seed, pre-fill its target, or replay a case over an already filled
   workbook. Verify en-US/default lists/1900 date settings and record Excel web version/build,
   Office.js version, locale, custom-list assumptions and recording time alongside the dump.
2. In the gym add-in, require `Office.context.requirements.isSetSupported("ExcelApi", "1.13")`.
   If unsupported, return an infrastructure failure, not an empty result or containment-style pass.
   Install `record-fill.js` in the add-in context. It declares one self-contained `recordFill`
   function, with no imports, external helpers, engine code, predictions or Graph dependency.
3. Pass the matching manifest probe to:

   ```js
   const dump = await Excel.run(context => recordFill(context, probe, new Date().toISOString()));
   ```

   The function recalculates, reads every target cell, calls exactly
   `sheet.getRange(probe.source).autoFill(probe.target, probe.autoFillType)`, synchronizes,
   recalculates and reads every target cell again. No fill retry or corrective write is performed.
   The before-state must round-trip the seed's known merge inventory before any fill is attempted;
   an API returning no containing merge for a covered cell cannot silently erase the merge evidence.
4. Save the raw returned object as `refs/officejs/<probe-id>.fill.json`, together with the environment
   metadata and untouched input pin. An `operation-error` is valuable evidence, not a failed
   recording: preserve its actual code/message and after-state, which may show partial mutation.
   Conversely, missing properties, unsupported APIs, invalid matrices and failed before/after reads
   throw and produce **no complete observation**. Retain their logs as recording failures.
5. Return exactly one dump per probe, all target cells exactly once in both snapshots, sorted in
   row-major order, with no omitted blanks. Preserve raw values and formulas, including non-formula
   values returned by `formulas`. Do not turn null into empty string, discard trailing zeros from
   text, normalize localized formats, or replace errors with a predicted result.

**Unverified recorder assumption:** a failed `autoFill` synchronization leaves the Office.js context
usable for the subsequent snapshot. The fake tests exercise that route, but only a gym run can
confirm it. If the real host refuses the after-read, keep the error and report incomplete recording;
do not invent an unchanged after-state. Missing gym integration is a known external prerequisite.
This records Office.js fill semantics, not a drag-handle pointer path or the unfinished UI gym itself.
Per-cell merged-area containment is also checked against the known seed, not assumed from API
presence: if that observation fails on Excel, the recorder needs adjustment before evidence is valid.

## Observed fields and reference checks

| Dump field | API read | Documentation checked | Actual Excel recording |
|---|---|---|---|
| cell address, zero-based row/column, dimensions | `Range.address`, `rowIndex`, `columnIndex`, `rowCount`, `columnCount` | Range reference; per-cell geometry verified before accepting coverage | pending |
| value, valueType | `Range.values`, `valueTypes` | Range reference | pending |
| formula | `Range.formulas` | A1 formulas; documented to return the cell value when no formula exists | pending |
| numberFormat, text | `Range.numberFormat`, `text` | Range reference; raw format code, not locale-normalized | pending |
| font bold/italic/color | `Range.format.font` | RangeFont reference | pending |
| fill color | `Range.format.fill.color` | RangeFill reference | pending |
| merged regions | per-cell `getMergedAreasOrNullObject().address`, `isNullObject` | Range/RangeAreas, ExcelApi 1.13; 512-area limit, avoided by per-cell reads | pending |
| completion or actual operation error | `autoFill` and `context.sync()` | method exists; exact error behavior is not established by docs | pending |

References fetched and read on 2026-11-10:

- [Graph v1.0 workbookRange methods and properties](https://learn.microsoft.com/en-us/graph/api/resources/range?view=graph-rest-1.0)
- [Office.js Range, including autoFill and string overload](https://learn.microsoft.com/en-us/javascript/api/excel/excel.range?view=excel-js-1.9)
- [Office.js AutoFillType definitions](https://learn.microsoft.com/en-us/javascript/api/excel/excel.autofilltype?view=excel-js-1.9)
- [Range merged-area API, stable ExcelApi 1.13 requirement shown](https://learn.microsoft.com/en-us/javascript/api/excel/excel.range?view=excel-js-preview)
- [RangeAreas](https://learn.microsoft.com/en-us/javascript/api/excel/excel.rangeareas?view=excel-js-preview)
- [RangeFont](https://learn.microsoft.com/en-us/javascript/api/excel/excel.rangefont?view=excel-js-1.9)
- [RangeFill](https://learn.microsoft.com/en-us/javascript/api/excel/excel.rangefill?view=excel-js-1.9)
- [Application.calculate](https://learn.microsoft.com/en-us/javascript/api/excel/excel.application?view=excel-js-1.9)

## Reproduction and tests

`make-probes.py` uses xlsxwriter 3.2.9; `verify-inputs.py` uses openpyxl 3.1.5. Both already appear
in the tools project's dependency/development sets; no new repository dependency is introduced.
They are installed only in an external validation environment. Node's built-in test runner tests
the distributed JavaScript against a strict load/sync fake; esbuild and TypeScript are existing
workspace tools. Rebuild only while preparing a new request: never silently replace frozen pins.

```sh
python eval/fill/make-probes.py
python eval/fill/verify-inputs.py
esbuild eval/fill/record-fill.ts --target=es2022 --outfile=eval/fill/record-fill.js
tsc --strict --noEmit --target ES2022 --lib ES2022 --skipLibCheck eval/fill/record-fill.ts
node --test eval/fill/record-fill.test.cjs
```

To regenerate predictions, check out the pinned **engine** commit separately (the current request
branch need not contain its WASM wrapper). Create a temporary Cargo binary whose source is
`predict-fill.rs`, with dependencies `rustsheet_wasm` by path to that checkout's
`crates/rustsheet-wasm` and `serde_json = "1.0"`. Seed its lockfile with that engine checkout's
`Cargo.lock`, then build with Rust 1.97. The validated run used Cargo/rustc 1.97.1 and an offline
registry. Run `predict-fill eval/fill/probes/probes.json <engine-commit>` into a **new** file and
compare it with `frozen-predictions.json`. Verify all seed pins before and after running. The driver
prints actual engine outputs and does not apply them or write to the seed workbooks.
