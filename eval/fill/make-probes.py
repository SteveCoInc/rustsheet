"""Build deterministic fill inputs; never generate Excel observations."""

from __future__ import annotations

import datetime as dt
import hashlib
import json
from pathlib import Path

import xlsxwriter
from xlsxwriter.utility import xl_cell_to_rowcol

ROOT = Path(__file__).resolve().parent
FORMATS = {
    "date": {"num_format": "yyyy-mm-dd"},
    "decimal-red": {"num_format": "0.00", "bold": True, "font_color": "#C00000", "bg_color": "#FFF2CC"},
    "decimal-blue": {"num_format": "0.000", "italic": True, "font_color": "#0070C0", "bg_color": "#DDEBF7"},
    "merged": {"bold": True, "bg_color": "#E2F0D9"},
}


def rectangle(address: str) -> dict[str, int]:
    start, _, end = address.partition(":")
    row_start, col_start = xl_cell_to_rowcol(start)
    row_end, col_end = xl_cell_to_rowcol(end or start)
    return {"r0": row_start, "c0": col_start, "r1": row_end, "c1": col_end}


def date_serial(year: int, month: int, day: int) -> int:
    return (dt.date(year, month, day) - dt.date(1899, 12, 30)).days


def probe(
    name: str,
    kind: str,
    source: str,
    target: str,
    seeds: list[tuple[str, str | float | int | None, str | None]],
    fill_type: str = "FillDefault",
    merges: list[str] | None = None,
) -> dict:
    source_rect, target_rect = rectangle(source), rectangle(target)
    changed = [edge for edge in source_rect if source_rect[edge] != target_rect[edge]]
    if len(changed) != 1:
        raise ValueError(f"{name}: target must extend exactly one source edge")
    edge = changed[0]
    if edge.endswith("0") and target_rect[edge] > source_rect[edge]:
        raise ValueError(f"{name}: target contracts source")
    if edge.endswith("1") and target_rect[edge] < source_rect[edge]:
        raise ValueError(f"{name}: target contracts source")
    date_units = {"FillDays": "day", "FillWeekdays": "weekday", "FillMonths": "month", "FillYears": "year"}
    options: dict = {"mode": "copy" if fill_type == "FillCopy" else "series", "ctrl": False, "series": fill_type == "FillSeries"}
    if fill_type in date_units:
        options["dateUnit"] = date_units[fill_type]
    return {
        "id": name,
        "file": f"{name}.xlsx",
        "sheet": "Fill",
        "source": source,
        "target": target,
        "sourceRect": source_rect,
        "targetRect": target_rect,
        "direction": {"r0": "up", "r1": "down", "c0": "left", "c1": "right"}[edge],
        "length": abs(target_rect[edge] - source_rect[edge]),
        "lengthUnit": "rows" if edge.startswith("r") else "columns",
        "seriesKindToProbe": kind,
        "autoFillType": fill_type,
        "engineOptions": options,
        "seeds": [{"cell": cell, "value": value, "format": style} for cell, value, style in seeds],
        "merges": merges or [],
        "recording": {"graph": "no documented autofill operation in v1.0", "officeJs": "Range.autoFill; actual behavior unverified"},
    }


