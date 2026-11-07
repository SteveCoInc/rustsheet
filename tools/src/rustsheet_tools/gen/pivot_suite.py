"""`rustsheet-gen --milestone M16` (or `--policy pivots`): the gen-pivot suite (T-4132, plan §6 layer 2).

Each generated pivot is one row of a seeded covering array (gen/covering.py) over the pivot dimensions of
stage PV1 (docs/research/pivot-tables-2026-11-04/plan.md §8; eval/catalog/pivot-features.tsv `stage`):

| factor | levels |
|---|---|
| layout | compact, outline, tabular |
| subtotals | top (default), bottom (`subtotalTop="0"`), none (`defaultSubtotal="0"`), custom functions |
| values | single data field, Values field last or first on columns, last or first on rows |
| filter | none, page field at (All), page field on one item, page field on several items, hidden axis items |
| rows, cols, data | 0-3 row fields, 0-2 column fields, 1-3 data fields |
| aggregate | the eleven ST_DataConsolidateFunction values, of the first data field |
| grand | both grand totals, the row one off, the column one off, both off |
| pool | the data edge pools of gen/pivot_data.py |
| caption | none, header captions, grandTotalCaption, missingCaption, errorCaption, field and data-field names, an item caption (`n`), subtotalCaption |
| extra | none, insertBlankRow, repeat labels (x14 fillDownLabels), a collapsed item (`sd="0"`), classic layout (gridDropZones), a data-field number format |

The array is 3-wise over layout x subtotals x values x filter (`THREE_WISE`) and pairwise over every factor.
`allowed` rules out the combinations a pivot cannot show (subtotals with fewer than two fields on an axis, a
hidden item with no axis field...), so they are not required. Show-values-as, the fifth 3-wise dimension of
the plan, and the other PV2+ dimensions are deferred (`DEFERRED`): they are not PV1 rows of the catalog, and
the engine's PV1 renderer is what this batch measures; each joins the factors when its stage opens.

Pivots are written with case builder v2's `definition` (rustsheet_tools/pivot_definition.py), exactly as the
`pivots` probes are: one pivot per sheet at `LOCATION`, over its own source block in A:I of the same sheet,
`PER_CASE` sheets per case (one oracle round trip each), and a `Q` sheet with one COUNTA control per pivot
sheet, so the case has a formula cell. Each case lists in `pivot_features` the catalog ids its pivots
exercise (`paths`, resolved through the catalog's `ooxml` column), and its description names each sheet's
levels, so a failing cell leads back to its row of the array. The page-field and hidden-item rows the filter
dimension exercises are stage PV2 in the catalog although the plan's PV1 scope lists them.
"""

from __future__ import annotations

import random
from collections.abc import Mapping
from functools import cache
from typing import Any

from rustsheet_tools import repo_root
from rustsheet_tools.gen import covering
from rustsheet_tools.gen.pivot_data import (
    DATE_FIELD,
    EDGE_VALUE_FIELD,
    HEADER,
    LABELS,
    POOL_NAMES,
    VALUE_EDGE,
    VALUES,
    Source,
    build_source,
    ensure_error,
)
from rustsheet_tools.pivot_catalog import read_table
from rustsheet_tools.pivot_definition import AGGREGATES, SUBTOTALS, X14_NS
from rustsheet_tools.pivot_definition import VALUES as VALUES_FIELD

