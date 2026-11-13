"""Writes T-5465's probe cases: what Excel does to references and records when cells are inserted or deleted
with a shift (SPEC-0039 Amendment 3, U7 to U14). One area, S!C4:D5, and four shifts."""
import json
import sys
from pathlib import Path

OUT = Path(sys.argv[1])
SHIFTS = {  # name -> (op, shift, the band in words)
    "insert-down": ("insert_cells", "down", "columns C:D, rows 4 on move down two"),
    "insert-right": ("insert_cells", "right", "rows 4:5, columns C on move right two"),
    "delete-up": ("delete_cells", "up", "columns C:D, rows 6 on move up two"),
    "delete-left": ("delete_cells", "left", "rows 4:5, columns E on move left two"),
}
AREA = "C4:D5"


def block():
    """S!B2:G9, each cell 10 * row + column (B2 is 22, G9 is 97): a value names the cell it was typed in."""
    return {f"{chr(64 + c)}{r}": 10 * r + c for r in range(2, 10) for c in range(2, 8)}


def with_text(formulas):
    """Sheet F's cells: each formula in column A, its FORMULATEXT beside it in column B."""
    cells, check = {}, []
    for i, f in enumerate(formulas, start=1):
        cells[f"A{i}"] = f
        cells[f"B{i}"] = f"=FORMULATEXT(A{i})"
        check += [f"F!A{i}", f"F!B{i}"]
    return cells, check


def picture(first_row, rows=12, last_column="I"):
    """Rows of sheet F that show S!A1:I12 after the edit, one text a row, read through INDIRECT so the
    edit cannot rewrite the reference."""
    cells, check = {}, []
    for i in range(rows):
        at = f"A{first_row + i}"
        cells[at] = f'=TEXTJOIN(",",FALSE,INDIRECT("S!A{i + 1}:{last_column}{i + 1}"))'
        check.append(f"F!{at}")
    return cells, check


def case(name, shift, description, s_cells, f_cells, check, extra_sheets=(), **top):
    op, way, band = SHIFTS[shift]
    spec = {
        "format": "rustsheet-case/v1",
        "description": f"T-5465 (SPEC-0039 Amendment 3, {description}) {op} of S!{AREA} shifting {way} ({band}). "
        "S!B2:G9 holds 10*row+column (B2 22, G9 97). Sheet F's formulas do not move; each has its FORMULATEXT beside it.",
        "tags": ["M4", "probe"],
        "sheets": [{"name": "S", **s_cells}, *extra_sheets, {"name": "F", "cells": f_cells}],
        "sheet_ops": [{"op": op, "sheet": "S", "range": AREA, "shift": way}],
        "expected": {},
        "check": check,
        **top,
    }
    (OUT / f"t5465-shift-{shift}-{name}.case.json").write_text(json.dumps(spec, indent=2) + "\n")


CELLS = ["S!C4", "S!D5", "S!C6", "S!D9", "S!E4", "S!G5", "S!C3", "S!C2", "S!B4", "S!B5", "S!E6", "S!$C$6", "S!$E$4"]
RANGES = [
    # the band of a down or up shift covers these whole (columns C:D)
    "S!C6:D7", "S!C3:D7", "S!C4:D5", "S!C2:C9", "S!C4:D9", "S!C6:C6",
    # the band of a right or left shift covers these whole (rows 4:5)
    "S!E4:F5", "S!B4:F5", "S!B4:G4", "S!C4:G5", "S!E4:E4",
    # across a band's edge, with and without a moved cell under them, and one that holds everything
    "S!B6:E7", "S!B2:E3", "S!B4:E5", "S!E3:F6", "S!B3:B6", "S!A1:H10",
]
LINES = ["S!4:4", "S!4:5", "S!6:6", "S!3:3", "S!C:C", "S!C:D", "S!E:E", "S!B:B"]

