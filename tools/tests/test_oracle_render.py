from __future__ import annotations

import base64
import hashlib
import json
from pathlib import Path
from types import SimpleNamespace
from typing import Any

import pytest

from rustsheet_tools.oracle.backends import GRAPH, GraphBackend, GraphRequestError, OracleRefusedError
from rustsheet_tools.oracle.render import (
    REFUSED_SCHEMA,
    SCHEMA,
    RenderError,
    check_dump,
    load_probes,
    pinned_bytes,
    range_bounds,
    record,
    render_dir,
    write_refusal,
)


@pytest.fixture
def graph(monkeypatch: pytest.MonkeyPatch) -> GraphBackend:
    for var, value in (("TENANT_ID", "t"), ("CLIENT_ID", "c"), ("CLIENT_SECRET", "s"), ("DRIVE_ID", "b!x")):
        monkeypatch.setenv(f"RUSTSHEET_ORACLE_{var}", value)
    return GraphBackend()


def _reply(body: Any, content: bytes = b"") -> Any:
    return SimpleNamespace(json=lambda: body, content=content, status_code=200)


def test_batch_get_chunks_by_twenty_and_retries_only_throttled_parts(
    graph: GraphBackend, monkeypatch: pytest.MonkeyPatch
) -> None:
    sent: list[list[str]] = []
    throttled_once: set[str] = set()

    def call(method: str, url: str, **kw: Any) -> Any:
        assert (method, url) == ("POST", f"{GRAPH}/$batch")
        parts = kw["json"]["requests"]
        sent.append([p["id"] for p in parts])
        responses = []
        for p in parts:
            assert p["headers"] == {"workbook-session-id": "s"}
            assert not p["url"].startswith("https://")  # relative to the Graph root
            if p["id"] == "3" and "3" not in throttled_once:
                throttled_once.add("3")
                responses.append({"id": "3", "status": 429})
            else:
                responses.append({"id": p["id"], "status": 200, "body": {"n": int(p["id"])}})
        return _reply({"responses": responses})

    monkeypatch.setattr(graph, "_call", call)
    monkeypatch.setattr("rustsheet_tools.oracle.backends.time.sleep", lambda _: None)
    bodies = graph.batch_get([f"{GRAPH}/r/{i}" for i in range(25)], {"workbook-session-id": "s"})
    assert bodies == [{"n": i} for i in range(25)]
    assert [len(ids) for ids in sent] == [20, 5, 1]
    assert sent[2] == ["3"]


def test_batch_get_raises_on_an_error_part(graph: GraphBackend, monkeypatch: pytest.MonkeyPatch) -> None:
    error = {"error": {"code": "ItemNotFound", "message": "no such sheet"}}
    monkeypatch.setattr(
        graph,
        "_call",
        lambda *_, **__: _reply({"responses": [{"id": "0", "status": 404, "body": error}]}),
    )
    with pytest.raises(GraphRequestError) as e:
        graph.batch_get([f"{GRAPH}/r"], {})
    assert e.value.code == "ItemNotFound"


def test_range_bounds() -> None:
    assert range_bounds("B2:D9") == (1, 1, 8, 3)
    assert range_bounds("C3") == (2, 2, 2, 2)
    with pytest.raises(RenderError):
        range_bounds("D9:B2")
    with pytest.raises(RenderError):
        range_bounds("A1:Z1000")


def _probe(tmp_path: Path) -> dict[str, Any]:
    data = b"not really an xlsx, but pinned"
    (tmp_path / "p.xlsx").write_bytes(data)
    probe = {
        "id": "p",
        "file": "p.xlsx",
        "sha256": hashlib.sha256(data).hexdigest(),
        "sheets": [{"name": "It's", "range": "A1:B2"}],
    }
    (tmp_path / "probes.json").write_text(
        json.dumps({"schema": "rustsheet.render-probes/1", "probes": [probe]})
    )
    return probe


