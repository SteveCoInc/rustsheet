"""Pivot tables defined field by field in spec cases (T-4122, case builder v2; decision 0274).

The original builder (pivot.py) writes one compact shape and renders its cells itself. A pivot with a
`"definition"` is different: the builder writes only the package parts and **Excel renders the pivot**
when it opens the file (the cache is `refreshOnLoad`; docs/research/pivot-tables-2026-11-04/
oracle-spike.md runs 1, 4-6). The definition maps one-to-one onto `pivotTableDefinition`:

    {"name": "Pv", "source": "Data!A1:D9", "location": "A3",
     "definition": {
       "table": {"outline": false, "compact": false},           # pivotTableDefinition attributes
       "rows": ["Region", "Product"],                           # rowFields, outer first
       "columns": ["Values"],                                   # colFields; "Values" is the Σ field (x=-2)
       "pages": [{"field": "Year", "item": 2024}],              # pageFields; "item" omitted is (All)
       "data": [{"field": "Sales"},                             # dataFields
                {"field": "Sales", "subtotal": "count", "name": "N"},
                {"field": "Sales", "showDataAs": "percentOfTotal", "numFmtId": 10}],
       "fields": {"Product": {"attrs": {"sortType": "descending"},   # pivotField attributes
                              "hidden": ["C"], "collapsed": ["A"],   # item h="1" / sd="0"
                              "subtotals": ["sum", "count"],         # custom subtotals (none: [])
                              "group": {"start": 0, "end": 100, "interval": 10}}},  # numeric fieldGroup
       "calculated": [{"name": "Double", "formula": "Sales*2"}],    # calculated cache fields
       "cache": {"missingItemsLimit": 0},                       # pivotCacheDefinition attributes
       "xml": {"filters": "<filters count=\\"1\\">...</filters>"}}}   # raw children, by slot

Field indices (`fld` in raw XML, `baseField`) count the source columns left to right, then the calculated
fields in order. Excel for the web refuses a package whose parts disagree with each other (run 2:
`OpenWorkbookBlockedWorkbook`), so the builder derives everything that must agree (shared items, records,
`<items>`, counts) from the source cells and refuses what Excel would refuse or crash on (ClosedXML's
invariants, docs/research/pivot-tables-2026-11-04/oss-engines.md §4). It does not need to agree with what
Excel will compute: Excel rebuilds the cache, the items and the layout on open. Source formulas are cached as
blank for that reason.

The pivot's area must hold no spec cell: a layout that meets a foreign cell collapses to `#SPILL!` (run 3).
The builder checks the anchor and the page-field area; the case author keeps the rest of the area empty.
"""

from __future__ import annotations

import math
import re
from dataclasses import dataclass, field
from typing import Any
from xml.sax.saxutils import quoteattr

import xlsxwriter.utility

from rustsheet_tools.excel_errors import ERRORS
from rustsheet_tools.pivot_xml import MAIN_NS, REL_NS, number_text

