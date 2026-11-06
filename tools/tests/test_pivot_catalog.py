"""The pivot feature catalog (T-4125): every schema node has a row, and the generator's walk, rules and
id merge behave."""

from __future__ import annotations

from pathlib import Path

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
    stale = sorted(set(table) - {n.path for n in nodes})
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
        "\t".join({**row, "web": "yes", "status": "done", "evidence": "probes/x"}[h] for h in HEADER),
    )
    tsv.write_text(text)
    write(xsd, out)
    again = read_table(tsv)["pivotTableDefinition/location@ref"]
    assert (again["id"], again["web"], again["status"], again["evidence"]) == (
        row["id"],
        "yes",
        "done",
        "probes/x",
    )
    added = rows([*walk(xsd), Node("ST_Axis=axisCol")], read_table(tsv))
    new = next(r for r in added if r["ooxml"] == "ST_Axis=axisCol")
    assert new["id"] == f"PV-{len(table) + 1:04d}"
