"""The pivot feature catalog, generated from the OOXML schemas (T-4125, M16).

`rustsheet-pivot-catalog --xsd DIR` walks the ECMA-376 transitional SpreadsheetML schema, plus the
Excel 2010 (x14) and 2013 (x15) extension schemas, from the pivot parts' root types. It writes:
- eval/catalog/pivot-schema-nodes.txt: every element and attribute path reached, with its default;
- eval/catalog/pivot-features.tsv: one row per node and per value of the enumerations a pivot computes
  with (aggregates, show-as modes, filter types, group levels, item types, sort types).

Every row carries a family, what it `affects` (values, layout, text or none, with the reason for none),
its scope (in, or out by decision 0274) and its M16 stage. A row that later work fills (web support,
status, evidence) starts `unknown` or `todo`. The schema files are not vendored. They come from a
pinned docx4j checkout (`SCHEMA_SOURCE`); docx4j is Apache-2.0 and the schemas are ECMA-376's. The node
list is committed, so the coverage test (every node has a row) runs without them.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from xml.etree import ElementTree

from rustsheet_tools import repo_root
from rustsheet_tools.oracle.package import parse_part

SCHEMA_SOURCE = (
    "https://github.com/plutext/docx4j at f960df59ef49f0d0a81f2ad7383b7dfaad14c46b: "
    "xsd/sml/sml_ECMA376_4ed_transitional.xsd, xsd/xlsx/office_spreadsheetml_2009_9_main.xsd, "
    "xsd/xlsx/office_spreadsheetml_2010_11_main.xsd"
)
SCHEMAS = {
    "": "sml/sml_ECMA376_4ed_transitional.xsd",
    "x14:": "xlsx/office_spreadsheetml_2009_9_main.xsd",
    "x15:": "xlsx/office_spreadsheetml_2010_11_main.xsd",
}
ROOTS = {
    "pivotTableDefinition": "CT_pivotTableDefinition",
    "pivotCacheDefinition": "CT_PivotCacheDefinition",
    "pivotCacheRecords": "CT_PivotCacheRecords",
}
# Extension roots: the x14/x15 elements a pivot, its cache, a slicer or a timeline carries, by element
# name (the walk also reaches their children). Types are walked from these, not one by one, so a type
# shared by two roots is listed under each path it can appear at, as for the main schema.
EXTENSION_ROOTS = {
    "x14:": (
        "pivotTableDefinition",
        "pivotCacheDefinition",
        "pivotField",
        "dataField",
        "cacheField",
        "pivotHierarchy",
        "cacheHierarchy",
        "slicers",
        "slicerCacheDefinition",
        "slicerStyles",
    ),
    "x15:": (
        "pivotTableData",
        "pivotTableUISettings",
        "pivotFilter",
        "dataField",
        "cacheHierarchy",
        "pivotCacheIdVersion",
        "pivotTableReferences",
        "timelines",
        "timelineCacheDefinition",
        "timelinePivotCacheDefinition",
        "tableSlicerCache",
        "slicerCacheHideItemsWithNoData",
        "timelineStyles",
    ),
}
ENUMS = (
    "ST_DataConsolidateFunction",
    "ST_ShowDataAs",
    "x14:ST_PivotShowAs",
    "ST_PivotFilterType",
    "ST_GroupBy",
    "ST_ItemType",
    "ST_FieldSortType",
    "ST_Axis",
)
XS = "{http://www.w3.org/2001/XMLSchema}"
MAX_NODES = 100_000
# Namespaces whose types the walk does not enter: XML Schema built-ins, the shared simple types, and
# the drawing and formula types a pivot part never holds.
NOT_WALKED = {"xsd", "s", "xdr", "xm"}
HEADER = ("id", "family", "ooxml", "default", "affects", "web", "scope", "stage", "status", "evidence")


@dataclass(frozen=True)
class Node:
    """One schema node: an element path (`a/b/c`) or an attribute of one (`a/b@x`), or an
    enumeration value (`ST_X=value`)."""

    path: str
    default: str = ""


def _resolve(type_ref: str, file_prefix: str) -> str:
    """A type reference as a key of the walk's type table: `x:` is the main schema, `x14:` and `x15:`
    the extensions, no prefix the referring file's own schema; built-in `xsd:` types have none."""
    if ":" not in type_ref:
        return file_prefix + type_ref
    ns, local = type_ref.split(":", 1)
    if ns in NOT_WALKED:
        return ""
    prefixes = {"x": "", "x14": "x14:", "x15": "x15:"}
    if ns not in prefixes:
        # A schema bump that references another namespace must not drop its nodes silently.
        raise ValueError(f"unknown schema namespace prefix in type {type_ref!r}")
    return prefixes[ns] + local


