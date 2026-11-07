"""The pivot feature catalog (T-4125): every schema node has a row, and the generator's walk, rules and
id merge behave.

The node list these tests read is the generator's own committed output; the schemas are not vendored,
so drift between it and the XSDs shows only when someone regenerates (catalog-findings.md says how).
"""

from __future__ import annotations

from pathlib import Path

import pytest

from rustsheet_tools import repo_root
from rustsheet_tools.pivot_catalog import HEADER, RULES, Node, classify, read_table, rows, walk, write

CATALOG = repo_root() / "eval" / "catalog"
FAMILIES = {family for _, family, _, _ in RULES}


def _nodes() -> list[Node]:
    lines = (CATALOG / "pivot-schema-nodes.txt").read_text().splitlines()
    return [Node(*line.split("\t")) for line in lines if not line.startswith("#")]


def test_every_schema_node_has_one_row() -> None:
    nodes = _nodes()
    table = read_table(CATALOG / "pivot-features.tsv")
    assert len(nodes) > 1000  # the walk reached the whole pivot schema
    missing = [n.path for n in nodes if n.path not in table]
    assert missing == []
    stale = [p for p in set(table) - {n.path for n in nodes} if table[p]["status"] != "retired"]
    assert stale == []
    for n in nodes:
        assert table[n.path]["default"] == n.default, n.path


def test_rows_are_well_formed() -> None:
    lines = (CATALOG / "pivot-features.tsv").read_text().splitlines()
    assert tuple(lines[0].split("\t")) == HEADER
    table = list(read_table(CATALOG / "pivot-features.tsv").values())
    assert len({r["id"] for r in table}) == len(table)
    for r in table:
        assert r["family"] in FAMILIES, r
        assert r["affects"] in {"values", "layout", "text", "none"}, r
        assert r["web"] in {"yes", "no", "unknown"}, r
        assert r["scope"] in {"in", "out (decision 0274)", "round-trip only (decision 0274)"}, r


def test_the_committed_rows_are_what_the_rules_give() -> None:
    table = read_table(CATALOG / "pivot-features.tsv")
    for node in _nodes():
        family, affects, scope, stage = classify(node)
        r = table[node.path]
        assert (r["family"], r["affects"], r["scope"], r["stage"]) == (family, affects, scope, stage)


def test_classification_examples() -> None:
    assert classify(Node("ST_DataConsolidateFunction=sum"))[:2] == ("aggregation", "values")
    assert classify(Node("x14:dataField@pivotShowAs"))[0] == "show-values-as"
    assert classify(Node("pivotTableDefinition/location@ref"))[0] == "layout"
    assert classify(Node("pivotCacheDefinition/cacheFields/cacheField/fieldGroup/rangePr@groupBy"))[0] == (
        "grouping"
    )
    olap = classify(Node("pivotCacheDefinition/cacheHierarchies/cacheHierarchy@uniqueName"))
    assert olap[2] == "out (decision 0274)"
    style = classify(Node("pivotTableDefinition/pivotTableStyleInfo@name"))
    assert style[:3] == ("formatting", "none", "round-trip only (decision 0274)")
    assert classify(Node("x14:slicerCacheDefinition/data/olap@pivotCacheId"))[0] == "olap-datamodel"
    assert classify(Node("x14:slicers/slicer@caption"))[0] == "slicers-timelines"
    # Slicer chrome and styles, and file metadata under a slicer, are round-trip only.
    assert classify(Node("x14:slicers/slicer@rowHeight"))[:2] == ("ui", "none")
    assert classify(Node("x14:slicerStyles/slicerStyle@name"))[0] == "formatting"
    assert classify(Node("x14:slicers/slicer@xr10:uid"))[0] == "metadata"
    # Features excel-docs.md marks as changing values or layout are in scope.
    assert classify(Node("pivotTableDefinition/pivotFields/pivotField@includeNewItemsInFilter"))[:2] == (
        "filtering",
        "values",
    )
    assert classify(Node("x14:pivotTableDefinition@hideValuesRow"))[0] == "layout"
    assert classify(Node("pivotTableDefinition/pivotFields/pivotField@subtotalTop"))[0] == "totals"
    assert classify(Node("pivotTableDefinition/dataFields/dataField@subtotal"))[0] == "aggregation"


