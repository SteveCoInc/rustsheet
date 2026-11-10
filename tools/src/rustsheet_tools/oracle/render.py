"""Render references: what Excel for the web shows for the render probes (`eval/render/`).

`rustsheet-oracle render` opens each pinned probe workbook through Microsoft Graph and records
the tier-1 dump `eval/render/README.md` specifies (`rustsheet.excel-render-graph/1`): every column
width, row height and cell of each sheet's probe range, plus every chart's placement. Beside it
go each chart's image and the workbook's print PDF (Excel's own page layout, the visual
reference). Tier-2 fields (merges, frozen panes, cell geometry, `####` fitting) are not readable
through Graph and are not recorded here.

Each property is read for one cell, one column or one row at a time, because Graph returns null
for a property that is not uniform across a range. `$batch` keeps the reads to a few hundred calls.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import sys
from datetime import UTC, datetime
from pathlib import Path
from typing import Any
from urllib.parse import quote

from xlsxwriter.utility import xl_col_to_name, xl_rowcol_to_cell

from rustsheet_tools import repo_root
from rustsheet_tools.oracle.backends import (
    OPEN_REFUSALS,
    GraphBackend,
    GraphRequestError,
    OracleError,
    OracleRefusedError,
)
from rustsheet_tools.spec2xlsx import parse_a1

SCHEMA = "rustsheet.excel-render-graph/1"
REFUSED_SCHEMA = "rustsheet.excel-render-refused/1"
PROBES_SCHEMA = "rustsheet.render-probes/1"
BORDER_SIDES = ("EdgeLeft", "EdgeTop", "EdgeRight", "EdgeBottom", "DiagonalUp", "DiagonalDown")
CELL_FIELDS = ("a1", "text", "value", "valueType", "numberFormat", "hAlign", "vAlign", "wrap")
FONT_FIELDS = ("name", "size", "bold", "italic", "underline", "color")
CHART_FIELDS = ("name", "left", "top", "width", "height")
MAX_PROBE_CELLS = 5_000  # per sheet: the probes hold at most a few hundred (directive 5)


class RenderError(OracleError):
    """A probe that cannot be recorded or a dump that does not cover its probe."""


def render_dir() -> Path:
    return repo_root() / "eval" / "render"


def load_probes(probe_dir: Path) -> list[dict[str, Any]]:
    """The probes of `probe_dir/probes.json` (`rustsheet.render-probes/1`)."""
    manifest = json.loads((probe_dir / "probes.json").read_text())
    if manifest.get("schema") != PROBES_SCHEMA:
        raise RenderError(f"{probe_dir / 'probes.json'}: schema is not {PROBES_SCHEMA}")
    probes: list[dict[str, Any]] = manifest["probes"]
    for probe in probes:
        # Both name files: the id the outputs, the file the input. Neither may leave its directory.
        for key in ("id", "file"):
            value = str(probe.get(key, ""))
            if not value or "/" in value or "\\" in value or value.startswith("."):
                raise RenderError(f"probes.json: probe {key} {value!r} is not a plain file name")
    return probes


def pinned_bytes(probe_dir: Path, probe: dict[str, Any]) -> bytes:
    """The probe's workbook, refused unless its sha256 is the pin: the bytes are the probe."""
    data: bytes = (probe_dir / str(probe["file"])).read_bytes()
    if hashlib.sha256(data).hexdigest() != probe["sha256"]:
        raise RenderError(f"{probe['file']}: sha256 differs from probes.json; do not edit or re-save probes")
    return data


def range_bounds(rng: str) -> tuple[int, int, int, int]:
    """Zero-based (top, left, bottom, right) of an A1 range `B2:D9` (or one cell)."""
    first, _, last = rng.replace("$", "").partition(":")
    (top, left), (bottom, right) = parse_a1(first), parse_a1(last or first)
    if bottom < top or right < left:
        raise RenderError(f"range {rng!r} is reversed")
    if (bottom - top + 1) * (right - left + 1) > MAX_PROBE_CELLS:
        raise RenderError(f"range {rng!r} has more than {MAX_PROBE_CELLS} cells")
    return top, left, bottom, right


def _sheet_url(item_url: str, sheet: str) -> str:
    ref = quote(sheet.replace("'", "''"), safe="")
    return f"{item_url}/workbook/worksheets('{ref}')"


def _range_url(ws: str, address: str, tail: str = "") -> str:
    return f"{ws}/range(address='{quote(address, safe='')}'){tail}"


