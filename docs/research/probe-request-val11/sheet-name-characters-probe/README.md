# Probe request `sheet-name-characters`: what a sheet name may hold, and how a formula spells it

*From poc-design-cc, 2026-11-12, for the Excel oracle owner (routed by the coordinator). For Taskei P560852728,
which was filed as a desktop probe. No desktop Excel fills probes (the project lead's ruling of 2026-11-12;
decision 0266 rule 2), so this is a request for Excel for the web, which has the rename box, a formula bar and
Office.js. Nothing here has been run in Excel.*

## What is asked

`docs/excel-semantics/syntax.md`, "A sheet name may hold a control character", is VERIFIED for one thing: files
Excel wrote hold a tab or a line feed in a sheet name. The collaborative sheet's rule for a sheet name
(`wire::sheet_name`, I-02 D-M1-72) goes further on Excel's published rule alone: it allows a control character
in a name and refuses a blank name. Blank is what Rust's `str::trim` empties: spaces only, and also a tab alone
or U+00A0 alone (`a_blank_sheet_name_is_refused`, crates/rustsheet-contracts/src/wire/ranges.rs). Four things
have no evidence:

1. Does Excel take a control character other than a tab or a line feed in a sheet name (U+0001, U+001F, U+007F,
   U+0000)?
2. Does it take a name of spaces only, of a tab only or of U+00A0 only, and does it keep or trim spaces at the
   ends of a name?
3. How does Excel spell such a name in a formula that names the sheet, as shown and as stored in the file?
4. Does the rename box take what the API takes?

Three parts of the published rule ride along, since the session is there: the length (31 UTF-16 units here, 31
characters in `rustsheet-sheet-ops`: Taskei P560850786), an apostrophe at an end, and the name `History`.

## The candidates

The script asks for each of these names in turn. A name is logged as its UTF-16 code units, which the third
column gives.

| Id | Name | Code units | Why |
|---|---|---|---|
| `plain` | `Plain` | `80 108 97 105 110` | the control: a name Excel takes |
| `tab` | `a`, a tab, `b` | `97 9 98` | stored in files Excel wrote: does the API take it, and how is it spelled in a formula? |
| `line-feed` | `c`, a line feed, `d` | `99 10 100` | likewise |
| `u0001` | `e`, U+0001, `f` | `101 1 102` | a control character no file of the sample holds |
| `u001f` | `g`, U+001F, `h` | `103 31 104` | the last one below a space |
| `u007f` | `i`, U+007F, `j` | `105 127 106` | DEL, which is not below a space |
| `spaces-only` | three spaces | `32 32 32` | blank: the collaborative sheet refuses it |
| `tab-only` | a tab | `9` | blank by the collaborative sheet's rule, and a control character, which it allows inside a name |
| `nbsp-only` | U+00A0 | `160` | blank by the same rule, and not U+0020 |
| `outer-spaces` | a space, `k`, a space | `32 107 32` | kept or trimmed? |
| `astral-16` | U+1F600 sixteen times | `55357 56832 55357 56832 55357 56832 55357 56832 55357 56832 55357 56832 55357 56832 55357 56832 55357 56832 55357 56832 55357 56832 55357 56832 55357 56832 55357 56832 55357 56832 55357 56832` | 16 characters and 32 UTF-16 units: taken if Excel counts characters to 31, refused if it counts units |
| `apostrophe-end` | `o`, an apostrophe | `111 39` | the published rule refuses an apostrophe at an end |
| `history` | `History` | `72 105 115 116 111 114 121` | the published rule reserves it |
| `u0000` | `m`, U+0000, `n` | `109 0 110` | in a workbook of its own (case `NUL`), since a host may stop at it |

No two names hold the same letters. A host that drops a control character, puts a space in its place, trims or
stops at U+0000 would otherwise make two candidates one name, and the second would be refused as taken, not for
its character.

## How

**In Excel for the web, with Script Lab**, in a **new, empty workbook** (the script refuses any other):

1. Note the date, the browser and its version, and the build Excel for the web reports (File > About), if it
   reports one.
2. Paste `cases.js` into the Script tab, in place of what the tab holds, and this one line into the HTML tab:
   `<div id="cases"></div><pre id="out"></pre>`. Press Run and click `NAMES`. The click turns both buttons
   off: a workbook takes one case.
3. Copy the lines shown under the buttons into `results.jsonl`: one a candidate and a last one with the
   workbook's sheet names.
4. Look at the sheet tabs and at B1:B13 of the first sheet, and note anything that looks wrong (a tab that
   shows a box or nothing, a formula that shows an error). Click B2 and copy what the formula bar shows.
