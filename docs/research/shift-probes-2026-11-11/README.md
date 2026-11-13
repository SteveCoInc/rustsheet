# Probe request: what Excel does when cells are inserted or deleted with a shift (T-5465)

From the M4 lane, 2026-11-11. SPEC-0039 Amendment 3 (A11) lists eight unknowns, U7 to U14, about
Home > Insert > Cells and Home > Delete > Cells (shift down, right, up, left). Every rule the engine will follow for
them is from memory today and marked UNVERIFIED; where a shape is doubtful the provisional rule refuses the edit.
These cases ask Excel. They need the two case ops of T-5464 (`insert_cells`, `delete_cells`), which the Graph
backend replays as `range(address='C4:D5')/insert` or `/delete` with a `shift`.

## Before the fill: the two case ops (one local patch)

The cases use two case ops, `insert_cells` and `delete_cells`, that your `tools/` does not have yet (they come
from T-5464, still in review on our side). `tools-case-ops.patch` here adds them: 4 files under `tools/`, 157 lines,
including their tests. It applies cleanly on today's `main`. Apply it locally for the fill and drop it afterwards;
it is not part of this request:

```sh
git apply docs/research/shift-probes-2026-11-11/tools-case-ops.patch     # before the fill
git apply -R docs/research/shift-probes-2026-11-11/tools-case-ops.patch  # after it
```

## What is asked

Excel-graph answers for **32 cases, 790 checked cells** (`rustsheet-case/v1`, `"expected": {}`), in `cases/`.

None is in a gate suite: the engine has no shift of cells yet (T-5466), so its adapter answers "not built" and a
pending case would be a whole-case failure. Each is run by path:

```sh
cd tools && uv run rustsheet-oracle recalc -b excel-graph ../docs/research/shift-probes-2026-11-11/cases/<name>.case.json
```

Wanted back: the JSON each prints (the checked cells' values), or the refusal (below).

For the four `*-records` cases the values are three cells only; what is wanted is **Excel's saved workbook after
the edit**, because a conditional format's range, a data validation's range and formula, and a defined name's
definition cannot be read through a formula:

```sh
cd tools && mkdir -p /tmp/shift-records/in
for c in ../docs/research/shift-probes-2026-11-11/cases/*-records.case.json; do
  uv run rustsheet-spec2xlsx "$c" "/tmp/shift-records/in/$(basename "$c" .case.json).xlsx"
done
uv run rustsheet-oracle resave -b excel-graph /tmp/shift-records/in --out /tmp/shift-records/out
```

Wanted back: the four saved `.xlsx` files and `resave.json`.

## A refusal is an answer

The case format cannot record a refused edit: the run fails at the edit. For the cases below a refusal is the
kind of answer expected; for every other case it would be a finding. Either way, please send back the case name,
the HTTP status, and Graph's error code and message.

| Case | Shape | Provisional rule |
|---|---|---|
| `insert-down-cuts-spill`, `delete-up-cuts-spill`, `insert-right-cuts-spill` | the band cuts a spilled array | refused |
| `insert-down-into-spill` | the area is inside a spilled array | refused |
| `insert-down-table-in-part`, `delete-up-table-in-part`, `insert-right-table-in-part` | the moved cells touch one column or two rows of a table | refused |
| `insert-down-off-the-grid`, `insert-right-off-the-grid` | a nonblank cell would be pushed off the sheet | refused |
| `insert-down-blank-off-the-grid` | only a formatted blank would be pushed off | not refused (control) |
| `insert-down-table-whole`, `delete-up-table-whole` | a table inside the band, below the area | moves with the cells (U14) |

## The cases

One area, `S!C4:D5`, and four edits: `insert-down` (columns C:D from row 4 on move down two rows),
`insert-right` (rows 4:5 from column C on move right two columns), `delete-up`, `delete-left`. `S!B2:G9` holds
`10 * row + column`, so a value names the cell it was typed in. Sheet F's formulas read S and do not move; each
has its `FORMULATEXT` beside it.

| Cases (one per edit) | Unknown | What it holds |
|---|---|---|
| `*-cells` (38 cells each) | U7 | 13 single-cell references: at the area, after it in the band, before it, beside the band, with and without `$`; rows 20 to 31 show `S!A1:I12` after the edit, read through `INDIRECT` |
| `*-ranges` (34) | U8, U9 | 17 ranges: ones a band covers whole (inside its tail, across the edit's first line, the area itself), ones across a band's edge with and without moved cells under them, and one that holds everything |
| `*-lines-spans-names` (28) | U10, U11 | whole rows and columns at, after, before and beside the edit; a 3-D span `S:T`; four names (a cell, a range across the edge, a cell before the edit, a whole row) |
| `*-on-sheet-and-formats` (71) | U7 to U10 on the edited sheet, U13, U14 | formulas in cells of S that move and that do not, each read back through `INDIRECT` at every place an edit could leave it (a moved cell that reads a cell outside the band is the shape the review of the amendment found: `D11 =B3`); `CELL("format")` at fixed places for a formatted cell that moves, a formatted blank, and the inserted cells |
| `*-records` (3 + the saved file) | U11, U13 | conditional formats and data validations inside a band's tail, across its edge, and outside both bands with a formula that reads a moved cell; three names |
| the twelve cases of the table above | U12, U14 | spills, tables, the grid's edge |

`predictions.tsv` has the provisional rule's answer for each reference of the `cells`, `ranges` and `lines`
cases, taken one reference at a time (152 lines): its coverage by the band (`Whole`, `Part`, `Outside`) and the
text the rule gives, `stays`, or the refusal. In the engine one refused reference refuses the whole edit, so by
these rules the `ranges` and `lines` cases are refused whole; Excel is expected to answer them all, and that
difference is what U9 is.

`gen_cases.py` wrote the cases and the predictions (`python3 gen_cases.py cases`).

## What is not asked here, and why

The case format cannot write these yet, so they wait for a second request: a merge (across the band's edge,
inside it, at the grid's edge), a link, a note or a comment thread, a legacy CSE array the band cuts, a what-if
data table, a pivot, an autofilter's range, a sparkline and a chart's series range.