for shift in SHIFTS:
    # U7: single cells.
    f, check = with_text([f"={c}" for c in CELLS])
    p, pc = picture(20)
    case("cells", shift, "U7: a cell reference at, after, before and beside the shift; rows 20 to 31 of F show S!A1:I12 after it,",
         {"cells": block()}, {**f, **p}, check + pc)
    # U8, U9: ranges.
    f, check = with_text([f"=SUM({r})" for r in RANGES])
    case("ranges", shift, "U8 and U9: ranges a band covers whole, ranges across a band's edge with and without a moved cell under them, and one that holds the area,",
         {"cells": block()}, f, check)
    # U10, U11: whole lines, a span of sheets, names.
    f, check = with_text([f"=SUM({r})" for r in LINES] + ["=SUM(S:T!C6)", "=SUM(S:T!C3:D7)", "=nmCell", "=SUM(nmAcross)", "=nmBefore", "=SUM(nmRow)"])
    case("lines-spans-names", shift,
         "U10 and U11: whole rows and columns, a 3-D span S:T, and names: nmCell is S!$C$6, nmAcross S!$B$6:$E$7, nmBefore S!$C$3, nmRow S!$4:$4,",
         {"cells": block()}, f, check, extra_sheets=[{"name": "T", "cells": {"C6": 1000, "C3": 2000, "D7": 4000}}],
         names={"nmCell": "=S!$C$6", "nmAcross": "=S!$B$6:$E$7", "nmBefore": "=S!$C$3", "nmRow": "=S!$4:$4"})
    # Formulas on S itself, in cells that move and cells that do not, read back through INDIRECT at
    # the place they were typed and at each place a shift could take them to; number formats likewise.
    on_sheet = {
        "C11": "=C6", "D11": "=B3", "C12": "=$C$6", "D12": "=F!A1",      # in columns C:D below the block
        "H4": "=E4", "I4": "=B3", "H5": "=$E$4", "I5": "=F!A1",          # in rows 4:5 right of the block
        "K14": "=C6", "L14": "=E4", "K15": "=SUM(C4:D5)", "L15": "=C3+B4",  # outside both bands
    }
    places = ["C11", "D11", "C12", "D12", "C13", "D13", "C14", "D14", "C9", "D9", "C10", "D10",
              "H4", "I4", "H5", "I5", "J4", "K4", "J5", "K5", "F4", "G4", "F5", "G5",
              "K14", "L14", "K15", "L15"]
    f_cells, check = {"A1": 7}, []
    for i, at in enumerate(places, start=2):
        f_cells[f"A{i}"] = f'=IFERROR(FORMULATEXT(INDIRECT("S!{at}")),"none")'
        f_cells[f"B{i}"] = f'=INDIRECT("S!{at}")'
        check += [f"F!A{i}", f"F!B{i}"]
    # U13, U14: a number format moves with its cell (C6, D9), a formatted blank (C10), and what an
    # inserted cell has (C4, D5 after an insert). CELL("format") is F2 for 0.00, P0 for 0%, G for General.
    formats = ["C4", "D5", "C6", "D6", "C8", "D7", "C10", "C12", "D9", "D11", "E4", "G4", "F5", "E5", "B6"]
    for i, at in enumerate(formats, start=40):
        f_cells[f"A{i}"] = f'=CELL("format",INDIRECT("S!{at}"))'
        check.append(f"F!A{i}")
    case("on-sheet-and-formats", shift,
         "U7 to U10 on the edited sheet and U13, U14: formulas in cells of S that move and that do not (C11 =C6, D11 =B3, C12 =$C$6, D12 =F!A1; H4 =E4, I4 =B3, H5 =$E$4, I5 =F!A1; K14 =C6, L14 =E4, K15 =SUM(C4:D5), L15 =C3+B4), read back through INDIRECT at every place a shift could leave them; S!C6 and S!D9 are formatted 0.00, S!E4 and S!F5 0%, the blank S!C10 0.00, and rows 40 to 54 of F give CELL(\"format\") of fixed places,",
         {"cells": {**block(), **on_sheet}, "number_formats": {"C6": "0.00", "D9": "0.00", "E4": "0%", "F5": "0%", "C10": "0.00"}},
         f_cells, check)

# U12 and U14: shapes Excel may refuse; the refusal and its error code are the answer wanted.
def refusal(name, shift, description, s_extra, f_formulas, area=AREA, **top):
    op, way, band = SHIFTS[shift]
    f, check = with_text(f_formulas)
    spec = {
        "format": "rustsheet-case/v1",
        "description": f"T-5465 (SPEC-0039 Amendment 3, {description}) {op} of S!{area} shifting {way}. If Excel refuses the edit, the answer wanted is the refusal: the HTTP status, Graph's error code and message.",
        "tags": ["M4", "probe"],
        "sheets": [{"name": "S", **s_extra}, {"name": "F", "cells": f}],
        "sheet_ops": [{"op": op, "sheet": "S", "range": area, "shift": way}],
        "expected": {},
        "check": check,
        **top,
    }
    (OUT / f"t5465-shift-{shift}-{name}.case.json").write_text(json.dumps(spec, indent=2) + "\n")

