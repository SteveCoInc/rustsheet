"""T-3815: the XNPV element-reading probe (eval/suites/probes/cases/t3815-xnpv-element-reading.case.json).

Sheet D holds four-row columns: A the values -100, 30, 40, 50 and B the dates 39448, 39508, 39600, 39700; each other
column changes one element (DATES for the dates against A, VALUES for the values against B). Calc row r takes one
variant: A the range form, B the array-constant form (blank where the element cannot be written in a constant), C the
computed-array form (`IF(TRUE,<range>)`), D a one-cell argument (that element alone against the first value or date).
Rows past the variants probe the rate and the shapes. Deterministic; run with `python3 -I`.
"""

from __future__ import annotations

import json
from pathlib import Path

OUT = Path(__file__).resolve().parents[3] / "eval/suites/probes/cases/t3815-xnpv-element-reading.case.json"
VALUES = [-100, 30, 40, 50]
DATES = [39448, 39508, 39600, 39700]
NA = "=NA()"
DIV0 = "=1/0"
EMPTY = None  # no cell written

# name, which row (0-based) changes, the element, its array-constant text (None: not writable)
DATE_VARIANTS = [
    ("later fractional, first + 0.5", 1, 39448.5, "39448.5"),
    ("later fractional, first + 1.5", 1, 39449.5, "39449.5"),
    ("first fractional 39448.7, later 39448.2", None, None, None),
    ("later negative", 1, -1, "-1"),
    ("later 0", 1, 0, "0"),
    ("first 0", 0, 0, "0"),
    ("first negative", 0, -1, "-1"),
    ("later past 2958465", 1, 2958466, "2958466"),
    ("last 2958465.5", 3, 2958465.5, "2958465.5"),
    ("later equal to the first", 1, 39448, "39448"),
    ("later before the first", 1, 39400, "39400"),
    ("later unsorted (39650 before 39600)", 1, 39650, "39650"),
    ("later numeric text", 1, "39508", '"39508"'),
    ("later date text", 1, "1/1/2025", '"1/1/2025"'),
    ("later non-numeric text", 1, "x", '"x"'),
    ("later TRUE", 1, True, "TRUE"),
    ("later FALSE", 1, False, "FALSE"),
    ("later empty", 1, EMPTY, None),
    ("later #N/A", 1, NA, "#N/A"),
    ("first numeric text", 0, "39448", '"39448"'),
    ("first empty", 0, EMPTY, None),
    ("first #N/A", 0, NA, "#N/A"),
    ("later -1 then later TRUE", None, None, None),
]
VALUE_VARIANTS = [
    ("later numeric text", 1, "30", '"30"'),
    ("later date text", 1, "1/1/2025", '"1/1/2025"'),
    ("later non-numeric text", 1, "x", '"x"'),
    ("later TRUE", 1, True, "TRUE"),
    ("later empty", 1, EMPTY, None),
    ("first 0", 0, 0, "0"),
    ("first numeric text", 0, "-100", '"-100"'),
    ("first TRUE", 0, True, "TRUE"),
    ("later #N/A", 1, NA, "#N/A"),
    ("first #N/A", 0, NA, "#N/A"),
]


def col_name(i: int) -> str:
    out = ""
    i += 1
    while i:
        i, r = divmod(i - 1, 26)
        out = chr(65 + r) + out
    return out


def constant(items: list[str]) -> str:
    return "{" + ";".join(items) + "}"


