# Probe request `render-pixels`: how Excel for the web draws a border, a gridline and a glyph, pixel by pixel

*From poc-design-cc, 2026-11-12, for the Excel oracle owner (routed by the coordinator). Asked by the grid's
WebGL lane (poc-render-gl-cc): requests P-RGL-1, P-RGL-2 and P-RGL-3 of `docs/ui/specs/render-webgl/SPEC.md`,
section "Probe requests", on its branch `docs/ui/render-gl-freeze` (8f7eb280f, not on main when this was
written). Taskei P561784965. Nothing here has been run in Excel. None of the three blocks that lane: each
decides a default.*

## What is asked, and what is not

Three things that only a screenshot at the screen's own resolution can show:

1. Whether a thin border and a gridline stay exactly one device pixel wide when the screen's scale or the
   sheet's zoom is not a whole number (Part 1).
2. Whether cell text is drawn with coloured edges (LCD antialiasing, ClearType) or grey ones (Part 2).
3. Whether text at a sheet zoom is drawn sharp at its zoomed size, and whether the spacing of its letters
   scales exactly or is rounded at each size (Part 3).

These are properties of the drawing, not behaviour Microsoft documents as such, and I did not search its pages
for them.

**Not asked:** the lane's P-RGL-2 also wanted the same text in desktop Excel 365 for comparison. No desktop
Excel fills probes (the project lead's ruling of 2026-11-12, decision 0464), so the comparison is left out.

**What it needs:** a Windows machine, Chrome and Edge, and a display whose scale can be set in Settings >
System > Display > Scale. If only a Mac is at hand, say so: Part 1 is then run at the scales the Mac's
displays have, Part 2 is not run (ClearType is Windows's), and Part 3 is run as written.

## The workbook

`render-pixels.xlsx` (sha256 `6e1c8e6a820daaff468c55c7261e7aa026c1ea5c5411eeb27c50f2f07e2f90c2`),
written by `python3 -I generate.py <dir>`; `python3 -I check.py` holds what follows without Excel.

Sheet `Borders`. Row 1 names the style of the boxed cell below it; every other cell is empty and unformatted,
so the default gridlines show between them.

| Cell | Its border, on all four sides |
|---|---|
| `B2` | thin |
| `D2` | medium |
| `F2` | thick |
| `H2` | double |
| `J2` | dashed |

B4:J4 have a thin line under them and nothing else: one long thin border, beside the gridline of the row
below.

Sheet `Text`, column A, black Calibri 11, the column 60 characters wide:

| Cell | Text | Fill |
|---|---|---|
| A1 | `Hamburgefonstiv 0123456789` | none |
| A2 | `Hamburgefonstiv 0123456789` | yellow |
| A3 | `illiill WAVE To AV fi fl` | none |
| A4 | `abcdefghijklmnopqrstuvwxyz` | none |
| A5 | `ABCDEFGHIJKLMNOPQRSTUVWXYZ` | none |

## How to take a screenshot that can be measured

Every picture is measured pixel by pixel, so each must be the screen's own pixels:

- the browser's own zoom at 100% (Ctrl+0), which is not the sheet's zoom at the bottom right of Excel;
- taken with Windows's own tool (Win+Shift+S, or PrtScn) and saved as **PNG**, never JPEG, and not resized,
  pasted into a document, or sent through a chat that recompresses pictures;
- with it, one line of text: the screen scale set in Windows, the sheet zoom, the browser and its version, and
  what `window.devicePixelRatio` answers in the browser's console (F12, Console, type it, Enter).

After changing the screen's scale, close and reopen the browser before taking the next picture. Please name
each file as the tables say, and **return** the pictures with a `RESULTS.md` that holds the lines of text.

### Part 1: borders and gridlines (P-RGL-1; sheet `Borders`, Chrome)

Nine pictures of the area A1:K5, one for each screen scale and sheet zoom. Click a cell far from it first
(M10), so that no selection box lies over a border.

| Screen scale | Sheet zoom 85% | Sheet zoom 100% | Sheet zoom 115% |
|---|---|---|---|
| 100% | `b-100-85.png` | `b-100-100.png` | `b-100-115.png` |
| 125% | `b-125-85.png` | `b-125-100.png` | `b-125-115.png` |
| 150% | `b-150-85.png` | `b-150-100.png` | `b-150-115.png` |

If Excel's zoom control does not offer 85% or 115%, use the nearest it gives on each side of 100%, and name
the file by the zoom used.

### Part 2: text antialiasing (P-RGL-2; sheet `Text`, screen scale 100%, sheet zoom 100%)

Before the pictures, check that ClearType is on (Windows search: "Adjust ClearType text"), and say so.

| Browser | Picture of A1:A5 |
|---|---|
| Chrome | `t-chrome.png` |
| Edge | `t-edge.png` |

### Part 3: text at a sheet zoom (P-RGL-3; sheet `Text`, screen scale 100%, Chrome)

Five pictures of row 1 alone, the whole line of A1 in each.

| Sheet zoom | Picture |
|---|---|
| 50% | `z-50.png` |
| 75% | `z-75.png` |
| 100% | `z-100.png` |
| 125% | `z-125.png` |
| 200% | `z-200.png` |

## What the pictures decide

The WebGL lane measures the pictures; this request only gathers them. The items are that lane's design items.

| # | Measured | From | What it decides |
|---|---|---|---|
| 1 | The width in device pixels of B2's thin border, of the line under B4:J4 and of a gridline, and whether any has a grey pixel beside it | the nine pictures of Part 1 | Design item 3: whether geometry is snapped to device pixels at every scale and zoom. One pixel with no fringe in all nine says Excel snaps; two blurred pixels at 125%, 150%, 85% or 115% says it does not |
| 2 | How medium, thick, double and dashed are built from device pixels at each scale | the same | the same item, for the other four styles |
| 3 | Of the pixels on the edge of a glyph, how many have red, green and blue that differ | `t-chrome.png`, `t-edge.png`, on white and on yellow | Design item 4: LCD antialiasing if they differ, greyscale if they do not, and whether a fill changes it |
| 4 | Whether glyph edges at 50%, 75%, 125% and 200% are as sharp as at 100% | Part 3 | whether each zoom has its own rasterization or a scaled picture of another |
| 5 | The distance between the same two letters at each zoom, against the zoom | Part 3 | whether advances scale exactly or are rounded at each size: whether text under a scale follows the browser's canvas or a rule of Excel's |

What Excel for the web draws is the record these defaults are compared with. Where the grid then differs, the
lane revises its default, or the difference is a known gap with a follow-up task (decision 0052).