MILESTONE = "M16"
SUITE = "gen-pivot"
LOCATION = "L5"  # right of the source block (A:I) with a gap column, and two page-field rows above
PER_CASE = 12
MAX_PAGES = 2
FACTORS: dict[str, list[str]] = {
    "layout": ["compact", "outline", "tabular"],
    "subtotals": ["top", "bottom", "none", "custom"],
    "values": ["single", "cols_end", "cols_start", "rows_end", "rows_start"],
    "filter": ["none", "page_all", "page_item", "page_multi", "hidden"],
    "rows": ["0", "1", "2", "3"],
    "cols": ["0", "1", "2"],
    "data": ["1", "2", "3"],
    "aggregate": list(AGGREGATES),
    "grand": ["both", "no_rows", "no_cols", "none"],
    "pool": POOL_NAMES,
    "caption": [
        "none",
        "headers",
        "grand_caption",
        "missing",
        "error",
        "names",
        "item_caption",
        "subtotal_caption",
    ],
    "extra": ["none", "blank_row", "repeat_labels", "collapsed", "classic", "num_format"],
}
THREE_WISE = ("layout", "subtotals", "values", "filter")
GROUPS = (covering.Group(tuple(FACTORS), 2), covering.Group(THREE_WISE, 3))
DEFERRED = {
    "show-values-as (dataField showDataAs, x14 pivotShowAs)": "PV2 (catalog PV-0108..PV-0116, PV-1254..PV-1259)",
    "label, value and date filters, Top-N": "PV2 (ST_PivotFilterType rows)",
    "sort by label or value (sortType, autoSortScope)": "PV2 (PV-0016..PV-0018, PV-1125)",
    "numeric, date and manual groups; calculated fields and items": "PV3",
}
# x14 "Repeat item labels" on a pivot field (docs/excel-semantics/pivots.md "Repeat item labels").
FILL_DOWN = (
    '<extLst><ext uri="{2946ED86-A175-432a-8AC1-64E0C546D7DE}" '
    f'xmlns:x14="{X14_NS}"><x14:pivotField fillDownLabels="1"/></ext></extLst>'
)
NUMBER_FORMATS = [3, 4, 10]  # #,##0  #,##0.00  0.00%

T = "pivotTableDefinition"
PF = f"{T}/pivotFields/pivotField"
CACHE_FIELD = "pivotCacheDefinition/cacheFields/cacheField"


def _under(prefix: str, suffixes: str) -> list[str]:
    """`prefix` followed by each space-separated suffix (catalog `ooxml` paths); a leading space stands for an
    empty suffix, `prefix` itself."""
    return [prefix + s for s in suffixes.split(" ")]


BASE_PATHS = [
    T,
    PF,
    CACHE_FIELD,
    *_under(T, "@name /pivotFields /pivotFields@count /dataFields /dataFields@count"),
    *_under(f"{T}/location", " @ref @firstHeaderRow @firstDataRow @firstDataCol"),
    *_under(f"{T}/dataFields/dataField", " @fld @name @subtotal"),
    *_under(PF, "@dataField"),
    *_under(CACHE_FIELD, "@name /sharedItems"),
    *_under(
        "pivotCacheDefinition",
        " /cacheFields /cacheFields@count /cacheSource /cacheSource@type @refreshOnLoad",
    ),
    *_under("pivotCacheDefinition/cacheSource/worksheetSource", " @ref @sheet"),
    *_under("pivotCacheRecords", " @count /r"),
]


def _axis_paths(axis: str) -> list[str]:
    fields, items = ("rowFields", "rowItems") if axis == "row" else ("colFields", "colItems")
    return [
        f"ST_Axis=axis{axis.capitalize()}",
        "ST_ItemType=data",
        *_under(f"{T}/{fields}", " @count /field /field@x"),
        *_under(f"{T}/{items}", " /i"),
        *_under(PF, "@axis /items /items@count /items/item /items/item@x"),
    ]