def cell_entry(a1: str, shown: dict[str, Any], fmt: Any) -> dict[str, Any]:
    """One `cells[]` entry from `shown` (the cell's `text`, `value`, `valueType` and `numberFormat`,
    from the range's arrays) and the cell's own `format` read with
    `$expand=font,fill,borders`. Values are kept as Graph returns them, nulls and empty strings
    included: what an unset property returns is part of the record."""
    font = fmt.get("font") or {}
    # Expanded, `borders` is the bare list; read on its own it is `{"value": [...]}`.
    sides = fmt.get("borders") or []
    sides = sides.get("value", []) if isinstance(sides, dict) else sides
    borders = {b.get("sideIndex"): b for b in sides}
    return {
        "a1": a1,
        **{k: shown[k] for k in ("text", "value", "valueType", "numberFormat")},
        "hAlign": fmt.get("horizontalAlignment"),
        "vAlign": fmt.get("verticalAlignment"),
        "wrap": fmt.get("wrapText"),
        "font": {k: font.get(k) for k in FONT_FIELDS},
        "fill": {"color": (fmt.get("fill") or {}).get("color")},
        "borders": {
            side: {k: borders[side].get(k) for k in ("style", "weight", "color")} if side in borders else {}
            for side in BORDER_SIDES
        },
    }


def read_sheet(
    graph: GraphBackend, item_url: str, sid: dict[str, str], name: str, rng: str
) -> dict[str, Any]:
    """The tier-1 record of sheet `name` over its probe range `rng`."""
    top, left, bottom, right = range_bounds(rng)
    ws = _sheet_url(item_url, name)
    block = graph._call(
        "GET", _range_url(ws, rng, "?$select=text,values,valueTypes,numberFormat"), headers=sid
    ).json()
    cols = [xl_col_to_name(c) for c in range(left, right + 1)]
    rows = list(range(top + 1, bottom + 2))
    cells = [(r, c) for r in range(top, bottom + 1) for c in range(left, right + 1)]
    urls = (
        [_range_url(ws, f"{c}:{c}", "?$select=columnHidden") for c in cols]
        + [_range_url(ws, f"{c}:{c}", "/format?$select=columnWidth") for c in cols]
        + [_range_url(ws, f"{r}:{r}", "?$select=rowHidden") for r in rows]
        + [_range_url(ws, f"{r}:{r}", "/format?$select=rowHeight") for r in rows]
        + [_range_url(ws, xl_rowcol_to_cell(r, c), "/format?$expand=font,fill,borders") for r, c in cells]
    )
    bodies = graph.batch_get(urls, sid)
    n, m = len(cols), len(rows)
    col_hidden, col_fmt = bodies[:n], bodies[n : 2 * n]
    row_hidden, row_fmt = bodies[2 * n : 2 * n + m], bodies[2 * n + m : 2 * n + 2 * m]
    cell_fmt = bodies[2 * n + 2 * m :]

    def at(key: str, r: int, c: int) -> Any:
        grid = block.get(key)
        try:
            return grid[r - top][c - left]
        except (TypeError, IndexError) as e:
            raise RenderError(f"{name}!{xl_rowcol_to_cell(r, c)}: Graph returned no {key}") from e

    charts = (
        graph._call("GET", f"{ws}/charts?$select={','.join(CHART_FIELDS)}", headers=sid)
        .json()
        .get("value", [])
    )
    return {
        "name": name,
        "columns": [
            {"index": left + 1 + i, "columnWidth": f.get("columnWidth"), "hidden": h.get("columnHidden")}
            for i, (h, f) in enumerate(zip(col_hidden, col_fmt, strict=True))
        ],
        "rows": [
            {"index": rows[i], "rowHeight": f.get("rowHeight"), "hidden": h.get("rowHidden")}
            for i, (h, f) in enumerate(zip(row_hidden, row_fmt, strict=True))
        ],
        "cells": [
            cell_entry(
                xl_rowcol_to_cell(r, c),
                {
                    "text": at("text", r, c),
                    "value": at("values", r, c),
                    "valueType": at("valueTypes", r, c),
                    "numberFormat": at("numberFormat", r, c),
                },
                fmt,
            )
            for (r, c), fmt in zip(cells, cell_fmt, strict=True)
        ],
        "charts": [{k: chart.get(k) for k in CHART_FIELDS} for chart in charts],
    }