spill_row = {"cells": {**block(), "B12": "=SEQUENCE(1,4)"}}       # spills B12:E12, across columns C:D
spill_col = {"cells": {**block(), "I3": "=SEQUENCE(4)"}}          # spills I3:I6, across rows 4:5
refusal("cuts-spill", "insert-down", "U12: a spill the band cuts: S!B12 =SEQUENCE(1,4) spills B12:E12 and the shift moves C12:D12,", spill_row,
        ['=INDIRECT("S!B12")', '=IFERROR(FORMULATEXT(INDIRECT("S!B12")),"none")', '=TEXTJOIN(",",FALSE,INDIRECT("S!A12:I12"))', '=TEXTJOIN(",",FALSE,INDIRECT("S!A14:I14"))', "=SUM(S!B12:E12)"])
refusal("cuts-spill", "delete-up", "U12: a spill the band cuts: S!B12 =SEQUENCE(1,4) spills B12:E12 and the shift moves C12:D12,", spill_row,
        ['=INDIRECT("S!B12")', '=IFERROR(FORMULATEXT(INDIRECT("S!B12")),"none")', '=TEXTJOIN(",",FALSE,INDIRECT("S!A12:I12"))', '=TEXTJOIN(",",FALSE,INDIRECT("S!A10:I10"))', "=SUM(S!B12:E12)"])
refusal("cuts-spill", "insert-right", "U12: a spill the band cuts: S!I3 =SEQUENCE(4) spills I3:I6 and the shift moves I4:I5,", spill_col,
        ['=INDIRECT("S!I3")', '=IFERROR(FORMULATEXT(INDIRECT("S!I3")),"none")', '=TEXTJOIN(",",FALSE,INDIRECT("S!I3:I6"))', '=TEXTJOIN(",",FALSE,INDIRECT("S!K3:K6"))', "=SUM(S!I3:I6)"])
refusal("into-spill", "insert-down", "U12: an insert inside a spill: S!C3 =SEQUENCE(4,2) spills C3:D6, which holds the area,", {"cells": {"C3": "=SEQUENCE(4,2)", "B2": 22}},
        ['=INDIRECT("S!C3")', '=IFERROR(FORMULATEXT(INDIRECT("S!C3")),"none")', '=TEXTJOIN(",",FALSE,INDIRECT("S!C3:D8"))', "=SUM(S!C3:D6)"])

table_cells = {"D7": "h1", "E7": "h2", "F7": "h3", "D8": 1, "E8": 2, "F8": 3, "D9": 4, "E9": 5, "F9": 6}
part = {"cells": {"C4": 44, "D5": 55, "C6": 66, **table_cells}, "tables": [{"name": "Sales", "ref": "D7:F9"}]}
refusal("table-in-part", "insert-down", "U12: a table the moved cells touch in part: the table Sales is S!D7:F9, and the band is columns C:D,", part,
        ["=SUM(Sales[h1])", "=SUM(S!D8:D9)", "=ROWS(Sales[#All])", '=TEXTJOIN(",",FALSE,INDIRECT("S!C7:F11"))'])
refusal("table-in-part", "delete-up", "U12: a table the moved cells touch in part: the table Sales is S!D7:F9, and the band is columns C:D,", part,
        ["=SUM(Sales[h1])", "=SUM(S!D8:D9)", "=ROWS(Sales[#All])", '=TEXTJOIN(",",FALSE,INDIRECT("S!C5:F9"))'])
whole_cells = {"C7": "h1", "D7": "h2", "C8": 1, "D8": 2, "C9": 4, "D9": 5}
whole = {"cells": {"C4": 44, "D5": 55, "C6": 66, "B8": 28, "E8": 58, **whole_cells}, "tables": [{"name": "Sales", "ref": "C7:D9"}]}
refusal("table-whole", "insert-down", "U14: a table the band holds whole: the table Sales is S!C7:D9, in columns C:D below the area,", whole,
        ["=SUM(Sales[h1])", "=SUM(S!C8:C9)", "=ROWS(Sales[#All])", '=TEXTJOIN(",",FALSE,INDIRECT("S!B7:E11"))'])
refusal("table-whole", "delete-up", "U14: a table the band holds whole: the table Sales is S!C7:D9, in columns C:D below the area,", whole,
        ["=SUM(Sales[h1])", "=SUM(S!C8:C9)", "=ROWS(Sales[#All])", '=TEXTJOIN(",",FALSE,INDIRECT("S!B5:E9"))'])
