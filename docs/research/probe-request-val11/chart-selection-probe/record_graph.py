"""Records the chart-selection and drawing-anchor probes on Excel for the web through Microsoft Graph.

Both READMEs (docs/research/chart-selection-probe, docs/research/drawing-anchor-probe) allow a Graph run, labelled
as Excel for the web. Each probe is one workbook with one sheet per case, built in a session that persists, saved
by Excel and downloaded. The answers are read from the saved parts:
- chart-selection: per series `c:tx`, `c:cat` (`numRef` or `strRef`) and `c:val` references, and the category
  axis kind (`c:catAx` or `c:dateAx`);
- drawing-anchor: `xdr:from` and `xdr:to` of the chart, and the chart's left/top/width/height in points before
  and after the operation, as Graph reports them.

Run from tools/ (it uses the oracle's Graph client):
    uv run python ../docs/research/chart-selection-probe/record_graph.py <out-dir>
"""

from __future__ import annotations

import io
import json
import re
import sys
import time
import uuid
import zipfile
from pathlib import Path
from typing import Any
from urllib.parse import quote

import xlsxwriter

from rustsheet_tools.oracle.backends import GraphBackend, OracleError

MONTHS = [45292 + 31 * i for i in range(12)]
SELECTION: dict[str, dict[str, Any]] = {
    "P1": {
        "range": "A1:D13",
        "values": [["Month", "Revenue", "Cost", "Margin"]]
        + [[d, 100 + i, 60 + i, 0.4] for i, d in enumerate(MONTHS)],
        "dates": ("A2:A13", 12, 1),
    },
    "P2": {"range": "A1:A5", "values": [["Cost"], [4], [5], [7], [6]]},
    "P3": {"range": "A1:D1", "values": [["Cost", 4, 5, 7]]},
    "P4": {
        "range": "A1:B6",
        "values": [["Year", "Sales"], [2014, 10], [2015, 12], [2016, 9], [2017, 14], [2018, 15]],
    },
    "P5": {"range": "A1:C4", "values": [["", 2023, 2024], ["North", 5, 6], ["South", 7, 8], ["East", 1, 2]]},
    "P6": {
        "range": "A1:C4",
        "values": [["Region", 45292, 45323], ["North", 5, 6], ["South", 7, 8], ["East", 1, 2]],
        "dates": ("B1:C1", 1, 2),
    },
    "P7": {
        "range": "A1:C4",
        "values": [["Region", 10, 20], ["North", 5, 6], ["South", 7, 8], ["East", 1, 2]],
    },
}
# (method, range tail, body): the anchor cases of drawing-anchor-probe/cases.js, in Graph's terms.
ANCHOR: dict[str, tuple[str, str, dict[str, Any]]] = {
    "R1": ("PATCH", "range(address='3:3')/format", {"rowHeight": 30}),
    "R2": ("PATCH", "range(address='12:12')/format", {"rowHeight": 30}),
    "R3": ("PATCH", "range(address='6:6')/format", {"rowHeight": 30}),
    "R4": ("PATCH", "range(address='3:3')/format", {"rowHeight": 5}),
    "C1": ("PATCH", "range(address='C:C')/format", {"columnWidth": 150}),
    "H1": ("PATCH", "range(address='3:3')", {"rowHidden": True}),
    "H2": ("PATCH", "range(address='12:12')", {"rowHidden": True}),
    "H3": ("PATCH", "range(address='6:6')", {"rowHidden": True}),
    "H4": ("PATCH", "range(address='C:C')", {"columnHidden": True}),
    "D1": ("POST", "range(address='3:3')/delete", {"shift": "Up"}),
    "D2": ("POST", "range(address='12:12')/delete", {"shift": "Up"}),
    "D3": ("POST", "range(address='3:12')/delete", {"shift": "Up"}),
    "D4": ("POST", "range(address='C:C')/delete", {"shift": "Left"}),
}
# The default grid (Calibri 11): rows 15 pt, columns 48 pt. Mid C3 and mid H12, as cases.js places the chart.
PLACE = {
    "left": 2 * 48 + 24,
    "top": 2 * 15 + 7.5,
    "width": (7 * 48 + 24) - (2 * 48 + 24),
    "height": (11 * 15 + 7.5) - (2 * 15 + 7.5),
}
FIELDS = ("left", "top", "width", "height")


