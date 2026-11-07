"""PivotCharts in spec cases (T-4130), so a probe can ask what Excel for the web does with a chart's caches.

A sheet's `"pivot_charts"` lists clustered column charts bound to a pivot of the workbook:

    "pivot_charts": [{"pivot": "P_C", "anchor": "E3", "series_name": "C!$B$3",
                      "categories": "C!$A$4:$A$5", "values": "C!$B$4:$B$5",
                      "cached_name": "Stale", "cached_categories": ["X", "Y"], "cached_values": [999, 999]}]

Each chart is a drawing part anchored at `anchor` holding one chart part with `c:pivotSource` (the pivot's
sheet and name; the workbook name in brackets is a placeholder, as the uploaded file is renamed) and one
series whose `c:strCache`/`c:numCache` hold the `cached_*` values as written. The caches are deliberately
stale: whether Excel rewrites them when it renders the pivot is the probe's question (plan.md §7). The
engine adapter ignores the key: RustSheet draws no charts (decision 0274).
"""

from __future__ import annotations

import re
from collections.abc import Callable
from typing import Any
from xml.sax.saxutils import escape

import xlsxwriter.utility

from rustsheet_tools.oracle.package import patch_parts, read_parts
from rustsheet_tools.pivot import relationships_xml, xml_part
from rustsheet_tools.pivot_xml import REL_NS, number_text

_KEYS = {
    "pivot",
    "anchor",
    "series_name",
    "categories",
    "values",
    "cached_name",
    "cached_categories",
    "cached_values",
}
_C_NS = "http://schemas.openxmlformats.org/drawingml/2006/chart"
_A_NS = "http://schemas.openxmlformats.org/drawingml/2006/main"
_XDR_NS = "http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing"
_CHART_TYPE = "application/vnd.openxmlformats-officedocument.drawingml.chart+xml"
_DRAWING_TYPE = "application/vnd.openxmlformats-officedocument.drawing+xml"
_REF = re.compile(r"[A-Za-z0-9_]+!\$?[A-Z]{1,3}\$?[1-9][0-9]{0,6}(?::\$?[A-Z]{1,3}\$?[1-9][0-9]{0,6})?")
# A worksheet's children that follow `<drawing>` in schema order (CT_Worksheet).
_AFTER_DRAWING = ("<legacyDrawing", "<legacyDrawingHF", "<picture", "<oleObjects", "<controls",
                  "<webPublishItems", "<tableParts", "<extLst", "</worksheet>")  # fmt: skip


def _ref(raw: Any, key: str) -> str:
    if not isinstance(raw, str) or not _REF.fullmatch(raw):
        raise ValueError(f"pivot chart {key} {raw!r} is a sheet-qualified A1 reference (Sheet!$A$1:$A$2)")
    return raw


def _str_cache(values: list[Any]) -> str:
    points = "".join(f'<c:pt idx="{k}"><c:v>{escape(str(v))}</c:v></c:pt>' for k, v in enumerate(values))
    return f'<c:strCache><c:ptCount val="{len(values)}"/>{points}</c:strCache>'


def _num_cache(values: list[Any]) -> str:
    if not all(isinstance(v, int | float) and not isinstance(v, bool) for v in values):
        raise ValueError("pivot chart cached_values are numbers")
    points = "".join(
        f'<c:pt idx="{k}"><c:v>{number_text(float(v))}</c:v></c:pt>' for k, v in enumerate(values)
    )
    return f'<c:numCache><c:formatCode>General</c:formatCode><c:ptCount val="{len(values)}"/>{points}</c:numCache>'


def chart_xml(chart: dict[str, Any], sheet_of_pivot: str) -> str:
    """The chart part of one `pivot_charts` entry."""
    if not isinstance(chart, dict):
        raise ValueError(f"a pivot chart is an object {{pivot, anchor, ...}}, not {chart!r}")
    if unknown := set(chart) - _KEYS:
        raise ValueError(f"a pivot chart has unknown keys {sorted(unknown)}")
    if not {"pivot", "anchor"} <= set(chart):
        raise ValueError("a pivot chart names its pivot and anchor")
    categories = list(chart.get("cached_categories", []))
    values = list(chart.get("cached_values", []))
    pivot = str(chart["pivot"])
    name = escape(f"[probe.xlsx]{sheet_of_pivot}!{pivot}")
    series_name = _ref(chart.get("series_name", f"{sheet_of_pivot}!$B$3"), "series_name")
    tx = (
        f"<c:tx><c:strRef><c:f>{series_name}</c:f>{_str_cache([chart.get('cached_name', 'Stale')])}"
        "</c:strRef></c:tx>"
    )
    cat = f"<c:cat><c:strRef><c:f>{_ref(chart.get('categories'), 'categories')}</c:f>{_str_cache(categories)}</c:strRef></c:cat>"
    val = f"<c:val><c:numRef><c:f>{_ref(chart.get('values'), 'values')}</c:f>{_num_cache(values)}</c:numRef></c:val>"
    return (
        f'<c:chartSpace xmlns:c="{_C_NS}" xmlns:a="{_A_NS}" xmlns:r="{REL_NS}"><c:roundedCorners val="0"/>'
        f'<c:pivotSource><c:name>{name}</c:name><c:fmtId val="0"/></c:pivotSource>'
        '<c:chart><c:autoTitleDeleted val="1"/><c:pivotFmts><c:pivotFmt><c:idx val="0"/></c:pivotFmt></c:pivotFmts>'
        '<c:plotArea><c:layout/><c:barChart><c:barDir val="col"/><c:grouping val="clustered"/><c:varyColors val="0"/>'
        f'<c:ser><c:idx val="0"/><c:order val="0"/>{tx}<c:invertIfNegative val="0"/>{cat}{val}</c:ser>'
        '<c:gapWidth val="150"/><c:axId val="1"/><c:axId val="2"/></c:barChart>'
        '<c:catAx><c:axId val="1"/><c:scaling><c:orientation val="minMax"/></c:scaling><c:delete val="0"/>'
        '<c:axPos val="b"/><c:crossAx val="2"/></c:catAx>'
        '<c:valAx><c:axId val="2"/><c:scaling><c:orientation val="minMax"/></c:scaling><c:delete val="0"/>'
        '<c:axPos val="l"/><c:crossAx val="1"/></c:valAx></c:plotArea><c:plotVisOnly val="1"/></c:chart>'
        "</c:chartSpace>"
    )