PAGE_PATHS = [
    "ST_Axis=axisPage",
    *_under(f"{T}/pageFields", " @count /pageField /pageField@fld /pageField@hier"),
    *_under(f"{T}/location", "@rowPageCount @colPageCount"),
]
_SHARED_LAYOUT = _under(T, "@compact @compactData") + _under(PF, "@compact")
LAYOUT_PATHS = {
    "compact": _SHARED_LAYOUT,
    "outline": [*_SHARED_LAYOUT, *_under(T, "@outline"), *_under(PF, "@outline")],
    "tabular": [*_SHARED_LAYOUT, *_under(T, "@outline @outlineData"), *_under(PF, "@outline")],
}
CAPTION_PATHS = {
    "headers": [f"{T}@rowHeaderCaption", f"{T}@colHeaderCaption", f"{T}@dataCaption"],
    "grand_caption": [f"{T}@grandTotalCaption"],
    "missing": [f"{T}@missingCaption", f"{T}@showMissing"],
    "error": [f"{T}@errorCaption", f"{T}@showError"],
    "names": [f"{PF}@name"],
    "item_caption": [f"{PF}/items/item@n"],
    "subtotal_caption": [f"{PF}@subtotalCaption"],
}
EXTRA_PATHS = {
    "blank_row": [f"{PF}@insertBlankRow", "ST_ItemType=blank"],
    "repeat_labels": ["x14:pivotField", "x14:pivotField@fillDownLabels"],
    "collapsed": [f"{PF}/items/item@sd"],
    "classic": [f"{T}@gridDropZones"],
    "num_format": [f"{T}/dataFields/dataField@numFmtId"],
}
# The cache shared-item element each kind of source value is listed as.
ITEM_KIND = {str: "s", float: "n", bool: "b"}


def _n(p: Mapping[str, str], key: str) -> int | None:
    return int(p[key]) if key in p else None


def _refused(p: Mapping[str, str]) -> list[bool]:
    """One entry per rule of `allowed`, True when the partial row `p` breaks it (unknown levels break none)."""
    rows, cols, data = _n(p, "rows"), _n(p, "cols"), _n(p, "data")
    no_axis = rows is not None and cols is not None and rows + cols == 0
    flat = rows is not None and cols is not None and rows < 2 and cols < 2  # no axis has subtotal lines
    values, caption = p.get("values"), p.get("caption")
    return [
        # One data field has no Values field; two or more need one (pivot_definition.py).
        values is not None and data is not None and (values == "single") != (data == 1),
        (values == "cols_start" and cols == 0) or (values == "rows_start" and rows == 0),
        p.get("subtotals", "top") != "top" and flat,
        p.get("filter") == "hidden" and no_axis,
        p.get("extra") in ("blank_row", "collapsed", "repeat_labels") and rows is not None and rows < 2,
        caption == "missing" and (rows == 0 or cols == 0),  # an empty intersection needs both axes
        caption == "item_caption" and no_axis,
        # subtotalCaption labels the default subtotal lines.
        caption == "subtotal_caption" and (p.get("subtotals") in ("none", "custom") or flat),
        p.get("pool") in ("dates", "many", "case") and no_axis,  # the edge field needs an axis
    ]


def allowed(p: Mapping[str, str]) -> bool:
    """Whether a (partial) row can be built and shows what its levels ask for (module docstring)."""
    return not any(_refused(p))


@cache
def catalog_ids() -> dict[str, str]:
    """The pivot catalog's id per OOXML path (eval/catalog/pivot-features.tsv)."""
    table = read_table(repo_root() / "eval" / "catalog" / "pivot-features.tsv")
    return {path: row["id"] for path, row in table.items()}


def feature_ids(paths: set[str]) -> list[str]:
    """The catalog ids of `paths`; a path the catalog does not have is an error."""
    ids = catalog_ids()
    if missing := sorted(p for p in paths if p not in ids):
        raise ValueError(f"paths not in the pivot catalog: {missing}")
    return sorted({ids[p] for p in paths})


def covering_rows(seed: int) -> list[dict[str, str]]:
    """The seeded covering array the suite is built from."""
    return covering.build(FACTORS, GROUPS, allowed, random.Random(seed))


class _Rotation:
    """The custom subtotal functions, handed out in turn across a suite so every one appears."""

    def __init__(self) -> None:
        self.counter = 0

    def take(self, count: int) -> list[str]:
        out = [SUBTOTALS[(self.counter + k) % len(SUBTOTALS)] for k in range(count)]
        self.counter += count
        return out


