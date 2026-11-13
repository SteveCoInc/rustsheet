# Probe request: a defined name of an .xls that the xlsx grid reads as a cell

> **val11 note:** the fixture `namesdemo.xls` (xlrd 1.2.0's example, BSD-3-Clause) is not on this branch's base, so this change does not carry it; it is needed only for the fallback step. `python3 -I ../fetch_public_inputs.py xls-name-as-cell-probe .` downloads it from xlrd 1.2.0 and checks its sha256 (b2185646...). `name-as-cell.xls` and `name-as-cell-excel.xls` are here.

*Design desk (poc-design-cc), 2026-11-12, for Taskei P560018445 (a follow-up of T-5384). Nothing here has been run
in Excel. The request is for Excel for the web: it was first written for desktop Excel for Windows, and the
project lead ruled the same day that only Excel for the web fills probes (as decision 0266 rule 2 has it for the
oracle). What only the desktop can show is under "No oracle" below. The answers go into `RESULTS.md` beside this file.*

## What is asked

BIFF8's grid ends at IV65536, so `TAX2024` or `USA1` is a legal name in an .xls: columns TAX and USA are past IV.
In an .xlsx it is a cell: column TAX, row 2024. (`FY2007` is no such name: FY is column 181, a cell of both
grids.) What does Excel do with such a name, and with the formulas that use it, when it converts the workbook
to .xlsx?

Today the .xls reader skips such a name, and the formulas that use it keep their cached values
(docs/excel-semantics/xls-import.md, "A defined name the xlsx grid reads as a cell", status UNVERIFIED). T-5384
counted the names the parser reads as no name, of which these are one kind (its outcome does not split them): 40
of the sample's 2,519 BIFF8 files hold one, and 322 formulas in 5 files keep their values for it. If Excel renames
the name and its uses, the reader can do the same and those formulas calculate.

Microsoft documents a rename. "Use Excel with earlier versions of Excel"
(support.microsoft.com/en-us/office/use-excel-with-earlier-versions-of-excel-2fd9ffcb-6fce-485b-85af-fecfd651a5ac,
read 2026-11-12) says that names such as `USA1`, `FOO100` and `MGR4` conflict with cell references in Excel 2007
and later, that Excel 2007 and later also reserves names that start with XL, and that on conversion to the new
file format "you will be alerted about the conflict" and "an underscore (_) is automatically added as a prefix to
the incompatible names"; "external workbook references and functions that take string references (such as
INDIRECT) are not updated". A page is not evidence here (docs/excel-semantics/README.md), so the request stands:
questions 1, 2, 9 and 11 have a documented answer to confirm, and the page does not answer the others (a
constant, the ends of the grid, a collision, a name's own definition, a sheet's name, a cell spelled in R1C1, a
lower-case spelling). The Office 2007 Resource Kit's "Changes in Office Excel 2007" (learn.microsoft.com,
previous-versions, cc179167) lists the larger grid and says nothing about names.

That page describes the desktop's conversion. Excel for the web converts too: "Excel 97-2003 workbook files
(.xls). Note: When you open this file format, Excel for the web will convert it to a newer .xlsx file. You can
always download the original by going to File > Info > Previous versions" ("Differences between using a workbook
in the browser and in Excel",
support.microsoft.com/en-us/office/differences-between-using-a-workbook-in-the-browser-and-in-excel-f0dc28ed-b85d-4e1d-be6d-5878005db3b6,
read 2026-11-12). No page says whether the web's conversion renames as the desktop's does: that is what this
request finds out. Where the two differ the rule follows the web (decision 0039 rule 4).

## The workbook

Two sheets, S and T. S!A1 holds 5.

| # | Name | Scope | Refers to | Why it is here |
|---|---|---|---|---|
| 1 | `TAX2024` | workbook | `=S!$A$1` | a cell of the xlsx grid, defined as a reference |
| 2 | `USA1` | workbook | `=7` | a cell of the xlsx grid, defined as a constant; Microsoft's own example |
| 3 | `Rate` | workbook | `=0.5` | control: a name in both grids |
| 4 | `XFD1` | workbook | `=3` | the xlsx grid's last column |
| 5 | `XFE1` | workbook | `=4` | one column past it: a name in both grids |
| 6 | `IW1` | workbook | `=5` | the first column past BIFF8's IV |
| 7 | `A65537` | workbook | `=9` | the first row past BIFF8's 65536 |
| 8 | `_TAX2024` | workbook | `=13` | the name a rename of name 1 would want |
| 9 | `Total` | workbook | `=TAX2024*2` | a name defined with name 1 |
| 10 | `TAX2024` | sheet T | `=11` | the same spelling, scoped to a sheet |
| 11 | `XLRate` | workbook | `=2` | starts with XL, which Microsoft's page says Excel 2007 and later reserves; a cell of neither grid |
| 12 | `R1C300` | workbook | `=4` | a cell of the xlsx grid spelled in R1C1, past BIFF8's 256 columns |
| 13 | `A1048576` | workbook | `=6` | the xlsx grid's last row |
| 14 | `A1048577` | workbook | `=8` | one row past it: a name in both grids |
| 15 | `tax2025` | workbook | `=10` | a cell of the xlsx grid, in lower case |