5. File > Save As > Download a Copy, as `sheet-names-script.xlsx`. This file is the evidence for what is
   stored: its sheet names and its formulas are read from it here.
6. In a second new workbook, open Script Lab again, press Run and click `NUL`. Add its two lines to
   `results.jsonl`, and download a copy as `sheet-names-nul.xlsx`. If Excel reports an error, or the workbook
   will not save or download, that is the answer: copy the message.

For each candidate the script asks Excel twice: it adds a sheet under the name (and deletes it), and it adds a
sheet and renames it, as the rename box does. It reads the name back each time. The sheet that took the name
holds the candidate's id in C1 and the candidate's row number in A1. The script then reads how Excel spells a
reference to that sheet's A1 and enters that spelling as a formula in column B of the first sheet, on the
candidate's row. A name Excel refuses is an answer: its line carries the error's code and message and the call
that failed, and the candidate's sheet keeps the name Excel gave it.

The script's part could also be recorded through Graph, labelled as Excel for the web: `worksheets/add` with a
name, a `PATCH` of a worksheet's `name`, a range's `address` and `formulas`, the list of worksheets, and the
saved file, in a session that persists, as `docs/research/chart-selection-probe/record_graph.py` recorded that
probe. The oracle tool's `sheet_ops` add and rename sheets through the same calls
(`tools/src/rustsheet_tools/oracle/backends.py`, `_replay_sheet_ops`). No such recorder is written here. Steps 7
to 12 stay by hand: no API drives the rename box.

**Then by hand, in a third new workbook**, for the rename box:

7. On the first sheet, type `a` in A1 and `b` in B1. Select A1:B1 and copy. Add a sheet, double-click its tab,
   paste, press Enter. Note the message, if there is one, and the tab's name. The first sheet keeps its own
   name: step 11 writes its formulas there.
8. On the first sheet, type `c` in A3 and `d` in A4. Select A3:A4 and copy. Add a sheet, double-click its tab,
   paste, press Enter. Note the same. A one-line box may drop what the clipboard holds between the two letters:
   the downloaded file says what the name holds.
9. Add a sheet and rename it to three spaces. Note the message, or the name it has.
10. Add a sheet and rename it to a space, `x`, a space. Is the tab's name ` x ` or `x`?
11. On the first sheet, click D1, type `=`, click the tab of step 7's sheet, click its cell A1 and press Enter.
    Do the same in D2, D3 and D4 for the sheets of steps 8, 9 and 10. Copy each formula as the formula bar shows
    it.
12. Download a copy, as `sheet-names-hand.xlsx`.

**Return** `results.jsonl`, the three downloaded files and the notes. They are committed beside this README
with a `RESULTS.md`.

`node check.js` runs the script against a fake of the parts of Office.js it uses: a host that takes every name;
hosts that refuse a control character at adding, at renaming or at both; hosts that change a name where others
refuse it (trim it, drop a control character, put a space in its place, stop at U+0000); every sync failing in
turn; a workbook that is not new and empty; and the buttons. It holds the table above to the script, code
units included. It needs no Excel and says nothing about what Excel does.

## What the answers decide

| # | Question | Read from | What it decides |
|---|---|---|---|
| 1 | Which control characters does Excel take in a sheet name? | the lines of `u0001`, `u001f`, `u007f`, `u0000`; the first two files' `<sheet name>` | whether `wire::sheet_name` allows every control character, as today, or only some |
| 2 | Blank names, and spaces at the ends | the lines of `spaces-only`, `tab-only`, `nbsp-only` and `outer-spaces`; steps 9 and 10 | whether a blank name is refused, as today, which names are blank, and whether a name is trimmed before it is checked |
| 3 | The spelling in a formula | each line's `address.ok` and `reader.ok.formula`; step 4; the first file's `<f>`; step 11 and the third file's `<f>` | how a formula that names such a sheet is printed and stored (`_xHHHH_` or the character itself) |
| 4 | The rename box against the API | steps 7 to 10 beside the lines of `tab`, `line-feed`, `spaces-only`, `outer-spaces` | whether a rule evidenced through the API holds for what a user types |
| 5 | Three published parts | the lines of `astral-16`, `apostrophe-end` and `history` | whether the length is counted in UTF-16 units or in characters (Taskei P560850786), and that the two refusals are Excel's |

Each answer becomes a rule of `docs/excel-semantics/syntax.md` under "A sheet name may hold a control
character", with this request as its evidence. If Excel for the web answers a question differently by the API
and by hand, both are recorded, and the rule for a user's edit follows the rename box.