def walk(xsd_dir: Path) -> list[Node]:
    """Every node the pivot parts' schemas reach, sorted."""
    types: dict[str, ElementTree.Element] = {}
    elements: dict[str, str] = {}  # top-level element name (with file prefix) -> its type key
    enums: dict[str, list[str]] = {}
    for prefix, rel in SCHEMAS.items():
        root = parse_part(rel, (xsd_dir / rel).read_bytes())
        for t in root.iter(XS + "complexType"):
            if t.get("name"):
                types[prefix + str(t.get("name"))] = t
        for e in root.findall(XS + "element"):
            elements[prefix + str(e.get("name"))] = _resolve(str(e.get("type", "")), prefix)
        for st in root.iter(XS + "simpleType"):
            if st.get("name"):
                enums[prefix + str(st.get("name"))] = [
                    str(v.get("value")) for v in st.iter(XS + "enumeration")
                ]
    nodes: set[Node] = set()

    def visit(type_key: str, path: str, seen: frozenset[str]) -> None:
        ct = types.get(type_key)
        if ct is None or type_key in seen:
            return
        if len(nodes) > MAX_NODES:  # a wrong --xsd directory, not the pinned schemas (about 1,600)
            raise ValueError(f"the schema walk passed {MAX_NODES} nodes")
        file_prefix = type_key[: type_key.index(":") + 1] if ":" in type_key else ""
        for a in ct.iter(XS + "attribute"):
            name = a.get("name") or a.get("ref")
            if name:
                nodes.add(Node(f"{path}@{name}", a.get("default", "")))
        for e in ct.iter(XS + "element"):
            name = e.get("name") or e.get("ref")
            if not name:
                continue
            child = f"{path}/{name}"
            nodes.add(Node(child))
            if e.get("name"):
                visit(_resolve(str(e.get("type", "")), file_prefix), child, seen | {type_key})

    for element, type_name in ROOTS.items():
        nodes.add(Node(element))
        visit(type_name, element, frozenset())
    for prefix, names in EXTENSION_ROOTS.items():
        for name in names:
            nodes.add(Node(prefix + name))
            visit(elements.get(prefix + name, ""), prefix + name, frozenset())
    for enum in ENUMS:
        nodes.update(Node(f"{enum}={v}") for v in enums.get(enum, []))
    return sorted(nodes, key=lambda n: n.path)