| Cell | Formula | Value in the .xls |
|---|---|---|
| S!B1 | `=TAX2024+1` | 6 |
| S!B2 | `=USA1*2` | 14 |
| S!B3 | `=Rate*2` | 1 |
| S!B4 | `=XFD1+XFE1` | 7 |
| S!B5 | `=SUM(TAX2024,USA1)` | 12 |
| S!B6 | `=Total` | 10 |
| S!B7 | `=INDIRECT("TAX2024")` | 5 |
| S!B8 | `=IW1+A65537` | 14 |
| S!B9 | `=_TAX2024` | 13 |
| S!B10 | `=XLRate*2` | 4 |
| S!B11 | `=R1C300+1` | 5 |
| S!B12 | `=A1048576+A1048577` | 14 |
| S!B13 | `=tax2025+1` | 11 |
| S!B14 | `=T!TAX2024` | 11 |
| T!A1 | `=TAX2024` | 11 |

The third column is what the formulas give by their definitions. No Excel has computed it: steps 4 and 6 ask.

S!B14 reaches the sheet's name from another sheet, and is stored as Excel stores that: `PtgNameX` through an
`ExternSheet` entry for the workbook, where T!A1, on the name's own sheet, is `PtgName`.

## The second workbook, one Excel wrote

`name-as-cell-excel.xls` goes with it. It is crates/rustsheet-xls/tests/fixtures/namesdemo.xls, a workbook Excel
wrote (from xlrd 1.2.0's examples, BSD 3-clause: the README beside the fixture), with one of its 34 names spelled
`USA1`, Microsoft's own example of such a name, where the fixture has `vrai`. That is four bytes in place: every
other byte is Excel's, and the names stay in Excel's order (`Union`, `USA1`, `Year_Tot`).

| Name | Scope | Refers to | Used by | Stored value |
|---|---|---|---|---|
| `USA1` | workbook | `=TRUE` | Sheet3!A17, `=USA1` | TRUE |

Sheet3 is the sheet it opens on. Its other 33 names and 27 formulas are the example's, and this repository's
reader carries them all. One of the 27, Sheet3!A6, is a natural-language label formula, which Excel 2007
removed: "When a workbook that contains NLFs is opened in Office Excel 2007 … the user is alerted by the
program that NLFs are not supported, and that the NLFs will be converted to static cell references if the
user continues the operation" ("Changes in Office Excel 2007", cited above). If Excel for the web says so,
continue: that message is about Sheet3!A6, not about names (docs/excel-semantics/xls-import.md, "A
natural-language label resolves to the reference after it").

It is here for two reasons. It gives questions 1 to 3 an answer on a file Excel wrote: a constant's name, used by
one formula. And it tells a refusal of the generated file from a refusal of .xls conversion (question 15).

## What to do

**In Excel for the web.** There is no way to write an .xls there, so there are two files (sha256 of each in "The
generated files" below). `name-as-cell.xls` was written by this repository's test writer, not by Excel, and no
Excel has opened it. `name-as-cell-excel.xls` is Excel's but for four bytes.

0. Note the date, and the build Excel for the web reports (File > About), if it reports one.
1. Upload both files to OneDrive or to a SharePoint library, and open `name-as-cell.xls` in Excel for the web.
2. Copy every message word for word, with the buttons it offers and the one chosen: about the conversion, about
   names, about anything else. If it shows the .xls for viewing only and offers to convert it for editing,
   accept. If Excel for the web says the file is damaged or cannot be opened, that says nothing about names:
   report the message and go on with the second file at step 7 (question 15).
3. When the workbook is open, do not edit it. Note the file's name in the title bar and in the folder (is there
   an .xlsx now, and is the .xls still there?), and whether File > Info > Previous versions lists the original.
4. Note, as shown: each formula's text and value in S!B1 to S!B14 and in T!A1, and the names with their scope and
   what they refer to, if this Excel for the web has Formulas > Name Manager.
5. File > Save As > Download a Copy, as `name-as-cell-web.xlsx`. This file is the evidence: its `definedNames`
   and its formulas are read from it here, so steps 3 and 4 only need to be rough.
6. Run the oracle tool on the downloaded file, as for any other request, and keep its output:
   `rustsheet-oracle recalc -b excel-graph name-as-cell-web.xlsx`. It gives each formula's value after a full
   calculation by Excel for the web, which the values stored in the download need not be.
7. Open `name-as-cell-excel.xls` in Excel for the web and do steps 2 to 6 with it. In step 4, note the formula's
   text and value in Sheet3!A17, and what the name `USA1` is called now. In step 5, download it as
   `name-as-cell-excel-web.xlsx`, and run the tool of step 6 on that file. If this file is refused too, or
   opens with any message, upload the fixture itself as well
   (crates/rustsheet-xls/tests/fixtures/namesdemo.xls, sha256
   `b2185646afdfcd47c0a2a10122690ac5ef1c0bd7e5f916887f57bfcd2c5ab5ba`: the same workbook without `USA1`), open
   it, and note what Excel for the web says: what it says of both files is not about the name. If it converts
   neither, the message is the answer: Excel for the web did not convert an .xls that Excel wrote.

**Return** `name-as-cell-web.xlsx` and `name-as-cell-excel-web.xlsx`, the oracle tool's output for each, and the
notes.

## No oracle

Two questions of the first request, and one of its steps, need desktop Excel. Nothing waits for them.

- *What Compatibility Mode lets a user define, and what its dialogs say* (question 10). It would have worded the
  reader's warning, which keeps its own words. It would also have said which of the fifteen names Excel 97-2003
  accepts at all: Microsoft's page vouches only for names such as `USA1`, so an answer for names 4 to 7 and 11
  to 14 is an answer about the generated file, which Excel may never write.
- *Whether Save As and Convert write the same names and formulas* (question 14). The reader follows the
  conversion Excel for the web makes, the only one that can be observed.
- *A workbook that Excel itself wrote as .xls, with these fifteen names.* The generated file stands in for it.
  If Excel for the web will not open the generated file, that is a fault of the file and no answer about names:
  the request stays open, and the file Excel wrote, `name-as-cell-excel.xls`, still answers questions 1 to 3.
  Only if Excel for the web converts no .xls that holds such a name does the rule have no oracle: the entry in
  docs/excel-semantics/xls-import.md is then restated as `NO-ORACLE` on Microsoft's page, in the form
  docs/excel-semantics/README.md gives, and the reader may follow the page (decision 0464).

## What the answers decide

| # | Question | Read from | What it decides |
|---|---|---|---|
| 1 | What is name 1 called in the .xlsx: `TAX2024`, `_TAX2024`, something else, or is it gone? | `definedNames` | whether the reader renames, and how |
| 2 | What does S!B1's formula say, and its value? | sheet S | whether uses are rewritten with the name |
| 3 | The same for name 2, a constant, and S!B2 | both | whether the rule depends on what the name refers to |
| 4 | Are `XFD1` and `XFE1` treated differently, and `A1048576` and `A1048577`? | `definedNames`, S!B4, S!B12 | that the test is "a cell of the xlsx grid", and where the grid ends on both sides |
| 5 | `IW1` and `A65537` | `definedNames`, S!B8 | the first column and the first row past BIFF8's grid |
| 6 | What happens to name 1 when `_TAX2024` exists already? | `definedNames`, S!B1, S!B9 | the rule for a collision |
| 7 | What does `Total` refer to afterwards? | `definedNames` | that a name's own definition is rewritten |
| 8 | The sheet's own `TAX2024` on T, used on T and from S | `definedNames` (`localSheetId`), T!A1, S!B14 | that the rule is per name, whatever its scope and however a formula writes it |
| 9 | Is the text in `INDIRECT("TAX2024")` changed? What is S!B7 as shown after the conversion, as stored in the download, and after the oracle tool's full calculation? | sheet S, the notes of step 4, the tool's output | whether such a formula changes meaning in Excel's own conversion (Taskei P560887404 waits for it) |
| 10 | Does Excel in Compatibility Mode let you define these names at all, and what do its dialogs say? | no oracle | nothing |
| 11 | Is `XLRate` renamed, and S!B10 with it? | `definedNames`, S!B10 | whether a name that starts with XL is renamed too (the reader writes it as it is) |
| 12 | Is `R1C300` renamed, and S!B11 with it? | `definedNames`, S!B11 | whether the rule covers a cell spelled in R1C1 (the reader skips such a name) |
| 13 | Is `tax2025` renamed, and how is it spelled afterwards? | `definedNames`, S!B13 | that the test ignores case and the rename keeps it |
| 14 | Do Save As and Convert give the same names and formulas? | no oracle | nothing |
| 15 | Does Excel for the web open the generated file, convert it on open, and say anything about names? | the notes of steps 2 and 3 | whether the generated file can stand in for one Excel wrote (if not, the file Excel wrote still answers questions 1 to 3: "No oracle" above), and what a user of Excel for the web is told |
| 16 | In the file Excel wrote: what is `USA1` called in the .xlsx, and what are Sheet3!A17's formula and value? Is that what name 2 and S!B2 of the generated file get? | both downloads (`definedNames`, Sheet3!A17, S!B2), the tool's output | questions 1 to 3 on a file Excel wrote; if the two files disagree, the answers on the generated file are about that file, and names 4 to 15 need files Excel wrote |

When the answers are in, they go to `RESULTS.md` here and to docs/excel-semantics/xls-import.md under "A defined
name the xlsx grid reads as a cell", with this request as the evidence. The reader's change is the xls lane's.

## The generated files

`generate.rs` beside this file writes `name-as-cell.xls` with `rustsheet_xls::test_support`, the writer the xls
crate's tests use, and `name-as-cell-excel.xls` from the fixture, and prints what this repository's reader makes
of each. It is not a member of the workspace. To run it with the workspace's locked dependencies:

```
mkdir /tmp/name-as-cell && cd /tmp/name-as-cell && cargo init --name generate
cp <repository>/docs/research/xls-name-as-cell-probe/generate.rs src/main.rs
cargo add --path <repository>/crates/rustsheet-xls --features test-support
cargo add cfb@=0.16.0
cp <repository>/Cargo.lock .
cargo run -- name-as-cell.xls
cargo run -- --excel-authored <repository>/crates/rustsheet-xls/tests/fixtures/namesdemo.xls name-as-cell-excel.xls
```

It gives `name-as-cell.xls`, 5,632 bytes, sha256
`8e5f118f3f62bba21eb12b4c7e3c261e352f3e4f44fe20503bb3af0873ef385a`. Only sheet S is selected and displayed in it.
And it gives `name-as-cell-excel.xls`, 22,528 bytes, sha256
`44ab5070c5daeb66151ee10ecac396f98177df3a6798f3a82d076ad988a20727`, which differs from the fixture in four bytes.

The generated file's container is a version 3 compound file with 512-byte sectors, which is what Excel writes for
an .xls. The test writer's own `Workbook::file` writes version 4 with 4,096-byte sectors, and the first file of
this request (16,384 bytes, sha256 `e7def7b1…`) was one. This repository's reader takes both. xlrd 2.0.1, a
reader written against Excel's files, refused the first ("file size (16384) not 512 + multiple of sector size
(4096)", then "directory corruption") and opens this one: BIFF 8, code page 1200, sheets S and T, the fifteen
names with their scopes, and the sixteen cells with the values of the table above. That is one reader more, not
Excel: question 15 is still open.

Each `BOF` record carries the build fields of an Excel-authored file (those of
crates/rustsheet-xls/tests/fixtures/namesdemo.xls: build 0x1FA9, year 0x07CD, last edited on Windows, lowest
BIFF version 6). The test writer's `bof` leaves them 0, which MS-XLS 2.4.21 rules out ("rupYear … MUST be
0x07CC or 0x07CD") and so does OpenOffice.org's format documentation, section 5.8.1 ("Build identifier, must
not be 0", "Build year, must not be 0"). This repository's reader gives the same .xlsx for either, and xlrd
opens either.

Of the file Excel wrote, xlrd 2.0.1 reads the four sheets, 34 names with `USA1` as the 32nd, defined as TRUE,
and TRUE in Sheet3!A17. This repository's reader reports 4 sheets, 61 cells and "1 defined name(s) a formula
reads as no name (a cell, a range): skipped": it writes the other 33 names and 27 of the 28 formulas, and
Sheet3!A17 keeps its stored value. Of the fixture as it is, it writes all 34 and all 28.

What this repository does with the generated file (measured on main 621a8367b and f59e216d3 with the first
file; the reader's .xlsx of this file is the same, byte for byte, on main 182efc31e; not Excel):

- The reader loads it and reports 2 sheets, 16 cells, "9 defined name(s) a formula reads as no name (a cell, a
  range): skipped" and "a defined name uses a defined name that is skipped: skipped". Of the fifteen names it
  writes five (`Rate`, `XFE1`, `_TAX2024`, `XLRate`, `A1048577`), and of the fifteen formulas four (S!B3, S!B7,
  S!B9 and S!B10); the other eleven formula cells keep their stored values.
- Recalculated by the engine, the reader's workbook gives the stored values in every cell of sheet S but one:
  S!B7, `=INDIRECT("TAX2024")`, is 0 where the file stores 5. The text is not a use of the name the reader can
  see, so the formula is carried, and it then reads cell TAX2024 of the xlsx grid, which is empty. Microsoft's
  page says Excel's own conversion leaves such text alone, which would give 0 there too; question 9 asks. The
  reader's side of it is Taskei P560887404 (the xls lane), which waits for that answer.
