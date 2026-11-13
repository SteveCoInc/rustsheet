# Probe request `protection-move`: what a cut and insert does to locked, hidden and allow-edit ranges

*From poc-design-cc, 2026-11-12, for the Excel oracle owner (routed by the coordinator). For Taskei P560075187,
whose answer decides the design of Taskei P559954551 (a protection record does not follow a block moved whole).
No desktop Excel fills probes (the project lead's ruling of 2026-11-12; decision 0266 rule 2), so this is a
request for Excel for the web. Nothing here has been run in Excel.*

## What is asked

A sheet's protection rests on three things: a cell's `locked` flag and its `hidden` flag (hide the formula),
which are parts of the cell's format, and the sheet's allow-edit ranges, each of which is a reference.
`docs/excel-semantics/references.md` says that cut cells take their format along, and gives the file format as
its only ground. The collaborative sheet moves rows by cut and insert, and its protection records name the
first and last line of a block (decision 0375), so it has to know:

1. When a row is cut out of a block and inserted elsewhere, do its `locked` and `hidden` flags go with it?
2. What becomes of an allow-edit range when an inner row leaves it, when its first or last row leaves it, when
   all of it is moved, and when a row from outside is inserted into it?
3. Does a plain reference to the same rows (`=SUM(B10:B14)`) change in the same way as the allow-edit range?

## The workbook

`SETUP` adds five sheets, one for each move below. On each sheet, A1:A30 hold the labels `r1` to `r30` (a row's
label is the row it started in), B1:B30 hold 1, and rows 10 to 14 are the block: A10:C14 is unlocked, has its
formulas hidden, and is the allow-edit range `Grant`. E1 is `=SUM(B10:B14)`. The sheets are not protected: a
protected sheet refuses the moves.

| Sheet | Cut | Insert cut cells above | What it asks |
|---|---|---|---|
| `INNER` | row 12 | row 25 | an inner row of the block moved out |
| `WHOLE` | rows 10 to 14 | row 25 | the whole block moved |
| `FIRST` | row 10 | row 25 | the block's first row moved out |
| `LAST` | row 14 | row 25 | the block's last row moved out |
| `INTO` | row 25 | row 12 | a row from outside moved into the block |

## How

**In Excel for the web, with Script Lab**, in a **new, empty workbook** (`SETUP` refuses any other):

1. Note the date and the build Excel for the web reports (File > About), if it reports one.
2. Paste `cases.js` into the Script tab and this one line into the HTML tab:
   `<div id="cases"></div><pre id="out"></pre>`. Press Run and click `SETUP`. Leave Script Lab's pane open.
3. On each of the five sheets, make that sheet's one move from the table: select the row or rows by their
   headers, cut (Ctrl+X), right-click the header of the row in the third column, and choose **Insert cut
   cells**. If Excel for the web has no such command, say so and stop: that is the answer, and the rule then
   has no oracle. Copy any message word for word.
4. Click `READ`. Copy the two lines shown under the buttons into `results.jsonl`.
5. On sheet `WHOLE`, open Review > Manage Protection (or what this build calls it) and note the range it shows
   for `Grant`, as a check on what the script read.
6. File > Save As > Download a Copy, as `protection-move-web.xlsx`: its `protectedRanges` and its cells'
   formats are read from it here.

While a case runs both buttons are off; afterwards its own button and the one before it stay off, since the
two run once each, in order. If Script Lab's pane was closed before `READ`, open it again and press Run: `READ`
works in a workbook `SETUP` has built.

Only the move itself needs a person. No call of Office.js is Excel's cut and insert: `Range.moveTo` "Moves cell
values, formatting, and formulas from current range to the destination range, replacing the old information in
those cells", and `Range.insert` shifts cells without moving any (`@types/office-js` 1.0.617). What `SETUP`
writes and `READ` reads could be recorded through Graph as well; no recorder is written here.

`READ` logs, for each sheet: `rows`, each row that holds a label or whose flags are not a plain cell's, as
`[the row now, its label, locked, formulaHidden]`; `grants`, each allow-edit range's title and address; and
`sum`, E1's formula and value. `SETUP`'s line says for each sheet whether the allow-edit range was added. If
this Excel refuses allow-edit ranges, both lines carry the error's code, message and the call that failed in
`grants`, and the flags and the SUM are still set up and read: questions 1 and 3 are then answered and question
2 is not.

**Return** `results.jsonl`, the downloaded file and the notes. They are committed beside this README with a
`RESULTS.md`.

`node check.js` runs the script against a fake of the parts of Office.js it uses, with a model of the rows, the
allow-edit range and the SUM. It makes the five moves on the model under two rules (everything travels with the
rows; nothing does) and holds `READ` to what the model then is, so that `READ` reports either answer; and it
runs both cases on a host that has no allow-edit ranges. It needs no Excel and says nothing about which rule
is Excel's.

## What the answers decide

| # | Question | Read from | What it decides |
|---|---|---|---|
| 1 | Do the flags go with a cut row? | `rows` of `INNER`, `FIRST`, `LAST` (the moved row's flags at its new place) and of `INTO` (the row moved in) | whether a locked or hidden format follows its cells, as references.md says from the file format |
| 2 | What becomes of the allow-edit range? | `grants` of each sheet; step 5; the file's `protectedRanges` | how a protection record follows a block moved whole, and what a partial move leaves it covering (Taskei P559954551) |
| 3 | Does a plain reference change in the same way? | `sum` beside `grants` on each sheet | whether an allow-edit range can be kept as a reference is kept |

Each answer becomes a rule of `docs/excel-semantics/references.md`, with this request as its evidence.
