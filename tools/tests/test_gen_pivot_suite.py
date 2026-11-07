"""The gen-pivot policy (T-4132): its covering array, its determinism, and cases the builder accepts."""

from __future__ import annotations

import itertools
import random
from functools import cache
from typing import Any

import pytest

from rustsheet_tools import repo_root
from rustsheet_tools.gen import covering
from rustsheet_tools.gen.pivot_data import POOL_NAMES, build_source
from rustsheet_tools.gen.pivot_suite import (
    FACTORS,
    GROUPS,
    PER_CASE,
    THREE_WISE,
    allowed,
    covering_rows,
    generate_pivot_suite,
)
from rustsheet_tools.pivot_catalog import read_table
from rustsheet_tools.pivot_definition import AGGREGATES, SUBTOTALS
from rustsheet_tools.spec2xlsx import build_xlsx


@cache
def _suite(seed: int) -> tuple[dict[str, Any], ...]:
    return tuple(generate_pivot_suite(seed))


def _catalog() -> dict[str, dict[str, str]]:
    return {
        row["id"]: row for row in read_table(repo_root() / "eval" / "catalog" / "pivot-features.tsv").values()
    }


def test_a_small_array_is_pairwise_and_respects_its_constraint() -> None:
    factors = {"a": ["0", "1", "2"], "b": ["x", "y"], "c": ["p", "q", "r"], "d": ["u", "v"]}

    def no_a2_with_x(p: Any) -> bool:
        return not (p.get("a") == "2" and p.get("b") == "x")

    groups = [covering.Group(tuple(factors), 2)]
    rows = covering.build(factors, groups, no_a2_with_x, random.Random(3))
    assert covering.uncovered(rows, factors, groups, no_a2_with_x) == set()
    assert all(no_a2_with_x(r) for r in rows)
    assert (("a", "2"), ("b", "x")) not in covering.required(factors, groups, no_a2_with_x)
    assert len(rows) <= 12  # 3 x 3 is the floor; greedy stays close
    assert rows == covering.build(factors, groups, no_a2_with_x, random.Random(3))


def test_three_wise_groups_cover_every_triple() -> None:
    factors = {"a": ["0", "1"], "b": ["0", "1", "2"], "c": ["0", "1"], "d": ["0", "1"]}
    groups = [covering.Group(("a", "b", "c"), 3), covering.Group(tuple(factors), 2)]
    rows = covering.build(factors, groups, lambda _p: True, random.Random(0))
    triples = {tuple((k, r[k]) for k in ("a", "b", "c")) for r in rows}
    assert len(triples) == 12
    assert covering.uncovered(rows, factors, groups, lambda _p: True) == set()


@pytest.mark.parametrize("seed", [0, 1])
def test_the_suite_array_covers_pairs_and_core_triples(seed: int) -> None:
    rows = covering_rows(seed)
    assert covering.uncovered(rows, FACTORS, GROUPS, allowed) == set()
    assert all(allowed(r) for r in rows)
    core = {tuple(r[k] for k in THREE_WISE) for r in rows}
    floor = max(
        len(FACTORS[a]) * len(FACTORS[b]) * len(FACTORS[c])
        for a, b, c in itertools.combinations(THREE_WISE, 3)
    )
    assert floor <= len(rows) <= floor + 30
    assert len(core) >= floor


def test_generation_is_deterministic_by_seed() -> None:
    assert list(_suite(0)) == generate_pivot_suite(0)
    assert [c["sheets"] for c in _suite(0)] != [c["sheets"] for c in _suite(1)]


def test_every_case_builds_and_holds_one_pivot_per_sheet() -> None:
    for case in _suite(0):
        build_xlsx(case)  # the v2 builder refuses a definition outside its schema
        pivots = [s for s in case["sheets"] if s.get("pivot_tables")]
        assert 1 <= len(pivots) <= PER_CASE
        assert all(len(s["pivot_tables"]) == 1 for s in pivots)
        assert case["sheets"][-1]["name"] == "Q"
        assert len(case["sheets"][-1]["cells"]) == len(pivots)
        assert case["tags"] == ["generated", "pivot", "M16"]


def test_cases_tag_in_scope_catalog_ids_covering_the_pv1_enumerations() -> None:
    catalog = _catalog()
    by_path = {row["ooxml"]: row["id"] for row in catalog.values()}
    tagged = {i for case in _suite(0) for i in case["pivot_features"]}
    assert all(catalog[i]["scope"] == "in" for i in tagged)
    # Page fields and hidden items: PV1 by the plan's stage table, PV2 rows in the catalog.
    pv2 = {i for i in tagged if catalog[i]["stage"] == "PV2"}
    assert {catalog[i]["ooxml"].split("/")[-1] for i in pv2} <= {
        "pageFields",
        "pageFields@count",
        "pageField",
        "pageField@fld",
        "pageField@hier",
        "pageField@item",
        "item@h",
        "pivotField@multipleItemSelectionAllowed",
    }
    assert all(catalog[i]["stage"] in ("PV1", "PV2") for i in tagged)
    for name in [
        *(f"ST_DataConsolidateFunction={a}" for a in AGGREGATES),
        *(f"ST_ItemType={s}" for s in SUBTOTALS),
    ]:
        assert by_path[name] in tagged, name
    for axis in ("axisRow", "axisCol", "axisPage", "axisValues"):
        assert by_path[f"ST_Axis={axis}"] in tagged


@pytest.mark.parametrize("pool", POOL_NAMES)
def test_each_pool_builds_a_source_with_its_edge(pool: str) -> None:
    source = build_source(random.Random(7), pool)
    values = [v for row in source.rows for v in row]
    if pool == "many":
        assert len(source.items("Rep")) >= 25
    if pool == "blanks":
        assert None in source.column("Channel")
    if pool == "errors":
        assert any(isinstance(v, dict) and v["t"] == "e" for v in values)
    if pool == "dates":
        assert any(v != int(v) for v in source.column("When"))
    assert all(not (isinstance(v, str) and (v == "" or v.startswith("="))) for v in values)


def test_every_level_reaches_the_definition() -> None:
    """Spot checks that a row's levels are written: `none` subtotals as an empty list, and the rest."""
    seen: dict[str, set[str]] = {k: set() for k in ("subtotals", "extra")}
    for case in _suite(0):
        for sheet in case["sheets"][:-1]:
            definition = sheet["pivot_tables"][0]["definition"]
            for entry in definition.get("fields", {}).values():
                if entry.get("subtotals") == []:
                    seen["subtotals"].add("none")
                elif "subtotals" in entry:
                    seen["subtotals"].add("custom")
                if "collapsed" in entry:
                    seen["extra"].add("collapsed")
                if "xml" in entry:
                    seen["extra"].add("repeat_labels")
                if entry.get("attrs", {}).get("insertBlankRow"):
                    seen["extra"].add("blank_row")
            if definition["table"].get("gridDropZones"):
                seen["extra"].add("classic")
            if any("numFmtId" in d for d in definition["data"]):
                seen["extra"].add("num_format")
    assert seen == {
        "subtotals": {"none", "custom"},
        "extra": {"collapsed", "repeat_labels", "blank_row", "classic", "num_format"},
    }