def test_pinned_bytes_refuses_a_resaved_probe(tmp_path: Path) -> None:
    probe = _probe(tmp_path)
    assert load_probes(tmp_path) == [probe]
    (tmp_path / "p.xlsx").write_bytes(b"re-saved")
    with pytest.raises(RenderError, match="sha256"):
        pinned_bytes(tmp_path, probe)


FORMAT = {
    "horizontalAlignment": "General",
    "verticalAlignment": "Bottom",
    "wrapText": False,
    "font": {
        "name": "Calibri",
        "size": 11.0,
        "bold": False,
        "italic": False,
        "underline": "None",
        "color": "#000000",
    },
    "fill": {"color": ""},
    "borders": [
        {"sideIndex": side, "style": "None", "weight": "Thin", "color": "#000000"}
        for side in ("EdgeTop", "EdgeBottom", "EdgeLeft", "EdgeRight", "DiagonalDown", "DiagonalUp")
    ],
}


def test_record_writes_a_complete_dump_pdf_and_chart_images(
    graph: GraphBackend, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    probe = _probe(tmp_path)
    calls: list[str] = []

    def call(method: str, url: str, **kw: Any) -> Any:
        tail = url.split("/items/1", 1)[-1]
        calls.append(f"{method} {tail}")
        if method == "POST" and tail == "/workbook/createSession":
            assert kw["json"] == {"persistChanges": False}  # the probe must stay as uploaded
            return _reply({"id": "s"})
        if "$select=text" in tail:
            assert "worksheets('It%27%27s')" in tail  # quote doubled, then URL-encoded
            grid = [["a", "b"], ["c", "d"]]
            return _reply(
                {
                    "text": grid,
                    "values": grid,
                    "valueTypes": [["String"] * 2] * 2,
                    "numberFormat": [["@"] * 2] * 2,
                }
            )
        if tail.endswith("/charts?$select=name,left,top,width,height"):
            return _reply(
                {"value": [{"name": "Chart 1", "left": 1.0, "top": 2.0, "width": 3.0, "height": 4.0}]}
            )
        if tail.endswith("/image"):
            return _reply({"value": base64.b64encode(b"PNG").decode()})
        if tail == "/content?format=pdf":
            return _reply({}, b"%PDF")
        return _reply({"id": "1"})

    def batch(urls: list[str], headers: dict[str, str]) -> list[Any]:
        assert headers == {"workbook-session-id": "s"}
        out: list[Any] = []
        for url in urls:
            if url.endswith("?$select=columnHidden"):
                out.append({"columnHidden": False})
            elif url.endswith("?$select=rowHidden"):
                out.append({"rowHidden": False})
            elif url.endswith("columnWidth"):
                out.append({"columnWidth": 48.0})
            elif url.endswith("rowHeight"):
                out.append({"rowHeight": 15.0})
            else:
                out.append(FORMAT)
        return out

    monkeypatch.setattr(graph, "_call", call)
    monkeypatch.setattr(graph, "batch_get", batch)
    out = tmp_path / "refs"
    dump = record(graph, tmp_path, probe, out, pdf=True, images=True)
    assert check_dump(dump, probe) == []
    assert json.loads((out / "p.render.json").read_text()) == dump
    assert (out / "p.pdf").read_bytes() == b"%PDF"
    assert (out / "p" / "It_s_Chart 1.png").read_bytes() == b"PNG"
    sheet = dump["sheets"][0]
    assert [c["a1"] for c in sheet["cells"]] == ["A1", "B1", "A2", "B2"]
    assert sheet["cells"][3]["text"] == "d"
    assert set(sheet["cells"][0]["borders"]) == {
        "EdgeLeft", "EdgeTop", "EdgeRight", "EdgeBottom", "DiagonalUp", "DiagonalDown"
    }  # fmt: skip
    # The PDF is converted from the stored file after the session closes; the scratch copy goes.
    assert calls.index("POST /workbook/closeSession") < calls.index("GET /content?format=pdf")
    assert calls[-1] == "DELETE "


def test_check_dump_refuses_a_dump_that_covers_less_than_the_probe(tmp_path: Path) -> None:
    probe = _probe(tmp_path)
    cell = {
        "text": "",
        "value": "",
        "valueType": "Empty",
        "numberFormat": "General",
        "hAlign": "General",
        "vAlign": "Bottom",
        "wrap": False,
        "font": dict.fromkeys(("name", "size", "bold", "italic", "underline", "color")),
        "fill": {"color": None},
        "borders": {
            side: {}
            for side in ("EdgeLeft", "EdgeTop", "EdgeRight", "EdgeBottom", "DiagonalUp", "DiagonalDown")
        },
    }
    sheet = {
        "name": "It's",
        "columns": [{"index": i, "columnWidth": 48.0, "hidden": False} for i in (1, 2)],
        "rows": [{"index": i, "rowHeight": 15.0, "hidden": False} for i in (1, 2)],
        "cells": [{"a1": a1, **cell} for a1 in ("A1", "B1", "A2", "B2")],
        "charts": [],
    }
    dump = {"schema": SCHEMA, "probe": "p", "sha256": probe["sha256"], "sheets": [sheet]}
    assert check_dump(dump, probe) == []
    sheet["cells"].pop()
    del sheet["cells"][0]["wrap"]
    sheet["rows"].append({"index": 3, "rowHeight": 15.0, "hidden": False})
    problems = check_dump(dump, probe)
    assert any("cells are not" in p for p in problems)
    assert any("rows are not" in p for p in problems)
    assert any("A1: lacks wrap" in p for p in problems)


def test_recorded_references_cover_their_probes() -> None:
    """Every committed tier-1 dump still matches its pinned probe and covers it completely."""
    probes = {p["id"]: p for p in load_probes(render_dir() / "probes")}
    for path in sorted((render_dir() / "refs" / "graph").glob("*.render.json")):
        dump = json.loads(path.read_text())
        assert dump["probe"] in probes, path
        assert check_dump(dump, probes[dump["probe"]]) == [], path


def test_a_probe_excel_will_not_open_is_recorded_as_refused(
    graph: GraphBackend, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    probe = _probe(tmp_path)
    deleted: list[str] = []

    def call(method: str, url: str, **_: Any) -> Any:
        if url.endswith("/createSession"):
            body = '{"error":{"code":"FileCorruptTryRepair","innerError":{"code":"unsupportedWorkbook"}}}'
            raise GraphRequestError(f"POST x -> 501: {body}", "FileCorruptTryRepair", "something went wrong")
        if url.endswith("format=pdf"):
            raise GraphRequestError("GET x -> 406: {}", None, None)
        if method == "DELETE":
            deleted.append(url)
        return _reply({"id": "1"})

    monkeypatch.setattr(graph, "_call", call)
    with pytest.raises(OracleRefusedError) as e:
        record(graph, tmp_path, probe, tmp_path / "refs", pdf=True, images=True)
    assert deleted  # the scratch copy is removed all the same
    refusal = json.loads(write_refusal(probe, tmp_path / "refs", e.value).read_text())
    assert (refusal["schema"], refusal["probe"], refusal["code"]) == (
        REFUSED_SCHEMA,
        "p",
        "FileCorruptTryRepair",
    )
    assert not (tmp_path / "refs" / "p.render.json").exists()


@pytest.mark.parametrize(
    ("key", "value"), [("id", "../x"), ("file", "sub/p.xlsx"), ("id", ".."), ("file", "")]
)
def test_load_probes_refuses_a_name_that_leaves_its_directory(tmp_path: Path, key: str, value: str) -> None:
    probe = _probe(tmp_path)
    probe[key] = value
    (tmp_path / "probes.json").write_text(
        json.dumps({"schema": "rustsheet.render-probes/1", "probes": [probe]})
    )
    with pytest.raises(RenderError, match="plain file name"):
        load_probes(tmp_path)
