# Probe request: a cell font that states bold, italic, strikethrough or underline off

*Design desk (poc-design-cc), 2026-11-12, for Taskei P560140699 (a follow-up of the change that reads a font
flag's `val`). Nothing here has been run in Excel. The request is for Excel for the web, through the oracle
tool. The answers go into `RESULTS.md` beside this file.*

## What is asked

`docs/excel-semantics/xlsx-storage.md`, "A font's `<b>`, `<i>` and `<strike>` hold their state in `val`", rests
on the schema: `<b val="0"/>` is not bold. Excel writes that off form in differential fonts, and no
Excel-written file of the repository holds it in a cell's font. Two questions:

1. Does Excel read a cell font's `<b val="0"/>`, `<i val="false"/>`, `<strike val="0"/>` and `<u val="none"/>`
   as off?
2. When Excel saves such a workbook, does it keep the off form or drop it? The engine keeps it, as a hint that
   lives from the import to the save (`Font::stated_off`, and `Underline::Off`). If Excel drops it, the hint can
   leave the model, and SPEC-0033's list of what a round trip may normalise gains a row with evidence.

Both are read from the workbook Excel writes back: a font Excel read as bold comes back bold. And one question
the entry marks as unevidenced: what Excel makes of a `val` it never writes, where the importer reads `FALSE`
and a padded ` 0 ` as off and anything else as on. Three of the four values tried are no `xsd:boolean` (`FALSE`,
`off`, the empty value). The fourth is one: the type collapses whitespace, so ` 0 ` is `0` to a schema
processor, and an Excel that reads it as off follows the schema.

## The workbooks

`python3 -I generate.py <dir>` writes five workbooks. Each has one sheet, `S`. Every cell of column A has a font
of its own, Calibri 11, with one flag stated in one form (A1 of the first file states none), and the cell's text
names the form; B1 counts the cells of column A.

| File | Column A | sha256 |
|---|---|---|
| `font-flags.xlsx` | A1 to A13: the thirteen forms of the table below, which the schema allows | `c7a0f79ad5b49734b370799cd85f4194d7b172bc7835984cbb42ea62c4bf433b` |
| `font-flag-upper-case.xlsx` | A1 `<b val="FALSE"/>`; A2 `<b/>` | `b7482ba6775b1f24d4968537e5fb89dcb777845aa9ea963965aad3bd112a1983` |
| `font-flag-padded.xlsx` | A1 `<b val=" 0 "/>`; A2 `<b/>` | `5ac9c1bd11f6c28568e2c40ed56e2eb698efd0457e370e586606815e5117dade` |
| `font-flag-word.xlsx` | A1 `<b val="off"/>`; A2 `<b/>` | `eaa55bcc9ce0a72328da04df6b16b88fa17529918612c37858334e4c5c51dd66` |
| `font-flag-empty.xlsx` | A1 `<b val=""/>`; A2 `<b/>` | `d5353637acc05f17f4ad0c95277e2804cc96d2a4b668cc94fb999a0609ecb336` |

The last four each hold one value Excel does not write, alone, so that a file Excel refuses or repairs says
which value it was. Their A2 is the control: a plain bold cell.

`font-flags.xlsx`, with what the engine does today (measured on main b55917c58 by loading the workbook through
the importer and saving it from the model; not Excel):

| Cell | The font states | The importer reads | A save from the model writes |
|---|---|---|---|
| A1 | nothing | no flag | no element |
| A2 | `<b/>` | bold | `<b/>` |
| A3 | `<b val="1"/>` | bold | `<b/>` |
| A4 | `<b val="true"/>` | bold | `<b/>` |
| A5 | `<b val="0"/>` | not bold, stated off | `<b val="0"/>` |
| A6 | `<b val="false"/>` | not bold, stated off | `<b val="0"/>` |
| A7 | `<i/>` | italic | `<i/>` |
| A8 | `<i val="false"/>` | not italic, stated off | `<i val="0"/>` |
| A9 | `<strike/>` | struck through | `<strike/>` |
| A10 | `<strike val="0"/>` | not struck through, stated off | `<strike val="0"/>` |
| A11 | `<u/>` | a single underline | `<u/>` |
| A12 | `<u val="none"/>` | no underline, stated off | `<u val="none"/>` |
| A13 | `<u val="single"/>` | a single underline | `<u/>` |

For the other four the importer reads `FALSE` and ` 0 ` as not bold, stated off (saved `<b val="0"/>`), and
`off` and the empty value as bold (saved `<b/>`). B1 is 13 in the first file and 2 in the others.

The files were written by `generate.py`, not by Excel, and no Excel has opened them.

## What to do

For each file, `font-flags.xlsx` first, with `<dir>` an absolute path (the command runs in `tools/`):

```
cd tools && uv run rustsheet-oracle recalc -b excel-graph <dir>/<file> > <dir>/<file>.graph.json 2> <dir>/<file>.graph.err
```

Please return, for each, the JSON output, or the `.err` file if the backend refused the workbook (a refusal such
as `FileCorruptTryRepair` is an answer, beside an empty JSON), and the recalculated workbook, which the tool
keeps as `eval/.cache/oracle/excel-graph/<sha256>.xlsx` under the file's sha256 above. If Excel refuses
`font-flags.xlsx`, stop there and send the error: the package is then the suspect, not a value. The tool's
canary writer accepts all five (checked here, without the network).

## What the answers decide

| # | Question | Read from | What it decides |
|---|---|---|---|
| 1 | Does Excel open each file, and is B1 the count? | the JSON, or the error | whether the form is one Excel reads at all |
| 2 | Which fonts come back bold, italic, struck through or underlined? | the styles part of the workbook written back for `font-flags.xlsx`: the font of each cell's format | that the off forms read as off: the entry becomes VERIFIED, or the importer changes |
| 3 | Is an off form written back as it was, as `val="0"`, or not at all? Do A1, A5 and A6 come back with one font? | the same | whether `Font::stated_off` and `Underline::Off` stay in the model |
| 4 | What does Excel make of `FALSE`, ` 0 `, `off` and an empty `val`? | the four other files | the importer's rule for a `val` Excel does not write: the padded one the schema allows, and three it does not |

Each answer goes into the entry named above, with this request as its evidence. The importer and the model are
the engine lane's (Taskei P560140699).

## Checks, without Excel

- `python3 -I check.py` runs the generator and reads the five workbooks back: every part well-formed, each cell
  of column A with a format of its own whose font states what the tables above give, the cell's text naming it,
  B1's formula, and the sha256 of each.
- A reader that is not this generator, openpyxl 3.1.5, loads all five and reads the thirteen allowed forms as
  the importer does. On the four values Excel does not write it guesses otherwise: `FALSE` and ` 0 ` as bold,
  the empty value as not bold. Two readers, two rules, and neither is Excel.