XSD = """<?xml version="1.0"?>
<xsd:schema xmlns:xsd="http://www.w3.org/2001/XMLSchema">
  <xsd:complexType name="CT_pivotTableDefinition">
    <xsd:sequence>
      <xsd:element name="location" type="CT_Location"/>
      <xsd:element name="self" type="CT_pivotTableDefinition"/>
    </xsd:sequence>
    <xsd:attribute name="name" type="xsd:string"/>
    <xsd:attribute name="compact" type="xsd:boolean" default="true"/>
  </xsd:complexType>
  <xsd:complexType name="CT_Location"><xsd:attribute name="ref" type="xsd:string"/></xsd:complexType>
  <xsd:simpleType name="ST_Axis">
    <xsd:restriction base="xsd:string"><xsd:enumeration value="axisRow"/></xsd:restriction>
  </xsd:simpleType>
</xsd:schema>
"""
EMPTY = '<?xml version="1.0"?><xsd:schema xmlns:xsd="http://www.w3.org/2001/XMLSchema"/>'


def _schemas(tmp: Path) -> Path:
    for rel, text in (
        ("sml/sml_ECMA376_4ed_transitional.xsd", XSD),
        ("xlsx/office_spreadsheetml_2009_9_main.xsd", EMPTY),
        ("xlsx/office_spreadsheetml_2010_11_main.xsd", EMPTY),
    ):
        (tmp / rel).parent.mkdir(parents=True, exist_ok=True)
        (tmp / rel).write_text(text)
    return tmp


def test_walk_lists_elements_attributes_defaults_and_enums_once(tmp_path: Path) -> None:
    paths = {n.path: n.default for n in walk(_schemas(tmp_path))}
    assert paths["pivotTableDefinition@compact"] == "true"
    assert "pivotTableDefinition/location@ref" in paths
    assert "ST_Axis=axisRow" in paths
    # A recursive type is walked once per path, not forever.
    assert "pivotTableDefinition/self" in paths
    assert "pivotTableDefinition/self/self" not in paths


def test_regeneration_keeps_ids_and_hand_filled_columns(tmp_path: Path) -> None:
    out = tmp_path / "out"
    out.mkdir()
    xsd = _schemas(tmp_path / "xsd")
    write(xsd, out)
    tsv = out / "pivot-features.tsv"
    table = read_table(tsv)
    row = table["pivotTableDefinition/location@ref"]
    text = tsv.read_text().replace(
        "\t".join(row[h] for h in HEADER),
        "\t".join(
            {**row, "web": "yes", "status": "done", "evidence": "probes/x", "owner": "T-0001"}[h]
            for h in HEADER
        ),
    )
    tsv.write_text(text)
    write(xsd, out)
    again = read_table(tsv)["pivotTableDefinition/location@ref"]
    assert (again["id"], again["web"], again["status"], again["evidence"], again["owner"]) == (
        row["id"],
        "yes",
        "done",
        "probes/x",
        "T-0001",
    )
    added = rows([*walk(xsd), Node("ST_Axis=axisCol")], read_table(tsv))
    new = next(r for r in added if r["ooxml"] == "ST_Axis=axisCol")
    assert new["id"] == f"PV-{len(table) + 1:04d}"


def test_a_table_without_the_owner_column_regenerates_with_it_empty(tmp_path: Path) -> None:
    xsd = _schemas(tmp_path / "xsd")
    previous = {r["ooxml"]: {k: v for k, v in r.items() if k != "owner"} for r in rows(walk(xsd), {})}
    gone = next(iter(previous))
    again = rows([n for n in walk(xsd) if n.path != gone], previous)
    assert {r["owner"] for r in again} == {""}
    assert next(r for r in again if r["ooxml"] == gone)["status"] == "retired"


def test_a_dropped_node_keeps_its_row_retired_and_its_id_is_not_reused(tmp_path: Path) -> None:
    xsd = _schemas(tmp_path / "xsd")
    previous = {r["ooxml"]: r for r in rows(walk(xsd), {})}
    last = max(previous.values(), key=lambda r: r["id"])
    survivors = [n for n in walk(xsd) if n.path != last["ooxml"]]
    again = rows([*survivors, Node("ST_Axis=axisNew")], previous)
    retired = next(r for r in again if r["ooxml"] == last["ooxml"])
    assert (retired["id"], retired["status"]) == (last["id"], "retired")
    new = next(r for r in again if r["ooxml"] == "ST_Axis=axisNew")
    assert new["id"] > last["id"]


def test_bad_inputs_fail_clearly(tmp_path: Path) -> None:
    bad = tmp_path / "pivot-features.tsv"
    bad.write_text("\t".join(HEADER) + "\nPV-0001\tlayout\n")
    with pytest.raises(ValueError, match="tab-separated"):
        read_table(bad)
    xsd = _schemas(tmp_path / "xsd")
    main = xsd / "sml/sml_ECMA376_4ed_transitional.xsd"
    main.write_text(XSD.replace('type="CT_Location"', 'type="zz:CT_Location"'))
    with pytest.raises(ValueError, match="namespace prefix"):
        walk(xsd)