def record(
    graph: GraphBackend, probe_dir: Path, probe: dict[str, Any], out: Path, *, pdf: bool, images: bool
) -> dict[str, Any]:
    """Records `probe` into `out`: `<id>.render.json`, and when asked `<id>.pdf` (the print
    layout) and `<id>/<chart>.png`. Returns the dump."""
    data = pinned_bytes(probe_dir, probe)
    drive = graph._drive()
    item = graph._call(
        "PUT", f"{drive}/root:/{graph.folder}/render-{probe['sha256']}.xlsx:/content", data=data
    ).json()
    item_url = f"{drive}/items/{item['id']}"
    pngs: dict[str, bytes] = {}
    try:
        # No edits are made, and none may persist: the probe stays as uploaded.
        try:
            session = graph._call(
                "POST", f"{item_url}/workbook/createSession", json={"persistChanges": False}
            ).json()
        except GraphRequestError as e:
            if e.code in OPEN_REFUSALS:
                raise OracleRefusedError(e.code, e.message) from e
            raise
        sid = {"workbook-session-id": session["id"]}
        sheets = [read_sheet(graph, item_url, sid, s["name"], s["range"]) for s in probe["sheets"]]
        if images:
            for sheet in sheets:
                ws = _sheet_url(item_url, sheet["name"])
                for chart in sheet["charts"]:
                    ref = quote(str(chart["name"]).replace("'", "''"), safe="")
                    img = graph._call("GET", f"{ws}/charts('{ref}')/image", headers=sid).json()
                    pngs[f"{sheet['name']}/{chart['name']}"] = base64.b64decode(img["value"])
        graph._call("POST", f"{item_url}/workbook/closeSession", headers=sid)
        print_pdf = graph._call("GET", f"{item_url}/content?format=pdf").content if pdf else None
    finally:
        try:
            graph._call("DELETE", item_url)
        except OracleError as e:
            print(f"WARN could not delete the scratch copy of {probe['file']}: {e}", file=sys.stderr)
    dump = {
        "schema": SCHEMA,
        "probe": probe["id"],
        "sha256": probe["sha256"],
        "source": {"api": "graph-v1.0", "recordedAt": _now()},
        "sheets": sheets,
    }
    if problems := check_dump(dump, probe):
        raise RenderError(f"{probe['id']}: incomplete dump: " + "; ".join(problems[:10]))
    out.mkdir(parents=True, exist_ok=True)
    (out / f"{probe['id']}.render.json").write_text(json.dumps(dump, indent=1, ensure_ascii=False) + "\n")
    if print_pdf is not None:
        (out / f"{probe['id']}.pdf").write_bytes(print_pdf)
    for key, png in pngs.items():
        path = out / probe["id"] / f"{_file_safe(key)}.png"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(png)
    return dump


def _now() -> str:
    return datetime.now(UTC).strftime("%Y-%m-%dT%H:%M:%SZ")


def write_refusal(probe: dict[str, Any], out: Path, e: OracleRefusedError) -> Path:
    """Records that Excel for the web would not open `probe`: the refusal is the probe's result (no
    dump can exist for it), so it is kept beside the dumps rather than lost in a log."""
    path = out / f"{probe['id']}.refused.json"
    out.mkdir(parents=True, exist_ok=True)
    record_ = {
        "schema": REFUSED_SCHEMA,
        "probe": probe["id"],
        "sha256": probe["sha256"],
        "source": {"api": "graph-v1.0", "recordedAt": _now()},
        "code": e.code,
        "detail": str(e),
    }
    path.write_text(json.dumps(record_, indent=1, ensure_ascii=False) + "\n")
    return path


def _file_safe(name: str) -> str:
    return "".join(ch if ch.isalnum() or ch in "-_ " else "_" for ch in name).strip()