VALUES = "Values"  # the Σ (data) field on an axis: `<field x="-2"/>`
_KEYS = {"name", "source", "location", "definition"}
_DEFINITION_KEYS = {"table", "rows", "columns", "pages", "data", "fields", "calculated", "cache", "xml"}
_FIELD_KEYS = {"attrs", "hidden", "collapsed", "subtotals", "group", "order"}
_DATA_KEYS = {"field", "name", "subtotal", "showDataAs", "baseField", "baseItem", "numFmtId"}
# ST_DataConsolidateFunction, with the verb Excel puts in a data field's default name.
_AGGREGATE_VERBS = [
    "sum",
    "Sum",
    "count",
    "Count",
    "average",
    "Average",
    "max",
    "Max",
    "min",
    "Min",
    "product",
    "Product",
    "countNums",
    "Count",
    "stdDev",
    "StdDev",
    "stdDevp",
    "StdDevp",
    "var",
    "Var",
    "varp",
    "Varp",
]
AGGREGATES = dict(zip(_AGGREGATE_VERBS[::2], _AGGREGATE_VERBS[1::2], strict=True))
# ST_ShowDataAs (ECMA-376) plus the Excel 2010 modes Excel writes as x14 `pivotShowAs`; the latter are
# accepted so probes can ask whether Excel for the web honours them (T-4129).
SHOW_DATA_AS = {
    "normal",
    "difference",
    "percent",
    "percentDiff",
    "runTotal",
    "percentOfRow",
    "percentOfCol",
    "percentOfTotal",
    "index",
}
# pivotTableDefinition attributes written unless `table` sets them (Excel's own new-pivot values).
_TABLE_DEFAULTS = {
    "dataCaption": "Values", "updatedVersion": "6", "minRefreshableVersion": "3", "createdVersion": "6",
    "applyNumberFormats": "0", "applyBorderFormats": "0", "applyFontFormats": "0", "applyPatternFormats": "0",
    "applyAlignmentFormats": "0", "applyWidthHeightFormats": "1", "useAutoFormatting": "1",
    "itemPrintTitles": "1", "indent": "0", "outline": "1", "outlineData": "1",
}  # fmt: skip
# Builder-managed attributes an author may not set through `table`, `attrs` or `cache`.
_MANAGED_TABLE = {"name", "cacheId", "dataOnRows", "dataPosition"}
_MANAGED_FIELD = {
    "axis",
    "dataField",
    "showAll",
    "defaultSubtotal",
    "multipleItemSelectionAllowed",
    *(f"{a}Subtotal" for a in AGGREGATES),
}
_MANAGED_CACHE = {"refreshOnLoad", "recordCount", "id"}
# CT_pivotTableDefinition's children after `dataFields`, in schema order: the raw-XML slots.
_SLOTS = ("formats", "conditionalFormats", "chartFormats", "pivotHierarchies", "filters", "extLst")
_ATTR_NAME = re.compile(r"[A-Za-z][A-Za-z0-9]*")

# A cached value: text, number, boolean, ("e", error text) or None (blank or not known to the builder).
Value = str | float | bool | tuple[str, str] | None


def _attr_value(value: Any) -> str:
    if isinstance(value, bool):
        return "1" if value else "0"
    if isinstance(value, int | float) and not isinstance(value, bool):
        return number_text(float(value))
    if isinstance(value, str):
        return value
    raise ValueError(f"attribute value {value!r} must be a string, number or boolean")


def _attrs(raw: Any, managed: set[str], where: str) -> str:
    """` k="v"` for each author attribute, refusing bad names and builder-managed ones."""
    if raw is None:
        return ""
    if not isinstance(raw, dict):
        raise ValueError(f"{where} must be an object of attributes")
    out = []
    for key, value in raw.items():
        if not _ATTR_NAME.fullmatch(key) or key in managed:
            raise ValueError(f"{where} may not set attribute {key!r}")
        out.append(f" {key}={quoteattr(_attr_value(value))}")
    return "".join(out)


def cached_value(raw: Any) -> Value:
    """The value a spec cell contributes to a pivot cache."""
    if raw is None or raw == "" or (isinstance(raw, str) and raw.startswith("=")):
        return None  # blank, `=""`, or a formula: Excel recomputes the cache on open
    if isinstance(raw, bool | str):
        return raw
    if isinstance(raw, int | float):
        if not math.isfinite(raw):
            raise ValueError(f"pivot source value {raw!r} is not finite")
        return float(raw)
    if isinstance(raw, dict) and raw.get("t") == "e" and raw.get("v") in ERRORS:
        return ("e", str(raw["v"]))
    if isinstance(raw, dict) and raw.get("t") == "s" and isinstance(raw.get("v"), str):
        return str(raw["v"])
    if isinstance(raw, dict) and ("entered" in raw or "entered_r1c1" in raw):
        return None  # typed input: Excel parses it after load
    raise ValueError(f"pivot source cell {raw!r} is not a value the builder can cache")