class _Pivot:
    """One covering-array row turned into one pivot sheet (`spec_sheet`) and the catalog paths it uses."""

    def __init__(self, row: Mapping[str, str], rng: random.Random, name: str, rotation: _Rotation) -> None:
        self.row, self.rng, self.name, self.rotation = row, rng, name, rotation
        self.src = build_source(rng, row["pool"])
        self.paths = set(BASE_PATHS)
        self.fields: dict[str, dict[str, Any]] = {f: {"attrs": {}} for f in HEADER}
        self.table: dict[str, Any] = {}
        self.definition: dict[str, Any] = {"table": self.table}
        self.rows_f, self.cols_f = self._axes(int(row["rows"]), int(row["cols"]))
        for axis, names in (("row", self.rows_f), ("col", self.cols_f)):
            if names:
                self.paths |= set(_axis_paths(axis))
        self.pages = self._filter(row["filter"])
        self.data = self._data(int(row["data"]))
        if row["caption"] == "error":
            ensure_error(rng, self.src, self.data[0]["field"])
        self._layout(row["layout"])
        self._subtotals(row["subtotals"])
        self._values(row["values"])
        self._grand(row["grand"])
        self._caption(row["caption"])
        self._extra(row["extra"])
        for field in [*self.rows_f, *self.cols_f, *self.pages]:
            self.paths |= {f"{CACHE_FIELD}/sharedItems/{k}" for k in _kinds(self.src, field)}

    def spec_sheet(self) -> dict[str, Any]:
        self.definition["data"] = self.data
        # An empty `subtotals` list is meaningful (no subtotal lines); other empty entries are left out.
        entries = {f: {k: v for k, v in e.items() if v or k == "subtotals"} for f, e in self.fields.items()}
        self.definition["fields"] = {f: e for f, e in entries.items() if e}
        pivot = {
            "name": f"Pv{self.name}",
            "source": f"{self.name}!{self.src.ref}",
            "location": LOCATION,
            "definition": self.definition,
        }
        sheet: dict[str, Any] = {"name": self.name, "cells": self.src.cells(), "pivot_tables": [pivot]}
        if self.src.formats:
            sheet["number_formats"] = self.src.formats
        return sheet

    @property
    def outer(self) -> list[str]:
        """The axis fields that have subtotal lines: every one but the innermost of its axis."""
        return self.rows_f[:-1] + self.cols_f[:-1]

    def _axes(self, n_rows: int, n_cols: int) -> tuple[list[str], list[str]]:
        """The row and column fields: the pool's edge field on rows (or columns) when there is one."""
        total, edge = n_rows + n_cols, self.src.edge
        others = [f for f in LABELS if f != edge]
        self.rng.shuffle(others)
        if edge is None or not total:
            chosen = others[:total]
        else:
            chosen = others[: total - 1]
            chosen.insert(self.rng.randrange(n_rows) if n_rows else self.rng.randrange(total), edge)
        return chosen[:n_rows], chosen[n_rows:]

    def _filter(self, kind: str) -> list[str]:
        """Hidden items on an axis field, or page fields not on an axis; returns the page fields."""
        rng, src = self.rng, self.src
        if kind == "none":
            return []
        if kind == "hidden":
            candidates = [f for f in self.rows_f + self.cols_f if len(src.items(f)) >= 2]
            if candidates:
                items = src.items(candidates[0])
                self.fields[candidates[0]]["hidden"] = rng.sample(
                    items, rng.randint(1, max(1, len(items) // 2))
                )
                self.paths.add(f"{PF}/items/item@h")
            return []
        free = [f for f in LABELS if f not in self.rows_f + self.cols_f and len(src.items(f)) >= 2]
        rng.shuffle(free)
        pages: list[dict[str, Any]] = [{"field": f} for f in free[: MAX_PAGES if rng.random() < 0.3 else 1]]
        if not pages:
            return []
        items = src.items(pages[0]["field"])
        if kind == "page_item":
            pages[0]["item"] = rng.choice(items)
            self.paths.add(f"{T}/pageFields/pageField@item")
        elif kind == "page_multi":
            self.fields[pages[0]["field"]]["hidden"] = rng.sample(items, rng.randint(1, len(items) - 1))
            self.paths |= {f"{PF}/items/item@h", f"{PF}@multipleItemSelectionAllowed"}
        self.definition["pages"] = pages
        self.paths |= set(PAGE_PATHS) | {
            f"{PF}@axis",
            f"{PF}/items",
            f"{PF}/items/item",
            f"{PF}/items/item@x",
        }
        return [p["field"] for p in pages]

    def _data(self, count: int) -> list[dict[str, Any]]:
        """The data fields: the row's aggregate over the pool's value field, then distinct random ones."""
        first = EDGE_VALUE_FIELD if self.src.pool in VALUE_EDGE else self.rng.choice(VALUES)
        pairs = [(first, self.row["aggregate"])]
        while len(pairs) < count:
            pair = (self.rng.choice(VALUES), self.rng.choice(list(AGGREGATES)))
            if pair not in pairs:
                pairs.append(pair)
        self.paths |= {f"ST_DataConsolidateFunction={agg}" for _, agg in pairs}
        return [{"field": f, "subtotal": agg} for f, agg in pairs]

    def _layout(self, layout: str) -> None:
        """Compact is the default; outline and tabular set the table and every field (pivots.md, T-4138)."""
        self.paths |= set(LAYOUT_PATHS[layout])
        if layout == "compact":
            return
        self.table |= {"compact": False, "compactData": False}
        if layout == "tabular":
            self.table |= {"outline": False, "outlineData": False}
        for entry in self.fields.values():
            entry["attrs"] |= (
                {"compact": False} if layout == "outline" else {"compact": False, "outline": False}
            )

    def _subtotals(self, kind: str) -> None:
        if not self.outer:
            return
        if kind == "top":
            self.paths |= {"ST_ItemType=default", f"{PF}@subtotalTop", f"{PF}@defaultSubtotal"}
        elif kind == "bottom":
            self.paths |= {"ST_ItemType=default", f"{PF}@subtotalTop"}
            for f in self.rows_f[:-1]:
                self.fields[f]["attrs"]["subtotalTop"] = False
        else:
            self.paths.add(f"{PF}@defaultSubtotal")
            for f in self.outer:
                functions = [] if kind == "none" else self.rotation.take(self.rng.randint(1, 2))
                self.fields[f]["subtotals"] = functions
                self.paths |= {f"ST_ItemType={s}" for s in functions} | {
                    f"{PF}@{s}Subtotal" for s in functions
                }

    def _values(self, kind: str) -> None:
        rows, cols = list(self.rows_f), list(self.cols_f)
        if kind != "single":
            self.paths.add("ST_Axis=axisValues")
            target = rows if kind.startswith("rows") else cols
            target.insert(0 if kind.endswith("start") else len(target), VALUES_FIELD)
            if kind.startswith("rows"):
                self.paths.add(f"{T}@dataOnRows")
        self.definition["rows"], self.definition["columns"] = rows, cols

    def _grand(self, kind: str) -> None:
        if kind != "none":
            self.paths.add("ST_ItemType=grand")
        if kind in ("no_rows", "none"):
            self.table["rowGrandTotals"] = False
            self.paths.add(f"{T}@rowGrandTotals")
        if kind in ("no_cols", "none"):
            self.table["colGrandTotals"] = False
            self.paths.add(f"{T}@colGrandTotals")

    def _caption(self, kind: str) -> None:
        rng, axis = self.rng, self.rows_f + self.cols_f
        if kind == "headers":
            self.table |= {"rowHeaderCaption": "Rows", "colHeaderCaption": "Cols", "dataCaption": "Measures"}
        elif kind == "grand_caption":
            self.table["grandTotalCaption"] = rng.choice(["All", "Everything"])
        elif kind == "missing":
            self.table |= {"missingCaption": rng.choice(["n/a", "-"]), "showMissing": True}
        elif kind == "error":
            self.table |= {"errorCaption": "ERR", "showError": True}
        elif kind == "names":
            for k, entry in enumerate(self.data, start=1):
                entry["name"] = f"Measure {k}"
            if axis:
                self.fields[axis[0]]["attrs"]["name"] = f"{axis[0]} field"
        elif kind == "item_caption":
            texts = [(f, v) for f in axis for v in self.src.items(f) if isinstance(v, str)]
            if not texts:
                return
            target, item = texts[0]
            self.fields[target]["item_attrs"] = {item: {"n": f"{item} shown"}}
        elif kind == "subtotal_caption":
            if not self.outer:
                return
            self.fields[self.outer[0]]["attrs"]["subtotalCaption"] = rng.choice(["Sub", "? Sub"])
        self.paths |= set(CAPTION_PATHS.get(kind, []))

    def _extra(self, kind: str) -> None:
        if kind == "none":
            return
        if kind == "classic":
            self.table["gridDropZones"] = True
        elif kind == "num_format":
            self.data[0]["numFmtId"] = self.rng.choice(NUMBER_FORMATS)
        else:
            outer = self.fields[self.rows_f[0]]
            if kind == "blank_row":
                outer["attrs"]["insertBlankRow"] = True
            elif kind == "repeat_labels":
                outer["xml"] = {"extLst": FILL_DOWN}
            else:  # collapsed
                outer["collapsed"] = [self.src.items(self.rows_f[0])[0]]
        self.paths |= set(EXTRA_PATHS[kind])


def _kinds(src: Source, name: str) -> set[str]:
    """The shared-item elements the field's values are cached as (`m` for a blank, `e` for an error)."""
    out = {"d"} if name == DATE_FIELD else set()  # date-formatted serials are cached as dates (pivots.md)
    for v in src.column(name):
        if v is None:
            out.add("m")
        elif name == DATE_FIELD:
            continue
        elif isinstance(v, dict):
            out.add("e" if v["t"] == "e" else "s")
        else:
            out.add(ITEM_KIND[float if isinstance(v, int) and not isinstance(v, bool) else type(v)])
    return out


def summary(row: Mapping[str, str]) -> str:
    """One sheet's levels, in factor order, for the case description."""
    return " ".join(f"{k}={row[k]}" for k in FACTORS)


def generate_pivot_suite(seed: int, per_case: int = PER_CASE) -> list[dict[str, Any]]:
    """The suite's cases for `seed` (module docstring)."""
    rows = covering_rows(seed)
    rotation = _Rotation()
    specs = []
    for start in range(0, len(rows), per_case):
        chunk = rows[start : start + per_case]
        sheets = []
        paths: set[str] = set()
        lines = []
        for k, row in enumerate(chunk, start=1):
            index = start + k - 1
            name = f"P{k:02d}"
            pivot = _Pivot(row, random.Random(seed * 1_000_003 + index), name, rotation)
            sheets.append(pivot.spec_sheet())
            paths |= pivot.paths
            lines.append(f"{name} (row {index}): {summary(row)}")
        controls = {f"A{k}": f"=COUNTA({s['name']}!A:A)" for k, s in enumerate(sheets, start=1)}
        specs.append(
            {
                "format": "rustsheet-case/v1",
                "description": (
                    f"generated pivots seed={seed} case={start // per_case} milestone={MILESTONE} "
                    f"(rustsheet-gen --milestone {MILESTONE}, T-4132): rows {start}-{start + len(chunk) - 1} of the "
                    f"covering array, one builder-v2 pivot per sheet at {LOCATION} over the sheet's own A:I source, "
                    "which Excel renders on open; Q holds one COUNTA control per sheet. " + "; ".join(lines)
                ),
                "tags": ["generated", "pivot", MILESTONE],
                "pivot_features": feature_ids(paths),
                "expected": {},
                "sheets": [*sheets, {"name": "Q", "cells": controls}],
            }
        )
    return specs
