# Probe request: where a chart's two-cell corners go when rows and columns are resized, hidden or deleted

*From the charts work, 2026-11-10, for the Excel oracle owner. Our object placement rules are provisional until this is answered. Not
urgent: nothing ships on it yet.*

## The questions

A chart anchored `twoCellAnchor` with no `editAs` (move and size with cells, as Excel writes a new chart) has each
corner at a cell plus an EMU offset into it (`xdr:from` and `xdr:to`: `col`, `colOff`, `row`, `rowOff`).
1. **Resize.** When the row (column) that holds a corner changes size, does Excel keep the corner's offset in EMU,
   keep it at the same fraction of its cell, or something else? When the row shrinks below the offset, where does
   the corner go?
2. **Hide.** On a hidden row (column), where does the corner go, and what does the saved file say?
3. **Delete.** When the row (column) that holds a corner is deleted, where does the corner go? When every row the
   chart spans is deleted, does the chart survive?

## Our provisional rules (what our placement code does today)

- Offsets are stored in absolute EMU, and an offset past its line's size is clamped at projection.
- A hidden line puts the corner at its start.
- A deleted line with no survivor sends the corner to the end of the axis.
- None of these changes a stored value.

## How (either way)

- **Office.js:** run `cases.js`, one case per fresh workbook, or one sheet each. Then save
  (`drawing-anchor.golden.xlsx`).
- **Graph:** the same operations through `workbook/worksheets/{s}/charts/add`, `range(address='3:3')/format`
  (`rowHeight`, `columnWidth`), `rowHidden`/`columnHidden`, and `range/delete` (`shift: Up` or `Left`). Use a session
  with `persistChanges: true`, then download.
- For every case we need `xdr:from` and `xdr:to` from the saved drawing part, and `chart.top/left/width/height` in
  points before and after the operation (`cases.js` logs them).
- The question is about the desktop product. Excel for the web (Graph) is welcome as a first set, labelled as such.
  The excel-probe rule puts a desktop-only difference in M14.

## The cases

Every case starts from the same chart. The default grid is Calibri 11: rows 15 pt, columns 8.43 characters
(48 pt). The chart's top-left corner is in the middle of C3 (`row 2`, `rowOff` half a row; `col 2`, `colOff` half a
column), and its bottom-right corner in the middle of H12.

| Case | Operation | Asks |
|---|---|---|
| R1 | row 3 (the top corner's) to 30 pt | 1 |
| R2 | row 12 (the bottom corner's) to 30 pt | 1 |
| R3 | row 6 (spanned, no corner) to 30 pt | control: the chart grows by 15 pt |
| R4 | row 3 to 5 pt (below the 7.5 pt offset) | 1, the shrink |
| C1 | column C (the left corner's) to 20 characters | 1, columns |
| H1 | hide row 3 | 2 |
| H2 | hide row 12 | 2 |
| H3 | hide row 6 | control: the chart shrinks by 15 pt |
| H4 | hide column C | 2, columns |
| D1 | delete row 3 (shift up) | 3 |
| D2 | delete row 12 | 3 |
| D3 | delete rows 3:12 (every row the chart spans) | 3, no survivor |
| D4 | delete column C (shift left) | 3, columns |
