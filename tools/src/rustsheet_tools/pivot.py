"""Pivot tables in spec cases (T-1705), so GETPIVOTDATA can be probed against a real pivot.

A sheet's `"pivot_tables"` lists pivots placed on that sheet:

    {"name": "Pivot1", "source": "Data!A1:C7", "location": "E3",
     "row": "Region", "column": "Product", "value": "Sales", "function": "sum"}

`source` is a range whose first row holds the field names (text) and whose other rows are literal
constants in `cells` (no formulas, no blanks); without a sheet name it is on the pivot's own sheet.
`row` names a text field, `column` (optional) another one, and `value` a number field aggregated by
`function` (`sum`, the default, `count`, `average`, `max` or `min`). Items show in ascending
case-folded order.

The builder renders the pivot as Excel lays out a new compact-form pivot (the caption cells, the
item labels, the aggregates and the grand totals) into the sheet's cells, and writes the pivot
cache definition, its records and the pivot table definition into the package. The cache is marked
`refreshOnLoad`, so Excel rebuilds it from `source` on open. The engine adapter writes the optional
`"rendered"` map (A1 address → cell, the layout above) as constants and builds the pivot's cache from
`source` (T-2084); `rustsheet-spec bake-pivots` fills it, and spec2xlsx refuses a `rendered` that
differs from the layout, so the layout stays defined here only.

A pivot may group its row field by numeric ranges (T-4228), the `<fieldGroup>` of ECMA-376:

    "group": {"start": 0, "end": 1, "interval": 0.1, "items": ["<0", "0-0.1", ..., ">1"]}

The row field must then hold numbers and the pivot has no column field. `items` are the group
labels in order, written as the `<groupItems>`: the first for values below `start`, the last for
values past `end`. A label may repeat and a number is written as an `<n>` item (T-4316 probes both;
Excel writes its own labels on load). A date group (T-4236) adds `"by"` (`seconds`, `minutes`, `hours`, `days`,
`months`, `quarters` or `years`, ECMA-376's `groupBy`) with `interval` 1, and `start` and `end` are
date serials (written as `startDate` and `endDate`); its row field holds date serials, which the
case should give a date number format (Excel groups a field by date only when its cells are dates),
and is cached as dates (truncated to the second; Excel refreshes the cache on load). Only a grouped
row field may hold blank (absent) cells, cached as `<m/>`.

A pivot may list `"unused_items"` (T-4248), texts the row field's cache keeps as shared items though no
record uses them (what a refresh that dropped an item's last record leaves, `missingItemsLimit`), and
`"show_all"` (default false), the row field's `showAll`. Excel rebuilds the cache from `source` on
load, which would drop such an item, so a pivot with unused items leaves `refreshOnLoad` off; the
rendered layout then shows an unused item (a blank aggregate) only when `show_all` is true.
`"column_unused_items"` and `"column_show_all"` (T-4311) are the same for the column field.

Excel stores no record's group, so the builder renders no cells for a grouped pivot (which group a
record falls in is what such a probe asks): Excel lays the pivot out itself on load (excel-graph,
T-4228), and the engine adapter has the engine lay it out (T-4235, T-4246), except a date group
over a field holding a blank, which it leaves unregistered (T-4260).
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from typing import Any
from xml.sax.saxutils import quoteattr

import xlsxwriter.utility

from rustsheet_tools.pivot_definition import DefinedPivot
from rustsheet_tools.pivot_xml import MAIN_NS, PACKAGE_REL_NS, REL_NS, number_text

_CONTENT_TYPE = "application/vnd.openxmlformats-officedocument.spreadsheetml.{}+xml"
_FUNCTIONS: dict[str, tuple[str, Callable[[list[float]], float]]] = {
    "sum": ("Sum", sum),
    "count": ("Count", lambda xs: float(len(xs))),
    "average": ("Average", lambda xs: sum(xs) / len(xs)),
    "max": ("Max", max),
    "min": ("Min", min),
}
_KEYS = {
    "name",
    "source",
    "location",
    "row",
    "column",
    "value",
    "function",
    "rendered",
    "group",
    "unused_items",
    "show_all",
    "column_unused_items",
    "column_show_all",
}
_GROUP_KEYS = {"start", "end", "interval", "items"}
_DATE_GROUPS = {"seconds", "minutes", "hours", "days", "months", "quarters", "years"}
# Pivot caches write a date before March 1900 on the OLE epoch (1899-12-30 is 0), so the same
# epoch serves every serial the builder writes (crates/rustsheet-xlsx `pivot_cache_date_serial`).
_OLE_EPOCH = datetime(1899, 12, 30)

Cell = str | float


@dataclass
class PivotParts:
    """What the pivots of a workbook add to its package: new parts and the edits to existing ones."""

    parts: dict[str, bytes] = field(default_factory=dict)
    workbook_caches: list[str] = field(default_factory=list)  # `<pivotCache>` elements
    workbook_rels: list[str] = field(default_factory=list)  # `<Relationship>` elements
    content_types: list[str] = field(default_factory=list)  # `<Override>` elements
    sheet_rels: dict[int, list[str]] = field(default_factory=dict)  # sheet number → `<Relationship>`s


def _split_range(ref: str) -> tuple[int, int, int, int]:
    first, _, last = ref.partition(":")
    r1, c1 = xlsxwriter.utility.xl_cell_to_rowcol(first.replace("$", ""))
    r2, c2 = xlsxwriter.utility.xl_cell_to_rowcol((last or first).replace("$", ""))
    if r2 < r1 or c2 < c1:
        msg = f"pivot source {ref} is not top-left to bottom-right"
        raise ValueError(msg)
    return r1, c1, r2, c2


def _source(
    spec: dict[str, Any], own_sheet: str, source: str, *, strict: bool = True
) -> tuple[str, str, list[list[Any]]]:
    """The source's sheet name, normalized range and rows of raw spec cells, header row first; an
    absent cell is None (the caller says where a blank is allowed). `strict` (the original builder)
    accepts only literal texts and numbers besides blanks; otherwise any cell passes as is
    (pivot_definition.py caches it)."""
    sheet_name, bang, ref = source.rpartition("!")
    sheet_name = sheet_name.strip("'").replace("''", "'") if bang else own_sheet
    by_name = {s["name"]: s for s in spec["sheets"]}
    if sheet_name not in by_name:
        msg = f"pivot source sheet {sheet_name!r} is not in the spec"
        raise ValueError(msg)
    r1, c1, r2, c2 = _split_range(ref)
    if r2 == r1:
        msg = f"pivot source {source} has no data rows"
        raise ValueError(msg)
    cells = by_name[sheet_name]["cells"]
    rows: list[list[Any]] = []
    for row in range(r1, r2 + 1):
        values = []
        for col in range(c1, c2 + 1):
            a1 = xlsxwriter.utility.xl_rowcol_to_cell(row, col)
            raw = cells.get(a1)
            if (
                strict
                and raw is not None
                and (
                    isinstance(raw, bool)
                    or not isinstance(raw, str | int | float)
                    or raw == ""
                    or (isinstance(raw, str) and raw.startswith("="))
                )
            ):
                msg = f"pivot source cell {sheet_name}!{a1} must be a literal text or number: {raw!r}"
                raise ValueError(msg)
            values.append(raw)
        rows.append(values)
    normalized = (
        f"{xlsxwriter.utility.xl_rowcol_to_cell(r1, c1)}:{xlsxwriter.utility.xl_rowcol_to_cell(r2, c2)}"
    )
    return sheet_name, normalized, rows


class _Pivot:
    """One pivot: its fields, items, aggregates and layout."""

    def __init__(self, spec: dict[str, Any], sheet: dict[str, Any], pivot: dict[str, Any]) -> None:
        if unknown := set(pivot) - _KEYS:
            msg = f"pivot table has unknown keys {sorted(unknown)}"
            raise ValueError(msg)
        self.name = str(pivot["name"])
        self.rendered: dict[str, Cell] | None = pivot.get("rendered")
        self.source_sheet, self.source_ref, rows = _source(spec, sheet["name"], str(pivot["source"]))
        header, self.records = rows[0], rows[1:]
        if not all(isinstance(h, str) for h in header) or len(set(header)) != len(header):
            msg = f"pivot source header {header!r} must be distinct texts"
            raise ValueError(msg)
        self.fields: list[str] = header
        self.function = str(pivot.get("function", "sum"))
        if self.function not in _FUNCTIONS:
            msg = f"pivot function {self.function!r} is not one of {sorted(_FUNCTIONS)}"
            raise ValueError(msg)
        self.group = _group(pivot.get("group"))
        if self.group is not None and pivot.get("column") is not None:
            msg = "a grouped pivot has no column field"
            raise ValueError(msg)
        self.row = self._field(pivot["row"], text=self.group is None)
        self.column = None if pivot.get("column") is None else self._field(pivot["column"], text=True)
        self.value = self._field(pivot["value"], text=False)
        for i, name in enumerate(header):
            if (i != self.row or self.group is None) and any(r[i] is None for r in self.records):
                msg = f"pivot source field {name!r} has a blank cell (only a grouped row field may)"
                raise ValueError(msg)
        if len({self.row, self.column, self.value} - {None}) != (2 if self.column is None else 3):
            msg = "a pivot's row, column and value fields must differ"
            raise ValueError(msg)
        # Shared items of each text field in order of first appearance, as Excel's cache keeps them.
        self.shared: list[list[str]] = [[] for _ in self.fields]
        for record in self.records:
            for i, value in enumerate(record):
                if isinstance(value, str) and value not in self.shared[i]:
                    self.shared[i].append(value)
        self.show_all = bool(pivot.get("show_all", False))
        self.unused: list[str] = self._unused(
            self.row, pivot.get("unused_items", []), grouped=self.group is not None, key="unused_items"
        )
        self.column_show_all = bool(pivot.get("column_show_all", False))
        column_unused = list(pivot.get("column_unused_items", []))
        if self.column is None and (column_unused or self.column_show_all):
            msg = "column_unused_items and column_show_all need a column field"
            raise ValueError(msg)
        self.column_unused: list[str] = (
            []
            if self.column is None
            else self._unused(self.column, column_unused, grouped=False, key="column_unused_items")
        )
        self.top, self.left = xlsxwriter.utility.xl_cell_to_rowcol(str(pivot["location"]).replace("$", ""))

    def _unused(self, index: int, raw: Any, *, grouped: bool, key: str) -> list[str]:
        """Field `index`'s unused items, appended to its shared items. Excel matches pivot items
        case-insensitively (the probe's "north" finds "North"), so an unused item must differ from
        every record's item, and from the others, ignoring case."""
        unused = list(raw)
        used = {str(v).casefold() for v in self.shared[index]}
        if unused and (
            grouped
            or not all(isinstance(u, str) and u.casefold() not in used for u in unused)
            or len({str(u).casefold() for u in unused}) != len(unused)
        ):
            msg = f"{key} are distinct texts no record uses, on an ungrouped field: {unused!r}"
            raise ValueError(msg)
        self.shared[index].extend(unused)
        return unused

    def _field(self, name: Any, *, text: bool) -> int:
        if name not in self.fields:
            msg = f"pivot field {name!r} is not in the source header {self.fields!r}"
            raise ValueError(msg)
        index = self.fields.index(name)
        if any(isinstance(r[index], str) != text for r in self.records if r[index] is not None):
            kind = "text" if text else "numbers"
            msg = f"pivot field {name!r} must hold only {kind}"
            raise ValueError(msg)
        return index

    def _dates(self, index: int) -> bool:
        """Whether field `index` is cached as dates: the row field of a date group."""
        return index == self.row and self.group is not None and self.group.by is not None

    def _order(self, index: int) -> list[str]:
        if index == self.row and self.group is not None:
            # Only the items' positions matter here: a grouped pivot renders no cells.
            return [str(v) for v in self.group.items]
        return sorted(self.shared[index], key=str.casefold)

    def _shows_all(self, index: int) -> bool:
        """Field `index`'s `showAll`: the row's `show_all`, the column's `column_show_all`."""
        return self.show_all if index == self.row else self.column_show_all and index == self.column

    def _shown(self, index: int) -> list[str]:
        """The items of field `index` laid out: all but an item without a record, unless `showAll`."""
        order = self._order(index)
        if self._shows_all(index):
            return order
        unused = self.unused if index == self.row else self.column_unused if index == self.column else []
        return [v for v in order if v not in unused]

    def _aggregate(self, row_item: str | None, column_item: str | None) -> float | None:
        values = [
            float(r[self.value])
            for r in self.records
            if (row_item is None or r[self.row] == row_item)
            and (column_item is None or self.column is None or r[self.column] == column_item)
        ]
        return _FUNCTIONS[self.function][1](values) if values else None

    @property
    def caption(self) -> str:
        return f"{_FUNCTIONS[self.function][0]} of {self.fields[self.value]}"

    def cells(self) -> dict[tuple[int, int], Cell]:
        """The rendered compact-form layout, by (row, column)."""
        rows, top, left = self._shown(self.row), self.top, self.left
        out: dict[tuple[int, int], Cell] = {}
        if self.group is not None:
            return out

        def put(row: int, col: int, value: Cell | None) -> None:
            if value is not None:
                out[(row, col)] = value

        if self.column is None:
            put(top, left, "Row Labels")
            put(top, left + 1, self.caption)
            for i, item in enumerate([*rows, None]):
                put(top + 1 + i, left, item if item is not None else "Grand Total")
                put(top + 1 + i, left + 1, self._aggregate(item, None))
            return out
        columns = self._shown(self.column)
        put(top, left, self.caption)
        put(top, left + 1, "Column Labels")
        put(top + 1, left, "Row Labels")
        for j, column in enumerate([*columns, None]):
            put(top + 1, left + 1 + j, column if column is not None else "Grand Total")
        for i, item in enumerate([*rows, None]):
            put(top + 2 + i, left, item if item is not None else "Grand Total")
            for j, column in enumerate([*columns, None]):
                put(top + 2 + i, left + 1 + j, self._aggregate(item, column))
        return out

    def a1_cells(self) -> dict[str, Cell]:
        """`cells()` by A1 address."""
        return {
            xlsxwriter.utility.xl_rowcol_to_cell(row, col): value
            for (row, col), value in self.cells().items()
        }

    def location(self) -> str:
        rows = len(self._shown(self.row)) + 1
        if self.column is None:
            bottom, right, first_data_row = self.top + rows, self.left + 1, 1
        else:
            bottom, right, first_data_row = (
                self.top + 1 + rows,
                self.left + 1 + len(self._shown(self.column)),
                2,
            )
        ref = (
            f"{xlsxwriter.utility.xl_rowcol_to_cell(self.top, self.left)}:"
            f"{xlsxwriter.utility.xl_rowcol_to_cell(bottom, right)}"
        )
        return f'<location ref="{ref}" firstHeaderRow="1" firstDataRow="{first_data_row}" firstDataCol="1"/>'

    def cache_definition(self) -> str:
        fields = []
        for i, name in enumerate(self.fields):
            if self.shared[i]:
                items = "".join(f"<s v={quoteattr(v)}/>" for v in self.shared[i])
                shared = f'<sharedItems count="{len(self.shared[i])}">{items}</sharedItems>'
            else:
                numbers = [float(r[i]) for r in self.records if r[i] is not None]
                blank = ' containsBlank="1"' if len(numbers) < len(self.records) else ""
                if self._dates(i):
                    shared = (
                        '<sharedItems containsSemiMixedTypes="0" containsNonDate="0" containsDate="1"'
                        f' containsString="0"{blank} minDate="{_iso(min(numbers))}" maxDate="{_iso(max(numbers))}"/>'
                    )
                else:
                    integer = ' containsInteger="1"' if all(n.is_integer() for n in numbers) else ""
                    shared = (
                        f'<sharedItems containsSemiMixedTypes="0" containsString="0" containsNumber="1"{integer}'
                        f'{blank} minValue="{number_text(min(numbers))}" maxValue="{number_text(max(numbers))}"/>'
                    )
            if i == self.row and self.group is not None:
                shared += self.group.xml(base=i)
            fields.append(f'<cacheField name={quoteattr(name)} numFmtId="0">{shared}</cacheField>')
        refresh = "" if self.unused or self.column_unused else ' refreshOnLoad="1"'
        return (
            f'<pivotCacheDefinition xmlns="{MAIN_NS}" xmlns:r="{REL_NS}" r:id="rId1"{refresh}'
            f' createdVersion="6" refreshedVersion="6" minRefreshableVersion="3" recordCount="{len(self.records)}">'
            f'<cacheSource type="worksheet"><worksheetSource ref="{self.source_ref}"'
            f" sheet={quoteattr(self.source_sheet)}/></cacheSource>"
            f'<cacheFields count="{len(fields)}">{"".join(fields)}</cacheFields></pivotCacheDefinition>'
        )

    def cache_records(self) -> str:
        records = "".join(
            "<r>"
            + "".join(
                f'<x v="{self.shared[i].index(v)}"/>'
                if isinstance(v, str)
                else "<m/>"
                if v is None
                else f'<d v="{_iso(float(v))}"/>'
                if self._dates(i)
                else f'<n v="{number_text(float(v))}"/>'
                for i, v in enumerate(record)
            )
            + "</r>"
            for record in self.records
        )
        return (
            f'<pivotCacheRecords xmlns="{MAIN_NS}" count="{len(self.records)}">{records}</pivotCacheRecords>'
        )

    def _axis_items(self, index: int) -> tuple[str, str]:
        """A field's `<items>` (display order, shared-item indices) and its `<rowItems>`/`<colItems>` body."""
        order = self._order(index)
        grouped = index == self.row and self.group is not None
        # A grouped field's items index its group labels, not shared items (it has none).
        items = "".join(
            f'<item x="{k if grouped else self.shared[index].index(v)}"/>' for k, v in enumerate(order)
        )
        shown = self._shown(index)
        lines = "".join(
            "<i><x/></i>" if d == 0 else f'<i><x v="{d}"/></i>' for d, v in enumerate(order) if v in shown
        )
        return (
            f'<items count="{len(order) + 1}">{items}<item t="default"/></items>',
            f'{lines}<i t="grand"><x/></i>',
        )

    def table_definition(self, cache_id: int) -> str:
        pivot_fields = []
        for i in range(len(self.fields)):
            if i in (self.row, self.column):
                axis = "axisRow" if i == self.row else "axisCol"
                pivot_fields.append(
                    f'<pivotField axis="{axis}" showAll="{int(self._shows_all(i))}">{self._axis_items(i)[0]}</pivotField>'
                )
            elif i == self.value:
                pivot_fields.append('<pivotField dataField="1" showAll="0"/>')
            else:
                pivot_fields.append('<pivotField showAll="0"/>')
        row_lines = self._axis_items(self.row)[1]
        rows = f'<rowFields count="1"><field x="{self.row}"/></rowFields>'
        rows += f'<rowItems count="{len(self._shown(self.row)) + 1}">{row_lines}</rowItems>'
        if self.column is None:
            columns = '<colItems count="1"><i/></colItems>'
        else:
            column_lines = self._axis_items(self.column)[1]
            columns = f'<colFields count="1"><field x="{self.column}"/></colFields>'
            columns += f'<colItems count="{len(self._shown(self.column)) + 1}">{column_lines}</colItems>'
        subtotal = "" if self.function == "sum" else f' subtotal="{self.function}"'
        return (
            f'<pivotTableDefinition xmlns="{MAIN_NS}" name={quoteattr(self.name)} cacheId="{cache_id}"'
            ' applyNumberFormats="0" applyBorderFormats="0" applyFontFormats="0" applyPatternFormats="0"'
            ' applyAlignmentFormats="0" applyWidthHeightFormats="1" dataCaption="Values" updatedVersion="6"'
            ' minRefreshableVersion="3" useAutoFormatting="1" itemPrintTitles="1" createdVersion="6" indent="0"'
            ' outline="1" outlineData="1" multipleFieldNames="0">'
            f'{self.location()}<pivotFields count="{len(pivot_fields)}">{"".join(pivot_fields)}</pivotFields>'
            f"{rows}{columns}"
            f'<dataFields count="1"><dataField name={quoteattr(self.caption)} fld="{self.value}"{subtotal}'
            ' baseField="0" baseItem="0"/></dataFields>'
            '<pivotTableStyleInfo name="PivotStyleLight16" showRowHeaders="1" showColHeaders="1"'
            ' showRowStripes="0" showColStripes="0" showLastColumn="1"/></pivotTableDefinition>'
        )


@dataclass(frozen=True)
class _Group:
    """A numeric range or date grouping of the row field (`<rangePr>` with `<groupItems>`)."""

    start: float
    end: float
    interval: float
    items: list[str | float]
    by: str | None = None  # `groupBy` of a date group; None groups numbers

    def xml(self, *, base: int) -> str:
        items = "".join(
            f"<s v={quoteattr(v)}/>" if isinstance(v, str) else f'<n v="{number_text(v)}"/>'
            for v in self.items
        )
        if self.by is None:
            bounds = (
                f'startNum="{number_text(self.start)}" endNum="{number_text(self.end)}"'
                f' groupInterval="{number_text(self.interval)}"'
            )
        else:
            bounds = f'groupBy="{self.by}" startDate="{_iso(self.start)}" endDate="{_iso(self.end)}"'
        return (
            f'<fieldGroup base="{base}"><rangePr autoStart="0" autoEnd="0" {bounds}/>'
            f'<groupItems count="{len(self.items)}">{items}</groupItems></fieldGroup>'
        )


def _iso(serial: float) -> str:
    """A date serial as a pivot cache writes it, to the whole second (truncated; the cache is
    refreshed on load, so it only has to parse)."""
    if not 0 <= serial < 2958466:
        msg = f"a pivot date serial must be 0 to 9999-12-31: {serial!r}"
        raise ValueError(msg)
    return (_OLE_EPOCH + timedelta(seconds=int(serial * 86400))).strftime("%Y-%m-%dT%H:%M:%S")


def _group(raw: Any) -> _Group | None:
    if raw is None:
        return None
    if not isinstance(raw, dict) or set(raw) - {"by"} != _GROUP_KEYS:
        msg = f"a pivot group has exactly the keys {sorted(_GROUP_KEYS)} (and optionally by): {raw!r}"
        raise ValueError(msg)
    by = raw.get("by")
    if by is not None and (by not in _DATE_GROUPS or raw["interval"] != 1):
        msg = f"a date group's by is one of {sorted(_DATE_GROUPS)} with interval 1: {raw!r}"
        raise ValueError(msg)
    bounds = [raw["start"], raw["end"], raw["interval"]]
    if any(isinstance(b, bool) or not isinstance(b, int | float) for b in bounds):
        msg = f"a pivot group's start, end and interval are numbers: {raw!r}"
        raise ValueError(msg)
    start, end, interval = (float(b) for b in bounds)
    if not start < end or not interval > 0:
        msg = f"a pivot group needs start < end and interval > 0: {raw!r}"
        raise ValueError(msg)
    items = raw["items"]
    if (
        not isinstance(items, list)
        or len(items) < 3
        or not all(
            isinstance(v, str) or (isinstance(v, int | float) and not isinstance(v, bool)) for v in items
        )
    ):
        msg = f"a pivot group's items are three or more texts or numbers: {items!r}"
        raise ValueError(msg)
    if by is not None:
        for bound in (start, end):
            _iso(bound)  # refuses a serial off the calendar
    return _Group(start, end, interval, items, by)


def rendered_cells(spec: dict[str, Any], sheet: dict[str, Any], pivot: dict[str, Any]) -> dict[str, Cell]:
    """The cells the builder renders for `pivot` (a pivot of `sheet`), by A1 address."""
    return _Pivot(spec, sheet, pivot).a1_cells()


def relationships_xml(*relationships: str) -> bytes:
    """A relationships part holding `relationships`."""
    return _xml(f'<Relationships xmlns="{PACKAGE_REL_NS}">{"".join(relationships)}</Relationships>')


def _relationship(rid: str, kind: str, target: str) -> str:
    return f'<Relationship Id="{rid}" Type="{REL_NS}/{kind}" Target="{target}"/>'


def _xml(text: str) -> bytes:
    return f'<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n{text}'.encode()


def pivot_tables(spec: dict[str, Any], write: Callable[[int, int, int, Cell], None]) -> PivotParts:
    """Renders every pivot of the spec through `write(sheet number, row, col, value)` and returns the
    package parts and edits they need. Raises ValueError for a malformed pivot or one that overlaps
    a spec cell or another pivot."""
    out = PivotParts()
    cache_id = 0
    for number, sheet in enumerate(spec["sheets"], start=1):
        taken = {xlsxwriter.utility.xl_cell_to_rowcol(a1) for a1 in sheet["cells"]}
        for raw in sheet.get("pivot_tables", ()):
            if "definition" in raw:
                cache_id += 1
                _defined(spec, sheet, raw, number=number, n=cache_id, taken=taken, out=out)
                continue
            pivot = _Pivot(spec, sheet, raw)
            cells = pivot.cells()
            if pivot.rendered is not None and pivot.rendered != pivot.a1_cells():
                msg = f"pivot {pivot.name} has `rendered` cells that differ from its layout (rustsheet-spec bake-pivots)"
                raise ValueError(msg)
            if overlap := taken & set(cells):
                msg = f"pivot {pivot.name} overlaps cells {sorted(overlap)}"
                raise ValueError(msg)
            taken |= set(cells)
            for (row, col), value in sorted(cells.items()):
                write(number, row, col, value)
            cache_id += 1
            _link(
                out,
                number=number,
                n=cache_id,
                definition_xml=pivot.cache_definition(),
                records_xml=pivot.cache_records(),
                table_xml=pivot.table_definition(cache_id),
            )
    return out


def _link(
    out: PivotParts, *, number: int, n: int, definition_xml: str, records_xml: str, table_xml: str
) -> None:
    """Adds pivot `n`'s cache definition, records and table parts to `out`, linked from the workbook,
    sheet `number` and the content types."""
    definition = f"xl/pivotCache/pivotCacheDefinition{n}.xml"
    records = f"xl/pivotCache/pivotCacheRecords{n}.xml"
    table = f"xl/pivotTables/pivotTable{n}.xml"
    out.parts[definition] = _xml(definition_xml)
    out.parts[records] = _xml(records_xml)
    out.parts[table] = _xml(table_xml)
    out.parts[f"xl/pivotCache/_rels/pivotCacheDefinition{n}.xml.rels"] = relationships_xml(
        _relationship("rId1", "pivotCacheRecords", f"pivotCacheRecords{n}.xml")
    )
    out.parts[f"xl/pivotTables/_rels/pivotTable{n}.xml.rels"] = relationships_xml(
        _relationship("rId1", "pivotCacheDefinition", f"../pivotCache/pivotCacheDefinition{n}.xml")
    )
    out.workbook_caches.append(f'<pivotCache cacheId="{n}" r:id="rIdPivotCache{n}"/>')
    out.workbook_rels.append(
        _relationship(f"rIdPivotCache{n}", "pivotCacheDefinition", f"pivotCache/pivotCacheDefinition{n}.xml")
    )
    out.sheet_rels.setdefault(number, []).append(
        _relationship(f"rIdPivotTable{n}", "pivotTable", f"../pivotTables/pivotTable{n}.xml")
    )
    for part, kind in (
        (definition, "pivotCacheDefinition"),
        (records, "pivotCacheRecords"),
        (table, "pivotTable"),
    ):
        out.content_types.append(f'<Override PartName="/{part}" ContentType="{_CONTENT_TYPE.format(kind)}"/>')


def _defined(
    spec: dict[str, Any],
    sheet: dict[str, Any],
    raw: dict[str, Any],
    *,
    number: int,
    n: int,
    taken: set[tuple[int, int]],
    out: PivotParts,
) -> None:
    """Adds a pivot with a `definition` (pivot_definition.py): parts only, Excel renders it on open."""
    source_sheet, source_ref, rows = _source(spec, sheet["name"], str(raw["source"]), strict=False)
    pivot = DefinedPivot(raw, rows[0], rows[1:])
    top, left = xlsxwriter.utility.xl_cell_to_rowcol(str(raw["location"]).replace("$", ""))
    cells = pivot.footprint(top, left)
    # A pivot saved without refreshOnLoad may hold spec cells: they are its saved render (T-4586).
    if pivot.refresh_on_load and (overlap := taken & cells):
        msg = f"pivot {pivot.name} needs empty cells at {sorted(overlap)} (Excel shows #SPILL! otherwise)"
        raise ValueError(msg)
    taken |= cells
    _link(
        out,
        number=number,
        n=n,
        definition_xml=pivot.cache_definition(source_sheet, source_ref),
        records_xml=pivot.cache_records(),
        table_xml=pivot.table_definition(n, top, left),
    )