def blank(sheets: list[str]) -> bytes:
    buf = io.BytesIO()
    wb = xlsxwriter.Workbook(buf, {"in_memory": True})
    for s in sheets:
        wb.add_worksheet(s)
    wb.close()
    return buf.getvalue()


def ws_url(item: str, sheet: str) -> str:
    return f"{item}/workbook/worksheets('{quote(sheet, safe='')}')"


def session_run(g: GraphBackend, sheets: list[str], build: Any) -> tuple[bytes, Any]:
    """Uploads a blank workbook of `sheets`, runs `build(item, sid)` in a persisting session, waits for Excel
    to save, and returns (the saved bytes, build's result). The scratch copy is deleted."""
    drive = g._drive()
    name = f"probe-{uuid.uuid4().hex[:12]}.xlsx"
    item = g._call("PUT", f"{drive}/root:/{g.folder}/{name}:/content", data=blank(sheets)).json()
    url = f"{drive}/items/{item['id']}"
    try:
        sid = {
            "workbook-session-id": g._call(
                "POST", f"{url}/workbook/createSession", json={"persistChanges": True}
            ).json()["id"]
        }
        result = build(url, sid)
        g._call("POST", f"{url}/workbook/closeSession", headers=sid)
        for _ in range(45):
            if g._call("GET", url).json().get("eTag") != item.get("eTag"):
                break
            time.sleep(2)
        else:
            raise OracleError("Excel never saved the workbook (eTag unchanged after 90 s)")
        return g._call("GET", f"{url}/content").content, result
    finally:
        g._call("DELETE", url)


def place(g: GraphBackend, ws: str, sid: dict[str, str], source: str) -> str:
    chart = g._call(
        "POST",
        f"{ws}/charts/add",
        headers=sid,
        json={"type": "ColumnClustered", "sourceData": source, "seriesBy": "Auto"},
    ).json()
    return str(chart["name"])


def box(g: GraphBackend, ws: str, sid: dict[str, str]) -> list[dict[str, Any]]:
    charts = g._call("GET", f"{ws}/charts?$select=name,left,top,width,height", headers=sid).json()["value"]
    return [{k: c.get(k) for k in ("name", *FIELDS)} for c in charts]


def build_selection(g: GraphBackend) -> Any:
    def build(item: str, sid: dict[str, str]) -> None:
        for case, c in SELECTION.items():
            ws = ws_url(item, case)
            g._call("PATCH", f"{ws}/range(address='{c['range']}')", headers=sid, json={"values": c["values"]})
            if "dates" in c:
                rng, rows, cols = c["dates"]
                g._call(
                    "PATCH",
                    f"{ws}/range(address='{rng}')",
                    headers=sid,
                    json={"numberFormat": [["mmm-yy"] * cols for _ in range(rows)]},
                )
            place(g, ws, sid, c["range"])

    return session_run(g, list(SELECTION), build)


def build_anchor(g: GraphBackend) -> Any:
    def build(item: str, sid: dict[str, str]) -> dict[str, Any]:
        log: dict[str, Any] = {}
        for case, (method, tail, body) in ANCHOR.items():
            ws = ws_url(item, case)
            g._call(
                "PATCH",
                f"{ws}/range(address='A1:B4')",
                headers=sid,
                json={"values": [["Name", "Value"], ["a", 10], ["b", 20], ["c", 15]]},
            )
            name = place(g, ws, sid, "A1:B4")
            g._call("PATCH", f"{ws}/charts('{quote(name, safe='')}')", headers=sid, json=PLACE)
            before = box(g, ws, sid)
            resp = g._call(method, f"{ws}/{tail}", headers=sid, json=body)
            log[case] = {
                "operation": f"{method} {tail} {json.dumps(body)}",
                "status": resp.status_code,
                "before": before,
                "after": box(g, ws, sid),
            }
        return log

    return session_run(g, list(ANCHOR), build)