# --------------------------------------------------------------------------- classification
# (pattern, family, affects, stage). First match wins, so the families that override come first: OLAP,
# metadata, formatting and UI, then slicers and timelines (whose styles and chrome are formatting). `out` and `round-trip` stages are decision 0274's scope
# rulings; an `affects none` row says why in its family (metadata, formatting, ui).
RULES: tuple[tuple[str, str, str, str], ...] = (
    (r"(?i)slicer.*olap", "olap-datamodel", "none", "out"),
    (
        r"(?i)(hierarch|kpi|dimension|measureGroup|/maps|calculatedMember|olap|@mdx|@serverField|"
        r"@uniqueMemberProperty|@uniqueName|consolidation|@connectionId|pivotEdit|pivotChange|userEdit|"
        r"editValue|tupleItem|tupleCache|@weightExpression|@enableEdit|@editData|@autoApply|"
        r"x15:pivotTableData|x15:dataField|x15:cacheHierarchy|cachedUniqueNames|@supportAddCalcMems|"
        r"@supportSubquery|@supportAdvancedDrill|@allocation|@showCalcMbrs|@showMemberPropertyTips|"
        r"@asteriskTotals|@member|@isMeasure|@visualTotals|@sourceField|@mpMap|@isDataModel|"
        # OLAP only per MS-OI29500 18.10.1.73 (excel-docs.md §2, totals and filters).
        r"@showEmptyRow|@showEmptyCol)",
        "olap-datamodel",
        "none",
        "out",
    ),
    (
        r"(createdVersion|updatedVersion|minRefreshableVersion|refreshedVersion|refreshedBy|refreshedDate|"
        r"upgradeOnRefresh|RefreshVersion|uid$|@mc:|@id$|@r:id|extLst|invalid|@pivotCacheId|@cacheId|"
        r"@tag$|@published|@altText|@ignore$|@recordCount|cacheField@level$|cacheField@uniqueList$)",
        "metadata",
        "none",
        "round-trip",
    ),
    (
        r"(?i)(style|/formats|/format/|conditionalFormat|chartFormat|@apply[A-Z]|autoFormat|Stripes|"
        r"@showRowHeaders|@showColHeaders|@showLastColumn|@indent|@mergeItem|@preserveFormatting|"
        r"@pageStyle|@dxfId)",
        "formatting",
        "none",
        "round-trip",
    ),
    (
        r"(@disableFieldList|@enableDrill|@enableFieldProperties|@enableWizard|@showDropZones|"
        r"@showDataDropDown|@showDataTips|@printDrill|@fieldPrintTitles|@itemPrintTitles|@immersive|"
        r"@showDrill|@dragTo|@dragOff|@showDropDowns|@showPropTip|@showPropCell|@showPropAsCaption|"
        r"@fieldListSortAscending|@insertPageBreak|pivotTableUISettings|@disableEdit|@trueOnly|"
        r"@backgroundQuery|@optimizeMemory|"
        # A slicer's or timeline's geometry and chrome: how it is drawn, not what it selects.
        r"(?i:slicer|timeline).*@(rowHeight|columnCount|lockedPosition|startItem|scrollPosition|showHeader|"
        r"showHorizontalScrollbar|showSelectionLabel|showTimeLevel|showCaption|level)$)",
        "ui",
        "none",
        "round-trip",
    ),
    (r"(?i)(slicer|timeline)", "slicers-timelines", "values", "PV5"),
    (
        r"(cacheSource|worksheetSource|@refreshOnLoad|@saveData|@missingItemsLimit|@enableRefresh|"
        r"pivotCacheIdVersion|pivotTableReferences)",
        "source-refresh",
        "values",
        "PV1",
    ),
    (r"(fieldGroup|rangePr|discretePr|groupItems|ST_GroupBy)", "grouping", "values", "PV3"),
    (r"(calculatedItem|@formula|@databaseField)", "calculated", "values", "PV3"),
    (r"(?i)(showDataAs|PivotShowAs|@baseField|@baseItem)", "show-values-as", "values", "PV2"),
    (
        # The legacy AutoShow attributes are a Top-N filter; `item@s` marks a manual filter (MS-OI29500
        # 18.10.1.45). Both decide which items exist (excel-docs.md §1.7).
        r"(filter|Filter|pageFields|pageField|@hideNewItems|@includeNewItemsInFilter|"
        r"@multipleItemSelectionAllowed|@measureFilter|@h$|@hidden|@hier$|@autoShow|@topAutoShow|"
        r"@itemPageCount|@rankBy|/item@s$)",
        "filtering",
        "values",
        "PV2",
    ),
    (
        r"(sortType|autoSort|FieldSortType|@sortOrder|@nonAutoSortDefault|@customListSort|@sortByTuple)",
        "sorting",
        "layout",
        "PV2",
    ),
    # Before totals: a data field's `subtotal` is its aggregate function, not a subtotal line.
    (
        r"(dataFields|^x14:dataField|DataConsolidateFunction|@numFmtId$|pivotField@dataField$)",
        "aggregation",
        "values",
        "PV1",
    ),
    # `Total` is deliberately broad: every *Total* name in the pivot schema is a totals feature.
    (r"(?i)(subtotal|GrandTotals|ST_ItemType|@t$|Total)", "totals", "layout", "PV1"),
    (
        r"(Caption|caption|@showError|@showMissing|@name$|@n$|@showHeaders|@showMultipleLabel)",
        "captions",
        "text",
        "PV1",
    ),
    (
        # The cache records' value elements are one letter (`x`, `n`, `s`, `b`, `e`, `m`, `d`, `r`).
        r"(sharedItems|cacheField|pivotCacheRecords|/[xnsbemd]$|/[xnsbemd]@|/r$|/r/|/r@|pivotCacheDefinition$)",
        "cache",
        "values",
        "PV1",
    ),
    (
        r"(location|rowItems|colItems|rowFields|colFields|pivotField|compact|outline|@showAll|"
        r"insertBlankRow|pageWrap|pageOverThenDown|@dataOnRows|@dataPosition|ST_Axis|@axis|@sd$|"
        r"fillDownLabels|@hideValuesRow|@showItems|@gridDropZones|pivotTableDefinition$|/i$|/i@|/i/)",
        "layout",
        "layout",
        "PV1",
    ),
)