def main() -> None:
    data: dict[str, object] = {}
    calc: dict[str, object] = {}
    notes: dict[str, str] = {}
    for r in range(4):
        data[f"A{r + 1}"] = VALUES[r]
        data[f"B{r + 1}"] = DATES[r]
    col = 2
    row = 1

    def add_column(items: list[object]) -> str:
        nonlocal col
        name = col_name(col)
        col += 1
        for r, item in enumerate(items):
            if item is not None:
                data[f"{name}{r + 1}"] = item
        return name

    def emit(label: str, values_rng: str, dates_rng: str, values_const: str | None, dates_const: str | None,
             one_value: str, one_date: str) -> None:
        nonlocal row
        calc[f"A{row}"] = f"=XNPV(0.1,D!{values_rng},D!{dates_rng})"
        if values_const is not None and dates_const is not None:
            calc[f"B{row}"] = f"=XNPV(0.1,{values_const},{dates_const})"
        calc[f"C{row}"] = f"=XNPV(0.1,IF(TRUE,D!{values_rng}),IF(TRUE,D!{dates_rng}))"
        calc[f"D{row}"] = f"=XNPV(0.1,{one_value},{one_date})"
        notes[str(row)] = label
        row += 1

    base_values = constant([str(v) for v in VALUES])
    for label, index, element, text in DATE_VARIANTS:
        dates: list[object] = list(DATES)
        texts = [str(d) for d in DATES]
        if label.startswith("first fractional"):
            dates[0], dates[1] = 39448.7, 39448.2
            texts[0], texts[1] = "39448.7", "39448.2"
            one_date_index = 1
        elif label.startswith("later -1 then"):
            dates[1], dates[2] = -1, True
            texts[1], texts[2] = "-1", "TRUE"
            one_date_index = 1
        else:
            assert index is not None
            dates[index] = element
            texts[index] = text if text is not None else ""
            one_date_index = index
        name = add_column(dates)
        dates_const = None if any(t == "" for t in texts) else constant(texts)
        one_value = "D!A1" if one_date_index == 0 else "D!A2"
        emit(f"dates: {label}", "$A$1:$A$4", f"{name}1:{name}4", base_values, dates_const, one_value,
             f"D!{name}{one_date_index + 1}")
    base_dates = constant([str(d) for d in DATES])
    for label, index, element, text in VALUE_VARIANTS:
        values: list[object] = list(VALUES)
        texts = [str(v) for v in VALUES]
        values[index] = element
        texts[index] = text if text is not None else ""
        name = add_column(values)
        values_const = None if any(t == "" for t in texts) else constant(texts)
        emit(f"values: {label}", f"{name}1:{name}4", "$B$1:$B$4", values_const, base_dates,
             f"D!{name}{index + 1}", "D!B1")
    # Values all 0; an error value against an error date and against a bad earlier date.
    zeros = add_column([0, 0, 0, 0])
    emit("values: all 0", f"{zeros}1:{zeros}4", "$B$1:$B$4", "{0;0;0;0}", base_dates, f"D!{zeros}1", "D!B1")
    v_err3 = add_column([-100, 30, NA, 50])
    d_err2 = add_column([39448, DIV0, 39600, 39700])
    d_true2 = add_column([39448, True, 39600, 39700])
    d_neg2 = add_column([39448, -1, 39600, 39700])
    emit("values #N/A at 3, dates #DIV/0! at 2", f"{v_err3}1:{v_err3}4", f"{d_err2}1:{d_err2}4",
         "{-100;30;#N/A;50}", "{39448;#DIV/0!;39600;39700}", f"D!{v_err3}3", f"D!{d_err2}2")
    emit("values #N/A at 3, dates TRUE at 2", f"{v_err3}1:{v_err3}4", f"{d_true2}1:{d_true2}4",
         "{-100;30;#N/A;50}", "{39448;TRUE;39600;39700}", f"D!{v_err3}3", f"D!{d_true2}2")
    emit("values #N/A at 3, dates -1 at 2", f"{v_err3}1:{v_err3}4", f"{d_neg2}1:{d_neg2}4",
         "{-100;30;#N/A;50}", "{39448;-1;39600;39700}", f"D!{v_err3}3", f"D!{d_neg2}2")
    v_err2 = add_column([-100, NA, 40, 50])
    d_err3 = add_column([39448, 39508, DIV0, 39700])
    emit("values #N/A at 2, dates #DIV/0! at 3", f"{v_err2}1:{v_err2}4", f"{d_err3}1:{d_err3}4",
         "{-100;#N/A;40;50}", "{39448;39508;#DIV/0!;39700}", f"D!{v_err2}2", f"D!{d_err3}3")
    v_true2 = add_column([-100, True, 40, 50])
    emit("values TRUE at 2, dates #DIV/0! at 3", f"{v_true2}1:{v_true2}4", f"{d_err3}1:{d_err3}4",
         "{-100;TRUE;40;50}", "{39448;39508;#DIV/0!;39700}", f"D!{v_true2}2", f"D!{d_err3}3")
    emit("values TRUE at 2, dates -1 at 2", f"{v_true2}1:{v_true2}4", f"{d_neg2}1:{d_neg2}4",
         "{-100;TRUE;40;50}", "{39448;-1;39600;39700}", f"D!{v_true2}2", f"D!{d_neg2}2")
    # The rate (A), with the good values and dates.
    rate_cells = {"text": "0.1", "empty": None, "date text": "1/1/2025"}
    rates = ["-1", "-2", "-0.5", '"0.1"', "D!ZZ1", "D!ZY1", "D!ZX1", "0"]
    data["ZY1"] = rate_cells["text"]
    data["ZX1"] = rate_cells["date text"]
    for rate in rates:
        calc[f"A{row}"] = f"=XNPV({rate},D!$A$1:$A$4,D!$B$1:$B$4)"
        calc[f"B{row}"] = f"=XNPV({rate},{base_values},{base_dates})"
        notes[str(row)] = f"rate {rate}"
        row += 1
    # Shapes.
    for r, v in enumerate(VALUES):
        data[f"{col_name(30 + r)}10"] = v  # a values row
    data["AE11"], data["AF11"], data["AE12"], data["AF12"] = -100, 30, 40, 50
    data["AE14"], data["AF14"], data["AE15"], data["AF15"] = 39448, 39508, 39600, 39700
    shapes = [
        ("values row, dates column", "D!AE10:AH10", "D!$B$1:$B$4"),
        ("values 2x2, dates column", "D!AE11:AF12", "D!$B$1:$B$4"),
        ("values 2x2, dates 2x2", "D!AE11:AF12", "D!AE14:AF15"),
        ("values column, dates 2x2", "D!$A$1:$A$4", "D!AE14:AF15"),
        ("values row constant, dates column constant", "{-100,30,40,50}", base_dates),
        ("values 2x2 constant, dates column", "{-100,30;40,50}", "D!$B$1:$B$4"),
    ]
    for label, v, d in shapes:
        calc[f"A{row}"] = f"=XNPV(0.1,{v},{d})"
        notes[str(row)] = label
        row += 1
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-3815: how XNPV reads each element of its values and dates (fractional, negative, 0 and out-of-range "
            "dates, later dates equal, earlier or unsorted, numeric, date and other text, booleans, empty cells and "
            "errors, in a range (A), an array constant (B), a computed array (C) and as one cell (D)), which error "
            "wins, the rate's forms and the shapes (docs/research/t3815/gen_case.py; rows: "
            + "; ".join(f"{k} {v}" for k, v in notes.items())
            + ")"
        ),
        "tags": ["M9", "financial", "xnpv", "probe", "T-3815"],
        "expected": {},
        "sheets": [{"name": "Calc", "cells": calc}, {"name": "D", "cells": data}],
    }
    OUT.write_text(json.dumps(case, indent=1) + "\n")


if __name__ == "__main__":
    main()
