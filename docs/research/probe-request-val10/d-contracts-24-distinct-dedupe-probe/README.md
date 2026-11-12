# Probe request: what Excel counts as one value in an AutoFilter list and in Remove Duplicates

*From poc-s-f1-cc (the F1 contracts steward), 2026-11-12. For the Excel oracle owner, routed by the coordinator.
Needed before D-CONTRACTS-24's `query.distinct@1` and `plan.dedupePlan@1` vectors freeze (wave-2 contracts, D7,
2026-11-15): the engine lane (E1) asked for an excel-probe of both equality rules before any vector is written.*

## Why

I-06 1.1.0 adds two engine answers that must group cells exactly as Excel does:
- `query.distinct` (U4's AutoFilter list): one item per distinct value of a column, with its count, in the list's
  order;
- `plan.dedupePlan` (M4's Remove Duplicates): which rows of a band are kept, ties keeping the first.
We have no evidence of either rule. Each case below is one column of cells; for each we need the two answers.

## How

**Office.js (preferred; both answers are observable):**
1. `D` (Remove Duplicates): in a fresh sheet, write the case's cells to `A1:A<n>` (inputs and number formats as
   given), then `sheet.getRange("A1:A<n>").removeDuplicates([0], false)`; record the returned
   `removed`/`uniqueRemaining` and the values left in `A1:A<n>` (`values`, `text`, `numberFormat`) after the call.
2. `F` (AutoFilter equality): rewrite the cells under a header `H` in `A1` (`A2:A<n+1>`), apply
   `sheet.autoFilter.apply("A1:A<n+1>", 0, { filterOn: "Values", values: [<v>] })` for each `<v>` listed in the case,
   and record which rows stay visible (`rowHidden` per row).

**Desktop (only for the list's order and grouping, which no API exposes):** open the AutoFilter dropdown on `A1`
for each case and record the list's items top to bottom, exactly as shown (including "(Blanks)").

## The cases

| Case | Cells (input; format when not General) | Filter values `F` to try | Question |
|---|---|---|---|
| E1 | `apple`, `Apple`, `APPLE`, `apple ` (trailing space) | `apple`, `Apple`, `apple ` | case and trailing space: one value or several? |
| E2 | `1`; `1` with format `0.0`; `1.0` typed; `'1` (text) | `1`, `1.0` | number vs display vs text |
| E3 | `0.1+0.2` as `=0.1+0.2`; `0.3` | `0.3` | a computed double that displays like a typed one |
| E4 | `#N/A` as `=NA()`; `#DIV/0!` as `=1/0`; text `#N/A` | `#N/A` | errors vs error-looking text |
| E5 | `TRUE`; `=1=1`; text `'TRUE` | `TRUE` | booleans vs text |
| E6 | 2026-01-05 with format `yyyy-mm-dd`; the same date with format `dd/mm/yyyy`; text `2026-01-05` | `2026-01-05` | one date under two formats; dates in the desktop list (the year/month/day tree) |
| E7 | empty; `=""`; a space | (Blanks) | which cells are blanks |
| E8 | `ß`, `ss`, `é` (U+00E9), `é` (e + U+0301) | `ss`, `é` | Unicode folding and normalisation |
| E9 | `10`, `9`, `100`, text `9a`, `TRUE`, `=NA()`, empty | (none) | the list's order across kinds (desktop only) |

Remove Duplicates for each case: which rows are removed, and which of two "equal" rows is kept (we expect the first).

## What we will do with it

The findings go into `docs/excel-semantics/` (a new `autofilter-and-dedupe.md`), and the vectors of
`query.distinct@1` (`DISTINCT_MAX` 10,000 items) and `plan.dedupePlan@1` are written from them. Until then both stay
proposed in D-CONTRACTS-24, items 1 and 5.

## Added 2026-11-12 09:00Z: Find's matching (D-CONTRACTS-24 item 2, `query.find@1`)

Office.js `worksheet.findAll(text, {completeMatch, matchCase})` (and `range.find` for one hit). Record which cells
match, in order.

| Case | Cells (A1:A6) | Searches | Question |
|---|---|---|---|
| W1 | `abc`, `a*c`, `aXc`, `ac`, `a?c`, `A~C` | `a*c`, `a~*c`, `a?c`, `a~?c`, `~~` | `*` and `?` wildcards and the `~` escape |
| W2 | `Total`, `total`, `TOTAL`, `subtotal`, `Total ` (trailing space), `=UPPER("total")` | `total` with matchCase false/true and completeMatch false/true | case and whole-cell, and whether a formula's value matches |
| W3 | 1 with format `0.00`; 1.5; `=1/3`; text `0.33` | `1.00`, `0.33`, `0.333` | values vs displayed text (Look in: Values matches the shown text?) |