side = {"cells": {"C4": 44, "D5": 55, "F3": "h1", "G3": "h2", "F4": 1, "G4": 2, "F5": 4, "G5": 5, "F6": 7, "G6": 8}, "tables": [{"name": "Sales", "ref": "F3:G6"}]}
refusal("table-in-part", "insert-right", "U12: a table the moved cells touch in part: the table Sales is S!F3:G6, and the band is rows 4:5,", side,
        ["=SUM(Sales[h1])", "=SUM(S!F4:F6)", "=COLUMNS(Sales[#All])", '=TEXTJOIN(",",FALSE,INDIRECT("S!E3:J6"))'])
refusal("off-the-grid", "insert-down", "U12: a nonblank cell pushed off the grid: S!C1048576 holds 5,", {"cells": {**block(), "C1048576": 5}},
        ["=S!C1048576", "=COUNTA(S!C:C)", "=S!C6"])
refusal("off-the-grid", "insert-right", "U12: a nonblank cell pushed off the grid: S!XFD4 holds 5,", {"cells": {**block(), "XFD4": 5}},
        ["=S!XFD4", "=COUNTA(S!4:4)", "=S!E4"])
refusal("blank-off-the-grid", "insert-down", "U12 control: only a formatted blank is pushed off the grid: S!C1048576 is blank with the format 0.00,", {"cells": block(), "number_formats": {"C1048576": "0.00"}},
        ['=CELL("format",INDIRECT("S!C1048576"))', "=S!C6"])
# U13: what cannot be read through a formula is read from Excel's saved file (see the README): the same
# workbooks, with a conditional format and a data validation in each place.
for shift in SHIFTS:
    op, way, band = SHIFTS[shift]
    spec = {
        "format": "rustsheet-case/v1",
        "description": f"T-5465 (SPEC-0039 Amendment 3, U11 and U13, for Excel's saved file) {op} of S!{AREA} shifting {way} ({band}). Conditional formats: C6:D7 (=C6>0, inside the down band's tail), E4:F5 (=E4>0, inside the right band's tail), B6:E7 (=B6>0, across the edge), K14 (=$C$6>0 and, a second rule, =$E$4>0: outside both bands, reading a moved cell). Data validations: C6 (=C6>0), E4 (=E4>0), B6:E7 (=B6>0), L14 (=$C$6>0), M14 (=$E$4>0). Names nmCell S!$C$6, nmAcross S!$B$6:$E$7, nmRight S!$E$4. Number formats as in on-sheet-and-formats. Wanted: Excel's saved workbook after the edit (rustsheet-oracle resave), or the refusal.",
        "tags": ["M4", "probe"],
        "sheets": [{
            "name": "S", "cells": block(),
            "number_formats": {"C6": "0.00", "D9": "0.00", "E4": "0%", "F5": "0%", "C10": "0.00"},
            "conditional_formats": [
                {"ref": "C6:D7", "formula": "=C6>0"}, {"ref": "E4:F5", "formula": "=E4>0"}, {"ref": "B6:E7", "formula": "=B6>0"},
                {"ref": "K14", "formula": "=$C$6>0"}, {"ref": "K14", "formula": "=$E$4>0"},
            ],
            "data_validations": [
                {"ref": "C6", "formula": "=C6>0"}, {"ref": "E4", "formula": "=E4>0"}, {"ref": "B6:E7", "formula": "=B6>0"},
                {"ref": "L14", "formula": "=$C$6>0"}, {"ref": "M14", "formula": "=$E$4>0"},
            ],
        }, {"name": "F", "cells": {"A1": "=nmCell", "A2": "=SUM(nmAcross)", "A3": "=nmRight"}}],
        "names": {"nmCell": "=S!$C$6", "nmAcross": "=S!$B$6:$E$7", "nmRight": "=S!$E$4"},
        "sheet_ops": [{"op": op, "sheet": "S", "range": AREA, "shift": way}],
        "expected": {},
        "check": ["F!A1", "F!A2", "F!A3"],
    }
    (OUT / f"t5465-shift-{shift}-records.case.json").write_text(json.dumps(spec, indent=2) + "\n")
print(len(list(OUT.glob("*.case.json"))), "cases")


# ---------------------------------------------------------------------------------------------
# The provisional rules of SPEC-0039 Amendment 3 (A11, U7 to U10), applied to each reference of the
# cells, ranges and lines cases, one reference at a time: the answers frozen beside the request.
import re

LAST_ROW, LAST_COLUMN = 1_048_576, 16_384


