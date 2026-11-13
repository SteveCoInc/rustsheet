# Probe request: table banding when a data row is hidden

From poc-ui-cc, 2026-11-12. For the UI's table styles: does a hidden row take part in the stripe order of Banded Rows?

## Steps (Excel for the web, by hand, about 10 minutes)

1. A new workbook. In A1:C11 put a header row (`Name`, `Qty`, `Note`) and ten data rows (any values; make `Name` in
   row 6 the only `x`).
2. Select A1:C11 and Insert > Table (My table has headers). Keep any built-in style with Banded Rows on (the default
   is fine).
3. **Case (a), hidden by a filter.** Filter `Name` to exclude `x`, so row 6 is hidden. Take a screenshot of rows 1-11.
4. Clear the filter so every row shows again.
5. **Case (b), hidden by hand.** Right-click the row 6 header and choose Hide Rows. Take a screenshot of rows 1-11.

## The question

In each screenshot, look at rows 5 and 7 (the rows on either side of the hidden one):

- they have **different** stripes: Excel bands the visible rows alternately; or
- they have **the same** stripe: each row keeps the stripe of its position in the table.

The two cases may differ. Wanted back: the two screenshots, and the style name used.