def cases() -> list[dict]:
    return [
        probe("number-single-default", "single-number default", "A1", "A1:A6", [("A1", 7, None)]),
        probe("number-single-series", "single-number forced series", "A1", "A1:A6", [("A1", 7, None)], "FillSeries"),
        probe("number-step-down", "two-number linear", "A1:A2", "A1:A7", [("A1", 1, None), ("A2", 3, None)]),
        probe("number-step-up", "two-number linear backwards", "A4:A5", "A1:A5", [("A4", 7, None), ("A5", 9, None)]),
        probe("number-step-right", "two-number linear horizontal", "B2:C2", "B2:H2", [("B2", 10, None), ("C2", 20, None)]),
        probe("number-step-left", "two-number linear horizontal backwards", "E2:F2", "A2:F2", [("E2", 10, None), ("F2", 20, None)]),
        probe("number-negative-decimal", "negative decimal linear", "A1:A2", "A1:A6", [("A1", -1.5, None), ("A2", -1.25, None)]),
        probe("number-copy", "explicit repeating copy", "A1:A2", "A1:A7", [("A1", 2, None), ("A2", 4, None)], "FillCopy"),
        probe("date-days-leap", "daily date across leap day", "A1", "A1:A5", [("A1", date_serial(2024, 2, 28), "date")], "FillDays"),
        probe("date-weekdays", "weekday dates skipping weekend", "A1", "A1:A6", [("A1", date_serial(2026, 11, 6), "date")], "FillWeekdays"),
        probe("date-month-end", "monthly date from month end", "A1", "A1:A5", [("A1", date_serial(2024, 1, 31), "date")], "FillMonths"),
        probe("date-years-leap", "yearly date from leap day", "A1", "A1:A6", [("A1", date_serial(2024, 2, 29), "date")], "FillYears"),
        probe("date-two-seeds", "two-date inferred interval", "A1:A2", "A1:A6", [("A1", date_serial(2026, 11, 9), "date"), ("A2", date_serial(2026, 11, 11), "date")]),
        probe("weekday-names", "English weekday custom list", "A1:A2", "A1:A8", [("A1", "Mon", None), ("A2", "Tue", None)]),
        probe("text-trailing-padded", "trailing digits with leading zeros", "A1:A2", "A1:A6", [("A1", "Item007", None), ("A2", "Item009", None)]),
        probe("text-trailing-single", "single trailing-number text", "A1", "A1:A5", [("A1", "Part 9", None)]),
        probe("formula-relative-down", "relative A1 formula", "B2", "B2:B6", [("B2", "=A2*2", None), *[(f"A{row}", row, None) for row in range(1, 7)]]),
        probe("formula-mixed-down", "relative and absolute row/column references", "B2", "B2:B6", [("B2", "=A2+$A2+A$1+$A$1", None), *[(f"A{row}", row, None) for row in range(1, 7)]]),
        probe("formula-mixed-right", "relative and absolute horizontal references", "B2", "B2:F2", [("B2", "=B1+$A1+B$1+$A$1", None), *[(f"{column}1", index, None) for index, column in enumerate("ABCDEF", 1)]]),
        probe("formula-relative-up", "backward formula adjustment", "B5", "B2:B5", [("B5", "=A4+$A$1", None), *[(f"A{row}", row, None) for row in range(1, 6)]]),
        probe("formatted-series", "numeric series with alternating source formats", "A1:A2", "A1:A6", [("A1", 1.25, "decimal-red"), ("A2", 1.5, "decimal-blue")]),
        probe("formatted-blank", "number and styled blank", "A1:A2", "A1:A6", [("A1", 2, "decimal-red"), ("A2", None, "decimal-blue")]),
        probe("merged-source-tiled", "merged source complete tiles", "A1:B1", "A1:F1", [("A1", "Tile 1", "merged")], merges=["A1:B1"]),
        probe("merged-source-partial", "merged source partial final tile or refusal", "A1:B1", "A1:E1", [("A1", "Tile 1", "merged")], merges=["A1:B1"]),
    ]


def build() -> None:
    output = ROOT / "probes"
    output.mkdir(exist_ok=True)
    entries = cases()
    for entry in entries:
        path = output / entry["file"]
        workbook = xlsxwriter.Workbook(path, {"in_memory": True})
        workbook.set_properties({"title": entry["id"], "created": dt.datetime(2026, 11, 10, tzinfo=dt.timezone.utc)})
        workbook.set_calc_mode("auto")
        sheet = workbook.add_worksheet(entry["sheet"])
        formats = {name: workbook.add_format(properties) for name, properties in FORMATS.items()}
        for merge in entry["merges"]:
            sheet.merge_range(merge, "", formats["merged"])
        for seed in entry["seeds"]:
            sheet.write(seed["cell"], seed["value"], formats.get(seed["format"]))
        workbook.close()
        entry["sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
    manifest = {"schema": "rustsheet.fill-probes/1", "dateSystem": "1900", "locale": "en-US", "formats": FORMATS, "probes": entries}
    (output / "probes.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    (output / "SHA256SUMS").write_text("".join(f"{entry['sha256']}  {entry['file']}\n" for entry in entries), encoding="utf-8")
    print(f"Built {len(entries)} input workbooks; no Excel results generated")


if __name__ == "__main__":
    build()
