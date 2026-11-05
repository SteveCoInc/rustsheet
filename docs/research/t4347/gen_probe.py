"""T-4347: write probes/t4347-numeral-bytes-24-27 (numeral bytes 0x24-0x27 over General, digits, fields, elapsed).

Run from the repo root: python3 -I docs/research/t4347/gen_probe.py. One row per (byte, question), column A; the
same questions for each byte, plus the `,N00` suffix form of each byte on two of them.
"""

import json

OUT = "eval/suites/probes/cases/t4347-numeral-bytes-24-27.case.json"
QUESTIONS = [
    (5, "General"),
    (10, "General"),
    (15, "General"),
    (20, "General"),
    (100, "General"),
    (105, "General"),
    (1010, "General"),
    (1230, "General"),
    (12345, "General"),
    (100000, "General"),
    (123456789, "General"),
    (100050000, "General"),
    (1000000005, "General"),
    (0.05, "General"),
    (12.05, "General"),
    (-15, "General"),
    (1230, "0"),
    (1234567890, "0"),
    (1234567.89, "#,##0.00"),
    (0.1234, "0.0%"),
    (1000, "0.00E+00"),
    (12.5, "# ?/?"),
    (45000, "yyyy-mm-dd yy d"),
    (45000.7027, "h:mm:ss"),
    (45000.0417, "hh:mm"),
    (45210, "m d dd"),
    (1.25, "[h]:mm"),
    (100000, "[s]"),
    (3600000, "[s]"),
    (0.2, "[hh]"),
]


def main() -> None:
    cells: dict[str, str] = {}
    row = 0
    for byte in (0x24, 0x25, 0x26, 0x27):
        for value, fmt in QUESTIONS:
            row += 1
            cells[f"A{row}"] = f'=TEXT({value},"[$-{byte:X}000409]{fmt}")'
        for value, fmt in ((12345, "General"), (45000, "yyyy-mm-dd")):
            row += 1
            cells[f"A{row}"] = f'=TEXT({value},"[$-,{byte:X}00]{fmt}")'
    # Appended after the first fill: 10^12 and past 10^16 in an elapsed count (`[s]` of 1E8 days is 8.64E12 s,
    # of 1.2E11 days 1.0368E16 s), and a zero gap after 兆 (`[s]` of 1.2345E11 days).
    for byte in (0x24, 0x25, 0x26, 0x27):
        for value in (100000000, 120000000000, 123450000000):
            row += 1
            cells[f"A{row}"] = f'=TEXT({value},"[$-{byte:X}000409][s]")'
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-4347: numeral bytes 0x24-0x27 (hex LCID form [$-24000409] to [$-27000409], and the ,2400 to "
        ",2700 suffix) over General, digit placeholders, percent, scientific, fractions, date and clock fields and "
        "elapsed counts; 32 rows per byte in byte order (rows 1-128), then [s] of 1E8, 1.2E11 and 1.2345E11 per byte (129-140).",
        "tags": ["M10", "probe", "number-format", "T-4347"],
        "expected": {},
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    with open(OUT, "w") as fh:
        json.dump(case, fh, indent=1, ensure_ascii=False)
        fh.write("\n")
    print(row, "cells")


if __name__ == "__main__":
    main()
