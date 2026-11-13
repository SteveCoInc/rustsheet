# Probe request `web-load`: how Excel for the web opens a large workbook, and what it says past its size limit

> **val11 note:** **Part 1 only.** Part 2 is held: its generator (`generate.py --over-limit`) has a known fault (its small control workbook names a `docProps/app.xml` it lacks, so the comparison would be void), and the fix is pending. This change does not carry `generate.py`; Part 2 follows when it is fixed. Part 1 uses `load-26-sheets.xlsx`, which is here (`SHA256SUMS`).

*From poc-design-cc, 2026-11-12, for the Excel oracle owner (routed by the coordinator). Asked by the import
performance lane (poc-cfix-uiperf-cc): its requests P-4 and P-5, which feed items R1, R2 and R7 of
`docs/ui/design/import-load-remaining.md` on its branch `poc/cfix-uiperf-freeze-docs` (e51ccea2c, not on main
when this was written). Taskei P561757146. Nothing here has been run in Excel, and nothing waits for it.*

## What is asked, and what is not

**The size limit is documented**, so it is not asked. Read on 2026-11-12:

- The Excel for the web service description
  (https://learn.microsoft.com/en-us/office365/servicedescriptions/office-online-service-description/excel-online):
  "Workbooks that exceed 100 megabytes (MB) cannot be viewed in Excel for the web from within Microsoft
  SharePoint Online. To view larger files, you'll need the Excel desktop app installed on your computer."
- "File size limits for workbooks in SharePoint"
  (https://support.microsoft.com/en-us/sharepoint/platform/file-size-limits-for-workbooks-in-sharepoint): a table
  with the row "SharePoint Online | A library in a site such as a team site | 0-100 MB", and "If a workbook
  exceeds the file size limit that's set for your Microsoft 365 subscription, you won't be able to view it in a
  browser window."
- "Data Model specification and limits"
  (https://support.microsoft.com/en-us/excel/data-model-specification-and-limits), for Excel for the web in
  Microsoft 365: "250 megabytes (MB) total file size limit.", with everything outside the data model held to
  the SharePoint Online limit, and "Limits in Microsoft 365 are not configurable, but can change over time."

**Not documented on these pages, and asked here:**

1. In what order a large workbook appears: is the first sheet painted before the rest is loaded, and how long
   is it from the click to the first cells and to the last sheet (Part 1)?
2. What a person sees past the limit, word for word, and whether the limit is on the file's size alone or also
   on how many cells it holds (Part 2, optional: it needs a 119 MB upload).

## The workbooks

`python3 -I generate.py <dir>` writes the first; `python3 -I generate.py <dir> --over-limit` writes all three.

| File | What it is | Size |
|---|---|---|
| `load-26-sheets.xlsx` (sha256 `8dff6ddf3f472412afa6d69d7406808940992e2abaadfe8a5e29cdf0a1cfec64`) | 26 sheets, `Sheet1` to `Sheet26`, of 700 rows by 11 columns: 200,200 cells. Columns A to J are numbers, column K a text; each sheet has three number-format areas and two rows with a height of their own. | 7.4 MB |
| `over-100-mb.xlsx` | One sheet of 270,000 rows by 11 columns, 2,970,000 cells, written without compression. | 119,159,809 bytes |
| `same-cells-deflated.xlsx` | The same cells as `over-100-mb.xlsx`, compressed as Excel compresses a file. Its bytes depend on the Python that wrote it, so it has no sha256 here. | about 12 MB |

In `load-26-sheets.xlsx` every number says where it is: `sheet * 1,000,000 + row * 100 + column`, so Sheet3!B2
is 3,000,202. Column K reads `s3r2` there. `Sheet26!A1` is `=SUM(Sheet1:Sheet25!A1)`, and its value,
325,002,525, needs a cell of every other sheet. The file stores no value for it and asks Excel to calculate on
load. `python3 -I check.py` holds all of this without Excel. The import performance lane times this editor's own
open on the same file.

## How

By hand, in Excel for the web, with the file in a OneDrive or SharePoint library. Please note the date, the
browser, the machine, the kind of connection, and the build Excel reports (File > About). **Return** the notes as
`RESULTS.md` beside this file, with the recordings or the pictures.

### Part 1: `load-26-sheets.xlsx`, three times

Record the screen (any recorder will do), so that the times can be read from the recording, frame by frame;
a stopwatch is the fallback. Do the steps three times: once right after the upload, then twice more after
closing the tab.

| Step | Do | Read from the recording, in seconds from the click of 1.1 |
|---|---|---|
| 1.1 | Click the file's name in the library. | 0, by definition. |
| 1.2 | Wait, touching nothing. | When Excel's frame (the ribbon, the sheet tabs) first shows. When the first cell values of Sheet1 show. When any sign of loading or calculating goes away, and what the sign was. Are all 26 tabs there when the first values show? |
| 1.3 | As soon as the first values show, click the tab `Sheet26`. | When Sheet26's cells show. What A1 shows at that moment: 325,002,525, another number, or nothing yet; and, if not the number at once, when it becomes 325,002,525. |
| 1.4 | Go back to Sheet1 and press Ctrl+End. | When the cells around K700 show: at once, or after a wait with empty cells? |
| 1.5 | Click the tab `Sheet13`, which has not been shown yet. | When its cells show. |

If you are at ease with the browser's developer tools, one more reading on the third run helps and is optional:
with the Network panel open, the number of requests and the bytes received up to the first values of step 1.2,
and whether steps 1.3 to 1.5 each start new requests.

### Part 2 (optional): past the documented limit

Generate the two files with `--over-limit`, upload both, and open each.

| Step | Do | Write down |
|---|---|---|
| 2.1 | Upload `over-100-mb.xlsx` (119 MB) and click it. | Does the upload itself say anything? What Excel for the web shows, word for word, with a picture: a refusal and its message, an offer to open in the desktop app or to download, or the workbook. |
| 2.2 | Upload `same-cells-deflated.xlsx` (about 12 MB, the same 2,970,000 cells) and click it. | Does it open? If it does: how long to the first values, and what A270000 reads (28,000,001). If not: the message, word for word. |

## What the answers decide

The items are the import performance lane's. For every row, what Excel for the web does is the record the
design is compared with; this editor's own budgets are that lane's and the owner's to set.

| # | Question | Read from | Item |
|---|---|---|---|
| 1 | Is the first sheet painted before the rest is ready? | 1.2 against 1.3 and 1.5: if Sheet26 or Sheet13 show at once after the first values, the rest was ready; if each waits, sheets come as they are asked for | R2 (active sheet first), which says of Excel for the web "It renders the first viewport before the rest (recall, not verified)" |
| 2 | How long to the first values, and to a sheet that depends on all the others? | 1.2, 1.3, three runs | R2's target, and the comparison the lane's runner makes on the same file |
| 3 | Is a sheet's far end there with its start? | 1.4 | R2 (what "first viewport" means within a sheet) |
| 4 | What does a person see past 100 MB? | 2.1 | R1 (what an open that is too large says) |
| 5 | Is the limit the file's size alone? | 2.1 against 2.2: the same cells, refused at 119 MB and opened at 12 MB, is a limit on size and not on cells | R7, which says that Excel for the web bounds a workbook by its file size and not by its cells |

If 2.2 is refused too, the limit is not the file's size alone, and R7's sentence is revised with the message as
its evidence.
