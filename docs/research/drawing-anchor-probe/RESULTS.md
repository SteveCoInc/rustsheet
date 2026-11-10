# Results: where a chart's two-cell corners go on Excel for the web (2026-11-10)

Recorded through Microsoft Graph (Excel for the web), labelled as such, as README.md allows. There is one workbook
with one sheet per case. Each sheet has `A1:B4` charted with `charts/add` (`ColumnClustered`). The chart is moved to
mid C3 to mid H12 by setting left 120, top 37.5, width 240 and height 135 pt on the default grid (rows 15 pt,
columns 48 pt), and then the case's operation is applied through Graph. The run used `rowHeight`/`columnWidth`
(C1 sets column C to 150 pt), `rowHidden`/`columnHidden`, and `range/delete`. Excel saved the workbook as
`drawing-anchor.graph.xlsx` (sha256 c936c32a69be654dfb69a59021190f0712dff4d8f8642a02a9196c429fa1f663).
`drawing-anchor.graph.json` holds `xdr:from`/`xdr:to` from the saved drawing parts and the chart's box in points
before and after each operation. Recorder: `../chart-selection-probe/record_graph.py`. Desktop Excel was not run:
the README's question is about desktop, and a desktop-only difference goes in M14 (decision 0039).

The anchor Excel wrote is a `twoCellAnchor` with no `editAs`, from col 2 colOff 304800 row 2 rowOff 95250 to col 7
colOff 304800 row 11 rowOff 95250 (mid C3 to mid H12). An unchanged anchor below means that one.

| Case | Box after (left, top, width, height) | Saved anchor | Reading |
|---|---|---|---|
| R1 row 3 → 30 pt | 120, 37.5, 240, 150 | unchanged | the offset stays 7.5 pt in EMU, not half the row |
| R2 row 12 → 30 pt | 120, 37.5, 240, 135 | unchanged | same, at the bottom corner |
| R3 row 6 → 30 pt | 120, 37.5, 240, 150 | unchanged | control: grows by 15 |
| R4 row 3 → 5 pt | 120, 35.25, 240, 127.5 | unchanged (rowOff still 95250) | an offset past the row is clamped at its end, 5.25 pt (7 px) in |
| C1 col C → 150 pt | 120, 37.5, 342, 135 | unchanged | the column offset stays 24 pt in EMU |
| H1 hide row 3 | 120, 30, 240, 127.5 | unchanged | the corner sits at the hidden row's start |
| H2 hide row 12 | 120, 37.5, 240, 127.5 | unchanged | same, bottom corner |
| H3 hide row 6 | 120, 37.5, 240, 120 | unchanged | control: shrinks by 15 |
| H4 hide col C | 96, 37.5, 216, 135 | unchanged | the corner sits at the hidden column's start |
| D1 delete row 3 | 120, 30, 240, 127.5 | from row 2 **rowOff 0**; to row 10 | the corner moves to the start of the row that took its place, and the file says so |
| D2 delete row 12 | 120, 37.5, 240, 127.5 | to row 11 **rowOff 0** | same, bottom corner |
| D3 delete rows 3:12 | no chart | no anchor, no chart part | **the chart is deleted** |
| D4 delete col C | 96, 37.5, 216, 135 | from col 2 **colOff 0**; to col 6 | as D1, for columns |

Against the provisional rules in README.md:
- "Offsets are stored in absolute EMU, and an offset past its line's size is clamped at projection": both hold
  (R1, R2, C1, R4). The clamp in R4 lands at 5.25 pt in a 5 pt row, Excel for the web's pixel rounding of the row.
- "A hidden line puts the corner at its start": holds (H1, H2, H4).
- "None of these changes a stored value": holds for resize and hide. It does **not** hold for delete. Excel rewrites
  the deleted corner to offset 0 in the line that took its place (D1, D2, D4).
- "A deleted line with no survivor sends the corner to the end of the axis": does **not** hold. Excel deletes the
  chart when every row it spans is deleted (D3).
