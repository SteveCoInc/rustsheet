# Probe request: a formula cell whose cached value is typed as a shared string

*Design desk (poc-design-cc), 2026-11-12, for Taskei P559389085. Nothing here has been run in Excel. The request
is for Excel for the web: one step through the oracle tool, one by hand. The answers go into `RESULTS.md` beside
this file.*

## What is asked

Excel writes a formula's text result as `t="str"`, with the text in `<v>`. `<c t="s"><f>…</f><v>3</v></c>`, a
formula cell whose value is an index into the shared strings, is a form Excel does not write, and 199 such cells
occur in 2 xlsx files of the corpus sample (`docs/excel-semantics/xlsx-storage.md`, "A formula whose value is
typed as a shared string", UNVERIFIED). The importer gives such a cell a cached `#N/IMPL!` until a calculation
computes it, and reports it.

1. Does Excel open such a workbook, or refuse or repair it?
2. What does the cell show before Excel calculates it: the shared string, an error, or something else?
3. What does Excel write back for the cell?

## The workbooks

`python3 -I generate.py <dir>` writes two workbooks. Each has one sheet, `S`, with cells in column A only, and
four shared strings: `zero`, `one`, `two`, `three`. Both are in Manual mode (`calcPr calcMode="manual"`) and do
not ask for a calculation on load, so that Excel may show the cached values first. Every cached value differs
from what its formula gives.

| File | Column A | sha256 |
|---|---|---|
| `formula-string-value.xlsx` | the four cells of the table below | `0f2b9aca59cfc25c9b3f4d6cfd1ad01c2213d590c95ee447580cde1c10762cf9` |
| `formula-string-value-out-of-range.xlsx` | A1 `<c t="s"><f>"q"</f><v>9</v></c>`; A2 `<c t="str"><f>"x"&amp;"y"</f><v>stale</v></c>` | `ba2ac23aeb81ac11d62315aeccb105c2e182d34ec836a2773c2a2953b7988192` |

The second file's A1 names shared string 9, and there are four: it is alone in its file, with a control, so that
a refusal says it was that.

`formula-string-value.xlsx`, with what the engine does today (measured on main b55917c58; not Excel):

| Cell | Stored as | Its formula gives | The importer shows as loaded | A save from the model writes |
|---|---|---|---|---|
| A1 | `<c t="s"><f>"x"&amp;"y"</f><v>3</v></c>` | `xy`, and shared string 3 is `three` | `#N/IMPL!` | `t="str"`, `xy` |
| A2 | `<c t="s"><f>1+1</f><v>2</v></c>` | 2, and shared string 2 is `two` | `#N/IMPL!` | a number, 2 |
| A3 | `<c t="str"><f>"x"&amp;"y"</f><v>stale</v></c>` | `xy`; the control: Excel's own form, with a stale value | `stale` | `t="str"`, `xy` |
| A4 | `<c t="s"><v>1</v></c>` | no formula; the control: a constant that is shared string 1 | `one` | `t="s"`, 1 |

In the second file the importer shows `#N/IMPL!` in A1 and `stale` in A2 as loaded, and `q` and `xy` after a
calculation.

The files were written by `generate.py`, not by Excel, and no Excel has opened them.

## What to do

**Through the oracle tool**, for each file, with `<dir>` an absolute path (the command runs in `tools/`):

```
cd tools && uv run rustsheet-oracle recalc -b excel-graph <dir>/<file> > <dir>/<file>.graph.json 2> <dir>/<file>.graph.err
```

Please return the JSON output, or the `.err` file if the backend refused the workbook (a refusal such as
`FileCorruptTryRepair` is an answer, beside an empty JSON), and the recalculated workbook, which the tool keeps
as `eval/.cache/oracle/excel-graph/<sha256>.xlsx` under the file's sha256 above. The tool makes Excel calculate
on load, so this step answers questions 1 and 3. Its canary writer accepts both files (checked here, without the
network).

**By hand, in Excel for the web**, for question 2, with `formula-string-value.xlsx` (and the other file if the
tool's run did not refuse it):

1. Upload the file and open it. Copy any message word for word, with its buttons and the one chosen.
2. Before anything else, note what A1 to A4 show, and what Formulas > Calculation Options says the mode is.
3. Formulas > Calculate Workbook. Note A1 to A4 again. A3 is the control: if it still shows `stale`, the command
   did not recalculate cells that nothing had changed.
4. Select A1, press F2 and then Enter, which enters its formula again. Note A1.

If the cells already show `xy`, 2, `xy` and `one` at step 2, Excel for the web calculated on open whatever the
file's mode, and question 2 has no answer there: say so.

## What the answers decide

| # | Question | Read from | What it decides |
|---|---|---|---|
| 1 | Does Excel open the file? | the tool's output or error; the message of step 1 | whether the importer should load such a file at all |
| 2 | What do A1 and A2 show before a calculation: `three` and `two`, an error, `stale`-like text, or nothing? | step 2, against the controls A3 (`stale`) and A4 (`one`) | the cached value the importer gives such a cell, where it gives `#N/IMPL!` today |
| 3 | What does Excel write back for A1 and A2? | the recalculated workbook's sheet part | that the export's `t="str"` and plain number are Excel's too |
| 4 | Is an index past the shared strings refused? | the second file | whether the importer's bound on the index matters for files Excel reads |

Each answer goes into the entry named above, with this request as its evidence; the importer follows (the import
lane, Taskei P559389085).

## Checks, without Excel

- `python3 -I check.py` runs the generator and reads the two workbooks back: every part well-formed, the four
  shared strings, Manual mode with no calculation on load, each cell of column A stored as the tables above give
  it, and the sha256 of each.
- A reader that is not this generator, openpyxl 3.1.5, loads both and gives back the formulas and the constant.
  It is not Excel.