def classify(node: Node) -> tuple[str, str, str, str]:
    """(family, affects, scope, stage) of a node. A node no rule matches is `uncategorized`, so the
    findings list it for a person to place."""
    for pattern, family, affects, stage in RULES:
        if re.search(pattern, node.path):
            scope = {"out": "out (decision 0274)", "round-trip": "round-trip only (decision 0274)"}
            return family, affects, scope.get(stage, "in"), stage
    return "uncategorized", "unknown", "in", "PV0"


# Columns later work fills by hand (probes, implementations); regeneration keeps them, and the id.
KEPT = ("web", "status", "evidence")
# The status of a row whose node left the schema walk. The row is kept so its id is never reused.
RETIRED = "retired"


def read_table(path: Path) -> dict[str, dict[str, str]]:
    """The committed TSV's rows by OOXML path (empty when there is none yet)."""
    if not path.exists():
        return {}
    lines = path.read_text().splitlines()
    header = lines[0].split("\t")
    table = {}
    for n, line in enumerate(lines[1:], start=2):
        cells = line.split("\t")
        if len(cells) != len(header) or not re.fullmatch(r"PV-\d{4,}", cells[0]):
            raise ValueError(f"{path}:{n}: expected {len(header)} tab-separated cells and a PV-nnnn id")
        row = dict(zip(header, cells, strict=True))
        table[row["ooxml"]] = row
    return table


def rows(nodes: list[Node], previous: dict[str, dict[str, str]]) -> list[dict[str, str]]:
    """One row per node, then the retired rows. A node the previous table had keeps its id and the
    hand-filled columns; a new one gets the next id after every id the table ever held. A previous
    row whose node left the walk stays, with status `retired`, so its id keeps naming it."""
    next_id = max((int(r["id"].removeprefix("PV-")) for r in previous.values()), default=0) + 1
    out = []
    for node in nodes:
        family, affects, scope, stage = classify(node)
        old = previous.get(node.path)
        if old is None:
            row_id, next_id = f"PV-{next_id:04d}", next_id + 1
        else:
            row_id = old["id"]
        row = {
            "id": row_id,
            "family": family,
            "ooxml": node.path,
            "default": node.default,
            "affects": affects,
            "web": "unknown",
            "scope": scope,
            "stage": stage,
            "status": "todo",
            "evidence": "",
        }
        if old is not None:
            row.update({k: old[k] for k in KEPT})
            if row["status"] == RETIRED:  # the node came back
                row["status"] = "todo"
        out.append(row)
    live = {n.path for n in nodes}
    out += [{**r, "status": RETIRED} for p, r in sorted(previous.items()) if p not in live]
    return out


def write(xsd_dir: Path, catalog_dir: Path) -> tuple[int, int]:
    """Writes the node list and the TSV; returns (nodes, uncategorized rows)."""
    nodes = walk(xsd_dir)
    catalog_dir.mkdir(parents=True, exist_ok=True)
    (catalog_dir / "pivot-schema-nodes.txt").write_text(
        f"# Generated by rustsheet-pivot-catalog from {SCHEMA_SOURCE}\n"
        + "".join(f"{n.path}\t{n.default}\n" for n in nodes)
    )
    tsv = catalog_dir / "pivot-features.tsv"
    table = rows(nodes, read_table(tsv))
    tsv.write_text("\t".join(HEADER) + "\n" + "".join("\t".join(r[h] for h in HEADER) + "\n" for r in table))
    return len(nodes), sum(1 for r in table if r["family"] == "uncategorized")


def main() -> None:
    ap = argparse.ArgumentParser(prog="rustsheet-pivot-catalog", description=__doc__.splitlines()[0])
    ap.add_argument("--xsd", type=Path, required=True, help="the docx4j checkout's xsd/ directory")
    ap.add_argument("--out", type=Path, default=None, help="catalog directory (default eval/catalog)")
    args = ap.parse_args()
    nodes, uncategorized = write(args.xsd, args.out or repo_root() / "eval" / "catalog")
    print(f"wrote {nodes} pivot catalog rows ({uncategorized} uncategorized)", file=sys.stderr)


if __name__ == "__main__":
    main()