def col(letters):
    n = 0
    for ch in letters:
        n = n * 26 + ord(ch) - 64
    return n


def letters(n):
    out = ""
    while n:
        n, rem = divmod(n - 1, 26)
        out = chr(65 + rem) + out
    return out


def parse(ref):
    """`C4`, `$C$6`, `C6:D7`, `4:5`, `C:D` -> (rows (first, last), columns (first, last), how to print it)."""
    a, _, b = ref.partition(":")
    b = b or a
    cell = re.fullmatch(r"(\$?)([A-Z]+)(\$?)(\d+)", a)
    if cell:
        c2 = re.fullmatch(r"(\$?)([A-Z]+)(\$?)(\d+)", b)
        marks = (cell[1], cell[3], c2[1], c2[3])
        return (int(cell[4]), int(c2[4])), (col(cell[2]), col(c2[2])), ("cells", marks, ":" in ref)
    if a.isdigit():
        return (int(a), int(b)), (1, LAST_COLUMN), ("rows", None, True)
    return (1, LAST_ROW), (col(a), col(b)), ("columns", None, True)


def printed(rows, columns, how):
    kind, marks, is_range = how
    if kind == "rows":
        return f"{rows[0]}:{rows[1]}"
    if kind == "columns":
        return f"{letters(columns[0])}:{letters(columns[1])}"
    first = f"{marks[0]}{letters(columns[0])}{marks[1]}{rows[0]}"
    last = f"{marks[2]}{letters(columns[1])}{marks[3]}{rows[1]}"
    return f"{first}:{last}" if is_range else first


def provisional(ref, shift):
    """(covered, outcome): the reference's text after the shift, `stays`, or the refusal."""
    rows, columns, how = parse(ref)
    by_rows = shift in ("insert-down", "delete-up")
    insert = shift.startswith("insert")
    # The area C4:D5: the edit is at line 4 (row or column C is 3), two lines; the band is the other axis's lines.
    at, count = (4, 2) if by_rows else (3, 2)
    band = (3, 4) if by_rows else (4, 5)
    edited, other = (rows, columns) if by_rows else (columns, rows)
    last = LAST_ROW if by_rows else LAST_COLUMN
    if edited == (1, last):
        # It spans the edited axis whole: no edit of that axis changes it (U10).
        covered = "Whole" if band[0] <= other[0] and other[1] <= band[1] else ("Outside" if other[1] < band[0] or other[0] > band[1] else "Part")
        return covered, "stays (it spans the edited axis)"
    if other[1] < band[0] or other[0] > band[1]:
        return "Outside", "stays"
    moved_from = at if insert else at + count  # the first line whose cells move
    if not (band[0] <= other[0] and other[1] <= band[1]):
        if edited[1] >= (at if insert else at):
            return "Part", "the shift is REFUSED (U9: a range across the band's edge with moved or deleted cells under it)"
        return "Part", "stays (nothing under it moves)"
    first, end = edited
    if insert:
        if first >= at:
            first, end = first + count, end + count
        elif end >= at:
            end += count
        else:
            return "Whole", "stays"
    else:
        gone = (at, at + count - 1)
        if gone[0] <= first and end <= gone[1]:
            return "Whole", "#REF!"
        if end < at:
            return "Whole", "stays"
        first = first if first < at else (at if first <= gone[1] else first - count)
        end = at - 1 if at <= end <= gone[1] else end - count
    new = (first, end)
    return "Whole", printed(new if by_rows else rows, columns if by_rows else new, how)


lines = ["case\tcell\tformula\tcovered\tprovisional rule (SPEC-0039 A11), for this reference alone"]
for shift in SHIFTS:
    for name, formulas in [("cells", [f"={c}" for c in CELLS]), ("ranges", [f"=SUM({r})" for r in RANGES]),
                           ("lines-spans-names", [f"=SUM({r})" for r in LINES])]:
        for i, formula in enumerate(formulas, start=1):
            ref = re.search(r"S!([$A-Z0-9:]+)", formula)[1]
            covered, outcome = provisional(ref, shift)
            text = outcome
            if not outcome.startswith(("stays", "the shift")):
                text = formula.replace(f"S!{ref}", "#REF!" if outcome == "#REF!" else f"S!{outcome}")
            lines.append(f"t5465-shift-{shift}-{name}\tF!B{i}\t{formula}\t{covered}\t{text}")
(OUT.parent / "predictions.tsv").write_text("\n".join(lines) + "\n")
print(len(lines) - 1, "provisional answers")
