"""Names defined through the API after load (`api_names`, T-3264)."""

from types import SimpleNamespace
from typing import Any

import pytest

from rustsheet_tools.api_names import api_names
from rustsheet_tools.oracle.backends import GraphBackend, LibreOfficeBackend, OracleError
from rustsheet_tools.spec2xlsx import build_xlsx

_BODY = "=ABS(" * 66 + "1" + ")" * 66  # long enough to be split across custom properties


def _spec(names: Any) -> dict[str, Any]:
    return {"sheets": [{"name": "S", "cells": {"A1": "=N"}}], "api_names": names}


def test_names_are_recorded_in_spec_order() -> None:
    xlsx = build_xlsx(_spec([{"name": "N", "formula": _BODY}, {"name": "M", "formula": "=1"}]))
    assert api_names(xlsx) == [("N", _BODY), ("M", "=1")]


def test_a_spec_without_api_names_records_none() -> None:
    xlsx = build_xlsx({"sheets": [{"name": "S", "cells": {"A1": 1}}]})
    assert api_names(xlsx) == []


@pytest.mark.parametrize(
    "names",
    [
        {"N": "=1"},
        [{"name": "N"}],
        [{"name": "", "formula": "=1"}],
        [{"name": "N", "formula": 1}],
        [{"name": "N", "formula": "=1", "scope": "S"}],
    ],
)
def test_malformed_api_names_are_refused(names: Any) -> None:
    with pytest.raises(ValueError, match="api"):
        build_xlsx(_spec(names))


@pytest.fixture
def graph(monkeypatch: pytest.MonkeyPatch) -> GraphBackend:
    for var in ("TENANT_ID", "CLIENT_ID", "CLIENT_SECRET", "DRIVE_ID"):
        monkeypatch.setenv(f"RUSTSHEET_ORACLE_{var}", "x")
    return GraphBackend()


def test_graph_adds_each_name_before_calculating_and_goes_on_when_refused(
    graph: GraphBackend, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    calls: list[tuple[str, str, Any]] = []

    def call(method: str, url: str, **kw: Any) -> Any:
        tail = url.rsplit("/items/1", 1)[-1]
        calls.append((method, tail, kw.get("json")))
        if tail == "/workbook/names/add":
            assert kw.get("refusable") is True
            if kw["json"]["name"] == "N":
                return SimpleNamespace(status_code=400, text='{"error":{"code":"InvalidArgument"}}')
        etag = "v2" if any(t == "/workbook/closeSession" for _, t, _ in calls) else "v1"
        return SimpleNamespace(json=lambda: {"id": "1", "eTag": etag}, content=b"saved", status_code=200)

    monkeypatch.setattr(graph, "_call", call)
    _, info = graph.recalc(
        build_xlsx(_spec([{"name": "N", "formula": _BODY}, {"name": "M", "formula": "=1"}]))
    )
    # The uploaded file's name, not the last name defined (T-3264 review).
    assert info.workbook_name is not None
    assert info.workbook_name.endswith(".xlsx")
    tails = [t for _, t, _ in calls]
    first = tails.index("/workbook/names/add")
    assert tails[first : first + 3] == [
        "/workbook/names/add",
        "/workbook/names/add",
        "/workbook/application/calculate",
    ]
    assert [j for m, t, j in calls if t == "/workbook/names/add"] == [
        {"name": "N", "reference": _BODY},
        {"name": "M", "reference": "=1"},
    ]
    err = capsys.readouterr().err
    assert "REFUSED name N" in err
    assert "ADDED name M '=1'" in err


def test_libreoffice_refuses_api_names() -> None:
    with pytest.raises(OracleError, match="api_names"):
        LibreOfficeBackend().recalc(build_xlsx(_spec([{"name": "N", "formula": "=1"}])))