def parts_by_sheet(xlsx: bytes) -> dict[str, dict[str, str]]:
    """Sheet name -> {'drawing': xml, 'charts': [xml, ...]} from the saved package."""
    z = zipfile.ZipFile(io.BytesIO(xlsx))
    wb = z.read("xl/workbook.xml").decode()
    rels = z.read("xl/_rels/workbook.xml.rels").decode()
    target = dict(re.findall(r'Id="(rId\d+)"[^>]*Target="([^"]+)"', rels))
    target.update({k: v for v, k in re.findall(r'Target="([^"]+)"[^>]*Id="(rId\d+)"', rels)})
    out: dict[str, dict[str, Any]] = {}
    for name, rid in re.findall(r'<sheet [^>]*name="([^"]+)"[^>]*r:id="(rId\d+)"', wb):
        sheet_path = "xl/" + target[rid].lstrip("/").removeprefix("xl/")
        srels = sheet_path.replace("worksheets/", "worksheets/_rels/") + ".rels"
        entry: dict[str, Any] = {"drawing": "", "charts": []}
        if srels in z.namelist():
            for t in re.findall(r'Target="([^"]*drawing[^"]*)"', z.read(srels).decode()):
                dpath = "xl/drawings/" + t.split("/")[-1]
                entry["drawing"] = z.read(dpath).decode()
                drels = dpath.replace("drawings/", "drawings/_rels/") + ".rels"
                for ct in re.findall(r'Target="([^"]*chart[^"]*)"', z.read(drels).decode()):
                    entry["charts"].append(z.read("xl/charts/" + ct.split("/")[-1]).decode())
        out[name] = entry
    return out


def series_of(chart: str) -> dict[str, Any]:
    series = []
    for series_xml in re.findall(r"<c:ser>(.*?)</c:ser>", chart, re.S):

        def ref(tag: str, s: str = series_xml) -> dict[str, str] | None:
            m = re.search(rf"<c:{tag}>(.*?)</c:{tag}>", s, re.S)
            if not m:
                return None
            kind = re.search(r"<c:(numRef|strRef|multiLvlStrRef|numLit|strLit)>", m[1])
            f = re.search(r"<c:f>(.*?)</c:f>", m[1])
            return {"kind": kind[1] if kind else None, "f": f[1] if f else None}

        series.append({"tx": ref("tx"), "cat": ref("cat"), "val": ref("val")})
    axes = [a for a in ("catAx", "dateAx", "valAx") if f"<c:{a}>" in chart]
    return {"series": series, "axes": axes}


def anchors_of(drawing: str) -> list[dict[str, Any]]:
    out = []
    for kind, body in re.findall(
        r"<xdr:(twoCellAnchor|oneCellAnchor|absoluteAnchor)([^>]*>.*?)</xdr:\1>", drawing, re.S
    ):

        def corner(tag: str, b: str = body) -> dict[str, int] | None:
            m = re.search(rf"<xdr:{tag}>(.*?)</xdr:{tag}>", b, re.S)
            if not m:
                return None
            return {k: int(v) for k, v in re.findall(r"<xdr:(col|colOff|row|rowOff)>(-?\d+)</xdr:\1>", m[1])}

        edit = re.search(r'editAs="([^"]+)"', body)
        out.append(
            {"anchor": kind, "editAs": edit[1] if edit else None, "from": corner("from"), "to": corner("to")}
        )
    return out


def main() -> None:
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    g = GraphBackend()
    recorded = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    sel_bytes, _ = build_selection(g)
    (out / "chart-selection.graph.xlsx").write_bytes(sel_bytes)
    sel = {s: [series_of(c) for c in e["charts"]] for s, e in parts_by_sheet(sel_bytes).items()}
    (out / "chart-selection.graph.json").write_text(
        json.dumps(
            {
                "source": {"api": "graph-v1.0", "application": "Excel for the web", "recordedAt": recorded},
                "cases": sel,
            },
            indent=1,
        )
        + "\n"
    )
    anchor_bytes, log = build_anchor(g)
    (out / "drawing-anchor.graph.xlsx").write_bytes(anchor_bytes)
    anchor_cases = {
        s: {"anchors": anchors_of(e["drawing"]), **log.get(s, {})}
        for s, e in parts_by_sheet(anchor_bytes).items()
    }
    (out / "drawing-anchor.graph.json").write_text(
        json.dumps(
            {
                "source": {
                    "api": "graph-v1.0",
                    "application": "Excel for the web",
                    "recordedAt": recorded,
                    "placed": PLACE,
                },
                "cases": anchor_cases,
            },
            indent=1,
        )
        + "\n"
    )
    print("ok", out)


if __name__ == "__main__":
    main()
