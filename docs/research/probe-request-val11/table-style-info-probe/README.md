# Probe request: a table's `tableStyleInfo` with a flag absent, and a style name its workbook does not define

*Design desk (poc-design-cc), 2026-11-12, for Taskei P560297170 (a follow-up of the change that reads a table's
`<tableStyleInfo>`; the importer's default stays with the engine lane that filed it). Nothing here has been run
in Excel. The request is for Excel for the web, through the oracle tool. The answers go into `RESULTS.md` beside
this file.*

## What is asked

`docs/excel-semantics/xlsx-storage.md`, "A table's style is its `tableStyleInfo`", leaves two forms unprobed,
because no Excel-written file of the repository holds them:

1. **A `<tableStyleInfo>` without one of its four flags.** ECMA-376 Part 1, §18.5.1.5, gives `showFirstColumn`,
   `showLastColumn`, `showRowStripes` and `showColumnStripes` no default. The importer reads an absent
   `showRowStripes` as on and the other three as off.
2. **A style name the workbook does not define.** The importer keeps the name, and a save from the model writes
   no `<tableStyles>`, so a table can be saved naming a custom style its workbook no longer has. §18.5.1.5 says
   an application "should use default style" then.

What does Excel read in each case? It is asked through what Excel writes back: the oracle tool opens a workbook
in Excel for the web, calculates it, and keeps the workbook Excel saved.

## The workbooks

`python3 -I generate.py <dir>` writes ten workbooks. Each has one sheet, `S`, with a table `Probe` over A1:B3
(the columns `k` and `n`; the rows a, 10 and b, 20) and D1 `=SUM(Probe[n])`, which is 30 while the table is one.
They differ only in the table part's `<tableStyleInfo>` and, for the last, in the styles part.

| File | `<tableStyleInfo>` | sha256 |
|---|---|---|
| `table-control.xlsx` | `name="TableStyleMedium2" showFirstColumn="0" showLastColumn="0" showRowStripes="1" showColumnStripes="0"` | `a0a1e265fa98b7b5ae959ae5900b9ae64d9cbb5c880127bc802b8c59b53775ca` |
| `table-control-opposite.xlsx` | `name="TableStyleMedium2" showFirstColumn="1" showLastColumn="1" showRowStripes="0" showColumnStripes="1"` | `627b21b16d3b6d33b710697777f40cb09b12614b64c05e885a3b7fbaadd5ce60` |
| `table-no-first-column.xlsx` | `name="TableStyleMedium2" showLastColumn="0" showRowStripes="1" showColumnStripes="0"` | `42ee9aa89d7e0af00e8f7a4d36e084affb495ce27f4043a6f5656d28e591a30c` |
| `table-no-last-column.xlsx` | `name="TableStyleMedium2" showFirstColumn="0" showRowStripes="1" showColumnStripes="0"` | `4536c10f48e4fc0e8b9a63d39f6fa70857e3395af11069b3a46a1002b98a759d` |
| `table-no-row-stripes.xlsx` | `name="TableStyleMedium2" showFirstColumn="0" showLastColumn="0" showColumnStripes="0"` | `c195d83e9c885045d830041f786fc011e653c00c692a7ddab51acfa8202ae5f4` |
| `table-no-column-stripes.xlsx` | `name="TableStyleMedium2" showFirstColumn="0" showLastColumn="0" showRowStripes="1"` | `e5fb344bdd039a7a7aead8ee1ef3d71085ea88125379850248df658ca55f9f4c` |
| `table-no-flags.xlsx` | `name="TableStyleMedium2"` | `537e65e75dcf898fdfcb89861dc6ea26e1e30f8b6335a965ddc47cf352710ea4` |
| `table-no-style-info.xlsx` | none: the table part has no such element | `40a5e8869f3755b46170c54dd01cc8d96a56b98d68f2acd01b7f5e6bd0b6f6b4` |
| `table-style-undefined.xlsx` | `name="ProbeStyle" showFirstColumn="0" showLastColumn="0" showRowStripes="1" showColumnStripes="0"` | `ebd320c1d8d26759b3bfaa843bc8f76831c88aa2dac02f433c35bcb213914313` |
| `table-style-defined.xlsx` | the same, and the styles part defines `ProbeStyle` | `70c992fe84cf0c2eda37559cfec5a222368702906eea183c8c9f468a031bbb86` |

- The two controls hold the four flags with opposite values. What Excel writes back for them shows how it writes
  a flag that is on and one that is off, which is what an absent flag's answer is read against.
- `ProbeStyle`, where it is defined, is one whole-table element with a bold font. No other styles part has a
  `<tableStyles>` element.
- The files were written by `generate.py`, not by Excel, and no Excel has opened them.

## What to do

For each file, `table-control.xlsx` first, with `<dir>` an absolute path (the command runs in `tools/`):

```
cd tools && uv run rustsheet-oracle recalc -b excel-graph <dir>/<file> > <dir>/<file>.graph.json 2> <dir>/<file>.graph.err
```

Please return, for each, the JSON output, or the `.err` file if the backend refused the workbook (a refusal such
as `FileCorruptTryRepair` is an answer, beside an empty JSON), and the recalculated workbook, which the tool
keeps as `eval/.cache/oracle/excel-graph/<sha256>.xlsx` under the file's sha256 above. If Excel refuses
`table-control.xlsx`, stop there and send the error: the package is then the suspect, not the element. The
tool's canary writer accepts all ten (checked here, without the network).

## What the answers decide

| # | Question | Read from | What it decides |
|---|---|---|---|
| 1 | Does Excel open each file, and is D1 30? | the JSON, or the error | whether the form is one Excel reads at all |
| 2 | For each flag: what does Excel write back where the file had none? | `xl/tables/table1.xml` of the four `table-no-…` workbooks, against the two controls | the importer's default for that flag |
| 3 | The same with all four absent | `table-no-flags.xlsx` | that the defaults do not depend on which other flags are there |
| 4 | With no `<tableStyleInfo>`: does Excel write one back, with which name and flags? | `table-no-style-info.xlsx` | what the importer gives a table with no such element (today: no style, row stripes on) |
| 5 | A name the workbook does not define: does Excel write the name back, another name, or none? | `table-style-undefined.xlsx`, against `table-style-defined.xlsx` | what a save from the model writes for a custom style it cannot carry |
| 6 | A name the workbook defines: are the name and the `<tableStyles>` element written back? | `table-style-defined.xlsx` | the control for 5 |

Each answer goes into the entry named above, with this request as its evidence. Where Excel's default for a flag
differs from the importer's, the importer changes (the engine lane's, Taskei P560297170).