def _drawing_xml(anchors: list[tuple[int, int]]) -> str:
    frames = []
    for k, (row, col) in enumerate(anchors, start=1):
        frames.append(
            f"<xdr:twoCellAnchor><xdr:from><xdr:col>{col}</xdr:col><xdr:colOff>0</xdr:colOff><xdr:row>{row}</xdr:row>"
            f"<xdr:rowOff>0</xdr:rowOff></xdr:from><xdr:to><xdr:col>{col + 7}</xdr:col><xdr:colOff>0</xdr:colOff>"
            f"<xdr:row>{row + 15}</xdr:row><xdr:rowOff>0</xdr:rowOff></xdr:to>"
            f'<xdr:graphicFrame macro=""><xdr:nvGraphicFramePr><xdr:cNvPr id="{k + 1}" name="Chart {k}"/>'
            '<xdr:cNvGraphicFramePr/></xdr:nvGraphicFramePr><xdr:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/>'
            f'</xdr:xfrm><a:graphic><a:graphicData uri="{_C_NS}"><c:chart xmlns:c="{_C_NS}" r:id="rId{k}"/>'
            "</a:graphicData></a:graphic></xdr:graphicFrame><xdr:clientData/></xdr:twoCellAnchor>"
        )
    return (
        f'<xdr:wsDr xmlns:xdr="{_XDR_NS}" xmlns:a="{_A_NS}" xmlns:r="{REL_NS}">{"".join(frames)}</xdr:wsDr>'
    )


def _pivot_sheets(spec: dict[str, Any]) -> dict[str, str]:
    return {
        str(p["name"]): str(sheet["name"]) for sheet in spec["sheets"] for p in sheet.get("pivot_tables", ())
    }


def _insert_drawing(rid: str) -> Callable[[str], str]:
    def apply(text: str) -> str:
        if "<drawing " in text:
            raise ValueError("cannot place a pivot chart: the sheet already has a drawing")
        at = min(i for i in (text.find(tag) for tag in _AFTER_DRAWING) if i >= 0)
        return text[:at] + f'<drawing r:id="{rid}"/>' + text[at:]

    return apply


def _append(before: str, element: str) -> Callable[[str], str]:
    def apply(text: str) -> str:
        if text.count(before) != 1:
            raise ValueError(f"cannot place pivot chart parts: {before} not found once")
        return text.replace(before, element + before)

    return apply


def write_pivot_charts(xlsx: bytes, spec: dict[str, Any]) -> bytes:
    """Adds each sheet's `pivot_charts` to the package `xlsx` (spec2xlsx's last pivot step)."""
    pivots = _pivot_sheets(spec)
    existing = set(read_parts(xlsx))
    added: dict[str, bytes] = {}
    patches: list[tuple[str, Callable[[str], str]]] = []
    types: list[str] = []
    chart_number = 0
    for number, sheet in enumerate(spec["sheets"], start=1):
        charts = sheet.get("pivot_charts", ())
        if not charts:
            continue
        if not isinstance(charts, list):
            raise ValueError(f"sheet {sheet['name']}: pivot_charts is a list")
        anchors = []
        drawing_rels = []
        for k, chart in enumerate(charts, start=1):
            if not isinstance(chart, dict) or str(chart.get("pivot")) not in pivots:
                raise ValueError(f"sheet {sheet['name']}: pivot chart {k} names no pivot of the workbook")
            chart_number += 1
            part = f"xl/charts/chart{chart_number}.xml"
            added[part] = xml_part(chart_xml(chart, pivots[str(chart["pivot"])]))
            types.append(f'<Override PartName="/{part}" ContentType="{_CHART_TYPE}"/>')
            drawing_rels.append(
                f'<Relationship Id="rId{k}" Type="{REL_NS}/chart" Target="../charts/chart{chart_number}.xml"/>'
            )
            anchors.append(xlsxwriter.utility.xl_cell_to_rowcol(str(chart["anchor"]).replace("$", "")))
        drawing = f"xl/drawings/drawing{number}.xml"
        added[drawing] = xml_part(_drawing_xml(anchors))
        added[f"xl/drawings/_rels/drawing{number}.xml.rels"] = relationships_xml(*drawing_rels)
        types.append(f'<Override PartName="/{drawing}" ContentType="{_DRAWING_TYPE}"/>')
        rel = f'<Relationship Id="rIdPivotChart{number}" Type="{REL_NS}/drawing" Target="../drawings/drawing{number}.xml"/>'
        rels = f"xl/worksheets/_rels/sheet{number}.xml.rels"
        if rels in existing:
            patches.append((rels, _append("</Relationships>", rel)))
        else:
            added[rels] = relationships_xml(rel)
        patches.append((f"xl/worksheets/sheet{number}.xml", _insert_drawing(f"rIdPivotChart{number}")))
    if not added:
        return xlsx
    patches.append(("[Content_Types].xml", _append("</Types>", "".join(types))))
    return patch_parts(xlsx, patches, added)