def _key(value: Value) -> tuple[str, str]:
    """Identity of a cache item for the builder's shared-items list. Texts that differ only in case are
    one item: Excel for the web refuses a cache listing both as corrupt (T-4122 runs). Excel rebuilds the
    items on open, so which casing it keeps is its own rule (T-4134), not this one."""
    if isinstance(value, bool):
        return ("b", str(value))
    if isinstance(value, float):
        return ("n", repr(value))
    if isinstance(value, tuple):
        return ("e", value[1])
    return ("m", "") if value is None else ("s", value.casefold())


def _item_xml(value: Value) -> str:
    if value is None:
        return "<m/>"
    if isinstance(value, bool):
        return f'<b v="{int(value)}"/>'
    if isinstance(value, float):
        return f'<n v="{number_text(value)}"/>'
    if isinstance(value, tuple):
        return f"<e v={quoteattr(value[1])}/>"
    return f"<s v={quoteattr(value)}/>"


@dataclass
class _Field:
    """One cache field and the pivot's use of it."""

    name: str
    values: list[Value]  # per record; empty for a calculated field
    formula: str | None = None
    items: list[Value] = field(default_factory=list)  # distinct values, first-seen order
    spec: dict[str, Any] = field(default_factory=dict)  # its `fields` entry
    axis: str | None = None  # axisRow, axisCol, axisPage
    data: bool = False

    def __post_init__(self) -> None:
        seen: set[tuple[str, str]] = set()
        for v in self.values:
            if _key(v) not in seen:
                seen.add(_key(v))
                self.items.append(v)

    def reorder(self, order: Any, pivot: str) -> None:
        """Puts the items `order` lists first, in that order: Excel keeps a field's item order on refresh
        (T-4122 runs), so the order is part of what a case asks Excel."""
        if not isinstance(order, list):
            raise ValueError(f"pivot {pivot}: order of {self.name!r} is a list of items")
        keys = [_key(cached_value(raw)) for raw in order]
        by_key = {_key(v): v for v in self.items}
        if missing := [raw for raw, k in zip(order, keys, strict=True) if k not in by_key]:
            raise ValueError(f"pivot {pivot}: order items {missing!r} are not in field {self.name!r}")
        if len(set(keys)) != len(keys):
            raise ValueError(f"pivot {pivot}: order of {self.name!r} repeats an item")
        self.items = [by_key[k] for k in keys] + [v for v in self.items if _key(v) not in set(keys)]

    @property
    def numeric_only(self) -> bool:
        return all(isinstance(v, float) or v is None for v in self.values)

    @property
    def group(self) -> dict[str, Any] | None:
        group = self.spec.get("group")
        return group if isinstance(group, dict) else None

    def shared_items(self) -> str:
        """`<sharedItems>`: the flags always, the list when records refer to it by index. The flags follow
        Excel-saved caches (LibreOffice and ClosedXML fixtures, oss-engines.md §5): booleans and errors count
        as strings; `containsSemiMixedTypes` is 0 only with no string and no blank; `containsMixedTypes` marks
        strings beside numbers. Excel for the web refuses other combinations as corrupt (T-4122 runs)."""
        vs = self.values
        strings = any(isinstance(v, str | bool | tuple) for v in vs)
        numbers = [v for v in vs if isinstance(v, float)]
        blank = any(v is None for v in vs)
        flags = []
        if not strings and not blank:
            flags.append(' containsSemiMixedTypes="0"')
        if not strings:
            flags.append(' containsString="0"')
        if strings and numbers:
            flags.append(' containsMixedTypes="1"')
        if numbers:
            flags.append(' containsNumber="1"')
            if all(n.is_integer() for n in numbers):
                flags.append(' containsInteger="1"')
            flags.append(f' minValue="{number_text(min(numbers))}" maxValue="{number_text(max(numbers))}"')
        if blank:
            flags.append(' containsBlank="1"')
        if not self.listed:
            return f"<sharedItems{''.join(flags)}/>"
        items = "".join(_item_xml(v) for v in self.items)
        return f'<sharedItems{"".join(flags)} count="{len(self.items)}">{items}</sharedItems>'

    @property
    def listed(self) -> bool:
        """Whether records refer to the shared items by index (`<x v>`): every field except an all-number
        field that is not on an axis or is grouped numerically (Excel's own shape)."""
        return not (self.numeric_only and (self.axis is None or self.group is not None))

    def group_labels(self) -> list[str]:
        """Numeric group items as the builder writes them; Excel relabels them on open (run 6)."""
        g = self.group or {}
        start, end, step = (float(g.get(k, 0)) for k in ("start", "end", "interval"))
        if not step > 0 or end < start or (end - start) / step > 10_000:
            raise ValueError(f"pivot field {self.name!r} group needs 0 < interval and at most 10,000 bins")
        bins = []
        lo = start
        while lo <= end:
            bins.append(f"{number_text(lo)}-{number_text(lo + step)}")
            lo += step
        return [f"<{number_text(start)}", *bins, f">{number_text(end)}"]