## What the engine does today

Measured on main b55917c58 by loading each workbook through the importer, calculating it, and saving it from the
model. It is what the entry says, and it is not Excel:

- D1 is 30 in all ten.
- Each flag is read as the file has it; an absent `showRowStripes` is read as on and an absent
  `showFirstColumn`, `showLastColumn` or `showColumnStripes` as off. So the four files with one flag absent, and
  the one with none, read as `table-control.xlsx` does, and a save writes all four flags for each:
  `<tableStyleInfo name="TableStyleMedium2" showFirstColumn="0" showLastColumn="0" showRowStripes="1"
  showColumnStripes="0"/>`.
- With no `<tableStyleInfo>` the table has no style name and the same four flags, and a save writes
  `<tableStyleInfo showFirstColumn="0" showLastColumn="0" showRowStripes="1" showColumnStripes="0"/>`.
- `ProbeStyle` is kept as the table's style name in both files, and a save writes no `<tableStyles>` in either:
  the defined style is lost, and the table still names it.

## Checks, without Excel

- `python3 -I check.py` runs the generator and reads the ten workbooks back: every part well-formed, every
  relationship's target present, the table over A1:B3 with its two columns, D1's formula, each file's
  `<tableStyleInfo>` as the table above gives it, `<tableStyles>` in the one styles part that should have it,
  and the sha256 of each.
- A reader that is not this generator, openpyxl 3.1.5, loads all ten and gives back the table, its columns, the
  formula and the style with each absent flag as absent. It is not Excel.