def check_dump(dump: dict[str, Any], probe: dict[str, Any]) -> list[str]:
    """Why `dump` cannot stand for `probe` (empty when it can): the gate compares only what a dump
    holds, so one that lists more or fewer columns, rows or cells than the probe range, or lacks a
    field, is refused rather than compared."""
    problems: list[str] = []
    if dump.get("schema") != SCHEMA:
        problems.append(f"schema is not {SCHEMA}")
    if dump.get("probe") != probe["id"] or dump.get("sha256") != probe["sha256"]:
        problems.append("probe id or sha256 does not match probes.json")
    sheets = {s.get("name"): s for s in dump.get("sheets", [])}
    if sorted(sheets) != sorted(s["name"] for s in probe["sheets"]):
        problems.append(f"sheets {sorted(sheets)} are not the probe's")
    for want in probe["sheets"]:
        sheet = sheets.get(want["name"])
        if sheet is None:
            continue
        top, left, bottom, right = range_bounds(want["range"])
        where = want["name"]
        cols = [c.get("index") for c in sheet.get("columns", [])]
        if cols != list(range(left + 1, right + 2)):
            problems.append(f"{where}: columns are not {want['range']}'s, once each")
        rows = [r.get("index") for r in sheet.get("rows", [])]
        if rows != list(range(top + 1, bottom + 2)):
            problems.append(f"{where}: rows are not {want['range']}'s, once each")
        expected = [xl_rowcol_to_cell(r, c) for r in range(top, bottom + 1) for c in range(left, right + 1)]
        if [c.get("a1") for c in sheet.get("cells", [])] != expected:
            problems.append(f"{where}: cells are not {want['range']}'s, once each")
        problems += [
            f"{where}: a column lacks {k}"
            for k in ("columnWidth", "hidden")
            if any(k not in c for c in sheet.get("columns", []))
        ]
        problems += [
            f"{where}: a row lacks {k}"
            for k in ("rowHeight", "hidden")
            if any(k not in r for r in sheet.get("rows", []))
        ]
        for cell in sheet.get("cells", []):
            missing = [k for k in CELL_FIELDS if k not in cell]
            missing += [f"font.{k}" for k in FONT_FIELDS if k not in cell.get("font", {})]
            missing += ["fill.color"] if "color" not in cell.get("fill", {}) else []
            missing += [f"borders.{s}" for s in BORDER_SIDES if s not in cell.get("borders", {})]
            if missing:
                problems.append(f"{where}!{cell.get('a1')}: lacks {', '.join(missing)}")
        if "charts" not in sheet or any(
            any(k not in chart for k in CHART_FIELDS) for chart in sheet["charts"]
        ):
            problems.append(f"{where}: charts missing or incomplete")
    return problems


def cmd_render(args: argparse.Namespace) -> int:
    """Records the tier-1 references of the selected probes (all by default), or with `--check`
    only validates the dumps already recorded."""
    probes = load_probes(args.probes)
    if args.probe:
        unknown = set(args.probe) - {p["id"] for p in probes}
        if unknown:
            sys.exit(f"unknown probe ids: {sorted(unknown)}")
        probes = [p for p in probes if p["id"] in args.probe]
    failed = 0
    if args.check:
        for probe in probes:
            path = args.out / f"{probe['id']}.render.json"
            refused = args.out / f"{probe['id']}.refused.json"
            if not path.exists() and refused.exists():
                # A recorded refusal is the probe's result; it is reported, and the gate sees INCOMPLETE.
                refusal = json.loads(refused.read_text())
                ok = refusal.get("schema") == REFUSED_SCHEMA and refusal.get("sha256") == probe["sha256"]
                print(
                    f"{probe['id']}: refused by Excel ({refusal.get('code')})"
                    if ok
                    else f"{probe['id']}: stale refusal"
                )
                failed += not ok
                continue
            problems = check_dump(json.loads(path.read_text()), probe) if path.exists() else ["not recorded"]
            for problem in problems:
                print(f"{probe['id']}: {problem}")
            failed += bool(problems)
        return 1 if failed else 0
    graph = GraphBackend()
    ok, why = graph.available()
    if not ok:
        sys.exit(f"excel-graph unavailable: {why}")
    for probe in probes:
        try:
            dump = record(graph, args.probes, probe, args.out, pdf=not args.no_pdf, images=not args.no_images)
        except OracleRefusedError as e:
            failed += 1
            print(f"REFUSED {probe['id']}: {e}; wrote {write_refusal(probe, args.out, e)}")
            continue
        except OracleError as e:
            failed += 1
            print(f"FAIL {probe['id']}: {e}", file=sys.stderr)
            continue
        cells = sum(len(s["cells"]) for s in dump["sheets"])
        charts = sum(len(s["charts"]) for s in dump["sheets"])
        print(f"recorded {probe['id']}: {len(dump['sheets'])} sheets, {cells} cells, {charts} charts")
    return 1 if failed else 0


def add_parser(sub: Any) -> None:
    """Adds the `render` subcommand to `rustsheet-oracle`."""
    p = sub.add_parser("render", help="record Excel for the web's tier-1 render references (eval/render)")
    p.add_argument(
        "--probes", type=Path, default=render_dir() / "probes", help="directory holding probes.json"
    )
    p.add_argument("--out", type=Path, default=render_dir() / "refs" / "graph")
    p.add_argument("--probe", action="append", default=[], help="only this probe id (repeatable)")
    p.add_argument("--no-pdf", action="store_true", help="skip the print PDF")
    p.add_argument("--no-images", action="store_true", help="skip the chart images")
    p.add_argument("--check", action="store_true", help="only validate the recorded dumps, offline")
    p.set_defaults(fn=cmd_render)