class DefinedPivot:
    """A pivot built from a `definition` (module docstring)."""

    def __init__(self, pivot: dict[str, Any], header: list[Any], rows: list[list[Any]]) -> None:
        if unknown := set(pivot) - _KEYS:
            raise ValueError(f"pivot table with a definition has unknown keys {sorted(unknown)}")
        self.name = str(pivot["name"])
        definition = pivot["definition"]
        if not isinstance(definition, dict) or (unknown := set(definition) - _DEFINITION_KEYS):
            raise ValueError(f"pivot {self.name}: definition has unknown keys {sorted(unknown or [])}")
        self.d = definition
        if not all(isinstance(h, str) and h for h in header) or len({h.casefold() for h in header}) != len(
            header
        ):
            raise ValueError(f"pivot {self.name}: source header {header!r} must be distinct non-empty texts")
        self.fields = [_Field(h, [cached_value(r[i]) for r in rows]) for i, h in enumerate(header)]
        self.records = len(rows)
        for calc in definition.get("calculated", []):
            if not isinstance(calc, dict) or set(calc) != {"name", "formula"}:
                raise ValueError(f"pivot {self.name}: a calculated field is {{name, formula}}")
            self.fields.append(_Field(str(calc["name"]), [], formula=str(calc["formula"])))
        names = [f.name.casefold() for f in self.fields]
        if len(set(names)) != len(names):
            raise ValueError(f"pivot {self.name}: calculated field names must differ from the source fields")
        for name, entry in (definition.get("fields") or {}).items():
            if not isinstance(entry, dict) or (unknown := set(entry) - _FIELD_KEYS):
                raise ValueError(
                    f"pivot {self.name}: field {name!r} entry has unknown keys {sorted(unknown or [])}"
                )
            target = self.fields[self._index(name)]
            target.spec = entry
            target.reorder(entry.get("order", []), self.name)
        self.rows = self._axis(definition.get("rows", []), "axisRow")
        self.columns = self._axis(definition.get("columns", []), "axisCol")
        self.pages = self._pages(definition.get("pages", []))
        self.data = self._data(definition.get("data", []))
        self._values_field()
        for f in self.fields:
            if f.formula is not None and (f.axis is not None or f.spec):
                raise ValueError(f"pivot {self.name}: calculated field {f.name!r} can only be a data field")
            if f.group is not None and not (f.numeric_only and f.axis in ("axisRow", "axisCol")):
                raise ValueError(
                    f"pivot {self.name}: grouped field {f.name!r} must be numbers on the row or column axis"
                )

    def _index(self, name: Any) -> int:
        for i, f in enumerate(self.fields):
            if isinstance(name, str) and f.name.casefold() == name.casefold():
                return i
        raise ValueError(f"pivot {self.name}: no field {name!r}")

    def _axis(self, names: Any, axis: str) -> list[int]:
        if not isinstance(names, list):
            raise ValueError(f"pivot {self.name}: rows and columns are lists of field names")
        out = []
        for name in names:
            if name == VALUES:
                out.append(-2)
                continue
            i = self._index(name)
            if self.fields[i].axis is not None:
                raise ValueError(f"pivot {self.name}: field {name!r} is on two axes")
            self.fields[i].axis = axis
            out.append(i)
        if out.count(-2) > 1:
            raise ValueError(f"pivot {self.name}: the Values field appears twice")
        return out

    def _pages(self, pages: Any) -> list[tuple[int, Value | None]]:
        if not isinstance(pages, list):
            raise ValueError(f"pivot {self.name}: pages is a list")
        out: list[tuple[int, Value | None]] = []
        for page in pages:
            if not isinstance(page, dict) or not set(page) <= {"field", "item"} or "field" not in page:
                raise ValueError(f"pivot {self.name}: a page is {{field, item?}}")
            i = self._index(page["field"])
            if self.fields[i].axis is not None or self.fields[i].formula is not None:
                raise ValueError(f"pivot {self.name}: page field {page['field']!r} is already used")
            self.fields[i].axis = "axisPage"
            item = cached_value(page["item"]) if "item" in page else None
            if "item" in page and _key(item) not in {_key(v) for v in self.fields[i].items}:
                raise ValueError(
                    f"pivot {self.name}: page item {page['item']!r} is not in field {page['field']!r}"
                )
            out.append((i, item if "item" in page else None))
        return out

    def _data(self, data: Any) -> list[dict[str, Any]]:
        if not isinstance(data, list) or not data:
            raise ValueError(f"pivot {self.name}: data lists at least one data field")
        out = []
        for entry in data:
            if not isinstance(entry, dict) or (unknown := set(entry) - _DATA_KEYS) or "field" not in entry:
                raise ValueError(
                    f"pivot {self.name}: a data field is {{field, ...}} without {sorted(unknown or [])}"
                )
            i = self._index(entry["field"])
            subtotal = str(entry.get("subtotal", "sum"))
            if subtotal not in AGGREGATES:
                raise ValueError(
                    f"pivot {self.name}: subtotal {subtotal!r} is not one of {sorted(AGGREGATES)}"
                )
            show = entry.get("showDataAs")
            if show is not None and show not in SHOW_DATA_AS:
                raise ValueError(
                    f"pivot {self.name}: showDataAs {show!r} is not one of {sorted(SHOW_DATA_AS)}"
                )
            self.fields[i].data = True
            name = str(entry.get("name") or f"{AGGREGATES[subtotal]} of {self.fields[i].name}")
            out.append({**entry, "fld": i, "subtotal": subtotal, "name": name})
        names = [e["name"].casefold() for e in out]
        if len(set(names)) != len(names):
            raise ValueError(f"pivot {self.name}: data field names must differ")
        return out

    def _values_field(self) -> None:
        """Excel crashes on a Values field with one data field and needs one with several
        (ClosedXML's invariants): add it to the columns when several data fields leave it out."""
        placed = -2 in self.rows or -2 in self.columns
        if len(self.data) == 1 and placed:
            raise ValueError(f"pivot {self.name}: the Values field needs two or more data fields")
        if len(self.data) > 1 and not placed:
            self.columns.append(-2)

    # ------------------------------------------------------------------ parts
    def footprint(self, top: int, left: int) -> set[tuple[int, int]]:
        """Cells that must hold no spec cell: the anchor and the page-field area above it."""
        cells = {(top, left)}
        if self.pages:
            if top < len(self.pages) + 1:
                raise ValueError(
                    f"pivot {self.name}: page fields need {len(self.pages) + 1} free rows above the location"
                )
            for k in range(len(self.pages)):
                cells |= {(top - len(self.pages) - 1 + k, left), (top - len(self.pages) - 1 + k, left + 1)}
        return cells

    def cache_definition(self, source_sheet: str, source_ref: str) -> str:
        fields = []
        for f in self.fields:
            if f.formula is not None:
                fields.append(
                    f'<cacheField name={quoteattr(f.name)} numFmtId="0" formula={quoteattr(f.formula)}'
                    ' databaseField="0"/>'
                )
                continue
            group = ""
            if f.group is not None:
                g = f.group
                labels = f.group_labels()
                items = "".join(f"<s v={quoteattr(label)}/>" for label in labels)
                group = (
                    f'<fieldGroup base="{self.fields.index(f)}"><rangePr autoStart="0" autoEnd="0"'
                    f' startNum="{number_text(float(g["start"]))}" endNum="{number_text(float(g["end"]))}"'
                    f' groupInterval="{number_text(float(g["interval"]))}"/>'
                    f'<groupItems count="{len(labels)}">{items}</groupItems></fieldGroup>'
                )
            fields.append(
                f'<cacheField name={quoteattr(f.name)} numFmtId="0">{f.shared_items()}{group}</cacheField>'
            )
        extra = _attrs(self.d.get("cache"), _MANAGED_CACHE, f"pivot {self.name} cache")
        return (
            f'<pivotCacheDefinition xmlns="{MAIN_NS}" xmlns:r="{REL_NS}" r:id="rId1" refreshOnLoad="1"'
            f' createdVersion="6" refreshedVersion="6" minRefreshableVersion="3" recordCount="{self.records}"{extra}>'
            f'<cacheSource type="worksheet"><worksheetSource ref="{source_ref}" sheet={quoteattr(source_sheet)}/>'
            f'</cacheSource><cacheFields count="{len(fields)}">{"".join(fields)}</cacheFields></pivotCacheDefinition>'
        )

    def cache_records(self) -> str:
        database = [f for f in self.fields if f.formula is None]
        index = [{_key(v): k for k, v in enumerate(f.items)} for f in database]
        rows = []
        for r in range(self.records):
            cells = []
            for f, at in zip(database, index, strict=True):
                v = f.values[r]
                cells.append(f'<x v="{at[_key(v)]}"/>' if f.listed else _item_xml(v))
            rows.append(f"<r>{''.join(cells)}</r>")
        return (
            f'<pivotCacheRecords xmlns="{MAIN_NS}" count="{self.records}">{"".join(rows)}</pivotCacheRecords>'
        )

    def _pivot_field(self, f: _Field) -> str:
        spec = f.spec
        attrs = _attrs(spec.get("attrs"), _MANAGED_FIELD, f"pivot {self.name} field {f.name!r} attrs")
        head = ""
        if f.axis:
            head += f' axis="{f.axis}"'
        if f.data:
            head += ' dataField="1"'
        if f.formula is not None:
            head += ' dragToRow="0" dragToCol="0" dragToPage="0"'
        subtotals = spec.get("subtotals")
        if subtotals is not None:
            if not isinstance(subtotals, list) or any(s not in AGGREGATES for s in subtotals):
                raise ValueError(
                    f"pivot {self.name}: subtotals of {f.name!r} must be a list of {sorted(AGGREGATES)}"
                )
            head += ' defaultSubtotal="0"' + "".join(f' {s}Subtotal="1"' for s in subtotals)
        if len(f.spec.get("hidden", [])) and f.axis == "axisPage":
            head += ' multipleItemSelectionAllowed="1"'
        if f.axis is None:
            return f'<pivotField{head} showAll="0"{attrs}/>'
        values: list[Value] = [*f.group_labels()] if f.group is not None else f.items
        keys = {_key(v): k for k, v in enumerate(values)}
        flags: dict[int, str] = {}
        for kind, attr in (("hidden", 'h="1"'), ("collapsed", 'sd="0"')):
            for raw in spec.get(kind, []):
                k = keys.get(_key(cached_value(raw)))
                if k is None:
                    raise ValueError(f"pivot {self.name}: {kind} item {raw!r} is not in field {f.name!r}")
                flags[k] = f"{flags.get(k, '')} {attr}"
        items = "".join(f'<item x="{k}"{flags.get(k, "")}/>' for k in range(len(values)))
        tails = ["default"] if subtotals is None else list(subtotals)
        items += "".join(f'<item t="{t}"/>' for t in tails)
        return f'<pivotField{head} showAll="0"{attrs}><items count="{len(values) + len(tails)}">{items}</items></pivotField>'

    def _page_item(self, i: int, item: Value | None) -> str:
        if item is None:
            return f'<pageField fld="{i}" hier="-1"/>'
        k = [_key(v) for v in self.fields[i].items].index(_key(item))
        return f'<pageField fld="{i}" item="{k}" hier="-1"/>'

    def table_definition(self, cache_id: int, top: int, left: int) -> str:
        table = self.d.get("table") or {}
        extra = _attrs(table, _MANAGED_TABLE, f"pivot {self.name} table")
        head = "".join(f' {k}="{v}"' for k, v in _TABLE_DEFAULTS.items() if k not in table)
        if (
            -2 in self.rows
        ):  # the Values field on rows: Excel for the web refuses the file without it (T-4138)
            head += ' dataOnRows="1"'
        anchor = xlsxwriter.utility.xl_rowcol_to_cell(top, left)
        corner = xlsxwriter.utility.xl_rowcol_to_cell(top + 1, left + 1)
        parts = [
            f'<location ref="{anchor}:{corner}" firstHeaderRow="1" firstDataRow="1" firstDataCol="1"'
            + (f' rowPageCount="{len(self.pages)}" colPageCount="1"' if self.pages else "")
            + "/>",
            f'<pivotFields count="{len(self.fields)}">{"".join(self._pivot_field(f) for f in self.fields)}</pivotFields>',
        ]
        for tag, axis in (("rowFields", self.rows), ("colFields", self.columns)):
            if axis:
                fields = "".join(f'<field x="{x}"/>' for x in axis)
                parts.append(f'<{tag} count="{len(axis)}">{fields}</{tag}>')
        if self.pages:
            pages = "".join(self._page_item(i, item) for i, item in self.pages)
            parts.append(f'<pageFields count="{len(self.pages)}">{pages}</pageFields>')
        data = "".join(self._data_field(e) for e in self.data)
        parts.append(f'<dataFields count="{len(self.data)}">{data}</dataFields>')
        raw = self.d.get("xml") or {}
        if not isinstance(raw, dict) or (unknown := set(raw) - set(_SLOTS)):
            raise ValueError(f"pivot {self.name}: xml slots are {list(_SLOTS)}, not {sorted(unknown or [])}")
        for slot in _SLOTS[:4]:
            parts.append(self._raw(raw, slot))
        parts.append(
            '<pivotTableStyleInfo name="PivotStyleLight16" showRowHeaders="1" showColHeaders="1"'
            ' showRowStripes="0" showColStripes="0" showLastColumn="1"/>'
        )
        parts.extend(self._raw(raw, slot) for slot in _SLOTS[4:])
        return (
            f'<pivotTableDefinition xmlns="{MAIN_NS}" name={quoteattr(self.name)} cacheId="{cache_id}"{head}{extra}>'
            f"{''.join(parts)}</pivotTableDefinition>"
        )

    def _data_field(self, e: dict[str, Any]) -> str:
        """One `<dataField>` (sum, `showDataAs`, `numFmtId` written only when given)."""
        attrs = f' name={quoteattr(e["name"])} fld="{e["fld"]}"'
        if e["subtotal"] != "sum":
            attrs += f' subtotal="{e["subtotal"]}"'
        if "showDataAs" in e:
            attrs += f' showDataAs="{e["showDataAs"]}"'
        base_field = e.get("baseField", 0)
        base = self._index(base_field) if isinstance(base_field, str) else int(base_field)
        attrs += f' baseField="{base}" baseItem="{int(e.get("baseItem", 0))}"'
        if "numFmtId" in e:
            attrs += f' numFmtId="{int(e["numFmtId"])}"'
        return f"<dataField{attrs}/>"

    def _raw(self, raw: dict[str, Any], slot: str) -> str:
        """The author's XML for `slot`: one `<slot>` element, no declarations, DTDs or entities. It is not
        parsed here (a malformed one is Excel's refusal to see, as `OpenWorkbookBlockedWorkbook`)."""
        if slot not in raw:
            return ""
        text = str(raw[slot]).strip()
        whole = re.fullmatch(rf"<{slot}(?:\s[^<>]*)?(?:/>|>.*</{slot}>)", text, flags=re.DOTALL)
        bare_amp = "&" in re.sub(r"&(?:amp|lt|gt|quot|apos);", "", text)
        if whole is None or "<!" in text or "<?" in text or bare_amp:
            raise ValueError(
                f"pivot {self.name}: xml slot {slot} must be one <{slot}> element without declarations"
            )
        return text
